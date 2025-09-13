// src-tauri/src/logging.rs
use tracing::{info, error};
use tracing_subscriber::{fmt, EnvFilter, prelude::*};
use std::fs::OpenOptions;
use anyhow::Result;

/// Initialize structured logging with file and console output
pub fn init_logging() -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let log_path = temp_dir.join("anafis_debug.log");

    // Create a file appender
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // Setup tracing with both file and console output
    let file_layer = fmt::layer()
        .with_writer(file)
        .with_ansi(false);

    let console_layer = fmt::layer()
        .with_writer(std::io::stderr);

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with(file_layer)
        .with(console_layer)
        .init();

    info!(log_file = ?log_path, "Logging initialized");
    Ok(())
}

/// Log an error with context
pub fn log_error(context: &str, error: &dyn std::error::Error) {
    error!(context = context, error = %error, "Error occurred");
}

/// Log an informational message
pub fn log_info(message: &str) {
    info!(message = message);
}
