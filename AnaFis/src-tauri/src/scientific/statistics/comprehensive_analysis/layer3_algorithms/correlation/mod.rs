//! Correlation analysis module
//!
//! This module provides comprehensive correlation analysis functionality,
//! organized into focused submodules for better maintainability.

pub mod hypothesis_testing;
pub mod correlation_methods;
pub mod correlation_matrix;
pub mod correlation_utils;
pub mod correlation_engine;

// Re-export main types and engines for convenient access
pub use crate::scientific::statistics::types::CorrelationTestResult;
pub use hypothesis_testing::HypothesisTestingEngine;
pub use correlation_engine::CorrelationEngine;

// Re-export key functions for backward compatibility
pub use correlation_engine::{
    CorrelationEngine as CorrelationComputationEngine,
};

// Re-export hypothesis testing functions
pub use hypothesis_testing::HypothesisTestingEngine as CorrelationHypothesisTestingEngine;