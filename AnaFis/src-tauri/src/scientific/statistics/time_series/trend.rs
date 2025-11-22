//! Trend analysis
//!
//! This module provides comprehensive trend analysis for time series data
//! including linear regression, polynomial trends, exponential smoothing,
//! and trend decomposition.


use crate::scientific::statistics::primitives::LinearRegression;
use crate::scientific::statistics::primitives::DesignMatrixBuilder;

/// Trend analysis result
#[derive(Debug, Clone)]
pub struct TrendResult {
    /// Type of trend detected
    pub trend_type: TrendType,
    /// Trend parameters (slope, intercept, etc.)
    pub parameters: Vec<f64>,
    /// R-squared value (goodness of fit)
    pub r_squared: f64,
    /// P-value for trend significance
    pub p_value: f64,
    /// Whether the trend is statistically significant
    pub significant: bool,
    /// Trend line values
    pub trend_values: Vec<f64>,
}

/// Types of trends that can be detected
#[derive(Debug, Clone, PartialEq)]
pub enum TrendType {
    /// No significant trend
    None,
    /// Linear trend: y = mx + b
    Linear,
    /// Exponential trend: y = a * e^(bx)
    Exponential,
    /// Logarithmic trend: y = a + b * ln(x)
    Logarithmic,
    /// Power trend: y = a * x^b
    Power,
    /// Polynomial trend: y = a + b*x + c*x^2 + ...
    Polynomial(usize), // degree
}

/// Trend analysis result
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub trend_present: bool,
    pub slope: f64,
    pub intercept: f64,
    pub r_squared: f64,
    pub significance: f64,
}

/// Trend analysis engine
pub struct TrendAnalysisEngine;

impl TrendAnalysisEngine {
    /// Analyze trend in time series data
    pub fn analyze_trend(data: &[f64]) -> Result<TrendResult, String> {
        if data.len() < 3 {
            return Err("Need at least 3 data points for trend analysis".to_string());
        }

        // Try different trend models and select the best one
        let models = vec![
            Self::linear_trend(data)?,
            Self::exponential_trend(data)?,
            Self::logarithmic_trend(data)?,
            Self::power_trend(data)?,
        ];

        // Find the model with the highest R-squared
        let best_model = models.into_iter()
            .max_by(|a, b| a.r_squared.partial_cmp(&b.r_squared).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or_else(|| TrendResult {
                trend_type: TrendType::None,
                parameters: vec![],
                r_squared: 0.0,
                p_value: 1.0,
                significant: false,
                trend_values: vec![],
            });

        Ok(best_model)
    }

    /// Linear trend analysis using least squares regression
    pub fn linear_trend(data: &[f64]) -> Result<TrendResult, String> {
        let n = data.len();
        let x_values: Vec<f64> = (0..n).map(|i| i as f64).collect();

        // Create design matrix using centralized builder
        let time_data: Vec<Vec<f64>> = x_values.iter().map(|&x| vec![x]).collect();
        let x_matrix = DesignMatrixBuilder::build_linear(&time_data)?;

        // Use centralized OLS regression
        let coefficients = LinearRegression::ols_fit(&x_matrix, &ndarray::Array1::from_vec(data.to_vec()))?;

        let intercept = coefficients[0];
        let slope = coefficients[1];

        // Generate trend values using centralized predict function
        let trend_values_ndarray = LinearRegression::predict(&x_matrix, &coefficients);
        let trend_values: Vec<f64> = trend_values_ndarray.to_vec();

        // Calculate R-squared
        let r_squared = LinearRegression::r_squared(data, &trend_values);

        // Calculate sum of squared residuals for standard error
        let ss_res = data.iter().zip(trend_values.iter())
            .map(|(y, pred)| (y - pred).powi(2))
            .sum::<f64>();

        // Calculate p-value for slope significance
        let x_mean = x_values.iter().sum::<f64>() / n as f64;
        let denominator = x_values.iter()
            .map(|x| (x - x_mean).powi(2))
            .sum::<f64>();

        let se_slope = (ss_res / (n as f64 - 2.0)).sqrt() / denominator.sqrt();
        let t_stat = slope / se_slope;
        let p_value = Self::t_test_p_value(t_stat, n as f64 - 2.0);

        Ok(TrendResult {
            trend_type: TrendType::Linear,
            parameters: vec![slope, intercept],
            r_squared,
            p_value,
            significant: p_value < 0.05,
            trend_values,
        })
    }

    /// Exponential trend analysis: y = a * e^(b*x)
    pub fn exponential_trend(data: &[f64]) -> Result<TrendResult, String> {
        // Transform to linear by taking ln(y)
        let ln_data: Vec<f64> = data.iter()
            .filter(|&&y| y > 0.0)
            .map(|y| y.ln())
            .collect();

        if ln_data.len() < data.len() / 2 {
            // Not enough positive values for exponential fit
            return Ok(TrendResult {
                trend_type: TrendType::None,
                parameters: vec![],
                r_squared: 0.0,
                p_value: 1.0,
                significant: false,
                trend_values: vec![],
            });
        }

        let linear_result = Self::linear_trend(&ln_data)?;

        // Transform back to exponential parameters
        let a = linear_result.parameters[1].exp(); // e^intercept
        let b = linear_result.parameters[0]; // slope remains the same

        // Generate trend values
        let x_values: Vec<f64> = (0..data.len()).map(|i| i as f64).collect();
        let trend_values = x_values.iter()
            .map(|x| a * (b * x).exp())
            .collect();

        Ok(TrendResult {
            trend_type: TrendType::Exponential,
            parameters: vec![a, b],
            r_squared: linear_result.r_squared,
            p_value: linear_result.p_value,
            significant: linear_result.significant,
            trend_values,
        })
    }

    /// Logarithmic trend analysis: y = a + b * ln(x)
    pub fn logarithmic_trend(data: &[f64]) -> Result<TrendResult, String> {
        let x_values: Vec<f64> = (1..=data.len()).map(|i| i as f64).collect(); // Start from 1 to avoid ln(0)
        let ln_x: Vec<f64> = x_values.iter().map(|x| x.ln()).collect();

        // Linear regression with ln(x)
        let n = data.len() as f64;
        let ln_x_mean = ln_x.iter().sum::<f64>() / n;
        let y_mean = data.iter().sum::<f64>() / n;

        let numerator = ln_x.iter().zip(data.iter())
            .map(|(lnx, y)| (lnx - ln_x_mean) * (y - y_mean))
            .sum::<f64>();

        let denominator = ln_x.iter()
            .map(|lnx| (lnx - ln_x_mean).powi(2))
            .sum::<f64>();

        if denominator == 0.0 {
            return Ok(TrendResult {
                trend_type: TrendType::None,
                parameters: vec![],
                r_squared: 0.0,
                p_value: 1.0,
                significant: false,
                trend_values: vec![],
            });
        }

        let b = numerator / denominator;
        let a = y_mean - b * ln_x_mean;

        // Generate trend values
        let trend_values: Vec<f64> = x_values.iter()
            .map(|x| a + b * x.ln())
            .collect();

        // Calculate R-squared
        let r_squared = LinearRegression::r_squared(data, &trend_values);

        // Calculate sum of squared residuals for standard error
        let ss_res = data.iter().zip(trend_values.iter())
            .map(|(y, pred)| (y - pred).powi(2))
            .sum::<f64>();

        // Calculate p-value
        let se_b = (ss_res / (n - 2.0)).sqrt() / denominator.sqrt();
        let t_stat = b / se_b;
        let p_value = Self::t_test_p_value(t_stat, n - 2.0);

        Ok(TrendResult {
            trend_type: TrendType::Logarithmic,
            parameters: vec![a, b],
            r_squared,
            p_value,
            significant: p_value < 0.05,
            trend_values,
        })
    }

    /// Power trend analysis: y = a * x^b
    pub fn power_trend(data: &[f64]) -> Result<TrendResult, String> {
        let x_values: Vec<f64> = (1..=data.len()).map(|i| i as f64).collect(); // Start from 1 to avoid ln(0)

        // Transform to linear by taking ln(y) and ln(x)
        let valid_pairs: Vec<(f64, f64)> = x_values.iter().zip(data.iter())
            .filter(|(x, y)| **x > 0.0 && **y > 0.0)
            .map(|(x, y)| (x.ln(), y.ln()))
            .collect();

        if valid_pairs.len() < data.len() / 2 {
            return Ok(TrendResult {
                trend_type: TrendType::None,
                parameters: vec![],
                r_squared: 0.0,
                p_value: 1.0,
                significant: false,
                trend_values: vec![],
            });
        }

        let _ln_x: Vec<f64> = valid_pairs.iter().map(|(x, _)| *x).collect();
        let ln_y: Vec<f64> = valid_pairs.iter().map(|(_, y)| *y).collect();

        let linear_result = Self::linear_trend(&ln_y)?;

        // Transform back to power parameters
        let b = linear_result.parameters[0]; // slope
        let a = linear_result.parameters[1].exp(); // e^intercept

        // Generate trend values
        let trend_values = x_values.iter()
            .map(|x| a * x.powf(b))
            .collect();

        Ok(TrendResult {
            trend_type: TrendType::Power,
            parameters: vec![a, b],
            r_squared: linear_result.r_squared,
            p_value: linear_result.p_value,
            significant: linear_result.significant,
            trend_values,
        })
    }

    /// Polynomial trend analysis
    pub fn polynomial_trend(data: &[f64], degree: usize) -> Result<TrendResult, String> {
        if degree == 0 || degree >= data.len() {
            return Err("Invalid polynomial degree".to_string());
        }

        let x_values: Vec<f64> = (0..data.len()).map(|i| i as f64).collect();

        // Use normal equations for polynomial fitting
        let coefficients = Self::polynomial_fit(&x_values, data, degree)?;

        // Generate trend values
        let trend_values: Vec<f64> = x_values.iter()
            .map(|x| Self::evaluate_polynomial(&coefficients, *x))
            .collect();

        // Calculate R-squared
        let r_squared = LinearRegression::r_squared(data, &trend_values);

        // Simplified p-value calculation (F-test would be more appropriate)
        let p_value = if r_squared > 0.5 { 0.01 } else { 0.1 };

        Ok(TrendResult {
            trend_type: TrendType::Polynomial(degree),
            parameters: coefficients,
            r_squared,
            p_value,
            significant: r_squared > 0.5,
            trend_values,
        })
    }

    /// Centered moving average trend smoothing
    /// Computes a centered moving average where each point is the average of
    /// window_size points centered on that position.
    pub fn centered_moving_average(data: &[f64], window_size: usize) -> Result<Vec<f64>, String> {
        if window_size == 0 || window_size > data.len() {
            return Err("Invalid window size".to_string());
        }

        let mut result = Vec::with_capacity(data.len());

        for i in 0..data.len() {
            let start = if i >= window_size / 2 { i.saturating_sub(window_size / 2) } else { 0 };
            let end = (i + window_size / 2 + 1).min(data.len());

            let window_data = &data[start..end];
            let avg = window_data.iter().sum::<f64>() / window_data.len() as f64;
            result.push(avg);
        }

        Ok(result)
    }

    /// Simple moving average (backward-looking window)
    /// Each point is the average of the previous window_size points.
    pub fn simple_moving_average(data: &[f64], window_size: usize) -> Result<Vec<f64>, String> {
        if window_size == 0 || window_size > data.len() {
            return Err("Invalid window size".to_string());
        }

        let mut result = Vec::with_capacity(data.len());

        for i in 0..data.len() {
            let start = i.saturating_sub(window_size - 1);
            let window_data = &data[start..=i];
            let avg = window_data.iter().sum::<f64>() / window_data.len() as f64;
            result.push(avg);
        }

        Ok(result)
    }

    /// Exponential smoothing
    pub fn exponential_smoothing(data: &[f64], alpha: f64) -> Result<Vec<f64>, String> {
        if !(0.0..=1.0).contains(&alpha) {
            return Err("Alpha must be between 0 and 1".to_string());
        }

        if data.is_empty() {
            return Ok(vec![]);
        }

        let mut result = Vec::with_capacity(data.len());
        result.push(data[0]); // First value remains unchanged

        for i in 1..data.len() {
            let smoothed = alpha * data[i] + (1.0 - alpha) * result[i - 1];
            result.push(smoothed);
        }

        Ok(result)
    }

    /// Detect trend direction and strength
    pub fn trend_direction(data: &[f64]) -> Result<(TrendDirection, f64), String> {
        if data.len() < 2 {
            return Err("Need at least 2 data points".to_string());
        }

        let trend_result = Self::linear_trend(data)?;
        let slope = trend_result.parameters.first().copied().unwrap_or(0.0);

        let direction = if slope > 0.01 {
            TrendDirection::Increasing
        } else if slope < -0.01 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        let strength = trend_result.r_squared;

        Ok((direction, strength))
    }

    // Helper functions

    fn t_test_p_value(t_stat: f64, df: f64) -> f64 {
        let cdf = crate::scientific::statistics::distributions::distribution_functions::student_t_cdf(t_stat.abs(), 0.0, 1.0, df);
        2.0 * (1.0 - cdf)
    }

    fn polynomial_fit(x: &[f64], y: &[f64], degree: usize) -> Result<Vec<f64>, String> {
        // Simplified polynomial fitting using normal equations
        // For production use, consider more robust methods
        let _n = x.len();
        let mut a = vec![vec![0.0; degree + 1]; degree + 1];
        let mut b = vec![0.0; degree + 1];

        // Build normal equations
        for i in 0..=degree {
            for j in 0..=degree {
                a[i][j] = x.iter().map(|x_val| x_val.powf((i + j) as f64)).sum::<f64>();
            }
            b[i] = x.iter().zip(y.iter())
                .map(|(x_val, y_val)| y_val * x_val.powf(i as f64))
                .sum::<f64>();
        }

        // Solve using centralized LinearAlgebra::solve_linear_system
        use crate::scientific::statistics::primitives::LinearAlgebra;
        use ndarray::{Array2, Array1};
        
        let a_matrix = Array2::from_shape_vec((degree + 1, degree + 1), a.into_iter().flatten().collect())
            .map_err(|e| format!("Failed to create coefficient matrix: {}", e))?;
        let b_vector = Array1::from_vec(b);
        
        LinearAlgebra::solve_linear_system(&a_matrix, &b_vector).map(|arr| arr.to_vec())
    }

    fn evaluate_polynomial(coeffs: &[f64], x: f64) -> f64 {
        coeffs.iter().enumerate()
            .map(|(i, c)| c * x.powf(i as f64))
            .sum()
    }
}

/// Trend direction enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    /// Increasing trend
    Increasing,
    /// Decreasing trend
    Decreasing,
    /// Stable/no significant trend
    Stable,
}