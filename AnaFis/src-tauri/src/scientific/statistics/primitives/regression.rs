//! Linear regression utilities
//!
//! This module provides centralized linear regression functionality
//! to avoid code duplication across the statistics library.

use ndarray::{Array1, Array2};
use crate::scientific::statistics::primitives::LinearAlgebra;
use crate::scientific::statistics::descriptive::StatisticalMoments;

/// Linear regression utilities
pub struct LinearRegression;

impl LinearRegression {
    /// Perform ordinary least squares (OLS) regression using SVD for numerical stability
    ///
    /// Uses SVD decomposition for better numerical stability with ill-conditioned matrices
    /// compared to solving normal equations directly.
    ///
    /// # Arguments
    /// * `x` - Design matrix (n_samples x n_features)
    /// * `y` - Response vector (n_samples)
    ///
    /// # Returns
    /// Vector of coefficients β (n_features)
    pub fn ols_fit(x: &Array2<f64>, y: &Array1<f64>) -> Result<Array1<f64>, String> {
        if x.nrows() != y.len() {
            return Err("Design matrix rows must match response vector length".to_string());
        }

        if x.nrows() < x.ncols() {
            return Err("Underdetermined system: more features than samples".to_string());
        }

        // Use SVD for numerical stability
        Self::ols_fit_svd(x, y)
    }

    /// OLS fit using SVD decomposition (numerically stable)
    fn ols_fit_svd(x: &Array2<f64>, y: &Array1<f64>) -> Result<Array1<f64>, String> {
        use crate::scientific::statistics::primitives::LinearAlgebra;

        // Perform SVD: X = U S V^T
        let (u, s, vt) = LinearAlgebra::svd(x)
            .map_err(|e| format!("SVD decomposition failed: {}", e))?;

        // For least squares, we solve X β = y
        // Using SVD: β = V S^+ U^T y
        // where S^+ is the pseudoinverse of S

        // Compute U^T y
        let ut_y = u.t().dot(y);

        // Apply pseudoinverse of singular values (regularized for numerical stability)
        let mut s_inv = Array1::zeros(s.len());
        let tolerance = 1e-12 * s[0]; // Relative tolerance based on largest singular value

        for i in 0..s.len() {
            if s[i] > tolerance {
                s_inv[i] = 1.0 / s[i];
            }
        }

        // Compute S^+ U^T y
        let s_inv_ut_y = &s_inv * &ut_y;

        // Compute V (S^+ U^T y)
        let coefficients = vt.t().dot(&s_inv_ut_y);

        Ok(coefficients)
    }

    /// Perform OLS regression with design matrix as Vec<Vec<f64>>
    ///
    /// # Arguments
    /// * `x` - Design matrix as vector of vectors
    /// * `y` - Response vector
    ///
    /// # Returns
    /// Vector of coefficients
    pub fn ols_fit_vec(x: &[Vec<f64>], y: &[f64]) -> Result<Vec<f64>, String> {
        if x.is_empty() || y.is_empty() {
            return Err("Empty input data".to_string());
        }

        let n_samples = x.len();
        let n_features = x[0].len();

        if n_samples != y.len() {
            return Err("Design matrix rows must match response vector length".to_string());
        }

        // Convert to Array2
        let x_flat: Vec<f64> = x.iter().flatten().cloned().collect();
        let x_matrix = Array2::from_shape_vec((n_samples, n_features), x_flat)
            .map_err(|e| format!("Failed to create design matrix: {}", e))?;

        let y_vector = Array1::from_vec(y.to_vec());

        Self::ols_fit(&x_matrix, &y_vector).map(|arr| arr.to_vec())
    }

    /// Calculate predictions for given design matrix and coefficients
    ///
    /// # Arguments
    /// * `x` - Design matrix
    /// * `coefficients` - Regression coefficients
    ///
    /// # Returns
    /// Predicted values
    pub fn predict(x: &Array2<f64>, coefficients: &Array1<f64>) -> Array1<f64> {
        x.dot(coefficients)
    }

    /// Calculate R-squared (coefficient of determination)
    ///
    /// # Arguments
    /// * `y_true` - True values
    /// * `y_pred` - Predicted values
    ///
    /// # Returns
    /// R-squared value between 0 and 1
    pub fn r_squared(y_true: &[f64], y_pred: &[f64]) -> f64 {
        if y_true.len() != y_pred.len() {
            return 0.0;
        }

        let n = y_true.len() as f64;
        let y_mean = y_true.iter().sum::<f64>() / n;

        let ss_tot: f64 = y_true.iter().map(|y| (y - y_mean).powi(2)).sum();
        let ss_res: f64 = y_true.iter().zip(y_pred.iter())
            .map(|(y, pred)| (y - pred).powi(2))
            .sum();

        if ss_tot == 0.0 {
            0.0
        } else {
            1.0 - (ss_res / ss_tot)
        }
    }

    /// Calculate root mean squared error (RMSE)
    ///
    /// # Arguments
    /// * `y_true` - True values
    /// * `y_pred` - Predicted values
    ///
    /// # Returns
    /// RMSE value
    pub fn rmse(y_true: &[f64], y_pred: &[f64]) -> f64 {
        if y_true.len() != y_pred.len() {
            return f64::INFINITY;
        }

        let n = y_true.len() as f64;
        let mse = y_true.iter().zip(y_pred.iter())
            .map(|(y, pred)| (y - pred).powi(2))
            .sum::<f64>() / n;

        mse.sqrt()
    }

    /// Calculate mean absolute error (MAE)
    ///
    /// # Arguments
    /// * `y_true` - True values
    /// * `y_pred` - Predicted values
    ///
    /// # Returns
    /// MAE value
    pub fn mae(y_true: &[f64], y_pred: &[f64]) -> f64 {
        if y_true.len() != y_pred.len() {
            return f64::INFINITY;
        }

        let n = y_true.len() as f64;
        y_true.iter().zip(y_pred.iter())
            .map(|(y, pred)| (y - pred).abs())
            .sum::<f64>() / n
    }

    /// Solve ridge regression: (X'X + lambda*I)^-1 X'y
    ///
    /// # Arguments
    /// * `x` - Design matrix
    /// * `y` - Response vector
    /// * `lambda` - Ridge regularization parameter
    ///
    /// # Returns
    /// Tuple of (coefficients, residual standard deviation)
    pub fn solve_ridge_regression(x: &Array2<f64>, y: &Array1<f64>, lambda: f64) -> Result<(Array1<f64>, f64), String> {
        let xt = x.t();
        let xtx = xt.dot(x);

        // Add lambda to diagonal
        let mut xtx_ridge = xtx.clone();
        for i in 0..xtx_ridge.nrows().min(xtx_ridge.ncols()) {
            xtx_ridge[[i, i]] += lambda;
        }

        let xty = xt.dot(y);
        let coeffs = LinearAlgebra::solve_linear_system(&xtx_ridge, &xty)?;

        // Calculate residuals
        let y_pred = x.dot(&coeffs);
        let residuals: Array1<f64> = y - &y_pred;
        let residual_std = residuals.as_slice().unwrap().std_dev();

        Ok((coeffs, residual_std))
    }
}