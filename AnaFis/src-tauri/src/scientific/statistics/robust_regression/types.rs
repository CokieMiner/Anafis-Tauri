//! Robust Regression Types
//!
//! Type definitions for robust regression methods and results.

use serde::{Deserialize, Serialize};

/// Result of robust regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustRegressionResult {
    pub coefficients: Vec<f64>,
    pub residuals: Vec<f64>,
    pub r_squared: f64,
    pub converged: bool,
    pub iterations: usize,
}

/// Configuration for robust regression methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustRegressionConfig {
    pub method: RobustRegressionMethod,
    pub max_iterations: usize,
    pub tolerance: f64,
    pub tuning_constant: Option<f64>,
}

/// Available robust regression methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RobustRegressionMethod {
    Huber,
    Ransac,
    Irls,
}