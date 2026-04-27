use std::env::temp_dir;
use std::fs::OpenOptions;
use std::io::{Result, stderr};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt::layer, prelude::*, registry};

/// Initialize structured logging with file and console output
pub fn init_logging() -> Result<()> {
    let log_temp_dir = temp_dir();
    let log_path = log_temp_dir.join("anafis_debug.log");

    // Create a file appender
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // Setup tracing with both file and console output
    let file_layer = layer().with_writer(file).with_ansi(false);

    let console_layer = layer().with_writer(stderr);
    registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(file_layer)
        .with(console_layer)
        .init();

    info!(log_file = ?log_path, "Logging initialized");
    Ok(())
}

/// Log an informational message
pub fn log_info(message: &str) {
    info!(message = message);
}
