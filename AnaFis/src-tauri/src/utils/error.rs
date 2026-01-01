// src-tauri/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnaFisError {
    #[error("Window management error: {0}")]
    Window(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO operation failed")]
    Io(#[from] std::io::Error),

    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("CSV processing error")]
    Csv(#[from] csv::Error),
}
