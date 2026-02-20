//! Curve fitting module for fitting data to various models using ODR.
/// Tauri commands for curve fitting.
pub mod commands;
/// Core engine for ODR computation.
pub mod engine;
/// Unit tests for curve fitting.
mod tests;
/// Shared types and result structures.
pub mod types;

pub use commands::*;
pub use types::*;
