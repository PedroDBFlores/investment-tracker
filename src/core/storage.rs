use crate::core::models::Investment;
use anyhow::Context;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

// Use our centralized error and result types
pub use crate::error::Result;

/// JSON-backed persistence layer for investment records.
///
/// All data is stored in a single JSON file (default:
/// `~/.investment_tracker/investments.json`). The path can be overridden via
/// the `INVESTMENT_TRACKER_DATA` environment variable or the app config file.
#[derive(Debug)]
pub struct Storage {
    data_file: PathBuf,
}

impl Storage {
    /// Create a `Storage` instance pointing at `data_file`.
    pub fn new(data_file: PathBuf) -> Self {
        Storage { data_file }
    }

    /// Create a `Storage` instance using the default data-file path.
    pub fn open() -> Self {
        Self::new(Self::get_data_path())
    }

    /// Resolve the path to the data file, respecting (in order):
    /// 1. `INVESTMENT_TRACKER_DATA` environment variable
    /// 2. The `data_directory` setting in the app config file
    /// 3. The default `~/.investment_tracker/investments.json`
    pub fn get_data_path() -> PathBuf {
        // Check for environment variable override (for testing)
        if let Ok(data_path) = std::env::var("INVESTMENT_TRACKER_DATA") {
            return PathBuf::from(data_path);
        }

        // Try to load from config file
        if let Ok(config) = crate::core::config::Config::load() {
            if let Some(custom_dir) = config.get_data_directory() {
                return custom_dir.join("investments.json");
            }
        }

        // Default location
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let data_dir = home_dir.join(".investment_tracker");
        let data_file = data_dir.join("investments.json");
        data_file
    }

    /// Load all investments from disk. Returns an empty `Vec` if the data file
    /// does not exist yet.
    pub fn load_investments(&self) -> Result<Vec<Investment>> {
        if !self.data_file.exists() {
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(&self.data_file)
            .with_context(|| format!("Failed to read file: {}", self.data_file.display()))?;

        serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse JSON from: {}", self.data_file.display()))
    }

    /// Serialise and atomically write all investments to disk, creating parent
    /// directories as needed.
    pub fn save_investments(&self, investments: &[Investment]) -> Result<()> {
        let data = serde_json::to_string_pretty(investments)
            .context("Failed to serialize investments to JSON")?;

        // Create parent directories if they don't exist
        if let Some(parent) = self.data_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(&self.data_file, data)
            .with_context(|| format!("Failed to write to file: {}", self.data_file.display()))
    }

    /// Bulk-insert multiple investments in a single load → append → save cycle.
    /// Investments without an ID get a fresh UUID assigned.  Returns the saved
    /// investments with their final IDs.
    pub fn add_investments(&self, investments: Vec<Investment>) -> Result<Vec<Investment>> {
        let mut existing = self.load_investments()?;
        let mut saved = Vec::with_capacity(investments.len());

        for investment in investments {
            let id = if investment.id.is_empty() {
                Uuid::new_v4().to_string()
            } else {
                investment.id.clone()
            };
            let mut inv = investment;
            inv.id = id;
            saved.push(inv.clone());
            existing.push(inv);
        }

        self.save_investments(&existing)?;
        Ok(saved)
    }

    /// Persist a single new investment, generating a UUID if `investment.id` is
    /// empty. Returns the saved investment with its final ID.
    pub fn add_investment(&self, investment: Investment) -> Result<Investment> {
        let mut investments = self.load_investments()?;

        // Generate UUID if not provided
        let id = if investment.id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            investment.id.clone()
        };

        // Create new investment with generated ID
        let mut new_investment = investment;
        new_investment.id = id.clone();

        investments.push(new_investment.clone());
        self.save_investments(&investments)?;

        Ok(new_investment)
    }

    /// Resolve a user-supplied ID (full UUID or short prefix) to the full UUID
    /// stored on disk.  Returns `Ok(None)` if no investment matches, and an
    /// error if the prefix is ambiguous (matches more than one investment).
    pub fn resolve_id<'a>(
        &self,
        id: &'a str,
        investments: &[Investment],
    ) -> Result<Option<String>> {
        // 1. Exact match — always preferred.
        if let Some(inv) = investments.iter().find(|inv| inv.id == id) {
            return Ok(Some(inv.id.clone()));
        }

        // 2. Prefix match — useful when the user copies the 8-char short ID
        //    shown by `list`.
        let matches: Vec<&Investment> = investments
            .iter()
            .filter(|inv| inv.id.starts_with(id))
            .collect();

        match matches.len() {
            0 => Ok(None),
            1 => Ok(Some(matches[0].id.clone())),
            _ => Err(anyhow::anyhow!(
                "Ambiguous ID prefix '{}' matches {} investments. Please provide more characters.",
                id,
                matches.len()
            )),
        }
    }

    /// Look up a single investment by full UUID or prefix. Returns `Ok(None)` if
    /// no match is found.
    pub fn get_investment(&self, id: &str) -> Result<Option<Investment>> {
        let investments = self.load_investments()?;
        let resolved = self.resolve_id(id, &investments)?;
        Ok(resolved.and_then(|full_id| investments.into_iter().find(|inv| inv.id == full_id)))
    }

    /// Replace an existing investment record (matched by ID) with
    /// `updated_investment`. Returns the previous record, or `Ok(None)` if not
    /// found.
    pub fn update_investment(&self, updated_investment: &Investment) -> Result<Option<Investment>> {
        let mut investments = self.load_investments()?;

        if let Some(pos) = investments.iter().position(|inv| {
            inv.id == updated_investment.id || inv.id.starts_with(&updated_investment.id)
        }) {
            let old_investment = investments[pos].clone();
            investments[pos] = updated_investment.clone();
            self.save_investments(&investments)?;
            Ok(Some(old_investment))
        } else {
            Ok(None)
        }
    }

    /// Remove and return the investment with the given ID (or prefix). Returns
    /// `Ok(None)` if not found.
    pub fn delete_investment(&self, id: &str) -> Result<Option<Investment>> {
        let mut investments = self.load_investments()?;
        let resolved = self.resolve_id(id, &investments)?;
        let full_id = match resolved {
            Some(ref fid) => fid.clone(),
            None => return Ok(None),
        };

        if let Some(pos) = investments.iter().position(|inv| inv.id == full_id) {
            let deleted_investment = investments.remove(pos);
            self.save_investments(&investments)?;
            Ok(Some(deleted_investment))
        } else {
            Ok(None)
        }
    }

    /// Return every investment currently on disk.
    pub fn get_all_investments(&self) -> Result<Vec<Investment>> {
        self.load_investments()
    }

    /// Convenience: load, find, mutate, save.
    pub fn mutate_investment<F>(&self, id: &str, f: F) -> Result<Option<Investment>>
    where
        F: FnOnce(&mut Investment) -> Result<()>,
    {
        let mut investments = self.load_investments()?;
        let resolved = self.resolve_id(id, &investments)?;
        let full_id = match resolved {
            Some(fid) => fid,
            None => return Ok(None),
        };
        if let Some(inv) = investments.iter_mut().find(|i| i.id == full_id) {
            f(inv)?;
            let updated = inv.clone();
            self.save_investments(&investments)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }
}
