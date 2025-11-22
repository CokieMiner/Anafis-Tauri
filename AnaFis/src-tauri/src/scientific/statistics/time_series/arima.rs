//! ARIMA modeling
//!
//! This module provides comprehensive ARIMA (AutoRegressive Integrated Moving Average)
//! modeling for time series analysis and forecasting.

use crate::scientific::statistics::correlation::CorrelationMethods;
use crate::scientific::statistics::descriptive::StatisticalMoments;
use rayon::prelude::*;

/// ARIMA model parameters
#[derive(Debug, Clone)]
pub struct ArimaParameters {
    /// Auto-regressive order (p)
    pub p: usize,
    /// Differencing order (d)
    pub d: usize,
    /// Moving average order (q)
    pub q: usize,
    /// AR coefficients
    pub ar_coeffs: Vec<f64>,
    /// MA coefficients
    pub ma_coeffs: Vec<f64>,
    /// Constant term
    pub constant: f64,
}

/// ARIMA model result
#[derive(Debug, Clone)]
pub struct ArimaResult {
    /// Model parameters
    pub parameters: ArimaParameters,
    /// Fitted values
    pub fitted_values: Vec<f64>,
    /// Residuals
    pub residuals: Vec<f64>,
    /// Log-likelihood
    pub log_likelihood: f64,
    /// AIC (Akaike Information Criterion)
    pub aic: f64,
    /// BIC (Bayesian Information Criterion)
    pub bic: f64,
    /// Model diagnostics
    pub diagnostics: ArimaDiagnostics,
}

/// ARIMA model diagnostics
#[derive(Debug, Clone)]
pub struct ArimaDiagnostics {
    /// Ljung-Box test statistic for residual autocorrelation
    pub ljung_box_stat: f64,
    /// P-value for Ljung-Box test
    pub ljung_box_p_value: f64,
    /// Residual standard deviation
    pub residual_sd: f64,
    /// Whether residuals are white noise
    pub residuals_white_noise: bool,
}

/// ARIMA forecasting result
#[derive(Debug, Clone)]
pub struct ArimaForecast {
    /// Point forecasts
    pub forecasts: Vec<f64>,
    /// Forecast standard errors
    pub forecast_se: Vec<f64>,
    /// Prediction intervals (lower bounds)
    pub lower_bounds: Vec<f64>,
    /// Prediction intervals (upper bounds)
    pub upper_bounds: Vec<f64>,
    /// Confidence level
    pub confidence_level: f64,
}

/// Temporary structure for ARMA fitting results
#[derive(Debug, Clone)]
struct ArmaResult {
    ar_coeffs: Vec<f64>,
    ma_coeffs: Vec<f64>,
    constant: f64,
    log_likelihood: f64,
}

/// ARIMA model implementation
pub struct ArimaModel;

impl ArimaModel {
    /// Fit ARIMA(p,d,q) model to time series data using internal implementation.
    pub fn fit_arima(data: &[f64], p: usize, d: usize, q: usize) -> Result<ArimaResult, String> {
        if data.len() < p.max(q) + d + 1 {
            return Err("Insufficient data for ARIMA model".to_string());
        }

        // Difference the series if d > 0
        let differenced_data = if d > 0 {
            Self::difference_series(data, d)?
        } else {
            data.to_vec()
        };

        // Fit ARMA on the differenced data
        let arma_result = Self::fit_arma(&differenced_data, p, q)?;

        // Calculate fitted values on the original scale
        let fitted_values = Self::calculate_fitted_values(data, p, d, &arma_result.ar_coeffs, &arma_result.ma_coeffs, arma_result.constant)?;

        // Residuals are for the differenced series
        let residuals = Self::calculate_arma_residuals(&differenced_data, &arma_result.ar_coeffs, &arma_result.ma_coeffs, arma_result.constant)?;

        // Calculate diagnostics on the residuals
        let diagnostics = Self::calculate_diagnostics(&residuals)?;

        let parameters = ArimaParameters {
            p,
            d,
            q,
            ar_coeffs: arma_result.ar_coeffs,
            ma_coeffs: arma_result.ma_coeffs,
            constant: arma_result.constant,
        };

        Ok(ArimaResult {
            parameters,
            fitted_values,
            residuals,
            log_likelihood: arma_result.log_likelihood,
            aic: 2.0 * (p + q + 1) as f64 - 2.0 * arma_result.log_likelihood, // Simplified AIC
            bic: (p + q + 1) as f64 * (data.len() as f64).ln() - 2.0 * arma_result.log_likelihood, // Simplified BIC
            diagnostics,
        })
    }

    /// Automatically select best ARIMA model using information criteria
    pub fn auto_arima(data: &[f64], max_p: usize, max_d: usize, max_q: usize) -> Result<ArimaResult, String> {
        if data.len() < 10 {
            return Err("Need at least 10 observations for auto ARIMA".to_string());
        }

        // Generate all parameter combinations
        let param_combinations: Vec<(usize, usize, usize)> = (0..=max_d.min(2))
            .flat_map(|d| {
                (0..=max_p.min(5)).flat_map(move |p| {
                    (0..=max_q.min(5)).map(move |q| (p, d, q))
                })
            })
            .filter(|(p, d, q)| !(*p == 0 && *q == 0 && *d == 0)) // Skip pure white noise
            .collect();

        // Fit models in parallel
        let model_results: Vec<Option<ArimaResult>> = param_combinations.into_par_iter()
            .map(|(p, d, q)| {
                match Self::fit_arima(data, p, d, q) {
                    Ok(model) if model.diagnostics.residuals_white_noise => Some(model),
                    _ => None,
                }
            })
            .collect();

        // Find best model by AIC
        let best_model = model_results.into_iter()
            .flatten()
            .min_by(|a, b| a.aic.partial_cmp(&b.aic).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| "Could not fit suitable ARIMA model".to_string())?;

        Ok(best_model)
    }

    /// Generate forecasts from fitted ARIMA model
    pub fn forecast(model: &ArimaResult, steps_ahead: usize, confidence_level: f64) -> Result<ArimaForecast, String> {
        if !(0.0..=1.0).contains(&confidence_level) {
            return Err("Confidence level must be between 0 and 1".to_string());
        }

        let mut forecasts = Vec::with_capacity(steps_ahead);
        let mut forecast_se = Vec::with_capacity(steps_ahead);

        // Get the original data (assuming model was fitted to it)
        // This is a simplification - in practice, we'd need to store the original data
        let _data_len = model.fitted_values.len();

        // Generate point forecasts
        let mut current_values = model.fitted_values.clone();

        for step in 0..steps_ahead {
            // Get past errors for MA part
            // For the first few steps, we use the last known residuals
            // For later steps, we assume error is 0
            let mut past_errors = Vec::new();
            if model.parameters.q > 0 {
                for i in 0..model.parameters.q {
                    if step < model.residuals.len() && i < model.residuals.len() {
                         let _idx = model.residuals.len() - 1 - i;
                         // If step > i, then the error term e_{t+step-i} is 0.
                         // If step <= i, then we need e_{t-(i-step)}.
                         if i >= step {
                             let past_idx = model.residuals.len() - 1 - (i - step);
                             if past_idx < model.residuals.len() {
                                 past_errors.push(model.residuals[past_idx]);
                             } else {
                                 past_errors.push(0.0);
                             }
                         } else {
                             past_errors.push(0.0);
                         }
                    } else {
                        past_errors.push(0.0);
                    }
                }
            }

            let next_value = Self::forecast_one_step(&current_values, &model.parameters, &past_errors)?;
            forecasts.push(next_value);

            // Update current values for next forecast
            current_values.push(next_value);
            if current_values.len() > model.parameters.p.max(model.parameters.q) + 1 {
                current_values.remove(0);
            }

            // Calculate forecast standard error (simplified)
            let se = model.diagnostics.residual_sd * (step as f64 + 1.0).sqrt();
            forecast_se.push(se);
        }

        // Calculate prediction intervals
        let z_score = crate::scientific::statistics::distributions::distribution_functions::normal_quantile(1.0 - (1.0 - confidence_level) / 2.0);
        let lower_bounds: Vec<f64> = forecasts.iter().zip(forecast_se.iter())
            .map(|(f, se)| f - z_score * se)
            .collect();
        let upper_bounds: Vec<f64> = forecasts.iter().zip(forecast_se.iter())
            .map(|(f, se)| f + z_score * se)
            .collect();

        Ok(ArimaForecast {
            forecasts,
            forecast_se,
            lower_bounds,
            upper_bounds,
            confidence_level,
        })
    }

    /// Check if time series is stationary
    pub fn check_stationarity(data: &[f64]) -> Result<(bool, f64), String> {
        if data.len() < 10 {
            return Err("Need at least 10 observations for stationarity test".to_string());
        }

        // Augmented Dickey-Fuller test (simplified)
        let adf_stat = Self::adf_test(data)?;
        let critical_value = -2.89; // Approximate 5% critical value for large samples

        let is_stationary = adf_stat < critical_value;
        let p_value = Self::adf_p_value(adf_stat, data.len());

        Ok((is_stationary, p_value))
    }

    // Internal helper methods

    /// Difference a time series
    fn difference_series(data: &[f64], order: usize) -> Result<Vec<f64>, String> {
        if order == 0 {
            return Ok(data.to_vec());
        }

        let mut result = data.to_vec();

        for _ in 0..order {
            let mut diff = Vec::with_capacity(result.len() - 1);
            for i in 1..result.len() {
                diff.push(result[i] - result[i - 1]);
            }
            result = diff;
        }

        Ok(result)
    }

    /// Fit ARMA(p,q) model using maximum likelihood estimation
    fn fit_arma(data: &[f64], p: usize, q: usize) -> Result<ArmaResult, String> {
        // Simplified ARMA fitting using conditional least squares
        // For production use, consider more sophisticated methods

        let _n = data.len();
        let mut ar_coeffs = vec![0.0; p];
        let mut ma_coeffs = vec![0.0; q];
        let mut constant = data.mean();

        // Initialize coefficients using Yule-Walker for AR part
        if p > 0 {
            ar_coeffs = Self::yule_walker_ar(data, p)?;
        }

        // Iterative refinement with convergence check
        let max_iter = 50; // Increased iterations
        let tolerance = 1e-6;
        let mut prev_log_likelihood = f64::NEG_INFINITY;

        for _iter in 0..max_iter {
            let residuals = Self::calculate_arma_residuals(data, &ar_coeffs, &ma_coeffs, constant)?;
            
            // Check convergence
            let current_log_likelihood = Self::calculate_log_likelihood(&residuals);
            if (current_log_likelihood - prev_log_likelihood).abs() < tolerance {
                break;
            }
            prev_log_likelihood = current_log_likelihood;

            if p > 0 {
                ar_coeffs = Self::update_ar_coeffs(data, &residuals, &ar_coeffs)?;
            }
            if q > 0 {
                ma_coeffs = Self::update_ma_coeffs(&residuals, &ma_coeffs)?;
            }
            constant = Self::update_constant(data, &ar_coeffs, &ma_coeffs);
        }

        // Calculate log-likelihood
        let residuals = Self::calculate_arma_residuals(data, &ar_coeffs, &ma_coeffs, constant)?;
        let log_likelihood = Self::calculate_log_likelihood(&residuals);

        Ok(ArmaResult {
            ar_coeffs,
            ma_coeffs,
            constant,
            log_likelihood,
        })
    }

    /// Yule-Walker equations for AR coefficient estimation
    fn yule_walker_ar(data: &[f64], p: usize) -> Result<Vec<f64>, String> {
        let n = data.len();
        if p >= n {
            return Err("AR order too high for data length".to_string());
        }

        // Calculate autocorrelations
        let mut acf = vec![0.0; p + 1];
        let mean = data.mean();
        let variance = data.variance();

        for lag in 0..=p {
            let mut sum = 0.0;
            for i in lag..n {
                sum += (data[i] - mean) * (data[i - lag] - mean);
            }
            acf[lag] = sum / ((n - lag) as f64 * variance);
        }

        // Solve Yule-Walker equations
        let mut coeffs = vec![0.0; p];
        if p == 1 {
            coeffs[0] = acf[1];
        } else {
            // Simplified solution for small p
            for i in 0..p {
                coeffs[i] = acf[i + 1];
                for j in 0..i {
                    coeffs[i] -= coeffs[j] * acf[i - j];
                }
            }
        }

        Ok(coeffs)
    }

    /// Calculate ARMA residuals
    fn calculate_arma_residuals(data: &[f64], ar_coeffs: &[f64], ma_coeffs: &[f64], constant: f64) -> Result<Vec<f64>, String> {
        let n = data.len();
        let p = ar_coeffs.len();
        let q = ma_coeffs.len();

        let mut residuals = vec![0.0; n];
        let mut errors = vec![0.0; n]; // MA error terms

        for i in 0..n {
            let mut prediction = constant;

            // AR part
            for j in 0..p.min(i + 1) {
                prediction += ar_coeffs[j] * data[i - j];
            }

            // MA part
            for j in 0..q.min(i + 1) {
                prediction += ma_coeffs[j] * errors[i - j];
            }

            residuals[i] = data[i] - prediction;
            errors[i] = residuals[i]; // For next MA term
        }

        Ok(residuals)
    }

    /// Update AR coefficients
    fn update_ar_coeffs(data: &[f64], residuals: &[f64], current_ar: &[f64]) -> Result<Vec<f64>, String> {
        // Simplified coefficient update
        let mut new_ar = current_ar.to_vec();

        // Use gradient descent step (very simplified)
        let learning_rate = 0.1 / data.len() as f64;
        for i in 0..current_ar.len() {
            let mut gradient = 0.0;
            for t in current_ar.len()..data.len() {
                gradient += residuals[t] * data[t - i - 1];
            }
            new_ar[i] += learning_rate * gradient;
        }

        Ok(new_ar)
    }

    /// Update MA coefficients
    fn update_ma_coeffs(residuals: &[f64], current_ma: &[f64]) -> Result<Vec<f64>, String> {
        let mut new_ma = current_ma.to_vec();

        // Simplified update
        let learning_rate = 0.1 / residuals.len() as f64;
        for i in 0..current_ma.len() {
            let mut gradient = 0.0;
            for t in current_ma.len()..residuals.len() {
                gradient += residuals[t] * residuals[t - i - 1];
            }
            new_ma[i] += learning_rate * gradient;
        }

        Ok(new_ma)
    }

    /// Update constant term
    fn update_constant(data: &[f64], ar_coeffs: &[f64], _ma_coeffs: &[f64]) -> f64 {
        let n = data.len();
        let mut sum = 0.0;

        for i in 0..n {
            let mut ar_sum = 0.0;
            for (j, &coeff) in ar_coeffs.iter().enumerate() {
                if i > j {
                    ar_sum += coeff * data[i - j - 1];
                }
            }
            sum += data[i] - ar_sum;
        }

        sum / n as f64
    }

    /// Calculate fitted values from ARIMA model
    fn calculate_fitted_values(data: &[f64], _p: usize, d: usize, ar_coeffs: &[f64], ma_coeffs: &[f64], constant: f64) -> Result<Vec<f64>, String> {
        let _n = data.len();

        // For differenced series, we need to integrate back
        let fitted = if d > 0 {
            // This is a simplification - proper integration would be more complex
            let differenced = Self::difference_series(data, d)?;
            let arma_fitted = Self::calculate_arma_fitted(&differenced, ar_coeffs, ma_coeffs, constant)?;

            // Integrate back (cumulative sum)
            Self::integrate_series(&arma_fitted, data, d)?
        } else {
            Self::calculate_arma_fitted(data, ar_coeffs, ma_coeffs, constant)?
        };

        Ok(fitted)
    }

    /// Calculate ARMA fitted values
    fn calculate_arma_fitted(data: &[f64], ar_coeffs: &[f64], ma_coeffs: &[f64], constant: f64) -> Result<Vec<f64>, String> {
        let residuals = Self::calculate_arma_residuals(data, ar_coeffs, ma_coeffs, constant)?;
        let mut fitted = vec![0.0; data.len()];

        for i in 0..data.len() {
            fitted[i] = data[i] - residuals[i];
        }

        Ok(fitted)
    }

    /// Integrate differenced series back to original scale
    fn integrate_series(differenced_fitted: &[f64], original: &[f64], d: usize) -> Result<Vec<f64>, String> {
        // Simplified integration - assumes we have the original series to work with
        let mut integrated = original.to_vec();

        // This is a very simplified integration
        // Proper implementation would require careful handling of initial conditions
        for i in d..integrated.len() {
            integrated[i] = original[i - d] + differenced_fitted[i - d];
        }

        Ok(integrated)
    }

    /// Forecast one step ahead
    fn forecast_one_step(current_values: &[f64], params: &ArimaParameters, past_errors: &[f64]) -> Result<f64, String> {
        let n = current_values.len();
        let mut forecast = params.constant;

        // AR part
        for i in 0..params.p {
            if n > i {
                forecast += params.ar_coeffs[i] * current_values[n - 1 - i];
            }
        }

        // MA part
        for i in 0..params.q {
            if i < past_errors.len() {
                forecast += params.ma_coeffs[i] * past_errors[i];
            }
        }

        Ok(forecast)
    }

    /// Calculate log-likelihood
    fn calculate_log_likelihood(residuals: &[f64]) -> f64 {
        let n = residuals.len() as f64;
        let ss_res = residuals.iter().map(|r| r * r).sum::<f64>();
        let sigma2 = ss_res / n;

        if sigma2 <= 0.0 {
            return f64::NEG_INFINITY;
        }

        -0.5 * n * (2.0 * std::f64::consts::PI * sigma2).ln() - ss_res / (2.0 * sigma2)
    }

    /// Calculate model diagnostics
    fn calculate_diagnostics(residuals: &[f64]) -> Result<ArimaDiagnostics, String> {
        let residual_sd = residuals.std_dev();

        // Ljung-Box test for residual autocorrelation
        let ljung_box_stat = Self::ljung_box_test(residuals, 10)?;
        let ljung_box_p_value = Self::ljung_box_p_value(ljung_box_stat, 10, residuals.len());

        let residuals_white_noise = ljung_box_p_value > 0.05;

        Ok(ArimaDiagnostics {
            ljung_box_stat,
            ljung_box_p_value,
            residual_sd,
            residuals_white_noise,
        })
    }

    /// Ljung-Box test statistic
    fn ljung_box_test(residuals: &[f64], lags: usize) -> Result<f64, String> {
        let n = residuals.len();
        if lags >= n {
            return Err("Too many lags for Ljung-Box test".to_string());
        }

        // Get autocorrelations using the centralized implementation
        let autocorrelations = CorrelationMethods::autocorrelation(residuals, lags)?;

        let mut q = 0.0;
        for (k, &acf_k) in autocorrelations.iter().enumerate() {
            q += acf_k * acf_k / (n - (k + 1)) as f64; // k starts from 0, so lag = k+1
        }

        Ok(q * n as f64 * (n + 2) as f64)
    }

    /// Ljung-Box p-value approximation
    fn ljung_box_p_value(q: f64, lags: usize, _n: usize) -> f64 {
        // Chi-squared approximation
        let cdf = crate::scientific::statistics::distributions::distribution_functions::chi_squared_cdf(q, lags as f64);
        1.0 - cdf
    }

    /// Augmented Dickey-Fuller test (simplified)
    fn adf_test(data: &[f64]) -> Result<f64, String> {
        let n = data.len();
        if n < 10 {
            return Err("Need more data for ADF test".to_string());
        }

        // Difference the series
        let diff: Vec<f64> = (1..n).map(|i| data[i] - data[i - 1]).collect();

        // Create lagged level
        let lagged_level = &data[0..n-1];

        // Simple regression: diff_t = a + b * level_{t-1} + error
        let mean_diff = diff.mean();
        let mean_level = lagged_level.mean();

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 0..lagged_level.len() {
            let diff_dev = diff[i] - mean_diff;
            let level_dev = lagged_level[i] - mean_level;
            numerator += diff_dev * level_dev;
            denominator += level_dev * level_dev;
        }

        if denominator == 0.0 {
            return Ok(0.0);
        }

        let b = numerator / denominator;
        Ok(b) // ADF statistic (simplified, should include more terms)
    }

    /// ADF p-value approximation
    fn adf_p_value(stat: f64, _n: usize) -> f64 {
        // Very simplified approximation
        if stat < -3.5 {
            0.01
        } else if stat < -3.0 {
            0.05
        } else if stat < -2.5 {
            0.10
        } else {
            0.5
        }
    }
}