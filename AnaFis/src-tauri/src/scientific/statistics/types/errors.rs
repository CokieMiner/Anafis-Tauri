use thiserror::Error;

/// A structured error type for analysis operations
#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Numerical instability: {0}")]
    NumericalInstability(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}