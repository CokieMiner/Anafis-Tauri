//! Stationarity Testing Module
//!
//! This module provides tests for time series stationarity:
//! - Augmented Dickey-Fuller (ADF) test
//! - Kwiatkowski-Phillips-Schmidt-Shin (KPSS) test
//! - Phillips-Perron test
//! - Rolling statistics analysis

pub mod types;

use types::*;
use crate::scientific::statistics::primitives::{LinearAlgebra, LinearRegression};
use ndarray::{Array1, Array2};
use rayon::prelude::*;


/// Stationarity testing engine
pub struct StationarityEngine;

impl StationarityEngine {
    /// Augmented Dickey-Fuller test for unit root
    /// Returns (test_statistic, p_value, critical_values, lags_used)
    pub fn adf_test(
        series: &[f64],
        regression_type: AdfRegressionType,
        max_lags: Option<usize>,
        autolag: bool,
    ) -> Result<AdfTestResult, String> {
        if series.len() < 10 {
            return Err("Series too short for ADF test (need at least 10 observations)".to_string());
        }

        let n = series.len();
        let max_lags = max_lags.unwrap_or((n as f64).powf(0.25).ceil() as usize);

        // Determine optimal lag length using AIC if autolag is true
        let lags = if autolag {
            Self::select_adf_lags(series, max_lags, regression_type)?
        } else {
            max_lags.min(n / 2 - 1)
        };

        // Create differenced series and lagged differences
        let mut y_diff = Vec::with_capacity(n - 1);
        let mut y_lagged = Vec::with_capacity(n - 1 - lags);

        for i in 1..n {
            y_diff.push(series[i] - series[i - 1]);
        }

        for &value in series.iter().skip(lags).take(n - 1 - lags) {
            y_lagged.push(value);
        }

        // Build design matrix based on regression type
        let (x_matrix, n_obs) = Self::build_adf_design_matrix(
            series,
            &y_diff,
            lags,
            regression_type,
        )?;

        // Perform OLS regression
        let coefficients = LinearRegression::ols_fit_vec(&x_matrix, &y_lagged)?;

        // Test statistic is the t-statistic for the coefficient of y_{t-1}
        let test_statistic = coefficients[0] / Self::standard_error(&x_matrix, &y_lagged, &coefficients)?;

        // Get critical values (approximate for large samples)
        let critical_values = Self::adf_critical_values(n_obs);

        // Calculate p-value using asymptotic distribution
        let p_value = Self::adf_p_value(test_statistic, regression_type);

        Ok(AdfTestResult {
            test_statistic,
            p_value,
            critical_values,
            lags_used: lags,
            n_obs,
            regression_type,
        })
    }

    /// KPSS test for stationarity around a deterministic trend
    /// Returns (test_statistic, p_value, critical_values, lags_used)
    pub fn kpss_test(
        series: &[f64],
        regression_type: KpssRegressionType,
        lags: Option<usize>,
    ) -> Result<KpssTestResult, String> {
        if series.len() < 5 {
            return Err("Series too short for KPSS test (need at least 5 observations)".to_string());
        }

        let n = series.len();
        let lags = lags.unwrap_or((n as f64).powf(0.25).ceil() as usize);

        // Detrend the series based on regression type
        let (residuals, _trend_coefficients) = Self::detrend_series(series, regression_type)?;

        // Compute cumulative sum of residuals
        let mut s = vec![0.0; n];
        for i in 1..n {
            s[i] = s[i - 1] + residuals[i];
        }

        // Compute long-run variance using Newey-West estimator
        let long_run_variance = Self::newey_west_variance(&residuals, lags)?;

        if long_run_variance <= 0.0 {
            return Err("Invalid long-run variance for KPSS test".to_string());
        }

        // Compute test statistic
        let test_statistic = s[n - 1].powi(2) / (n as f64 * long_run_variance);

        // Critical values for KPSS test
        let critical_values = Self::kpss_critical_values();

        // P-value calculation (approximate)
        let p_value = Self::kpss_p_value(test_statistic);

        Ok(KpssTestResult {
            test_statistic,
            p_value,
            critical_values,
            lags_used: lags,
            regression_type,
            long_run_variance,
        })
    }

    /// Phillips-Perron test (similar to ADF but with nonparametric correction)
    pub fn phillips_perron_test(
        series: &[f64],
        regression_type: AdfRegressionType,
    ) -> Result<PhillipsPerronResult, String> {
        if series.len() < 10 {
            return Err("Series too short for PP test".to_string());
        }

        // First perform regular ADF test
        let adf_result = Self::adf_test(series, regression_type, None, true)?;

        // Compute Newey-West long-run variance of residuals
        let residuals = Self::compute_adf_residuals(series, adf_result.lags_used, regression_type)?;
        let long_run_variance = Self::newey_west_variance(&residuals, adf_result.lags_used)?;

        // Adjust test statistic
        let adjustment = long_run_variance / Self::residual_variance(&residuals)?;
        let test_statistic_pp = adf_result.test_statistic * adjustment.sqrt();

        // P-value using the same distribution as ADF
        let p_value = Self::adf_p_value(test_statistic_pp, regression_type);

        Ok(PhillipsPerronResult {
            test_statistic: test_statistic_pp,
            p_value,
            critical_values: adf_result.critical_values,
            lags_used: adf_result.lags_used,
            regression_type,
            long_run_variance,
        })
    }

    /// Rolling statistics analysis for stationarity assessment
    pub fn rolling_statistics(
        series: &[f64],
        window_size: usize,
        step: usize,
    ) -> Result<RollingStatsResult, String> {
        if window_size >= series.len() {
            return Err("Window size must be smaller than series length".to_string());
        }

        // Generate window indices
        let window_indices: Vec<usize> = (0..=(series.len().saturating_sub(window_size)))
            .step_by(step)
            .collect();

        // Compute statistics for each window in parallel
        let window_stats: Vec<(f64, f64, f64, usize)> = window_indices.into_par_iter()
            .map(|start_idx| {
                let window = &series[start_idx..start_idx + window_size];

                // Mean
                let mean = window.iter().sum::<f64>() / window_size as f64;

                // Variance
                let variance = window.iter()
                    .map(|x| (x - mean).powi(2))
                    .sum::<f64>() / (window_size - 1) as f64;

                // Autocorrelation at lag 1
                let autocorr = if window_size > 1 {
                    let mut numerator = 0.0;
                    for i in 1..window_size {
                        numerator += (window[i] - mean) * (window[i - 1] - mean);
                    }
                    if variance > 0.0 {
                        numerator / ((window_size - 1) as f64 * variance)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                (mean, variance, autocorr, start_idx)
            })
            .collect();

        // Sort by position and extract results
        let mut sorted_stats: Vec<_> = window_stats.into_iter().collect();
        sorted_stats.sort_by_key(|&(_, _, _, pos)| pos);

        // Extract results manually
        let mut means = Vec::with_capacity(sorted_stats.len());
        let mut variances = Vec::with_capacity(sorted_stats.len());
        let mut autocorrelations = Vec::with_capacity(sorted_stats.len());
        let mut positions = Vec::with_capacity(sorted_stats.len());

        for (mean, var, autocorr, pos) in sorted_stats {
            means.push(mean);
            variances.push(var);
            autocorrelations.push(autocorr);
            positions.push(pos);
        }

        Ok(RollingStatsResult {
            means,
            variances,
            autocorrelations,
            positions,
            window_size,
        })
    }

    // Helper methods

    fn select_adf_lags(
        series: &[f64],
        max_lags: usize,
        regression_type: AdfRegressionType,
    ) -> Result<usize, String> {
        let n = series.len();
        let mut best_lags = 0;
        let mut best_aic = f64::INFINITY;

        for lags in 0..=max_lags.min(n / 2 - 1) {
            if let Ok((x_matrix, _)) = Self::build_adf_design_matrix(series, &[], lags, regression_type) {
                let y_lagged: Vec<f64> = series[lags..n - 1].to_vec();

                if let Ok(coeffs) = LinearRegression::ols_fit_vec(&x_matrix, &y_lagged) {
                    if let Ok(residuals) = Self::compute_residuals(&x_matrix, &y_lagged, &coeffs) {
                        let aic = Self::aic_criterion(&residuals, x_matrix.len());
                        if aic < best_aic {
                            best_aic = aic;
                            best_lags = lags;
                        }
                    }
                }
            }
        }

        Ok(best_lags)
    }

    fn build_adf_design_matrix(
        series: &[f64],
        y_diff: &[f64],
        lags: usize,
        regression_type: AdfRegressionType,
    ) -> Result<(Vec<Vec<f64>>, usize), String> {
        let n = series.len();
        let n_obs = n - 1 - lags;

        let mut x_matrix = Vec::with_capacity(n_obs);

        for i in lags..(n - 1) {
            let mut row = Vec::new();

            // y_{t-1}
            row.push(series[i]);

            // Lagged differences
            for lag in 1..=lags {
                row.push(y_diff[i - lag]);
            }

            // Deterministic terms
            match regression_type {
                AdfRegressionType::Constant => {
                    row.push(1.0);
                }
                AdfRegressionType::ConstantTrend => {
                    row.push(1.0);
                    row.push((i + 1) as f64);
                }
                AdfRegressionType::NoConstant => {
                    // No additional terms
                }
            }

            x_matrix.push(row);
        }

        Ok((x_matrix, n_obs))
    }

    fn standard_error(x: &[Vec<f64>], y: &[f64], coeffs: &[f64]) -> Result<f64, String> {
        let residuals = Self::compute_residuals(x, y, coeffs)?;
        let variance = Self::residual_variance(&residuals)?;
        let n = x.len() as f64;
        let p = x[0].len() as f64;

        // Standard error of first coefficient
        let x_matrix = Array2::from_shape_vec(
            (n as usize, p as usize),
            x.iter().flatten().cloned().collect(),
        )
        .map_err(|e| format!("Failed to create X matrix: {}", e))?;

        let xtx_inv = LinearAlgebra::matrix_inverse(&x_matrix.t().dot(&x_matrix))?;
        let se = (variance * xtx_inv[[0, 0]]).sqrt();

        Ok(se)
    }

    fn compute_residuals(x: &[Vec<f64>], y: &[f64], coeffs: &[f64]) -> Result<Vec<f64>, String> {
        let mut residuals = Vec::with_capacity(y.len());

        for (i, y_val) in y.iter().enumerate() {
            let mut pred = 0.0;
            for (j, coeff) in coeffs.iter().enumerate() {
                pred += coeff * x[i][j];
            }
            residuals.push(y_val - pred);
        }

        Ok(residuals)
    }

    fn residual_variance(residuals: &[f64]) -> Result<f64, String> {
        let n = residuals.len();
        if n <= 1 {
            return Err("Not enough residuals for variance calculation".to_string());
        }

        let mean = residuals.iter().sum::<f64>() / n as f64;
        let variance = residuals.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (n - 1) as f64;

        Ok(variance)
    }

    fn aic_criterion(residuals: &[f64], n_params: usize) -> f64 {
        let n = residuals.len() as f64;
        let rss = residuals.iter().map(|r| r * r).sum::<f64>();
        let sigma2 = rss / n;

        if sigma2 <= 0.0 {
            return f64::INFINITY;
        }

        n * sigma2.ln() + 2.0 * n_params as f64
    }

    fn adf_critical_values(n: usize) -> AdfCriticalValues {
        // Corrected MacKinnon (1996, 2010) critical value approximations
        // These use the proper 1/N asymptotic expansion, not 1/sqrt(N)
        let n_f = n as f64;
        let n_inv = 1.0 / n_f;
        let n_inv2 = n_inv * n_inv;

        // MacKinnon (2010) coefficients for Model 1 (constant only)
        // These are the correct coefficients for the 1/N expansion
        let one_percent = -3.43035 - 6.5393 * n_inv - 16.786 * n_inv2;
        let five_percent = -2.86213 - 2.738 * n_inv - 8.36 * n_inv2;
        let ten_percent = -2.56710 - 1.438 * n_inv - 4.48 * n_inv2;

        AdfCriticalValues {
            one_percent,
            five_percent,
            ten_percent,
        }
    }

    fn adf_p_value(test_stat: f64, regression_type: AdfRegressionType) -> f64 {
        // Use MacKinnon (1996) p-value approximations
        // These are based on extensive Monte Carlo simulations and are much more accurate
        // than simple table lookups

        let (tau, n_params) = match regression_type {
            AdfRegressionType::Constant => (test_stat, 1),
            AdfRegressionType::ConstantTrend => (test_stat, 2),
            AdfRegressionType::NoConstant => (test_stat, 0),
        };

        // MacKinnon approximation coefficients
        let coeffs = match n_params {
            0 => vec![0.6344, 1.2378, 3.2496, 9.0893, 19.775], // No constant
            1 => vec![0.4797, 1.1048, 2.8898, 8.0688, 18.26],  // Constant
            2 => vec![0.4919, 1.1646, 3.0172, 8.5731, 19.633], // Constant + trend
            _ => vec![0.4797, 1.1048, 2.8898, 8.0688, 18.26],  // Default to constant
        };

        // Compute p-value using the approximation: 1 - exp(coeffs[0] + coeffs[1]*tau + ...)
        let mut poly = 0.0;
        for (i, &coeff) in coeffs.iter().enumerate() {
            poly += coeff * tau.powi(i as i32);
        }

        let p_value = (-poly).exp();
        p_value.clamp(0.0, 1.0) // Ensure p-value is in valid range
    }

    fn detrend_series(series: &[f64], regression_type: KpssRegressionType) -> Result<(Vec<f64>, Vec<f64>), String> {
        let n = series.len();
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();

        match regression_type {
            KpssRegressionType::Constant => {
                // Remove mean
                let mean = series.iter().sum::<f64>() / n as f64;
                let residuals = series.iter().map(|y| y - mean).collect();
                Ok((residuals, vec![mean]))
            }
            KpssRegressionType::ConstantTrend => {
                // Remove linear trend
                let x_matrix = Array2::from_shape_vec(
                    (n, 2),
                    x.iter().cloned().chain(vec![1.0; n]).collect(),
                )
                .map_err(|e| format!("Failed to create design matrix: {}", e))?;

                let y_vector = Array1::from_vec(series.to_vec());
                let coeffs = LinearAlgebra::solve_linear_system(
                    &x_matrix.t().dot(&x_matrix),
                    &x_matrix.t().dot(&y_vector),
                )?;

                let trend = x_matrix.dot(&coeffs);
                let residuals = (&y_vector - &trend).to_vec();

                Ok((residuals, coeffs.to_vec()))
            }
        }
    }

    fn newey_west_variance(residuals: &[f64], lags: usize) -> Result<f64, String> {
        let n = residuals.len();

        // Base variance
        let mean = residuals.iter().sum::<f64>() / n as f64;
        let mut variance = residuals.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n as f64;

        // Add autocovariances with Bartlett kernel
        for lag in 1..=lags {
            let weight = 1.0 - lag as f64 / (lags + 1) as f64;
            let mut autocov = 0.0;

            for i in lag..n {
                autocov += (residuals[i] - mean) * (residuals[i - lag] - mean);
            }
            autocov /= n as f64;

            variance += 2.0 * weight * autocov;
        }

        Ok(variance)
    }

    fn kpss_critical_values() -> KpssCriticalValues {
        KpssCriticalValues {
            one_percent: 0.739,
            two_point_five_percent: 0.574,
            five_percent: 0.463,
            ten_percent: 0.347,
        }
    }

    fn kpss_p_value(test_stat: f64) -> f64 {
        // Approximate p-values for KPSS test
        if test_stat > 0.739 { 0.01 }
        else if test_stat > 0.574 { 0.025 }
        else if test_stat > 0.463 { 0.05 }
        else if test_stat > 0.347 { 0.10 }
        else { 0.5 }
    }

    fn compute_adf_residuals(
        series: &[f64],
        lags: usize,
        regression_type: AdfRegressionType,
    ) -> Result<Vec<f64>, String> {
        let n = series.len();
        let mut y_diff = Vec::with_capacity(n - 1);
        let mut y_lagged = Vec::with_capacity(n - 1 - lags);

        for i in 1..n {
            y_diff.push(series[i] - series[i - 1]);
        }

        for &value in series.iter().skip(lags).take(n - 1 - lags) {
            y_lagged.push(value);
        }

        let (x_matrix, _) = Self::build_adf_design_matrix(series, &y_diff, lags, regression_type)?;
        let coeffs = LinearRegression::ols_fit_vec(&x_matrix, &y_lagged)?;

        Self::compute_residuals(&x_matrix, &y_lagged, &coeffs)
    }
}