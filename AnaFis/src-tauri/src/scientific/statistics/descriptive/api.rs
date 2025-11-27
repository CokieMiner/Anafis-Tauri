//! Main API for descriptive statistics
//!
//! This module provides the primary interface for computing descriptive statistics
//! with comprehensive configuration support for uncertainties and method selection.

use crate::scientific::statistics::descriptive::core::moments::StatisticalMoments;
use crate::scientific::statistics::descriptive::types::*;
use crate::scientific::statistics::descriptive::uncertainty::{
    DescriptiveUncertainty, UncertaintyMethod,
};

/// Unified API for descriptive statistics.
///
/// Provides a comprehensive interface to compute descriptive statistics
/// with full configuration support for uncertainties and method selection.
pub struct DescriptiveStatistics;

impl DescriptiveStatistics {
    /// Compute descriptive statistics with comprehensive options.
    ///
    /// This unified API handles all types of descriptive statistics computation:
    /// - Regular statistics (default)
    /// - Weighted statistics (when weights provided)
    /// - Streaming statistics (when streaming=true in config)
    /// - Uncertainty propagation (when uncertainties provided)
    /// - Selective computation (when StatisticsToCompute specified)
    /// - Arbitrary quantiles (when quantiles specified)
    ///
    /// # Arguments
    /// * `data` - The data values
    /// * `config` - Optional configuration for computation type and options
    ///
    /// # Returns
    /// * `DescriptiveResult` - Either regular or streaming statistics based on config
    ///doe
    /// # Example
    ///
    ///
    /// use anafis_lib::scientific::statistics::descriptive::{
    ///     DescriptiveStatistics, DescriptiveConfig, StatisticsToCompute, UncertaintyMethod, DescriptiveResult
    /// };
    ///
    /// let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    ///
    /// // Regular statistics (all computed)
    /// match DescriptiveStatistics::compute(&data, None)? {
    ///     DescriptiveResult::Regular(stats) => println!("Mean: {}", stats.mean.unwrap()),
    ///     _ => unreachable!(),
    /// }
    ///
    /// // Selective computation
    /// let config = DescriptiveConfig::new()
    ///     .with_statistics(StatisticsToCompute {
    ///         moments: true,
    ///         quantiles: false,
    ///         dispersion: false,
    ///         mode: false,
    ///         kde: false,
    ///     });
    /// let result = DescriptiveStatistics::compute(&data, Some(&config))?;
    ///
    /// // Weighted statistics
    /// let weights = vec![1.0, 2.0, 3.0, 2.0, 1.0];
    /// let config = DescriptiveConfig::new().with_weights(&weights);
    /// let result = DescriptiveStatistics::compute(&data, Some(&config))?;
    ///
    /// // Arbitrary quantiles
    /// let quantiles_to_compute = vec![0.1, 0.5, 0.9];
    /// let config = DescriptiveConfig::new().with_quantiles(&quantiles_to_compute);
    /// let result = DescriptiveStatistics::compute(&data, Some(&config))?;
    ///
    /// // With uncertainties for uncertainty propagation
    /// let errors = vec![0.1, 0.1, 0.1, 0.1, 0.1];
    /// let config = DescriptiveConfig::new()
    ///     .with_uncertainties(&errors)
    ///     .with_uncertainty_method(UncertaintyMethod::MonteCarlo);
    /// let result = DescriptiveStatistics::compute(&data, Some(&config))?;
    ///
    /// // Streaming statistics
    /// let config = DescriptiveConfig::new().with_streaming(true);
    /// match DescriptiveStatistics::compute(&data, Some(&config))? {
    ///     DescriptiveResult::Streaming(stats) => println!("Streaming mean: {}", stats.mean),
    ///     _ => unreachable!(),
    /// }
    ///

    pub fn compute(
        data: &[f64],
        config: Option<&DescriptiveConfig>,
    ) -> Result<DescriptiveResult, String> {
        if data.is_empty() {
            return Err("Data cannot be empty".to_string());
        }

        let default_config = DescriptiveConfig::default();
        let config = config.unwrap_or(&default_config);

        // Check if streaming is requested
        if config.streaming {
            let mut streaming =
                crate::scientific::statistics::descriptive::methods::StreamingStatistics::new();

            for &value in data {
                streaming.update(value);
            }

            let streaming_stats = StreamingStats {
                count: streaming.count(),
                mean: streaming.mean(),
                variance: streaming.variance(),
                std_dev: streaming.std_dev(),
                min: streaming.min(),
                max: streaming.max(),
                skewness: streaming.skewness(),
                kurtosis: streaming.kurtosis(),
            };

            return Ok(DescriptiveResult::Streaming(streaming_stats));
        }

        // Regular statistics computation
        use crate::scientific::statistics::descriptive::core::*;
        use crate::scientific::statistics::descriptive::methods::WeightedStatistics;

        let compute_all = config.compute_stats.is_none();
        let stats_to_compute = config
            .compute_stats
            .as_ref()
            .unwrap_or(&StatisticsToCompute {
                moments: true,
                quantiles: true,
                dispersion: true,
                mode: true,
                kde: true,
            });

        // Check if weights are provided - if so, use weighted statistics
        let use_weights = config.weights.is_some();
        let weights = config.weights.unwrap_or(&[]);

        // Moments (mean, variance, std_dev, skewness, kurtosis)
        let moments = if compute_all || stats_to_compute.moments {
            if use_weights {
                // Use weighted statistics
                let mean_val = data.weighted_mean(weights)?;
                let var_val = data.weighted_variance(weights)?;
                let skew_val = data.weighted_skewness(weights)?;
                let kurt_val = data.weighted_kurtosis(weights)?;
                Some(
                    crate::scientific::statistics::descriptive::types::StatisticalMomentsResult {
                        mean: mean_val,
                        variance: var_val,
                        skewness: skew_val,
                        kurtosis: kurt_val,
                    },
                )
            } else {
                Some(data.moments()?)
            }
        } else {
            None
        };

        // Location statistics
        let (median, mode) = if compute_all || stats_to_compute.quantiles || stats_to_compute.mode {
            let median_val = if compute_all || stats_to_compute.quantiles {
                if use_weights {
                    Some(data.weighted_median(weights)?)
                } else {
                    Some(Location::median(data, None).0)
                }
            } else {
                None
            };
            let mode_val = if compute_all || stats_to_compute.mode {
                if use_weights {
                    // Use weighted KDE to find mode
                    use crate::scientific::statistics::descriptive::core::kde::{
                        BandwidthMethod, Kernel, KernelDensityEstimator,
                    };

                    match KernelDensityEstimator::with_weights(
                        data,
                        weights,
                        Kernel::Gaussian,
                        BandwidthMethod::Silverman,
                    ) {
                        Ok(kde) => {
                            // Evaluate density at all data points
                            let density_points: Vec<(f64, f64)> =
                                data.iter().map(|&x| (x, kde.evaluate(x))).collect();

                            // Find the point(s) with maximum density
                            if let Some((_, max_density)) = density_points
                                .iter()
                                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                            {
                                // Collect all points within small tolerance of max density
                                let tolerance = max_density * 0.01; // 1% tolerance
                                let modes: Vec<f64> = density_points
                                    .iter()
                                    .filter(|(_, density)| {
                                        (density - max_density).abs() < tolerance
                                    })
                                    .map(|(x, _)| *x)
                                    .collect();

                                Some(modes)
                            } else {
                                None
                            }
                        }
                        Err(_) => None, // If KDE fails (e.g., zero variance), return None
                    }
                } else {
                    Some(Location::modes(data))
                }
            } else {
                None
            };
            (median_val, mode_val)
        } else {
            (None, None)
        };

        // Dispersion measures
        let dispersion = if compute_all || stats_to_compute.dispersion {
            if use_weights {
                // For weighted dispersion, we need to implement weighted versions
                // For now, fall back to unweighted
                Some(Dispersion::all_measures(data, None)?)
            } else {
                Some(Dispersion::all_measures(data, None)?)
            }
        } else {
            None
        };

        // Standard quantiles (Q1, Q3)
        let (q1, q3) = if compute_all || stats_to_compute.quantiles {
            if use_weights {
                // For weighted quantiles, use weighted quantile method
                let q1_val = data.weighted_quantile(weights, 0.25)?;
                let q3_val = data.weighted_quantile(weights, 0.75)?;
                (Some(q1_val), Some(q3_val))
            } else {
                let sorted = data.to_vec();
                let (q1_val, q3_val) = super::Quantiles::quartiles(&sorted)?;
                (Some(q1_val), Some(q3_val))
            }
        } else {
            (None, None)
        };

        // Arbitrary quantiles
        let arbitrary_quantiles = if let Some(quantiles_to_compute) = config.quantiles {
            let mut quantiles_vec = Vec::new();
            for &p in quantiles_to_compute {
                if use_weights {
                    let val = data.weighted_quantile(weights, p)?;
                    quantiles_vec.push((p, val));
                } else {
                    let mut sorted_data = data.to_vec();
                    sorted_data
                        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let val = super::Quantiles::quantile(&sorted_data, p, config.quantile_method)?;
                    quantiles_vec.push((p, val));
                }
            }
            Some(quantiles_vec)
        } else {
            None
        };

        // Robust measures (trimmed mean and Winsorized variance)
        let (trimmed_mean, winsorized_variance) = if compute_all || stats_to_compute.dispersion {
            let trimmed = Dispersion::trimmed_mean(data, 0.1).ok(); // 10% trimmed mean
            let winsorized = Dispersion::winsorized_variance(data, 0.1).ok(); // 10% Winsorized variance
            (trimmed, winsorized)
        } else {
            (None, None)
        };

        // Confidence intervals if requested
        let confidence_intervals = if config.compute_ci {
            Some(Self::compute_confidence_intervals(data, config)?)
        } else {
            None
        };

        let regular_stats = DescriptiveStats {
            count: data.len(),
            mean: moments.as_ref().map(|m| m.mean),
            median,
            mode,
            std_dev: moments.as_ref().map(|m| m.variance.sqrt()),
            variance: moments.as_ref().map(|m| m.variance),
            min: dispersion.as_ref().map(|d| d.min),
            max: dispersion.as_ref().map(|d| d.max),
            range: dispersion.as_ref().map(|d| d.range),
            q1,
            q3,
            iqr: dispersion.as_ref().map(|d| d.iqr),
            skewness: moments.as_ref().map(|m| m.skewness),
            kurtosis_population: if compute_all || stats_to_compute.moments {
                if use_weights {
                    Some(data.weighted_kurtosis(weights)?)
                } else {
                    Some(data.kurtosis_population()?)
                }
            } else {
                None
            },
            kurtosis_sample: if compute_all || stats_to_compute.moments {
                if use_weights {
                    None // Sample kurtosis not implemented for weighted
                } else {
                    Some(data.kurtosis_sample()?)
                }
            } else {
                None
            },
            cv: dispersion.as_ref().map(|d| d.coefficient_of_variation),
            mad: dispersion.as_ref().map(|d| d.mad),
            confidence_intervals,
            robust_cv: dispersion
                .as_ref()
                .and_then(|d| d.robust_coefficient_of_variation),
            quantiles: arbitrary_quantiles,
            trimmed_mean,
            winsorized_variance,
            kde: if compute_all || stats_to_compute.kde {
                if let Some(kde_cfg) = &config.kde_config {
                    use crate::scientific::statistics::descriptive::core::kde::KernelDensityEstimator;

                    let kde_estimator = KernelDensityEstimator::new(
                        data,
                        kde_cfg.kernel,
                        kde_cfg.bandwidth_method,
                    )?;

                    // Use FFT for large datasets or if explicitly requested (implied by grid_points > 0)
                    // Here we always use FFT as it returns a grid which is what KDEResult expects
                    let (grid, density) = kde_estimator
                        .evaluate_fft(kde_cfg.grid_points)
                        .map_err(|e| e.to_string())?;

                    let confidence_bands = if kde_cfg.compute_uncertainty {
                        // Evaluate uncertainty on the same grid
                        kde_estimator
                            .evaluate_with_uncertainty(
                                &grid,
                                kde_cfg.n_bootstraps,
                                kde_cfg.confidence_level,
                            )
                            .ok()
                    } else {
                        None
                    };

                    Some(KDEResult {
                        grid,
                        density,
                        confidence_bands,
                        bandwidth: kde_estimator.bandwidth(),
                    })
                } else {
                    None
                }
            } else {
                None
            },
        };

        Ok(DescriptiveResult::Regular(regular_stats))
    }

    /// Internal method to compute confidence intervals
    fn compute_confidence_intervals(
        data: &[f64],
        config: &DescriptiveConfig,
    ) -> Result<ConfidenceIntervals, String> {
        // Determine which uncertainty method to use
        let method = match config.uncertainty_method {
            UncertaintyMethod::Auto => {
                // Auto-select based on available information
                if config.covariance.is_some() {
                    UncertaintyMethod::Analytical
                } else if config.uncertainties.is_some() {
                    UncertaintyMethod::MonteCarlo
                } else {
                    UncertaintyMethod::Bootstrap
                }
            }
            other => other,
        };

        match method {
            UncertaintyMethod::Bootstrap => {
                let (mean_ci, _variance_ci, std_dev_ci) =
                    DescriptiveUncertainty::bootstrap_moments(
                        data,
                        config.n_samples,
                        config.confidence_level,
                    )?;

                let (median_val, median_err) = DescriptiveUncertainty::quantile_uncertainty(
                    data,
                    &vec![0.0; data.len()],
                    0.5,
                    config.n_samples,
                )?;

                let z = 1.96; // 95% CI
                let median_ci = (median_val - z * median_err, median_val + z * median_err);

                Ok(ConfidenceIntervals {
                    mean_ci,
                    median_ci,
                    std_dev_ci,
                })
            }

            UncertaintyMethod::MonteCarlo => {
                let errors = config
                    .uncertainties
                    .ok_or("Monte Carlo method requires uncertainties")?;

                let (mean_val, mean_err) =
                    DescriptiveUncertainty::mean_uncertainty(data, errors, config.n_samples)?;
                let (std_val, std_err) =
                    DescriptiveUncertainty::std_dev_uncertainty(data, errors, config.n_samples)?;
                let (median_val, median_err) = DescriptiveUncertainty::quantile_uncertainty(
                    data,
                    errors,
                    0.5,
                    config.n_samples,
                )?;

                let z = 1.96;
                Ok(ConfidenceIntervals {
                    mean_ci: (mean_val - z * mean_err, mean_val + z * median_err),
                    median_ci: (median_val - z * median_err, median_val + z * median_err),
                    std_dev_ci: (std_val - z * std_err, std_val + z * std_err),
                })
            }

            UncertaintyMethod::Analytical => {
                if let Some(cov) = config.covariance {
                    let (mean_err, std_err) =
                        DescriptiveUncertainty::analytical_uncertainty_with_covariance(data, cov)?;

                    let mean_val = data.mean()?;
                    let std_val = data.std_dev()?;

                    let z = 1.96;
                    // Note: Analytical method doesn't support median/quantile uncertainty
                    // because quantiles are non-differentiable functions.
                    // Use Bootstrap or MonteCarlo for quantile uncertainty instead.
                    Ok(ConfidenceIntervals {
                        mean_ci: (mean_val - z * mean_err, mean_val + z * mean_err),
                        median_ci: (f64::NAN, f64::NAN),
                        std_dev_ci: (std_val - z * std_err, std_val + z * std_err),
                    })
                } else if let Some(errors) = config.uncertainties {
                    let (mean_err, std_err) =
                        DescriptiveUncertainty::analytical_uncertainty(data, errors)?;

                    let mean_val = data.mean()?;
                    let std_val = data.std_dev()?;

                    let z = 1.96;
                    // Note: Analytical method uses automatic differentiation which
                    // doesn't work for quantiles (non-differentiable).
                    // Consider using UncertaintyMethod::Bootstrap for quantile CI.
                    Ok(ConfidenceIntervals {
                        mean_ci: (mean_val - z * mean_err, mean_val + z * mean_err),
                        median_ci: (f64::NAN, f64::NAN),
                        std_dev_ci: (std_val - z * std_err, std_val + z * std_err),
                    })
                } else {
                    Err("Analytical method requires uncertainties or covariance matrix".to_string())
                }
            }

            UncertaintyMethod::Auto => unreachable!("Auto should have been resolved"),
        }
    }
}
