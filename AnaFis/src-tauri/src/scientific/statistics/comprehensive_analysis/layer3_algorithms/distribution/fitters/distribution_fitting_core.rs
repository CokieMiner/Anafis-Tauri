//! Core distribution fitting functionality

use super::super::moments;
use rayon::prelude::*;
use crate::scientific::statistics::types::DistributionFit;

// Import fitting functions from submodules
use super::continuous_distributions::{
    fit_normal_distribution,
    fit_lognormal_distribution,
    fit_exponential_distribution,
    fit_weibull_distribution,
    fit_gamma_distribution,
    fit_beta_distribution,
};

use super::extreme_distributions::{
    fit_gumbel_distribution,
    fit_pareto_distribution,
    fit_johnson_su_distribution,
    fit_burr_distribution,
};

use super::heavy_tail_distributions::{
    fit_students_t_distribution,
    fit_cauchy_distribution,
};

/// Statistical distribution engine - coordinates distribution-related computations
pub struct StatisticalDistributionEngine;

impl StatisticalDistributionEngine {
    /// Fit multiple distributions to data and return best fits
    pub fn fit_distributions(data: &[f64]) -> Result<Vec<DistributionFit>, String> {
        if data.is_empty() {
            return Err("Cannot fit distributions to empty dataset".to_string());
        }

        // Define all distribution fitting functions
        #[allow(clippy::type_complexity)]
        let fitting_functions: Vec<fn(&[f64]) -> Result<DistributionFit, String>> = vec![
            fit_normal_distribution,
            fit_lognormal_distribution,
            fit_exponential_distribution,
            fit_weibull_distribution,
            fit_gamma_distribution,
            fit_beta_distribution,
            fit_gumbel_distribution,
            fit_pareto_distribution,
            fit_johnson_su_distribution,
            fit_burr_distribution,
            fit_students_t_distribution,
            fit_cauchy_distribution,
        ];

        // Fit all distributions in parallel
        let fits: Vec<DistributionFit> = fitting_functions
            .into_par_iter()
            .filter_map(|fit_fn| fit_fn(data).ok())
            .collect();

        // Filter out fits with invalid AIC values and sort by goodness of fit (lower AIC is better)
        let mut sorted_fits = fits
            .into_iter()
            .filter(|fit| fit.aic.is_finite())
            .collect::<Vec<_>>();
        sorted_fits.sort_by(|a, b| match a.aic.partial_cmp(&b.aic) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        Ok(sorted_fits)
    }

    /// Compute statistical moments (mean, variance, skewness, kurtosis)
    pub fn moments(data: &[f64]) -> Result<(f64, f64, f64, f64), String> {
        moments::moments(data)
    }

    /// Rank transformation for statistical tests
    pub fn rank_transformation(data: &[f64]) -> Vec<f64> {
        moments::rank_transformation(data)
    }
}