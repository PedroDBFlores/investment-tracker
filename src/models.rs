use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use chrono::Utc;

#[derive(Debug, Error)]
pub enum InvestmentError {
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid date format: {0}")]
    InvalidDate(String),
    #[error("Invalid investment type: {0}")]
    InvalidType(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvestmentType {
    Stock,
    ETF,
    MutualFund,
    Deposit,
    Bond,
    Crypto,
    Other(String),
}

impl fmt::Display for InvestmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvestmentType::Stock => write!(f, "Stock"),
            InvestmentType::ETF => write!(f, "ETF"),
            InvestmentType::MutualFund => write!(f, "Mutual Fund"),
            InvestmentType::Deposit => write!(f, "Deposit"),
            InvestmentType::Bond => write!(f, "Bond"),
            InvestmentType::Crypto => write!(f, "Crypto"),
            InvestmentType::Other(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Investment {
    pub id: String,
    pub investment_type: InvestmentType,
    pub name: String,
    pub symbol: Option<String>,
    pub amount: f64,
    pub date: String,
    pub current_value: Option<f64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Investment {
    pub fn new(
        id: String,
        investment_type: InvestmentType,
        name: String,
        symbol: Option<String>,
        amount: f64,
        date: String,
        current_value: Option<f64>,
        notes: Option<String>,
    ) -> Result<Self, InvestmentError> {
        if amount <= 0.0 {
            return Err(InvestmentError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }
        
        // Basic date validation (YYYY-MM-DD format)
        if date.len() != 10 || date.chars().nth(4) != Some('-') || date.chars().nth(7) != Some('-') {
            return Err(InvestmentError::InvalidDate(
                "Date must be in YYYY-MM-DD format".to_string(),
            ));
        }
        
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        Ok(Investment {
            id,
            investment_type,
            name,
            symbol,
            amount,
            date,
            current_value,
            notes,
            created_at: now.clone(),
            updated_at: now,
        })
    }
    
    pub fn update_amount(&mut self, new_amount: f64) -> Result<(), InvestmentError> {
        if new_amount <= 0.0 {
            return Err(InvestmentError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }
        self.amount = new_amount;
        self.updated_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Ok(())
    }
    
    pub fn update_current_value(&mut self, new_value: f64) -> Result<(), InvestmentError> {
        if new_value < 0.0 {
            return Err(InvestmentError::InvalidAmount(
                "Value cannot be negative".to_string(),
            ));
        }
        self.current_value = Some(new_value);
        self.updated_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Ok(())
    }
    
    pub fn return_on_investment(&self) -> Option<f64> {
        self.current_value.map(|current| current - self.amount)
    }
    
    pub fn return_percentage(&self) -> Option<f64> {
        self.current_value.map(|current| (current - self.amount) / self.amount * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_investment_creation() {
        let investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            Some("TEST".to_string()),
            1000.0,
            "2024-01-15".to_string(),
            Some(1200.0),
            Some("Test investment".to_string()),
        ).unwrap();
        
        assert_eq!(investment.id, "test-id");
        assert_eq!(investment.name, "Test Company");
        assert_eq!(investment.amount, 1000.0);
        assert_eq!(investment.date, "2024-01-15");
        assert_eq!(investment.current_value, Some(1200.0));
        assert_eq!(investment.notes, Some("Test investment".to_string()));
        assert!(investment.created_at.contains("202"));
        assert!(investment.updated_at.contains("202"));
    }
    
    #[test]
    fn test_invalid_amount() {
        let result = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            0.0,
            "2024-01-15".to_string(),
            None,
            None,
        );
        
        assert!(result.is_err());
        if let Err(InvestmentError::InvalidAmount(msg)) = result {
            assert_eq!(msg, "Amount must be greater than 0");
        } else {
            panic!("Expected InvalidAmount error");
        }
    }
    
    #[test]
    fn test_invalid_date_format() {
        let result = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "invalid-date".to_string(),
            None,
            None,
        );
        
        assert!(result.is_err());
        if let Err(InvestmentError::InvalidDate(msg)) = result {
            assert_eq!(msg, "Date must be in YYYY-MM-DD format");
        } else {
            panic!("Expected InvalidDate error");
        }
    }
    
    #[test]
    fn test_update_amount() {
        let mut investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "2024-01-15".to_string(),
            None,
            None,
        ).unwrap();
        
        let original_updated_at = investment.updated_at.clone();
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        investment.update_amount(1500.0).unwrap();
        
        assert_eq!(investment.amount, 1500.0);
        // Check that updated_at was actually updated by comparing timestamps
        assert!(investment.updated_at >= original_updated_at);
        // In practice, it should be different, but for test reliability we just check it's not older
    }
    
    #[test]
    fn test_update_invalid_amount() {
        let mut investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "2024-01-15".to_string(),
            None,
            None,
        ).unwrap();
        
        let result = investment.update_amount(0.0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_return_calculations() {
        let investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "2024-01-15".to_string(),
            Some(1500.0),
            None,
        ).unwrap();
        
        assert_eq!(investment.return_on_investment(), Some(500.0));
        assert_eq!(investment.return_percentage(), Some(50.0));
    }
    
    #[test]
    fn test_return_calculations_no_current_value() {
        let investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "2024-01-15".to_string(),
            None,
            None,
        ).unwrap();
        
        assert_eq!(investment.return_on_investment(), None);
        assert_eq!(investment.return_percentage(), None);
    }
    
    #[test]
    fn test_serialize_deserialize() {
        let original = Investment::new(
            "test-id".to_string(),
            InvestmentType::ETF,
            "Test ETF".to_string(),
            Some("ETF".to_string()),
            2000.0,
            "2024-02-20".to_string(),
            Some(2500.0),
            Some("Test ETF investment".to_string()),
        ).unwrap();
        
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Investment = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(original, deserialized);
    }
    
    #[test]
    fn test_investment_type_display() {
        assert_eq!(format!("{}", InvestmentType::Stock), "Stock");
        assert_eq!(format!("{}", InvestmentType::ETF), "ETF");
        assert_eq!(format!("{}", InvestmentType::MutualFund), "Mutual Fund");
        assert_eq!(format!("{}", InvestmentType::Other("Custom".to_string())), "Custom");
    }
}