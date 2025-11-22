//! Distribution Analysis Types
//!
//! Type definitions for distribution fitting, testing, and analysis.

use serde::{Deserialize, Serialize};
use super::fitting::DistributionFit;

/// Distribution fitting result with multiple candidate distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionFittingResult {
    pub fits: Vec<DistributionFit>,
    pub best_fit: Option<DistributionFit>,
    pub ranking_criteria: String, // "aic", "bic", or "goodness_of_fit"
}

/// Goodness of fit test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodnessOfFitTest {
    pub test_name: String,
    pub statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom: usize,
    pub distribution_name: String,
    pub parameters: Vec<(String, f64)>,
}