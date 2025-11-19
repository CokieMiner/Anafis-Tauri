use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::*;
use statrs::distribution::ContinuousCDF;
use rand_pcg::Pcg64;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use crate::scientific::statistics::comprehensive_analysis::traits::ProgressCallback;

#[derive(Debug, Clone)]
pub struct BootstrapConvergence {
    pub is_converged: bool,
    pub stability_score: f64,
    pub assessment: String,
}

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
            RandomSampling::sample_with_replacement(rng, data, data.len()).unwrap()
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
            "Computing bootstrap samples...",
        )?;

        let mut bootstrap_statistics = bootstrap_statistics;
        bootstrap_statistics.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        let lower_percentile = (1.0 - confidence_level) / 2.0;
        let upper_percentile = 1.0 - lower_percentile;

        let lower_idx = (lower_percentile * n_bootstrap as f64) as usize;
        let upper_idx = (upper_percentile * n_bootstrap as f64) as usize;

        let lower_bound = bootstrap_statistics[lower_idx.min(n_bootstrap - 1)];
        let upper_bound = bootstrap_statistics[upper_idx.min(n_bootstrap - 1)];

        Ok((lower_bound, upper_bound))
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
            "Computing block bootstrap samples...",
        )?;

        let mut bootstrap_statistics = bootstrap_statistics;
        bootstrap_statistics.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        let lower_percentile = (1.0 - confidence_level) / 2.0;
        let upper_percentile = 1.0 - lower_percentile;

        let lower_idx = (lower_percentile * n_bootstrap as f64) as usize;
        let upper_idx = (upper_percentile * n_bootstrap as f64) as usize;

        let lower_bound = bootstrap_statistics[lower_idx.min(n_bootstrap - 1)];
        let upper_bound = bootstrap_statistics[upper_idx.min(n_bootstrap - 1)];

        Ok((lower_bound, upper_bound))
    }

    /// Generate a block bootstrap sample
    fn block_bootstrap_sample(rng: &mut Pcg64, data: &[f64], block_size: usize, n_blocks: usize, n: usize) -> Vec<f64> {
        let mut sample = Vec::with_capacity(n);
        
        for _ in 0..n_blocks {
            // Sample a block index with replacement
            let block_idx = rng.random_range(0..n_blocks);
            let start = block_idx * block_size;
            let end = (start + block_size).min(n);
            
            // Add the block to the sample
            sample.extend_from_slice(&data[start..end]);
        }
        
        // Trim to original size if necessary
        sample.truncate(n);
        sample
    }

    /// Bootstrap standard error estimation
    pub fn bootstrap_standard_error<F>(
        data: &[f64],
        statistic_fn: F,
        n_bootstrap: usize,
        _rng: &mut Pcg64,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        if data.len() > 10000 {
            Self::block_bootstrap_standard_error(data, statistic_fn, n_bootstrap)
        } else {
            Self::standard_bootstrap_standard_error(data, statistic_fn, n_bootstrap)
        }
    }

    /// Standard bootstrap standard error
    fn standard_bootstrap_standard_error<F>(
        data: &[f64],
        statistic_fn: F,
        n_bootstrap: usize,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let bootstrap_statistics = Self::run_bootstrap(
            data,
            &statistic_fn,
            Self::standard_bootstrap_sample(data),
            n_bootstrap,
            None::<&dyn crate::scientific::statistics::comprehensive_analysis::traits::ProgressCallback>,
            "",
        )?;

        let mean_bootstrap = bootstrap_statistics.iter().sum::<f64>() / n_bootstrap as f64;
        let variance_bootstrap = bootstrap_statistics.iter()
            .map(|x| (x - mean_bootstrap).powi(2))
            .sum::<f64>() / (n_bootstrap - 1) as f64;

        Ok(variance_bootstrap.sqrt())
    }

    /// Block bootstrap standard error
    fn block_bootstrap_standard_error<F>(
        data: &[f64],
        statistic_fn: F,
        n_bootstrap: usize,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let bootstrap_statistics = Self::run_bootstrap(
            data,
            &statistic_fn,
            Self::block_bootstrap_sample_fn(data),
            n_bootstrap,
            None::<&dyn crate::scientific::statistics::comprehensive_analysis::traits::ProgressCallback>,
            "",
        )?;

        let mean_bootstrap = bootstrap_statistics.iter().sum::<f64>() / n_bootstrap as f64;
        let variance_bootstrap = bootstrap_statistics.iter()
            .map(|x| (x - mean_bootstrap).powi(2))
            .sum::<f64>() / (n_bootstrap - 1) as f64;

        Ok(variance_bootstrap.sqrt())
    }

    /// Bootstrap bias estimation
    pub fn bootstrap_bias<F>(
        data: &[f64],
        statistic_fn: F,
        n_bootstrap: usize,
        _rng: &mut Pcg64,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        if data.len() > 10000 {
            Self::block_bootstrap_bias(data, statistic_fn, n_bootstrap)
        } else {
            Self::standard_bootstrap_bias(data, statistic_fn, n_bootstrap)
        }
    }

    /// Standard bootstrap bias
    fn standard_bootstrap_bias<F>(
        data: &[f64],
        statistic_fn: F,
        n_bootstrap: usize,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let original_statistic = statistic_fn(data);

        let bootstrap_statistics = Self::run_bootstrap(
            data,
            &statistic_fn,
            Self::standard_bootstrap_sample(data),
            n_bootstrap,
            None::<&dyn crate::scientific::statistics::comprehensive_analysis::traits::ProgressCallback>,
            "",
        )?;

        let mean_bootstrap = bootstrap_statistics.iter().sum::<f64>() / n_bootstrap as f64;
        let bias = mean_bootstrap - original_statistic;

        Ok(bias)
    }

    /// Block bootstrap bias
    fn block_bootstrap_bias<F>(
        data: &[f64],
        statistic_fn: F,
        n_bootstrap: usize,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let original_statistic = statistic_fn(data);

        let bootstrap_statistics = Self::run_bootstrap(
            data,
            &statistic_fn,
            Self::block_bootstrap_sample_fn(data),
            n_bootstrap,
            None::<&dyn crate::scientific::statistics::comprehensive_analysis::traits::ProgressCallback>,
            "",
        )?;

        let mean_bootstrap = bootstrap_statistics.iter().sum::<f64>() / n_bootstrap as f64;
        let bias = mean_bootstrap - original_statistic;

        Ok(bias)
    }

    /// BCa (Bias-Corrected and Accelerated) bootstrap confidence intervals
    /// More accurate than percentile bootstrap, especially for skewed statistics
    pub fn bca_confidence_intervals<F>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_bootstrap: usize,
        _rng: &mut Pcg64,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        if data.is_empty() {
            return Err("Cannot compute BCa bootstrap CI for empty dataset".to_string());
        }

        if data.len() > 10000 {
            Self::block_bootstrap_bca_confidence_intervals(data, statistic_fn, confidence_level, n_bootstrap)
        } else {
            Self::standard_bootstrap_bca_confidence_intervals(data, statistic_fn, confidence_level, n_bootstrap)
        }
    }

    /// Standard BCa bootstrap confidence intervals
    fn standard_bootstrap_bca_confidence_intervals<F>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_bootstrap: usize,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let n = data.len();
        let theta_hat = statistic_fn(data);

        // 1. Generate bootstrap samples and compute bootstrap statistics in parallel
        let boot_stats = Self::run_bootstrap(
            data,
            &statistic_fn,
            Self::standard_bootstrap_sample(data),
            n_bootstrap,
            None::<&dyn crate::scientific::statistics::comprehensive_analysis::traits::ProgressCallback>,
            "",
        )?;

        let mut boot_stats = boot_stats;
        boot_stats.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        // 2. Calculate bias correction (z0)
        let p_below = boot_stats.iter().filter(|&&x| x < theta_hat).count() as f64 / n_bootstrap as f64;
        let z0 = Self::normal_quantile(p_below, 0.0, 1.0);

        // 3. Calculate acceleration (a) using jackknife
        let mut jackknife_stats = Vec::with_capacity(n);
        for i in 0..n {
            let jack_sample: Vec<f64> = data.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, &x)| x)
                .collect();
            jackknife_stats.push(statistic_fn(&jack_sample));
        }

        let jack_mean = jackknife_stats.iter().sum::<f64>() / n as f64;
        let numerator: f64 = jackknife_stats.iter()
            .map(|&x| (jack_mean - x).powi(3))
            .sum();
        let denominator: f64 = jackknife_stats.iter()
            .map(|&x| (jack_mean - x).powi(2))
            .sum::<f64>()
            .powf(1.5);

        let a = if denominator > 0.0 {
            numerator / (6.0 * denominator)
        } else {
            0.0
        };

        // 4. Adjust percentiles
        let alpha = 1.0 - confidence_level;
        let z_alpha = Self::normal_quantile(alpha / 2.0, 0.0, 1.0);
        let z_alpha2 = Self::normal_quantile(1.0 - alpha / 2.0, 0.0, 1.0);

        let p1 = Self::normal_cdf(z0 + (z0 + z_alpha) / (1.0 - a * (z0 + z_alpha)), 0.0, 1.0);
        let p2 = Self::normal_cdf(z0 + (z0 + z_alpha2) / (1.0 - a * (z0 + z_alpha2)), 0.0, 1.0);

        let lower_idx = (p1 * n_bootstrap as f64) as usize;
        let upper_idx = (p2 * n_bootstrap as f64) as usize;

        let lower_bound = boot_stats[lower_idx.min(n_bootstrap - 1)];
        let upper_bound = boot_stats[upper_idx.min(n_bootstrap - 1)];

        Ok((lower_bound, upper_bound))
    }

    /// Block bootstrap BCa confidence intervals
    fn block_bootstrap_bca_confidence_intervals<F>(
        data: &[f64],
        statistic_fn: F,
        confidence_level: f64,
        n_bootstrap: usize,
    ) -> Result<(f64, f64), String>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
    {
        let n = data.len();
        let theta_hat = statistic_fn(data);

        // 1. Generate bootstrap samples and compute bootstrap statistics in parallel
        let boot_stats = Self::run_bootstrap(
            data,
            &statistic_fn,
            Self::block_bootstrap_sample_fn(data),
            n_bootstrap,
            None::<&dyn crate::scientific::statistics::comprehensive_analysis::traits::ProgressCallback>,
            "",
        )?;

        let mut boot_stats = boot_stats;
        boot_stats.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        // 2. Calculate bias correction (z0)
        let p_below = boot_stats.iter().filter(|&&x| x < theta_hat).count() as f64 / n_bootstrap as f64;
        let z0 = Self::normal_quantile(p_below, 0.0, 1.0);

        // 3. Calculate acceleration (a) using jackknife (simplified for large n)
        // For large datasets, approximate jackknife with fewer samples
        let jackknife_samples = if n > 1000 { 100 } else { n };
        let step = n / jackknife_samples;
        let mut jackknife_stats = Vec::with_capacity(jackknife_samples);
        for i in (0..n).step_by(step).take(jackknife_samples) {
            let jack_sample: Vec<f64> = data.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, &x)| x)
                .collect();
            jackknife_stats.push(statistic_fn(&jack_sample));
        }

        let jack_mean = jackknife_stats.iter().sum::<f64>() / jackknife_samples as f64;
        let numerator: f64 = jackknife_stats.iter()
            .map(|&x| (jack_mean - x).powi(3))
            .sum();
        let denominator: f64 = jackknife_stats.iter()
            .map(|&x| (jack_mean - x).powi(2))
            .sum::<f64>()
            .powf(1.5);

        let a = if denominator > 0.0 {
            numerator / (6.0 * denominator)
        } else {
            0.0
        };

        // 4. Adjust percentiles
        let alpha = 1.0 - confidence_level;
        let z_alpha = Self::normal_quantile(alpha / 2.0, 0.0, 1.0);
        let z_alpha2 = Self::normal_quantile(1.0 - alpha / 2.0, 0.0, 1.0);

        let p1 = Self::normal_cdf(z0 + (z0 + z_alpha) / (1.0 - a * (z0 + z_alpha)), 0.0, 1.0);
        let p2 = Self::normal_cdf(z0 + (z0 + z_alpha2) / (1.0 - a * (z0 + z_alpha2)), 0.0, 1.0);

        let lower_idx = (p1 * n_bootstrap as f64) as usize;
        let upper_idx = (p2 * n_bootstrap as f64) as usize;

        let lower_bound = boot_stats[lower_idx.min(n_bootstrap - 1)];
        let upper_bound = boot_stats[upper_idx.min(n_bootstrap - 1)];

        Ok((lower_bound, upper_bound))
    }
    
    /// Normal quantile function (inverse CDF)
    fn normal_quantile(p: f64, mean: f64, std_dev: f64) -> f64 {
        let normal = statrs::distribution::Normal::new(mean, std_dev).expect("Invalid normal parameters");
        normal.inverse_cdf(p)
    }
    
    /// Normal cumulative distribution function
    fn normal_cdf(x: f64, mean: f64, std_dev: f64) -> f64 {
        let normal = statrs::distribution::Normal::new(mean, std_dev).expect("Invalid normal parameters");
        normal.cdf(x)
    }
}
