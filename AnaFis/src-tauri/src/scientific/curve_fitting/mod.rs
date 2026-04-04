//! Curve fitting module using profiled ODR with per-point latent x-corrections.
// Note(odr-option-2): Profiled latent-variable ODR is implemented; future work is
// optional and focused on stronger trust-region/Schur-complement step control.
mod logic;
/// Tauri commands for curve fitting.
pub mod commands;
/// Unit tests for curve fitting.
mod tests;
/// Shared types and result structures.
pub mod types;
/// Core engine for profiled ODR computation.
pub use logic::engine;

pub use commands::*;
pub use types::*;
