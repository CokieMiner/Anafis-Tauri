//! ndarray-specific linear algebra optimizations
//!
//! This module provides linear algebra operations optimized for ndarray
//! with additional functionality for large matrices and memory efficiency.

use ndarray::{Array2, Array1, Axis};
use super::linear_algebra::{LinearAlgebra, SvdResult};

/// Type alias for ndarray SVD result
pub type NdSvdResult = SvdResult;

/// High-performance linear algebra operations using ndarray ecosystem
pub struct NdLinearAlgebra;

impl NdLinearAlgebra {
    /// Matrix multiplication: C = A * B using ndarray
    pub fn matrix_multiply(a: &Array2<f64>, b: &Array2<f64>) -> Result<Array2<f64>, String> {
        LinearAlgebra::matrix_multiply(a, b)
    }

    /// Matrix-vector multiplication: y = A * x using ndarray
    pub fn matrix_vector_multiply(matrix: &Array2<f64>, vector: &Array1<f64>) -> Result<Array1<f64>, String> {
        LinearAlgebra::matrix_vector_multiply(matrix, vector)
    }

    /// Cholesky decomposition for positive definite matrices using ndarray-linalg BLAS/LAPACK
    pub fn cholesky_decomposition(matrix: &Array2<f64>) -> Result<Array2<f64>, String> {
        LinearAlgebra::cholesky_decomposition(matrix)
    }

    /// Eigenvalue decomposition for symmetric matrices using ndarray-linalg BLAS/LAPACK
    pub fn eigenvalue_decomposition(matrix: &Array2<f64>) -> Result<(Array1<f64>, Array2<f64>), String> {
        LinearAlgebra::eigenvalue_decomposition(matrix)
    }

    /// Singular value decomposition using ndarray-linalg BLAS/LAPACK
    pub fn svd(matrix: &Array2<f64>) -> Result<NdSvdResult, String> {
        LinearAlgebra::svd(matrix)
    }

    /// Solve linear system A * x = b using ndarray-linalg BLAS/LAPACK
    pub fn solve_linear_system(a: &Array2<f64>, b: &Array1<f64>) -> Result<Array1<f64>, String> {
        LinearAlgebra::solve_linear_system(a, b)
    }

    /// Check if matrix is symmetric
    pub fn is_symmetric(matrix: &Array2<f64>) -> bool {
        LinearAlgebra::is_symmetric(matrix)
    }

    /// Check if matrix is symmetric positive definite
    pub fn is_symmetric_positive_definite(matrix: &Array2<f64>) -> bool {
        LinearAlgebra::is_symmetric_positive_definite(matrix)
    }

    /// Matrix inverse using ndarray
    pub fn matrix_inverse(matrix: &Array2<f64>) -> Result<Array2<f64>, String> {
        LinearAlgebra::matrix_inverse(matrix)
    }

    /// Matrix determinant using ndarray
    pub fn determinant(matrix: &Array2<f64>) -> Result<f64, String> {
        LinearAlgebra::determinant(matrix)
    }

    /// Matrix trace using ndarray
    pub fn trace(matrix: &Array2<f64>) -> f64 {
        LinearAlgebra::trace(matrix)
    }

    /// Covariance matrix computation using ndarray (more memory efficient for large datasets)
    pub fn covariance_matrix(data: &Array2<f64>) -> Result<Array2<f64>, String> {
        if data.nrows() < 2 {
            return Err("Need at least 2 observations for covariance".to_string());
        }

        let n = data.nrows() as f64;
        let mean = data.mean_axis(Axis(0)).ok_or("Failed to compute mean")?;

        // Center the data using broadcasting
        let centered = data - &mean.insert_axis(Axis(0));

        // Compute covariance matrix
        let cov = centered.t().dot(&centered) / (n - 1.0);
        Ok(cov)
    }

    /// Efficient matrix operations for very large matrices using ndarray's memory layout
    pub fn large_matrix_multiply(a: &Array2<f64>, b: &Array2<f64>) -> Result<Array2<f64>, String> {
        // ndarray's dot product is optimized for large matrices
        Self::matrix_multiply(a, b)
    }

    /// Memory-efficient eigenvalue computation for large symmetric matrices
    pub fn large_eigenvalue_decomposition(matrix: &Array2<f64>) -> Result<(Array1<f64>, Array2<f64>), String> {
        // Use parallel eigenvalue decomposition for large matrices
        LinearAlgebra::parallel_eigenvalue_decomposition(matrix)
    }
}