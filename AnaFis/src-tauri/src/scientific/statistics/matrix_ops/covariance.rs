//! Covariance and correlation matrix operations

use ndarray::Array2;
use crate::scientific::statistics::primitives::LinearAlgebra;

/// Covariance operations
pub struct CovarianceOps;

impl CovarianceOps {
    /// Compute covariance matrix from data matrix
    /// Data should be (n_samples x n_features)
    pub fn covariance_matrix(
        data: &[Vec<f64>],
        ddof: usize, // Delta degrees of freedom (0 for population, 1 for sample)
    ) -> Result<Array2<f64>, String> {
        if data.is_empty() {
            return Err("Empty data".to_string());
        }

        let n_samples = data.len();
        let n_features = data[0].len();

        if n_samples < 2 {
            return Err("Need at least 2 samples for covariance".to_string());
        }

        // Convert to ndarray
        let matrix = Array2::from_shape_vec(
            (n_samples, n_features),
            data.iter().flatten().cloned().collect(),
        )
        .map_err(|e| format!("Failed to create ndarray from data: {}", e))?;

        // Use the centralized LinearAlgebra implementation
        LinearAlgebra::covariance_matrix_with_ddof(&matrix, ddof as f64)
    }

    /// Compute correlation matrix from covariance matrix
    pub fn correlation_matrix_from_covariance(cov: &Array2<f64>) -> Result<Array2<f64>, String> {
        if !cov.is_square() {
            return Err("Covariance matrix must be square".to_string());
        }

        let n = cov.nrows();
        let mut corr = Array2::zeros((n, n));

        // Extract standard deviations (square root of diagonal elements)
        let stds: Vec<f64> = (0..n)
            .map(|i| cov[[i, i]].sqrt())
            .collect();

        for i in 0..n {
            for j in 0..n {
                if stds[i] > 0.0 && stds[j] > 0.0 {
                    corr[[i, j]] = cov[[i, j]] / (stds[i] * stds[j]);
                } else {
                    corr[[i, j]] = 0.0; // Handle zero variance
                }
            }
        }

        Ok(corr)
    }
}
