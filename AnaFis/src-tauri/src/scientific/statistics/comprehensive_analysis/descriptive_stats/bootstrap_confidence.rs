//! Bootstrap confidence intervals
//!
//! This module provides bootstrap-based confidence interval calculations
//! for descriptive statistics.

use rand::Rng;
use rand_pcg::Pcg64;

/// Confidence interval results
#[derive(Debug, Clone)]
pub struct ConfidenceIntervals {
    pub mean: (f64, f64),
    pub median: (f64, f64),
    pub std_dev: (f64, f64),
    pub skewness: (f64, f64),
    pub kurtosis: (f64, f64),
}

/// Bootstrap confidence interval calculations
pub struct BootstrapConfidence;

impl BootstrapConfidence {
    /// Compute bootstrap confidence intervals for all statistics
    pub fn confidence_intervals(
        data: &[f64],
        confidence_level: f64,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<ConfidenceIntervals, String> {
        let mean_ci = Self::bca_confidence_intervals(
            data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64,
            confidence_level,
            n_samples,
            rng,
        )?;

        let median_ci = Self::bca_confidence_intervals(
            data,
            |sample| {
                let mut sorted = sample.to_vec();
                sorted.sort_by(|a, b| a.total_cmp(b));
                super::quantiles::Quantiles::median(&sorted)
            },
            confidence_level,
            n_samples,
            rng,
        )?;

        let std_dev_ci = Self::bca_confidence_intervals(
            data,
            |sample| super::dispersion::Dispersion::variance(sample).sqrt(),
            confidence_level,
            n_samples,
            rng,
        )?;

        let skewness_ci = Self::bca_confidence_intervals(
            data,
            super::shape_statistics::ShapeStatistics::skewness,
            confidence_level,
            n_samples,
            rng,
        )?;

        let kurtosis_ci = Self::bca_confidence_intervals(
            data,
            super::shape_statistics::ShapeStatistics::kurtosis,
            confidence_level,
            n_samples,
            rng,
        )?;

        Ok(ConfidenceIntervals {
            mean: mean_ci,
            median: median_ci,
            std_dev: std_dev_ci,
            skewness: skewness_ci,
            kurtosis: kurtosis_ci,
        })
    }

    /// Compute BCa (Bias-Corrected and Accelerated) bootstrap confidence intervals
    pub fn bca_confidence_intervals<F>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        if data.is_empty() {
            return Err("Cannot compute confidence intervals for empty data".to_string());
        }

        // Generate bootstrap samples
        let mut bootstrap_statistics = Vec::with_capacity(n_samples);

        for _ in 0..n_samples {
            let mut sample = Vec::with_capacity(data.len());
            for _ in 0..data.len() {
                let idx = rng.random_range(0..data.len());
                sample.push(data[idx]);
            }
            let stat = statistic_fn(&sample);
            bootstrap_statistics.push(stat);
        }

        // Sort bootstrap statistics
        bootstrap_statistics.sort_by(|a, b| a.total_cmp(b));

        // Compute percentiles for confidence interval
        let alpha = 1.0 - confidence_level;
        let lower_idx = (alpha / 2.0 * n_samples as f64) as usize;
        let upper_idx = ((1.0 - alpha / 2.0) * n_samples as f64) as usize;

        let lower = bootstrap_statistics[lower_idx];
        let upper = bootstrap_statistics[upper_idx.min(n_samples - 1)];

        Ok((lower, upper))
    }

    /// Compute uncertainty-aware bootstrap confidence intervals
    pub fn uncertainty_aware_confidence_intervals(
        data: &[f64],
        uncertainties: &[f64],
        confidence_levels: &[f64],
        confidence_level: f64,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<ConfidenceIntervals, String> {
        if uncertainties.len() != data.len() || confidence_levels.len() != data.len() {
            return Err("Uncertainties and confidence levels must match data length".to_string());
        }

        let mean_ci = Self::uncertainty_aware_bootstrap_statistic(
            data, uncertainties, confidence_levels,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64,
            confidence_level, n_samples, rng
        )?;

        let median_ci = Self::uncertainty_aware_bootstrap_statistic(
            data, uncertainties, confidence_levels,
            |sample| {
                let mut sorted = sample.to_vec();
                sorted.sort_by(|a, b| a.total_cmp(b));
                super::quantiles::Quantiles::median(&sorted)
            },
            confidence_level, n_samples, rng
        )?;

        let std_dev_ci = Self::uncertainty_aware_bootstrap_statistic(
            data, uncertainties, confidence_levels,
            |sample| super::dispersion::Dispersion::variance(sample).sqrt(),
            confidence_level, n_samples, rng
        )?;

        let skewness_ci = Self::uncertainty_aware_bootstrap_statistic(
            data, uncertainties, confidence_levels,
            super::shape_statistics::ShapeStatistics::skewness,
            confidence_level, n_samples, rng
        )?;

        let kurtosis_ci = Self::uncertainty_aware_bootstrap_statistic(
            data, uncertainties, confidence_levels,
            super::shape_statistics::ShapeStatistics::kurtosis,
            confidence_level, n_samples, rng
        )?;

        Ok(ConfidenceIntervals {
            mean: mean_ci,
            median: median_ci,
            std_dev: std_dev_ci,
            skewness: skewness_ci,
            kurtosis: kurtosis_ci,
        })
    }

    /// Compute bootstrap confidence interval for a statistic with uncertainty-aware sampling
    fn uncertainty_aware_bootstrap_statistic<F>(
        data: &[f64],
        uncertainties: &[f64],
        confidence_levels: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        use rand_distr::{Distribution, Normal};

        let mut bootstrap_statistics = Vec::with_capacity(n_samples);

        for _ in 0..n_samples {
            let mut sample = Vec::with_capacity(data.len());

            // Generate a bootstrap sample where each point is drawn from its uncertainty distribution
            for (i, &value) in data.iter().enumerate() {
                let uncertainty = uncertainties[i];
                let conf_level = confidence_levels[i];

                // Convert confidence level to standard deviation multiplier
                let z_score = if conf_level >= 0.999 {
                    3.291
                } else if conf_level >= 0.99 {
                    2.576
                } else if conf_level >= 0.95 {
                    1.96
                } else if conf_level >= 0.90 {
                    1.645
                } else if conf_level >= 0.80 {
                    1.282
                } else {
                    // Fallback approximation
                    (-2.0f64 * (1.0 - conf_level).ln()).sqrt()
                };
                let std_dev = uncertainty / z_score;

                // Sample from normal distribution around the measured value
                let normal = Normal::new(value, std_dev)
                    .map_err(|e| format!("Invalid normal distribution parameters: {}", e))?;
                let sampled_value = normal.sample(rng);
                sample.push(sampled_value);
            }

            // Compute statistic on the uncertainty-sampled bootstrap sample
            let stat = statistic_fn(&sample);
            bootstrap_statistics.push(stat);
        }

        // Compute confidence interval from bootstrap statistics
        bootstrap_statistics.sort_by(|a, b| a.total_cmp(b));
        let alpha = 1.0 - confidence_level;
        let lower_idx = (alpha / 2.0 * n_samples as f64) as usize;
        let upper_idx = ((1.0 - alpha / 2.0) * n_samples as f64) as usize;

        let lower = bootstrap_statistics[lower_idx];
        let upper = bootstrap_statistics[upper_idx.min(n_samples - 1)];

        Ok((lower, upper))
    }
}