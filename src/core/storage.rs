use crate::core::models::Investment;
use anyhow::Context;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

// Use our centralized error and result types
pub use crate::error::Result;

/// The core set of persistence operations required by the application.
///
/// Implementing this trait for a new backend (e.g. SQLite) is sufficient to
/// make the entire command layer work with that backend without any changes to
/// the command code.  The current implementation is [`Storage`] (JSON files).
#[allow(dead_code)]
pub trait StorageBackend {
    /// Load all investments. Returns an empty `Vec` when no data exists yet.
    fn load_investments(&self) -> Result<Vec<Investment>>;
    /// Persist the full investment list, replacing any previous contents.
    fn save_investments(&self, investments: &[Investment]) -> Result<()>;
    /// Add a single investment, assigning a UUID if `id` is empty.
    fn add_investment(&self, investment: Investment) -> Result<Investment>;
    /// Bulk-add multiple investments in one write cycle.
    fn add_investments(&self, investments: Vec<Investment>) -> Result<Vec<Investment>>;
    /// Look up by full UUID or short prefix. Returns `None` when not found.
    fn get_investment(&self, id: &str) -> Result<Option<Investment>>;
    /// Replace the matching investment. Returns the previous record or `None`.
    fn update_investment(&self, updated: &Investment) -> Result<Option<Investment>>;
    /// Remove and return the matching investment, or `None` if not found.
    fn delete_investment(&self, id: &str) -> Result<Option<Investment>>;
    /// Return every investment on disk.
    fn get_all_investments(&self) -> Result<Vec<Investment>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::InvestmentType;
    use tempfile::tempdir;

    fn make_storage() -> (Storage, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("investments.json");
        (Storage::new(path), dir)
    }

    fn make_investment(name: &str, amount: f64, inv_type: InvestmentType) -> Investment {
        Investment::new(
            String::new(),
            inv_type,
            name.to_string(),
            None,
            amount,
            "2024-01-15".to_string(),
            Some(amount),
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    // ── Basic CRUD ────────────────────────────────────────────────────────────

    #[test]
    fn test_load_empty_when_no_file() {
        let (storage, _dir) = make_storage();
        let investments = storage.load_investments().unwrap();
        assert!(investments.is_empty());
    }

    #[test]
    fn test_add_and_get_investment() {
        let (storage, _dir) = make_storage();
        let inv = make_investment("Apple", 1000.0, InvestmentType::Stock);
        let saved = storage.add_investment(inv).unwrap();
        assert!(!saved.id.is_empty(), "ID should have been assigned");

        let loaded = storage.get_investment(&saved.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Apple");
    }

    #[test]
    fn test_add_investment_assigns_uuid() {
        let (storage, _dir) = make_storage();
        let inv = make_investment("ACME", 500.0, InvestmentType::Stock);
        assert!(inv.id.is_empty());
        let saved = storage.add_investment(inv).unwrap();
        assert_eq!(saved.id.len(), 36, "UUID should be 36 chars");
    }

    #[test]
    fn test_get_investment_by_prefix() {
        let (storage, _dir) = make_storage();
        let saved = storage
            .add_investment(make_investment("MSFT", 2000.0, InvestmentType::Stock))
            .unwrap();
        let prefix = &saved.id[..8];
        let found = storage.get_investment(prefix).unwrap().unwrap();
        assert_eq!(found.id, saved.id);
    }

    #[test]
    fn test_get_investment_not_found() {
        let (storage, _dir) = make_storage();
        let result = storage.get_investment("nonexistent-id").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_id_ambiguous_prefix() {
        let (storage, _dir) = make_storage();
        // Use a fixed prefix so both IDs start with the same characters.
        let mut inv1 = make_investment("A", 100.0, InvestmentType::Stock);
        let mut inv2 = make_investment("B", 200.0, InvestmentType::Stock);
        inv1.id = "aaa00001-0000-0000-0000-000000000000".to_string();
        inv2.id = "aaa00002-0000-0000-0000-000000000000".to_string();
        storage.add_investment(inv1).unwrap();
        storage.add_investment(inv2).unwrap();

        let investments = storage.load_investments().unwrap();
        let result = storage.resolve_id("aaa0000", &investments);
        assert!(result.is_err(), "Ambiguous prefix should return an error");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Ambiguous"), "Error message should say 'Ambiguous'");
    }

    #[test]
    fn test_update_investment() {
        let (storage, _dir) = make_storage();
        let saved = storage
            .add_investment(make_investment("Tesla", 3000.0, InvestmentType::Stock))
            .unwrap();

        let mut updated = saved.clone();
        updated.amount = 3500.0;
        let old = storage.update_investment(&updated).unwrap().unwrap();
        assert_eq!(old.amount, 3000.0);

        let reloaded = storage.get_investment(&saved.id).unwrap().unwrap();
        assert_eq!(reloaded.amount, 3500.0);
    }

    #[test]
    fn test_delete_investment() {
        let (storage, _dir) = make_storage();
        let saved = storage
            .add_investment(make_investment("NVDA", 4000.0, InvestmentType::Stock))
            .unwrap();

        let deleted = storage.delete_investment(&saved.id).unwrap().unwrap();
        assert_eq!(deleted.name, "NVDA");

        let all = storage.get_all_investments().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_delete_investment_not_found() {
        let (storage, _dir) = make_storage();
        let result = storage.delete_investment("no-such-id").unwrap();
        assert!(result.is_none());
    }

    // ── Error scenarios ───────────────────────────────────────────────────────

    #[test]
    fn test_load_corrupted_json_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("investments.json");
        std::fs::write(&path, b"this is not valid json {{{").unwrap();
        let storage = Storage::new(path);
        let result = storage.load_investments();
        assert!(result.is_err(), "Corrupted JSON should return an error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Failed to parse JSON"),
            "Error should mention JSON parsing: {msg}"
        );
    }

    #[test]
    fn test_load_empty_json_array_returns_empty_vec() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("investments.json");
        std::fs::write(&path, b"[]").unwrap();
        let storage = Storage::new(path);
        let investments = storage.load_investments().unwrap();
        assert!(investments.is_empty());
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c").join("data.json");
        let storage = Storage::new(nested.clone());
        storage.save_investments(&[]).unwrap();
        assert!(nested.exists(), "save_investments should create parent dirs");
    }

    #[test]
    fn test_add_investments_bulk() {
        let (storage, _dir) = make_storage();
        let batch = vec![
            make_investment("A", 100.0, InvestmentType::Stock),
            make_investment("B", 200.0, InvestmentType::ETF),
            make_investment("C", 300.0, InvestmentType::Crypto),
        ];
        let saved = storage.add_investments(batch).unwrap();
        assert_eq!(saved.len(), 3);
        assert!(saved.iter().all(|inv| !inv.id.is_empty()));

        let all = storage.get_all_investments().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_mutate_investment() {
        let (storage, _dir) = make_storage();
        let saved = storage
            .add_investment(make_investment("BTC", 5000.0, InvestmentType::Crypto))
            .unwrap();

        let result = storage
            .mutate_investment(&saved.id, |inv| {
                inv.amount = 6000.0;
                Ok(())
            })
            .unwrap()
            .unwrap();

        assert_eq!(result.amount, 6000.0);
        let reloaded = storage.get_investment(&saved.id).unwrap().unwrap();
        assert_eq!(reloaded.amount, 6000.0);
    }

    #[test]
    fn test_mutate_investment_not_found() {
        let (storage, _dir) = make_storage();
        let result = storage
            .mutate_investment("nonexistent", |_| Ok(()))
            .unwrap();
        assert!(result.is_none());
    }
}

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

        for mut inv in investments {
            if inv.id.is_empty() {
                inv.id = Uuid::new_v4().to_string();
            }
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

        let mut new_investment = investment;
        if new_investment.id.is_empty() {
            new_investment.id = Uuid::new_v4().to_string();
        }

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

impl StorageBackend for Storage {
    fn load_investments(&self) -> Result<Vec<Investment>> {
        self.load_investments()
    }
    fn save_investments(&self, investments: &[Investment]) -> Result<()> {
        self.save_investments(investments)
    }
    fn add_investment(&self, investment: Investment) -> Result<Investment> {
        self.add_investment(investment)
    }
    fn add_investments(&self, investments: Vec<Investment>) -> Result<Vec<Investment>> {
        self.add_investments(investments)
    }
    fn get_investment(&self, id: &str) -> Result<Option<Investment>> {
        self.get_investment(id)
    }
    fn update_investment(&self, updated: &Investment) -> Result<Option<Investment>> {
        self.update_investment(updated)
    }
    fn delete_investment(&self, id: &str) -> Result<Option<Investment>> {
        self.delete_investment(id)
    }
    fn get_all_investments(&self) -> Result<Vec<Investment>> {
        self.get_all_investments()
    }
}
