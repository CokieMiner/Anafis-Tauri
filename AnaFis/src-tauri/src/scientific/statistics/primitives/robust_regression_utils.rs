//! Robust regression utilities
//!
//! This module provides centralized utilities for robust regression methods
//! to avoid code duplication across different robust regression algorithms.

use ndarray::{Array1, Array2};
use crate::scientific::statistics::primitives::LinearRegression;

/// Robust regression utilities
pub struct RobustRegressionUtils;

impl RobustRegressionUtils {
    /// Convert Vec<Vec<f64>> design matrix and Vec<f64> response to ndarray format
    ///
    /// # Arguments
    /// * `x_vec` - Design matrix as vector of vectors
    /// * `y_vec` - Response vector
    ///
    /// # Returns
    /// Tuple of (design_matrix, response_vector)
    pub fn convert_to_ndarray(
        x_vec: &[Vec<f64>],
        y_vec: &[f64],
    ) -> Result<(Array2<f64>, Array1<f64>), String> {
        let n_samples = x_vec.len();
        if n_samples == 0 {
            return Err("Empty data".to_string());
        }

        let n_features = x_vec[0].len();
        if n_samples != y_vec.len() {
            return Err("X and y must have the same number of samples".to_string());
        }

        // Convert to ndarray
        let x = Array2::from_shape_vec(
            (n_samples, n_features),
            x_vec.iter().flatten().cloned().collect(),
        ).map_err(|e| format!("Failed to create ndarray from x_vec: {}", e))?;

        let y = Array1::from_vec(y_vec.to_vec());

        Ok((x, y))
    }

    /// Calculate R-squared from residuals and response vector
    ///
    /// # Arguments
    /// * `residuals` - Residuals vector
    /// * `y` - Original response vector
    ///
    /// # Returns
    /// R-squared value
    pub fn calculate_r_squared_from_residuals(
        residuals: &Array1<f64>,
        y: &Array1<f64>,
    ) -> Result<f64, String> {
        let ss_res: f64 = residuals.mapv(|r| r * r).sum();
        let y_mean = y.mean().ok_or("Failed to compute mean of y for R-squared calculation")?;
        let ss_tot: f64 = y.mapv(|yi| (yi - y_mean).powi(2)).sum();
        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };
        Ok(r_squared)
    }

    /// Calculate R-squared from predictions and true values
    ///
    /// # Arguments
    /// * `y_true` - True response values
    /// * `y_pred` - Predicted values
    ///
    /// # Returns
    /// R-squared value
    pub fn calculate_r_squared(y_true: &[f64], y_pred: &[f64]) -> f64 {
        LinearRegression::r_squared(y_true, y_pred)
    }

    /// Check convergence based on weight changes
    ///
    /// # Arguments
    /// * `old_weights` - Previous iteration weights
    /// * `new_weights` - Current iteration weights
    /// * `tolerance` - Convergence tolerance
    ///
    /// # Returns
    /// Tuple of (converged, max_weight_change)
    pub fn check_weight_convergence(
        old_weights: &Array1<f64>,
        new_weights: &Array1<f64>,
        tolerance: f64,
    ) -> (bool, f64) {
        let max_weight_change = (new_weights - old_weights)
            .mapv(f64::abs)
            .into_iter()
            .fold(0.0, f64::max);

        (max_weight_change < tolerance, max_weight_change)
    }

    /// Perform weighted least squares update
    ///
    /// # Arguments
    /// * `x` - Design matrix
    /// * `y` - Response vector
    /// * `weights` - Weight vector
    ///
    /// # Returns
    /// Updated coefficients
    pub fn weighted_least_squares_update(
        x: &Array2<f64>,
        y: &Array1<f64>,
        weights: &Array1<f64>,
    ) -> Result<Array1<f64>, String> {
        // Create diagonal weight matrix
        let w_sqrt = weights.mapv(f64::sqrt);
        let w_diag = Array2::from_diag(&w_sqrt);

        // Weighted design matrix and response
        let x_weighted = w_diag.dot(x);
        let y_weighted = w_diag.dot(y);

        LinearRegression::ols_fit(&x_weighted, &y_weighted)
    }

    /// Validate input data for robust regression
    ///
    /// # Arguments
    /// * `x_vec` - Design matrix
    /// * `y_vec` - Response vector
    ///
    /// # Returns
    /// Tuple of (n_samples, n_features) if valid
    pub fn validate_input_data(
        x_vec: &[Vec<f64>],
        y_vec: &[f64],
    ) -> Result<(usize, usize), String> {
        let n_samples = x_vec.len();
        if n_samples == 0 {
            return Err("Empty data".to_string());
        }

        let n_features = x_vec[0].len();
        if n_samples != y_vec.len() {
            return Err("X and y must have the same number of samples".to_string());
        }

        // Validate all rows have the same number of features
        for row in x_vec {
            if row.len() != n_features {
                return Err("All rows must have the same number of features".to_string());
            }
        }

        Ok((n_samples, n_features))
    }
}