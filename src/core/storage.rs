use crate::core::models::Investment;
use anyhow::Context;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

// Use our centralized error and result types
pub use crate::error::Result;

#[derive(Debug)]
pub struct Storage {
    data_file: PathBuf,
}

impl Storage {
    pub fn new(data_file: PathBuf) -> Self {
        Storage { data_file }
    }

    pub fn open() -> Self {
        Self::new(Self::get_data_path())
    }

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

    pub fn load_investments(&self) -> Result<Vec<Investment>> {
        if !self.data_file.exists() {
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(&self.data_file)
            .with_context(|| format!("Failed to read file: {}", self.data_file.display()))?;

        serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse JSON from: {}", self.data_file.display()))
    }

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

    pub fn get_investment(&self, id: &str) -> Result<Option<Investment>> {
        let investments = self.load_investments()?;
        Ok(investments.into_iter().find(|inv| inv.id == id))
    }

    pub fn update_investment(&self, updated_investment: &Investment) -> Result<Option<Investment>> {
        let mut investments = self.load_investments()?;

        if let Some(pos) = investments
            .iter()
            .position(|inv| inv.id == updated_investment.id)
        {
            let old_investment = investments[pos].clone();
            investments[pos] = updated_investment.clone();
            self.save_investments(&investments)?;
            Ok(Some(old_investment))
        } else {
            Ok(None)
        }
    }

    pub fn delete_investment(&self, id: &str) -> Result<Option<Investment>> {
        let mut investments = self.load_investments()?;

        if let Some(pos) = investments.iter().position(|inv| inv.id == id) {
            let deleted_investment = investments.remove(pos);
            self.save_investments(&investments)?;
            Ok(Some(deleted_investment))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_investments(&self) -> Result<Vec<Investment>> {
        self.load_investments()
    }

    /// Convenience: load, find, mutate, save.
    pub fn mutate_investment<F>(&self, id: &str, f: F) -> Result<Option<Investment>>
    where
        F: FnOnce(&mut Investment) -> Result<()>,
    {
        let mut investments = self.load_investments()?;
        if let Some(inv) = investments.iter_mut().find(|i| i.id == id) {
            f(inv)?;
            let updated = inv.clone();
            self.save_investments(&investments)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }
}
