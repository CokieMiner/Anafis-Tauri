//! Descriptive Statistics Module
//!
//! This module provides comprehensive descriptive statistical analysis
//! including central tendency, dispersion, shape statistics, and confidence intervals.

pub mod coordinator;
pub mod kde;
pub mod quantiles;
pub mod central_tendency;
pub mod dispersion;
pub mod shape_statistics;
pub mod bootstrap_confidence;

// Re-export the main coordinator for backward compatibility
pub use coordinator::DescriptiveStatsCoordinator;