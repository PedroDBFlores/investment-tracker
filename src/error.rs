use thiserror::Error;

/// Domain-specific errors produced by investment operations.
///
/// These are typically wrapped in [`anyhow::Error`] via the blanket
/// `From<InvestmentError>` impl, so callers can use the `?` operator freely.
#[derive(Debug, Error)]
pub enum InvestmentError {
    /// A monetary value or price failed a positive-number constraint.
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    /// A date string failed YYYY-MM-DD parsing or calendar validation.
    #[error("Invalid date format: {0}")]
    InvalidDate(String),
    /// No investment matched the supplied ID or prefix.
    #[error("Investment not found: {0}")]
    NotFound(String),
    /// A sell operation requested more units than are available.
    #[error("Insufficient units: {0}")]
    InsufficientUnits(String),
}

/// Standard `Result` alias used throughout the crate (`anyhow::Result<T>`).
pub type Result<T> = anyhow::Result<T>;
