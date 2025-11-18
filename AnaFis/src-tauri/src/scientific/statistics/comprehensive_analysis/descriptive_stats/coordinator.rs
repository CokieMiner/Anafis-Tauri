//! Descriptive Statistics Coordinator
//!
//! This module provides the main coordinator for descriptive statistical analysis.
//! It orchestrates computations from the various descriptive statistics submodules.

use rand_pcg::Pcg64;
use crate::scientific::statistics::comprehensive_analysis::descriptive_stats::{
    central_tendency::CentralTendency,
    dispersion::Dispersion,
    shape_statistics::ShapeStatistics,
    bootstrap_confidence::BootstrapConfidence,
    quantiles::{Quantiles, QuantileMethod},
};
use crate::scientific::statistics::types::descriptive::DescriptiveStats;

/// Descriptive Statistics Coordinator
/// Coordinates basic statistical summary computations
pub struct DescriptiveStatsCoordinator;

impl DescriptiveStatsCoordinator {
    /// Analyze descriptive statistics for a dataset
    pub fn analyze(data: &[f64], bootstrap_samples: Option<usize>, rng: &mut Pcg64) -> Result<DescriptiveStats, String> {
        Self::analyze_with_uncertainties(data, None, None, bootstrap_samples, rng)
    }

    /// Analyze descriptive statistics for a dataset with uncertainty consideration
    pub fn analyze_with_uncertainties(
        data: &[f64],
        _uncertainties: Option<&[f64]>,
        _confidence_levels: Option<&[f64]>,
        bootstrap_samples: Option<usize>,
        rng: &mut Pcg64
    ) -> Result<DescriptiveStats, String> {
        if data.is_empty() {
            return Err("Cannot analyze empty dataset".to_string());
        }

        // Basic descriptive statistics
        let (mean, variance, skewness, kurtosis) = ShapeStatistics::moments(data);
        let std_dev = variance.sqrt();

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.total_cmp(b));

        let min = Dispersion::min(data);
        let max = Dispersion::max(data);
        let range = Dispersion::range(data);

        let (q1, q3) = Dispersion::quartiles(data);
        let iqr = Dispersion::iqr(data);

        let cv = ShapeStatistics::coefficient_of_variation(mean, std_dev);
        let mad = Dispersion::median_absolute_deviation(data, CentralTendency::median(data));

        // Bootstrap confidence intervals if requested
        let confidence_intervals = if let Some(n_samples) = bootstrap_samples {
            let bootstrap_ci = BootstrapConfidence::confidence_intervals(
                data,
                0.95,
                n_samples,
                rng,
            )?;
            Some(crate::scientific::statistics::types::descriptive::ConfidenceIntervals {
                mean_ci: bootstrap_ci.mean,
                median_ci: bootstrap_ci.median,
                std_dev_ci: bootstrap_ci.std_dev,
            })
        } else {
            None
        };

        // If bootstrap is requested, also compute robust CV
        let robust_cv = if bootstrap_samples.is_some() {
            Some(ShapeStatistics::robust_coefficient_of_variation(data))
        } else {
            None
        };

        Ok(DescriptiveStats {
            count: data.len(),
            mean,
            median: CentralTendency::median(data),
            mode: CentralTendency::modes(data),
            std_dev,
            variance,
            min,
            max,
            range,
            q1,
            q3,
            iqr,
            skewness,
            kurtosis,
            cv,
            mad,
            confidence_intervals,
            robust_cv,
        })
    }

    /// Compute quantile from sorted data using specified method
    pub fn quantile(sorted_data: &[f64], p: f64, method: QuantileMethod) -> f64 {
        Quantiles::quantile(sorted_data, p, method)
    }

    /// Legacy quantile method (Type 7) for backward compatibility
    pub fn quantile_legacy(sorted_data: &[f64], p: f64) -> f64 {
        Quantiles::quantile_legacy(sorted_data, p)
    }
}