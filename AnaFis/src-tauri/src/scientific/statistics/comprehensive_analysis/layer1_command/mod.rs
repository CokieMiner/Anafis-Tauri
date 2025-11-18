//! Layer 1: Command Interface & Orchestration
//!
//! This is the main entry point for the comprehensive statistical analysis system.
//! It handles input validation, analysis orchestration, and output formatting.

// Re-export the main command interface for backward compatibility
pub mod command;
pub use command::ComprehensiveAnalysisCommand;

// Submodules for organized functionality
pub mod validation;
pub mod detection;
pub mod orchestration;
pub mod formatting;
pub mod quality;