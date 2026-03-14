use crate::models::Investment;
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug)]
pub struct Storage {
    data_file: PathBuf,
}

impl Storage {
    pub fn new(data_file: PathBuf) -> Self {
        Storage { data_file }
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
    
    pub fn add_investment(&self, mut investment: Investment) -> Result<Investment> {
        let mut investments = self.load_investments()?;
        
        // Generate UUID if not provided
        let id = if investment.id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            investment.id.clone()
        };
        
        // Update investment with generated ID
        investment.id = id.clone();
        
        investments.push(investment.clone());
        self.save_investments(&investments)?;
        
        Ok(investment)
    }
    
    pub fn get_investment(&self, id: &str) -> Result<Option<Investment>> {
        let investments = self.load_investments()?;
        Ok(investments.into_iter().find(|inv| inv.id == id))
    }
    
    pub fn update_investment(&self, updated_investment: &Investment) -> Result<Option<Investment>> {
        let mut investments = self.load_investments()?;
        
        if let Some(pos) = investments.iter().position(|inv| inv.id == updated_investment.id) {
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
}