//! Uncertainty Analysis Types
//!
//! Type definitions for uncertainty propagation and bootstrap methods.

use serde::{Deserialize, Serialize};

/// Bootstrap analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapResult {
    pub original_statistic: f64,
    pub bootstrap_statistics: Vec<f64>,
    pub confidence_interval: (f64, f64),
    pub bias: f64,
    pub standard_error: f64,
}

/// Uncertainty propagation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyPropagationResult {
    pub output_uncertainty: f64,
    pub sensitivity_coefficients: Vec<f64>,
    pub covariance_matrix: Vec<Vec<f64>>,
}

/// Bootstrap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub n_resamples: usize,
    pub confidence_level: f64,
    pub method: BootstrapMethod,
}

/// Bootstrap sampling methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BootstrapMethod {
    Standard,
    Percentile,
    BCa, // Bias-corrected and accelerated
}

/// Monte Carlo simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    pub samples: Vec<f64>,
    pub mean: f64,
    pub standard_deviation: f64,
    pub confidence_interval: (f64, f64),
    pub distribution_fit: Option<String>, // Placeholder for DistributionFit
}