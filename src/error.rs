use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvestmentError {
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid date format: {0}")]
    InvalidDate(String),
    #[allow(dead_code)]
    #[error("Invalid investment type: {0}")]
    InvalidType(String),
    #[allow(dead_code)]
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Investment not found: {0}")]
    NotFound(String),
}

// Use anyhow::Result as our standard Result type
pub type Result<T> = anyhow::Result<T>;

// Since InvestmentError implements StdError, anyhow::Error already provides
// From<InvestmentError> for anyhow::Error automatically
