//! Bootstrap methods for uncertainty propagation and confidence intervals
//!
//! This module provides comprehensive bootstrap resampling techniques for
//! uncertainty analysis, confidence interval estimation, and statistical inference.

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;
use crate::scientific::statistics::descriptive::moments::StatisticalMoments;
use crate::scientific::statistics::descriptive::quantiles::Quantiles;

/// Progress callback trait for reporting bootstrap progress
pub trait ProgressCallback: Send + Sync {
    fn report_progress(&self, current: usize, total: usize, message: &str);
}

/// Bootstrap convergence assessment
#[derive(Debug, Clone)]
pub struct BootstrapConvergence {
    /// Whether the bootstrap has converged
    pub is_converged: bool,
    /// Stability score (0-1, higher is more stable)
    pub stability_score: f64,
    /// Assessment description
    pub assessment: String,
}

/// Confidence interval results for descriptive statistics
#[derive(Debug, Clone)]
pub struct ConfidenceIntervals {
    /// Mean confidence interval (lower, upper)
    pub mean: (f64, f64),
    /// Median confidence interval (lower, upper)
    pub median: (f64, f64),
    /// Standard deviation confidence interval (lower, upper)
    pub std_dev: (f64, f64),
    /// Skewness confidence interval (lower, upper)
    pub skewness: (f64, f64),
    /// Kurtosis confidence interval (lower, upper)
    pub kurtosis: (f64, f64),
}

/// Bootstrap methods for uncertainty propagation
pub struct BootstrapMethods;

/// Bootstrap resampling algorithms
pub struct BootstrapEngine;

impl BootstrapEngine {
    /// Generic bootstrap runner that takes a sampling function
    fn run_bootstrap<F, S, P>(
        _data: &[f64],
        statistic_fn: &F,
        sampling_fn: S,
        n_bootstrap: usize,
        progress_callback: Option<&P>,
        progress_message: &str,
    ) -> Result<Vec<f64>, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
        S: Fn(&mut Pcg64) -> Vec<f64> + Send + Sync,
        P: ProgressCallback + ?Sized,
    {
        // Generate random seeds for parallel RNGs using thread_rng for better entropy
        let mut seed_rng = rand::rng();
        let seeds: Vec<u64> = (0..n_bootstrap).map(|_| seed_rng.random::<u64>()).collect();

        let bootstrap_statistics: Vec<f64> = seeds.into_par_iter()
            .enumerate()
            .map(|(i, seed)| {
                if let Some(callback) = progress_callback {
                    if i % 1000 == 0 { // Report progress every 1000 iterations
                        callback.report_progress(i, n_bootstrap, progress_message);
                    }
                }
                let mut thread_rng = Pcg64::seed_from_u64(seed);
                let sample = sampling_fn(&mut thread_rng);
                statistic_fn(&sample)
            })
            .collect();

        if let Some(callback) = progress_callback {
            callback.report_progress(n_bootstrap, n_bootstrap, &format!("{} completed", progress_message.to_lowercase()));
        }

        Ok(bootstrap_statistics)
    }

    /// Standard bootstrap sampling function
    fn standard_bootstrap_sample(data: &[f64]) -> impl Fn(&mut Pcg64) -> Vec<f64> + '_ {
        move |rng: &mut Pcg64| {
            BootstrapMethods::sample_with_replacement(data, rng)
        }
    }

    /// Block bootstrap sampling function
    fn block_bootstrap_sample_fn(data: &[f64]) -> impl Fn(&mut Pcg64) -> Vec<f64> + '_ {
        let n = data.len();
        let block_size = ((n as f64).sqrt() as usize).max(1);
        let n_blocks = n.div_ceil(block_size);
        move |rng: &mut Pcg64| {
            Self::block_bootstrap_sample(rng, data, block_size, n_blocks, n)
        }
    }

    /// Bootstrap confidence intervals with progress reporting
    pub fn confidence_intervals_with_progress<F, P>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_bootstrap: usize,
        progress_callback: &P,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
        P: ProgressCallback,
    {
        if data.is_empty() {
            return Err("Cannot compute bootstrap CI for empty dataset".to_string());
        }

        // For large datasets, use block bootstrap for efficiency
        if data.len() > 10000 {
            Self::block_bootstrap_confidence_intervals_with_progress(data, statistic_fn, confidence_level, n_bootstrap, progress_callback)
        } else {
            Self::standard_bootstrap_confidence_intervals_with_progress(data, statistic_fn, confidence_level, n_bootstrap, progress_callback)
        }
    }

    /// Standard bootstrap confidence intervals with progress reporting
    fn standard_bootstrap_confidence_intervals_with_progress<F, P>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_bootstrap: usize,
        progress_callback: &P,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
        P: ProgressCallback,
    {
        let bootstrap_statistics = Self::run_bootstrap(
            data,
            &statistic_fn,
            Self::standard_bootstrap_sample(data),
            n_bootstrap,
            Some(progress_callback),
            "Computing standard bootstrap confidence intervals",
        )?;

        let mut sorted_stats = bootstrap_statistics;
        sorted_stats.sort_by(|a, b| a.total_cmp(b));

        let alpha = 1.0 - confidence_level;
        let lower_idx = (alpha / 2.0 * n_bootstrap as f64) as usize;
        let upper_idx = ((1.0 - alpha / 2.0) * n_bootstrap as f64) as usize;

        Ok((sorted_stats[lower_idx], sorted_stats[upper_idx.min(n_bootstrap - 1)]))
    }

    /// Block bootstrap confidence intervals with progress reporting
    fn block_bootstrap_confidence_intervals_with_progress<F, P>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_bootstrap: usize,
        progress_callback: &P,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
        P: ProgressCallback,
    {
        let bootstrap_statistics = Self::run_bootstrap(
            data,
            &statistic_fn,
            Self::block_bootstrap_sample_fn(data),
            n_bootstrap,
            Some(progress_callback),
            "Computing block bootstrap confidence intervals",
        )?;

        let mut sorted_stats = bootstrap_statistics;
        sorted_stats.sort_by(|a, b| a.total_cmp(b));

        let alpha = 1.0 - confidence_level;
        let lower_idx = (alpha / 2.0 * n_bootstrap as f64) as usize;
        let upper_idx = ((1.0 - alpha / 2.0) * n_bootstrap as f64) as usize;

        Ok((sorted_stats[lower_idx], sorted_stats[upper_idx.min(n_bootstrap - 1)]))
    }

    /// Block bootstrap sampling
    fn block_bootstrap_sample(
        rng: &mut Pcg64,
        data: &[f64],
        block_size: usize,
        n_blocks: usize,
        n: usize,
    ) -> Vec<f64> {
        let mut sample = Vec::with_capacity(n);
        for _ in 0..n_blocks {
            let block_start = rng.random_range(0..n.saturating_sub(block_size - 1));
            let block_end = (block_start + block_size).min(n);
            sample.extend_from_slice(&data[block_start..block_end]);
        }
        sample.truncate(n);
        sample
    }

    /// Compute bootstrap confidence intervals for descriptive statistics
    pub fn confidence_intervals(
        data: &[f64],
        confidence_level: f64,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<ConfidenceIntervals, String> {
        BootstrapMethods::confidence_intervals(data, confidence_level, n_samples, rng)
    }

    /// Compute uncertainty-aware bootstrap confidence intervals
    /// Each data point has associated measurement uncertainty
    pub fn uncertainty_aware_confidence_intervals(
        data: &[f64],
        uncertainties: &[f64],
        confidence_levels: &[f64],
        confidence_level: f64,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<ConfidenceIntervals, String> {
        BootstrapMethods::uncertainty_aware_confidence_intervals(data, uncertainties, confidence_levels, confidence_level, n_samples, rng)
    }

    /// Generic bootstrap confidence intervals using percentile method
    pub fn percentile_confidence_intervals<F>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        BootstrapMethods::percentile_confidence_intervals(data, statistic_fn, confidence_level, n_samples, rng)
    }

    /// BCa (Bias-Corrected and Accelerated) bootstrap confidence intervals
    /// More accurate than percentile bootstrap, especially for skewed statistics
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
        BootstrapMethods::bca_confidence_intervals(data, statistic_fn, confidence_level, n_samples, rng)
    }

    /// Bootstrap standard error estimation
    pub fn bootstrap_standard_error<F>(
        data: &[f64],
        statistic_fn: F,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        BootstrapMethods::bootstrap_standard_error(data, statistic_fn, n_samples, rng)
    }

    /// Bootstrap bias estimation
    pub fn bootstrap_bias<F>(
        data: &[f64],
        statistic_fn: F,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        BootstrapMethods::bootstrap_bias(data, statistic_fn, n_samples, rng)
    }

    /// Block bootstrap for dependent data
    pub fn block_bootstrap_confidence_intervals<F>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_samples: usize,
        block_size: Option<usize>,
        rng: &mut Pcg64,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        BootstrapMethods::block_bootstrap_confidence_intervals(data, statistic_fn, confidence_level, n_samples, block_size, rng)
    }

    /// Bootstrap convergence assessment
    pub fn assess_convergence<F>(
        data: &[f64],
        statistic_fn: F,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<BootstrapConvergence, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        BootstrapMethods::assess_convergence(data, statistic_fn, n_samples, rng)
    }
}

impl BootstrapMethods {
    /// Compute bootstrap confidence intervals for descriptive statistics
    pub fn confidence_intervals(
        data: &[f64],
        confidence_level: f64,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<ConfidenceIntervals, String> {
        if data.is_empty() {
            return Err("Cannot compute confidence intervals for empty data".to_string());
        }

        // Use BCa method for better accuracy
        let mean_ci = Self::bca_confidence_intervals(
            data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64,
            confidence_level,
            n_samples,
            rng,
        )?;

        let median_ci = Self::bca_confidence_intervals(
            data,
            Self::median_statistic,
            confidence_level,
            n_samples,
            rng,
        )?;

        let std_dev_ci = Self::bca_confidence_intervals(
            data,
            Self::std_dev_statistic,
            confidence_level,
            n_samples,
            rng,
        )?;

        let skewness_ci = Self::bca_confidence_intervals(
            data,
            Self::skewness_statistic,
            confidence_level,
            n_samples,
            rng,
        )?;

        let kurtosis_ci = Self::bca_confidence_intervals(
            data,
            Self::kurtosis_statistic,
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

    /// Compute uncertainty-aware bootstrap confidence intervals
    /// Each data point has associated measurement uncertainty
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
            Self::median_statistic,
            confidence_level, n_samples, rng
        )?;

        let std_dev_ci = Self::uncertainty_aware_bootstrap_statistic(
            data, uncertainties, confidence_levels,
            Self::std_dev_statistic,
            confidence_level, n_samples, rng
        )?;

        let skewness_ci = Self::uncertainty_aware_bootstrap_statistic(
            data, uncertainties, confidence_levels,
            Self::skewness_statistic,
            confidence_level, n_samples, rng
        )?;

        let kurtosis_ci = Self::uncertainty_aware_bootstrap_statistic(
            data, uncertainties, confidence_levels,
            Self::kurtosis_statistic,
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

    /// Generic bootstrap confidence intervals using percentile method
    pub fn percentile_confidence_intervals<F>(
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

        let bootstrap_stats = Self::bootstrap_statistics(data, statistic_fn, n_samples, rng)?;
        let mut sorted_stats = bootstrap_stats;
        sorted_stats.sort_by(|a, b| a.total_cmp(b));

        let alpha = 1.0 - confidence_level;
        let lower_idx = (alpha / 2.0 * n_samples as f64) as usize;
        let upper_idx = ((1.0 - alpha / 2.0) * n_samples as f64) as usize;

        let lower = sorted_stats[lower_idx];
        let upper = sorted_stats[upper_idx.min(n_samples - 1)];

        Ok((lower, upper))
    }

    /// BCa (Bias-Corrected and Accelerated) bootstrap confidence intervals
    /// More accurate than percentile bootstrap, especially for skewed statistics
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
            return Err("Cannot compute BCa confidence intervals for empty data".to_string());
        }

        let theta_hat = statistic_fn(data);

        // Generate bootstrap statistics
        let bootstrap_stats = Self::bootstrap_statistics(data, &statistic_fn, n_samples, rng)?;
        let mut boot_stats = bootstrap_stats;
        boot_stats.sort_by(|a, b| a.total_cmp(b));

        // Calculate bias correction (z0)
        let p_below = boot_stats.iter().filter(|&&x| x < theta_hat).count() as f64 / n_samples as f64;
        let z0 = Self::normal_quantile(p_below);

        // Calculate acceleration (a) using jackknife
        let jackknife_stats = Self::jackknife_statistics(data, &statistic_fn);
        let a = Self::calculate_acceleration(&jackknife_stats);

        // Adjust percentiles
        let alpha = 1.0 - confidence_level;
        let z_alpha = Self::normal_quantile(alpha / 2.0);
        let z_alpha2 = Self::normal_quantile(1.0 - alpha / 2.0);

        let p1 = Self::normal_cdf(z0 + (z0 + z_alpha) / (1.0 - a * (z0 + z_alpha)));
        let p2 = Self::normal_cdf(z0 + (z0 + z_alpha2) / (1.0 - a * (z0 + z_alpha2)));

        let lower_idx = (p1 * n_samples as f64) as usize;
        let upper_idx = (p2 * n_samples as f64) as usize;

        let lower = boot_stats[lower_idx.min(n_samples - 1)];
        let upper = boot_stats[upper_idx.min(n_samples - 1)];

        Ok((lower, upper))
    }

    /// Bootstrap standard error estimation
    pub fn bootstrap_standard_error<F>(
        data: &[f64],
        statistic_fn: F,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let bootstrap_stats = Self::bootstrap_statistics(data, statistic_fn, n_samples, rng)?;
        let mean_bootstrap = bootstrap_stats.iter().sum::<f64>() / n_samples as f64;
        let variance = bootstrap_stats.iter()
            .map(|x| (x - mean_bootstrap).powi(2))
            .sum::<f64>() / (n_samples - 1) as f64;

        Ok(variance.sqrt())
    }

    /// Bootstrap bias estimation
    pub fn bootstrap_bias<F>(
        data: &[f64],
        statistic_fn: F,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let original_statistic = statistic_fn(data);
        let bootstrap_stats = Self::bootstrap_statistics(data, statistic_fn, n_samples, rng)?;
        let mean_bootstrap = bootstrap_stats.iter().sum::<f64>() / n_samples as f64;

        Ok(mean_bootstrap - original_statistic)
    }

    /// Block bootstrap for dependent data
    pub fn block_bootstrap_confidence_intervals<F>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_samples: usize,
        block_size: Option<usize>,
        rng: &mut Pcg64,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        if data.is_empty() {
            return Err("Cannot compute block bootstrap CI for empty data".to_string());
        }

        let block_size = block_size.unwrap_or_else(|| ((data.len() as f64).sqrt() as usize).max(1));
        let bootstrap_stats = Self::block_bootstrap_statistics(data, statistic_fn, n_samples, block_size, rng)?;
        let mut sorted_stats = bootstrap_stats;
        sorted_stats.sort_by(|a, b| a.total_cmp(b));

        let alpha = 1.0 - confidence_level;
        let lower_idx = (alpha / 2.0 * n_samples as f64) as usize;
        let upper_idx = ((1.0 - alpha / 2.0) * n_samples as f64) as usize;

        let lower = sorted_stats[lower_idx];
        let upper = sorted_stats[upper_idx.min(n_samples - 1)];

        Ok((lower, upper))
    }

    /// Bootstrap convergence assessment
    pub fn assess_convergence<F>(
        data: &[f64],
        statistic_fn: F,
        n_samples: usize,
        rng: &mut Pcg64,
    ) -> Result<BootstrapConvergence, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        if n_samples < 100 {
            return Ok(BootstrapConvergence {
                is_converged: false,
                stability_score: 0.0,
                assessment: "Insufficient bootstrap samples for convergence assessment".to_string(),
            });
        }

        let bootstrap_stats = Self::bootstrap_statistics(data, statistic_fn, n_samples, rng)?;

        // Split into halves and compare
        let half = n_samples / 2;
        let first_half_mean = bootstrap_stats[..half].iter().sum::<f64>() / half as f64;
        let second_half_mean = bootstrap_stats[half..].iter().sum::<f64>() / half as f64;

        let first_half_std = Self::std_dev_statistic(&bootstrap_stats[..half]);
        let second_half_std = Self::std_dev_statistic(&bootstrap_stats[half..]);

        let mean_diff = (first_half_mean - second_half_mean).abs();
        let avg_std = (first_half_std + second_half_std) / 2.0;

        let stability_score = if avg_std > 0.0 {
            1.0 - (mean_diff / avg_std).min(1.0)
        } else {
            1.0
        };

        let is_converged = stability_score > 0.95 && n_samples >= 1000;

        let assessment = if is_converged {
            "Bootstrap has converged with high stability".to_string()
        } else if stability_score > 0.8 {
            "Bootstrap shows reasonable stability, consider more samples".to_string()
        } else {
            "Bootstrap shows poor stability, increase sample size".to_string()
        };

        Ok(BootstrapConvergence {
            is_converged,
            stability_score,
            assessment,
        })
    }

    // Helper methods

    /// Generate bootstrap statistics in parallel
    fn bootstrap_statistics<F>(
        data: &[f64],
        statistic_fn: F,
        n_samples: usize,
        _rng: &mut Pcg64,
    ) -> Result<Vec<f64>, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        // Generate seeds for parallel RNGs
        let mut seed_rng = rand::rng();
        let seeds: Vec<u64> = (0..n_samples).map(|_| seed_rng.random::<u64>()).collect();

        let statistics: Vec<f64> = seeds.into_par_iter()
            .map(|seed| {
                let mut thread_rng = Pcg64::seed_from_u64(seed);
                let sample = Self::sample_with_replacement(data, &mut thread_rng);
                statistic_fn(&sample)
            })
            .collect();

        Ok(statistics)
    }

    /// Generate block bootstrap statistics
    fn block_bootstrap_statistics<F>(
        data: &[f64],
        statistic_fn: F,
        n_samples: usize,
        block_size: usize,
        _rng: &mut Pcg64,
    ) -> Result<Vec<f64>, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let mut seed_rng = rand::rng();
        let seeds: Vec<u64> = (0..n_samples).map(|_| seed_rng.random::<u64>()).collect();

        let statistics: Vec<f64> = seeds.into_par_iter()
            .map(|seed| {
                let mut thread_rng = Pcg64::seed_from_u64(seed);
                let sample = Self::block_sample(data, block_size, &mut thread_rng);
                statistic_fn(&sample)
            })
            .collect();

        Ok(statistics)
    }

    /// Sample with replacement
    fn sample_with_replacement(data: &[f64], rng: &mut Pcg64) -> Vec<f64> {
        let size = data.len();
        let mut sample = Vec::with_capacity(size);
        for _ in 0..size {
            let idx = rng.random_range(0..data.len());
            sample.push(data[idx]);
        }
        sample
    }

    /// Block sampling for dependent data
    fn block_sample(data: &[f64], block_size: usize, rng: &mut Pcg64) -> Vec<f64> {
        let n = data.len();
        let n_blocks = n.div_ceil(block_size);
        let mut sample = Vec::with_capacity(n);

        for _ in 0..n_blocks {
            let block_idx = rng.random_range(0..n_blocks);
            let start = block_idx * block_size;
            let end = (start + block_size).min(n);
            sample.extend_from_slice(&data[start..end]);
        }

        sample.truncate(n);
        sample
    }

    /// Uncertainty-aware bootstrap statistic
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
        let mut bootstrap_statistics = Vec::with_capacity(n_samples);

        for _ in 0..n_samples {
            let mut sample = Vec::with_capacity(data.len());

            for i in 0..data.len() {
                let value = data[i];
                let uncertainty = uncertainties[i];
                let conf_level = confidence_levels[i];

                // Convert confidence level to standard deviation multiplier
                let z_score = Self::confidence_to_z_score(conf_level);
                let std_dev = uncertainty / z_score;

                // Sample from normal distribution around the measured value
                let normal = Normal::new(value, std_dev)
                    .map_err(|e| format!("Invalid normal distribution parameters: {}", e))?;
                let sampled_value = normal.sample(rng);
                sample.push(sampled_value);
            }

            let stat = statistic_fn(&sample);
            bootstrap_statistics.push(stat);
        }

        bootstrap_statistics.sort_by(|a, b| a.total_cmp(b));
        let alpha = 1.0 - confidence_level;
        let lower_idx = (alpha / 2.0 * n_samples as f64) as usize;
        let upper_idx = ((1.0 - alpha / 2.0) * n_samples as f64) as usize;

        let lower = bootstrap_statistics[lower_idx];
        let upper = bootstrap_statistics[upper_idx.min(n_samples - 1)];

        Ok((lower, upper))
    }

    /// Jackknife statistics for acceleration calculation
    fn jackknife_statistics<F>(data: &[f64], statistic_fn: &F) -> Vec<f64>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let n = data.len();
        let mut jackknife_stats = Vec::with_capacity(n);

        for i in 0..n {
            let jack_sample: Vec<f64> = data.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, &x)| x)
                .collect();
            jackknife_stats.push(statistic_fn(&jack_sample));
        }

        jackknife_stats
    }

    /// Calculate acceleration parameter for BCa bootstrap
    fn calculate_acceleration(jackknife_stats: &[f64]) -> f64 {
        let n = jackknife_stats.len() as f64;
        let jack_mean = jackknife_stats.iter().sum::<f64>() / n;

        let numerator: f64 = jackknife_stats.iter()
            .map(|&x| (jack_mean - x).powi(3))
            .sum();
        let denominator: f64 = jackknife_stats.iter()
            .map(|&x| (jack_mean - x).powi(2))
            .sum::<f64>()
            .powf(1.5);

        if denominator > 0.0 {
            numerator / (6.0 * denominator)
        } else {
            0.0
        }
    }

    // Statistical functions for bootstrap

    fn median_statistic(data: &[f64]) -> f64 {
        Quantiles::median(data)
    }

    fn std_dev_statistic(data: &[f64]) -> f64 {
        data.std_dev()
    }

    fn skewness_statistic(data: &[f64]) -> f64 {
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let std_dev = Self::std_dev_statistic(data);

        if std_dev == 0.0 {
            return 0.0;
        }

        let skewness = data.iter()
            .map(|x| ((x - mean) / std_dev).powi(3))
            .sum::<f64>() / n;

        skewness
    }

    fn kurtosis_statistic(data: &[f64]) -> f64 {
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let std_dev = Self::std_dev_statistic(data);

        if std_dev == 0.0 {
            return 0.0;
        }

        let kurtosis = data.iter()
            .map(|x| ((x - mean) / std_dev).powi(4))
            .sum::<f64>() / n;

        kurtosis - 3.0 // Excess kurtosis
    }

    // Utility functions

    fn normal_quantile(p: f64) -> f64 {
        crate::scientific::statistics::distributions::distribution_functions::normal_quantile(p)
    }

    fn normal_cdf(x: f64) -> f64 {
        crate::scientific::statistics::distributions::distribution_functions::normal_cdf(x, 0.0, 1.0)
    }

    fn confidence_to_z_score(confidence_level: f64) -> f64 {
        if confidence_level >= 0.999 {
            3.291
        } else if confidence_level >= 0.99 {
            2.576
        } else if confidence_level >= 0.95 {
            1.96
        } else if confidence_level >= 0.90 {
            1.645
        } else if confidence_level >= 0.80 {
            1.282
        } else {
            // Approximation for other confidence levels
            (-2.0f64 * (1.0 - confidence_level).ln()).sqrt()
        }
    }


}