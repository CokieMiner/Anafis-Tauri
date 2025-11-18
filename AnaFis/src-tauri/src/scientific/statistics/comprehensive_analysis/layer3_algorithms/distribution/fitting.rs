//! Distribution fitting functionality - Facade module

use crate::scientific::statistics::types::DistributionFit;

// Import the core fitting engine
use super::fitters::distribution_fitting_core::StatisticalDistributionEngine as CoreEngine;

/// Statistical distribution engine - coordinates distribution-related computations
/// This is a facade that delegates to the organized fitting modules
pub struct StatisticalDistributionEngine;

impl StatisticalDistributionEngine {
    /// Fit multiple distributions to data and return best fits
    pub fn fit_distributions(data: &[f64]) -> Result<Vec<DistributionFit>, String> {
        CoreEngine::fit_distributions(data)
    }

    /// Compute statistical moments (mean, variance, skewness, kurtosis)
    pub fn moments(data: &[f64]) -> Result<(f64, f64, f64, f64), String> {
        CoreEngine::moments(data)
    }

    /// Compute variance of a dataset
    pub fn variance(data: &[f64]) -> f64 {
        CoreEngine::variance(data)
    }

    /// Rank transformation for statistical tests
    pub fn rank_transformation(data: &[f64]) -> Vec<f64> {
        CoreEngine::rank_transformation(data)
    }
}