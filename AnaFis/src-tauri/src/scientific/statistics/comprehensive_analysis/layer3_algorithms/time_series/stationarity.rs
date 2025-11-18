use crate::scientific::statistics::types::*;
use ndarray::Array2;

/// Stationarity testing engine for time series analysis
pub struct StationarityEngine;

impl StationarityEngine {
    /// Proper Augmented Dickey-Fuller test for unit root
    /// Tests H0: unit root (non-stationary) vs H1: stationary
    /// Returns test statistic and p-value approximation
    pub fn adf_test(data: &[f64]) -> Result<StationarityResult, String> {
        if data.len() < 10 {
            return Err("Insufficient data for ADF test (need at least 10 observations)".to_string());
        }

        // Determine optimal lag length using AIC-like criterion
        let max_lags = ((data.len() as f64).powf(1.0/3.0) as usize).min(data.len() / 4);
        let optimal_lags = Self::select_adf_lags(data, max_lags);

        // Perform ADF regression: Δy_t = α + β*t + γ*y_{t-1} + Σ δ_i*Δy_{t-i}
        let (gamma_coeff, gamma_se) = Self::adf_regression(data, optimal_lags)?;

        // Test statistic is t-statistic for γ coefficient
        let t_statistic = gamma_coeff / gamma_se;

        // Approximate p-value using MacKinnon (1994) response surface
        let p_value = Self::mackinnon_p_value(t_statistic, data.len() as f64, optimal_lags, true, true);

        // For ADF: reject H0 (unit root) if p < α, meaning series is stationary
        Ok(StationarityResult {
            is_stationary: p_value < 0.05,
            p_value
        })
    }

    /// Proper KPSS test for stationarity around deterministic trend
    /// Tests H0: stationary vs H1: unit root (non-stationary)
    pub fn kpss_test(data: &[f64]) -> Result<StationarityResult, String> {
        if data.len() < 10 {
            return Err("Insufficient data for KPSS test (need at least 10 observations)".to_string());
        }

        // Estimate deterministic trend: y_t = α + β*t + ε_t
        let n = data.len() as f64;
        let t_values: Vec<f64> = (0..data.len()).map(|i| i as f64).collect();

        let sum_t = t_values.iter().sum::<f64>();
        let sum_y = data.iter().sum::<f64>();
        let sum_ty = t_values.iter().zip(data.iter()).map(|(t, y)| t * y).sum::<f64>();
        let sum_t2 = t_values.iter().map(|t| t * t).sum::<f64>();

        let beta = (n * sum_ty - sum_t * sum_y) / (n * sum_t2 - sum_t * sum_t);
        let alpha = (sum_y - beta * sum_t) / n;

        // Compute residuals
        let residuals: Vec<f64> = data.iter().zip(t_values.iter())
            .map(|(y, t)| y - alpha - beta * t)
            .collect();

        // Estimate long-run variance using Newey-West with automatic bandwidth
        let bandwidth = (4.0 * (n / 100.0).powf(2.0/9.0)) as usize;
        let long_run_variance = Self::newey_west_variance(&residuals, bandwidth);

        // Compute partial sums
        let mut partial_sums = vec![0.0; data.len() + 1];
        for i in 1..=data.len() {
            partial_sums[i] = partial_sums[i - 1] + residuals[i - 1];
        }

        // KPSS statistic: (1/n²) * Σ S_t² / σ²
        let mut sum_squared_s = 0.0;
        for &s in &partial_sums[1..] {
            sum_squared_s += s * s;
        }

        let kpss_stat = sum_squared_s / (n * n * long_run_variance);

        // Critical values for KPSS (trend stationary case)
        // Approximate p-value based on asymptotic distribution
        let p_value = Self::kpss_p_value(kpss_stat);

        // For KPSS: reject H0 (stationary) if p < α, meaning series has unit root
        Ok(StationarityResult {
            is_stationary: p_value >= 0.05,
            p_value
        })
    }

    /// Select optimal lag length for ADF test using AIC-like criterion
    fn select_adf_lags(data: &[f64], max_lags: usize) -> usize {
        let mut best_lags = 0;
        let mut best_aic = f64::INFINITY;

        for lags in 0..=max_lags {
            if let Ok((_, _, aic)) = Self::adf_regression_aic(data, lags) {
                if aic < best_aic {
                    best_aic = aic;
                    best_lags = lags;
                }
            }
        }

        best_lags
    }

    /// Perform ADF regression and return coefficient and standard error for γ
    fn adf_regression(data: &[f64], lags: usize) -> Result<(f64, f64), String> {
        let n = data.len();
        if n < lags + 2 {
            return Err("Insufficient data for ADF regression".to_string());
        }

        // Create design matrix for: Δy_t = α + β*t + γ*y_{t-1} + Σ δ_i*Δy_{t-i}
        let mut x_matrix = Vec::new();
        let mut y_vector = Vec::new();

        for t in (lags + 1)..n {
            // Dependent variable: Δy_t
            y_vector.push(data[t] - data[t - 1]);

            // Independent variables: [1, t, y_{t-1}, Δy_{t-1}, ..., Δy_{t-lags}]
            let mut row = vec![1.0, t as f64, data[t - 1]];
            for lag in 1..=lags {
                row.push(data[t - lag] - data[t - lag - 1]);
            }
            x_matrix.push(row);
        }

        Self::ols_regression(&x_matrix, &y_vector)
    }

    /// ADF regression with AIC calculation
    fn adf_regression_aic(data: &[f64], lags: usize) -> Result<(f64, f64, f64), String> {
        let (gamma, se) = Self::adf_regression(data, lags)?;

        // Calculate AIC-like criterion
        let n = data.len() - lags - 1;
        let k = lags + 3; // parameters: constant, trend, gamma, and lags
        let sigma2 = se * se; // approximate residual variance
        let aic = (n as f64) * sigma2.ln() + 2.0 * k as f64;

        Ok((gamma, se, aic))
    }

    /// Simple OLS regression returning coefficient and SE for last variable
    fn ols_regression(x_matrix: &[Vec<f64>], y_vector: &[f64]) -> Result<(f64, f64), String> {
        let n = x_matrix.len();
        let p = x_matrix[0].len();

        if n < p {
            return Err("Insufficient observations for regression".to_string());
        }

        // Convert to ndarray matrix
        let mut x_data = Vec::new();
        for row in x_matrix {
            x_data.extend_from_slice(row);
        }
        let x = Array2::from_shape_vec((n, p), x_data).map_err(|e| e.to_string())?;
        let y = ndarray::Array1::from_vec(y_vector.to_vec());

        // Compute (X^T X)^(-1) X^T y
        let xt = x.t().to_owned();
        let xtx = xt.dot(&x);
        let xtx_inv = crate::scientific::statistics::comprehensive_analysis::layer4_primitives::LinearAlgebra::matrix_inverse(&xtx)?;
        let xty = xt.dot(&y);
        let beta = xtx_inv.dot(&xty);

        // Compute residual variance
        let y_hat = x.dot(&beta);
        let residuals = &y - &y_hat;
        let sigma2 = residuals.iter().map(|&r| r * r).sum::<f64>() / (n - p) as f64;

        // Standard error for last coefficient (gamma in ADF)
        let gamma_se = (sigma2 * xtx_inv[[p-1, p-1]]).sqrt();

        Ok((beta[p-1], gamma_se))
    }

    /// Newey-West long-run variance estimation
    fn newey_west_variance(residuals: &[f64], bandwidth: usize) -> f64 {
        let n = residuals.len() as f64;
        let mut s2 = 0.0;

        // Base variance
        for &e in residuals {
            s2 += e * e;
        }
        s2 /= n;

        // Add autocovariances with Bartlett kernel
        for lag in 1..=bandwidth {
            let weight = 1.0 - lag as f64 / (bandwidth + 1) as f64;
            let mut gamma = 0.0;
            let mut count = 0;

            for i in lag..residuals.len() {
                gamma += residuals[i] * residuals[i - lag];
                count += 1;
            }

            if count > 0 {
                gamma /= count as f64;
                s2 += 2.0 * weight * gamma;
            }
        }

        s2.max(1e-10) // Ensure positive variance
    }

    /// Approximate p-value for ADF test using MacKinnon (1994) response surface
    fn mackinnon_p_value(stat: f64, n: f64, _lags: usize, constant: bool, trend: bool) -> f64 {
        // Simplified MacKinnon approximation for trend + constant case
        // These are approximate critical value regressions
        let (a, b, c, d) = if trend && constant {
            (-3.43, -6.32, -16.06, -28.01)
        } else if constant {
            (-2.86, -2.52, -3.50, -5.10)
        } else {
            (-1.95, -0.23, 1.65, 3.51)
        };

        let n_inv = 1.0 / n;
        let n_inv2 = n_inv * n_inv;

        let critical_value = a + b * n_inv + c * n_inv2 + d * n_inv2 * n_inv;

        // For p-value approximation, use normal CDF
        // This is a rough approximation - in practice, more sophisticated methods exist
        if stat < critical_value {
            0.01 // Reject H0 at 1% level
        } else if stat < critical_value + 0.5 {
            0.05 // Reject H0 at 5% level
        } else if stat < critical_value + 1.0 {
            0.10 // Reject H0 at 10% level
        } else {
            0.20 // Fail to reject H0
        }
    }

    /// Approximate p-value for KPSS test
    fn kpss_p_value(stat: f64) -> f64 {
        // Critical values for KPSS (trend case): 0.119 (10%), 0.146 (5%), 0.176 (2.5%), 0.216 (1%)
        if stat > 0.216 {
            0.01
        } else if stat > 0.176 {
            0.025
        } else if stat > 0.146 {
            0.05
        } else if stat > 0.119 {
            0.10
        } else {
            0.20
        }
    }
}