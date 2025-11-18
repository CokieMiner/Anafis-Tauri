use crate::scientific::statistics::types::*;
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::StatisticalDistributions;
use statrs::distribution::ContinuousCDF;
use rayon::prelude::*;
use statrs::distribution::ChiSquared;

/// Time series decomposition engine
pub struct TimeSeriesDecompositionEngine;

impl TimeSeriesDecompositionEngine {
    /// Decompose time series using STL (Seasonal and Trend decomposition using Loess)
    /// More robust than simple moving averages
    pub fn decompose_stl(data: &[f64], period: usize) -> Result<TimeSeriesComponents, String> {
        if data.len() < 2 * period {
            return Err("Time series too short for STL decomposition".to_string());
        }

        // STL parameters
        let seasonal_loess_span = 0.3; // Span for seasonal smoothing
        let trend_loess_span = 0.3;    // Span for trend smoothing
        let max_iterations = 5;

        // Initial trend estimate using LOESS
        let mut trend = Self::loess_smooth(data, trend_loess_span)?;

        // Iteratively refine seasonal and trend components
        let mut seasonal = vec![0.0; data.len()];

        for _ in 0..max_iterations {
            // Detrend the series
            let detrended: Vec<f64> = data.iter().zip(trend.iter())
                .map(|(y, t)| y - t)
                .collect();

            // Estimate seasonal component using LOESS on detrended series
            let seasonal_raw = Self::loess_smooth(&detrended, seasonal_loess_span)?;

            // Extract periodic seasonal pattern
            let seasonal_pattern = Self::extract_seasonal_pattern(&seasonal_raw, period)?;

            // Expand seasonal pattern to full length
            seasonal = (0..data.len())
                .map(|i| seasonal_pattern[i % period])
                .collect();

            // Update trend estimate
            let deseasonalized: Vec<f64> = data.iter().zip(seasonal.iter())
                .map(|(y, s)| y - s)
                .collect();

            trend = Self::loess_smooth(&deseasonalized, trend_loess_span)?;
        }

        // Compute residuals
        let residuals = data.iter().zip(trend.iter().zip(seasonal.iter()))
            .map(|(y, (t, s))| y - t - s)
            .collect();

        Ok(TimeSeriesComponents {
            trend,
            seasonal,
            residuals,
            period,
        })
    }

    /// LOESS (Locally Estimated Scatterplot Smoothing) implementation
    fn loess_smooth(data: &[f64], span: f64) -> Result<Vec<f64>, String> {
        let n = data.len();
        let mut smoothed = vec![0.0; n];

        // Window size based on span
        let window_size = (span * n as f64).max(3.0) as usize;

        // For large datasets, use parallel processing
        if n > 1000 {
            smoothed.par_iter_mut().enumerate().for_each(|(i, val)| {
                let start = i.saturating_sub(window_size / 2);
                let end = (i + window_size / 2 + 1).min(n);

                // Weights based on tricubic kernel
                let weights: Vec<f64> = (start..end)
                    .map(|j| {
                        let distance = (j as f64 - i as f64).abs() / (window_size as f64 / 2.0);
                        if distance >= 1.0 {
                            0.0
                        } else {
                            (1.0 - distance.powi(3)).powi(3)
                        }
                    })
                    .collect();

                // Weighted linear regression in local window
                *val = Self::weighted_linear_regression(
                    &(start..end).map(|j| j as f64).collect::<Vec<f64>>(),
                    &data[start..end],
                    &weights
                ).unwrap_or(0.0);
            });
        } else {
            for (i, smoothed_val) in smoothed.iter_mut().enumerate().take(n) {
                let start = i.saturating_sub(window_size / 2);
                let end = (i + window_size / 2 + 1).min(n);

                // Weights based on tricubic kernel
                let weights: Vec<f64> = (start..end)
                    .map(|j| {
                        let distance = (j as f64 - i as f64).abs() / (window_size as f64 / 2.0);
                        if distance >= 1.0 {
                            0.0
                        } else {
                            (1.0 - distance.powi(3)).powi(3)
                        }
                    })
                    .collect();

                // Weighted linear regression in local window
                *smoothed_val = Self::weighted_linear_regression(
                    &(start..end).map(|j| j as f64).collect::<Vec<f64>>(),
                    &data[start..end],
                    &weights
                )?;
            }
        }

        Ok(smoothed)
    }

    /// Weighted linear regression for LOESS
    fn weighted_linear_regression(x: &[f64], y: &[f64], weights: &[f64]) -> Result<f64, String> {
        let n = x.len();
        if n < 2 {
            return Ok(y[0]); // Not enough points for regression
        }

        let sum_w = weights.iter().sum::<f64>();
        let sum_wx = x.iter().zip(weights.iter()).map(|(xi, wi)| xi * wi).sum::<f64>();
        let sum_wy = y.iter().zip(weights.iter()).map(|(yi, wi)| yi * wi).sum::<f64>();
        let sum_wxx = x.iter().zip(weights.iter()).map(|(xi, wi)| xi * xi * wi).sum::<f64>();
        let sum_wxy = x.iter().zip(y.iter().zip(weights.iter())).map(|(xi, (yi, wi))| xi * yi * wi).sum::<f64>();

        // For prediction at x = n/2 (middle of window), but since we're doing local smoothing,
        // we'll predict at the center of the x values
        let x_center = x.iter().sum::<f64>() / n as f64;

        // Weighted least squares: y = a + b*x
        let denominator = sum_w * sum_wxx - sum_wx * sum_wx;
        if denominator.abs() < 1e-10 {
            return Ok(sum_wy / sum_w); // No slope, return weighted mean
        }

        let b = (sum_w * sum_wxy - sum_wx * sum_wy) / denominator;
        let a = (sum_wy - b * sum_wx) / sum_w;

        Ok(a + b * x_center)
    }

    /// Extract periodic seasonal pattern from smoothed detrended series
    fn extract_seasonal_pattern(smoothed: &[f64], period: usize) -> Result<Vec<f64>, String> {
        let n_periods = smoothed.len() / period;
        let mut seasonal = vec![0.0; period];

        // Average each season across all periods
        for (p, seasonal_val) in seasonal.iter_mut().enumerate().take(period) {
            let mut sum = 0.0;
            let mut count = 0;

            for i in 0..n_periods {
                let idx = i * period + p;
                if idx < smoothed.len() {
                    sum += smoothed[idx];
                    count += 1;
                }
            }

            *seasonal_val = if count > 0 { sum / count as f64 } else { 0.0 };
        }

        // Center the seasonal component (sum to zero)
        let seasonal_mean = seasonal.iter().sum::<f64>() / period as f64;
        for s in seasonal.iter_mut() {
            *s -= seasonal_mean;
        }

        Ok(seasonal)
    }

    /// Decompose time series into trend, seasonal, and residual components
    pub fn decompose_additive(data: &[f64], period: usize) -> Result<TimeSeriesComponents, String> {
        if data.len() < 2 * period {
            return Err("Time series too short for decomposition".to_string());
        }

        // Estimate trend using moving average
        let trend = Self::moving_average_trend(data, period)?;

        // Estimate seasonal component
        let detrended = data.iter().zip(trend.iter())
            .map(|(y, t)| y - t)
            .collect::<Vec<f64>>();

        let seasonal_pattern = Self::estimate_seasonal(&detrended, period)?;

        // Expand seasonal component to match data length
        let seasonal = (0..data.len())
            .map(|i| seasonal_pattern[i % period])
            .collect::<Vec<f64>>();

        // Compute residuals
        let residuals = data.iter().zip(trend.iter().zip(seasonal.iter()))
            .map(|(y, (t, s))| y - t - s)
            .collect::<Vec<f64>>();

        Ok(TimeSeriesComponents {
            trend,
            seasonal,
            residuals,
            period,
        })
    }

    fn moving_average_trend(data: &[f64], period: usize) -> Result<Vec<f64>, String> {
        let n = data.len();
        let mut trend = vec![0.0; n];

        // For large datasets, use parallel processing
        if n > 1000 {
            trend.par_iter_mut().enumerate().for_each(|(i, val)| {
                let start = i.saturating_sub(period / 2);
                let end = (i + period / 2 + 1).min(n);

                let sum: f64 = data[start..end].iter().sum();
                let count = end - start;
                *val = sum / count as f64;
            });
        } else {
            for (i, trend_val) in trend.iter_mut().enumerate().take(n) {
                let start = i.saturating_sub(period / 2);
                let end = (i + period / 2 + 1).min(n);

                let sum: f64 = data[start..end].iter().sum();
                let count = end - start;
                *trend_val = sum / count as f64;
            }
        }

        Ok(trend)
    }

    /// Estimate seasonal component
    fn estimate_seasonal(detrended: &[f64], period: usize) -> Result<Vec<f64>, String> {
        let n_periods = detrended.len() / period;
        let mut seasonal = vec![0.0; period];

        // Average each season
        for (p, seasonal_val) in seasonal.iter_mut().enumerate().take(period) {
            let mut sum = 0.0;
            let mut count = 0;

            for i in 0..n_periods {
                let idx = i * period + p;
                if idx < detrended.len() {
                    sum += detrended[idx];
                    count += 1;
                }
            }

            *seasonal_val = sum / count as f64;
        }

        // Center the seasonal component
        let seasonal_mean = seasonal.iter().sum::<f64>() / period as f64;
        for s in seasonal.iter_mut() {
            *s -= seasonal_mean;
        }

        Ok(seasonal)
    }

    /// Test for trend presence
    pub fn trend_test(data: &[f64]) -> Result<TrendAnalysis, String> {
        if data.len() < 3 {
            return Err("Need at least 3 observations for trend analysis".to_string());
        }

        // Simple linear regression for trend
        let n = data.len() as f64;
        let x_values: Vec<f64> = (0..data.len()).map(|i| i as f64).collect();

        let sum_x = x_values.iter().sum::<f64>();
        let sum_y = data.iter().sum::<f64>();
        let sum_xy = x_values.iter().zip(data.iter()).map(|(x, y)| x * y).sum::<f64>();
        let sum_x2 = x_values.iter().map(|x| x * x).sum::<f64>();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // R-squared
        let y_mean = sum_y / n;
        let ss_tot = data.iter().map(|y| (y - y_mean).powi(2)).sum::<f64>();
        let ss_res = data.iter().zip(x_values.iter())
            .map(|(y, x)| (y - (slope * x + intercept)).powi(2))
            .sum::<f64>();
        let r_squared = 1.0 - ss_res / ss_tot;

        // t-test for slope significance
        let se_slope = (ss_res / (n - 2.0)).sqrt() / (sum_x2 - sum_x * sum_x / n).sqrt();
        let t_statistic = slope / se_slope;
        let p_value = 2.0 * (1.0 - StatisticalDistributions::t_cdf(t_statistic.abs(), n - 2.0));

        Ok(TrendAnalysis {
            trend_present: p_value < 0.05,
            slope,
            intercept,
            r_squared,
            significance: p_value,
        })
    }

    /// Autocorrelation function
    pub fn autocorrelation(data: &[f64], max_lag: usize) -> Result<Vec<f64>, String> {
        if data.len() < max_lag + 1 {
            return Err("Data too short for requested lag".to_string());
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>();

        let autocorr: Vec<f64> = (1..=max_lag).into_par_iter().map(|lag| {
            let mut covariance = 0.0;
            for i in lag..data.len() {
                covariance += (data[i] - mean) * (data[i - lag] - mean);
            }
            covariance / variance
        }).collect();

        Ok(autocorr)
    }

    /// Ljung-Box test for overall randomness / lack of autocorrelation
    /// Returns (Q_statistic, p_value)
    pub fn ljung_box_test(data: &[f64], lags: usize) -> Result<(f64, f64), String> {
        let n = data.len() as f64;
        if data.len() < 2 || lags == 0 {
            return Err("Insufficient data for Ljung-Box test".to_string());
        }

        let m = lags.min(data.len() - 1);
        let autocorr = Self::autocorrelation(data, m)?; // len = m

        // Compute Q statistic
        let mut q = 0.0f64;
        for (k, &rho_k) in autocorr.iter().enumerate() {
            let k1 = (k + 1) as f64;
            q += rho_k * rho_k / (n - k1);
        }
        q *= n * (n + 2.0);

        // Degrees of freedom = m (no AR parameters assumed)
        let df = m as f64;
        let chi2 = ChiSquared::new(df).map_err(|e| e.to_string())?;
        let p_value = 1.0 - chi2.cdf(q);
        Ok((q, p_value))
    }
}