//! Core linear algebra operations
//!
//! This module provides fundamental linear algebra operations
//! using BLAS/LAPACK through ndarray-linalg.

use ndarray::{Array2, Array1, s};
use ndarray_linalg::cholesky::Cholesky;
use ndarray_linalg::eigh::Eigh;
use ndarray_linalg::svd::SVD;
use ndarray_linalg::solve::{Solve, Inverse, Determinant};
use ndarray_linalg::UPLO;

/// Type alias for SVD decomposition result
pub type SvdResult = (Array2<f64>, Array1<f64>, Array2<f64>);

/// High-performance linear algebra operations
pub struct LinearAlgebra;

impl LinearAlgebra {
    /// Matrix multiplication: C = A * B
    pub fn matrix_multiply(a: &Array2<f64>, b: &Array2<f64>) -> Result<Array2<f64>, String> {
        if a.ncols() != b.nrows() {
            return Err("Matrix dimension mismatch for multiplication".to_string());
        }
        Ok(a.dot(b))
    }

    /// Matrix-vector multiplication: y = A * x
    pub fn matrix_vector_multiply(matrix: &Array2<f64>, vector: &Array1<f64>) -> Result<Array1<f64>, String> {
        if matrix.ncols() != vector.len() {
            return Err("Matrix-vector dimension mismatch".to_string());
        }
        Ok(matrix.dot(vector))
    }

    /// Cholesky decomposition for positive definite matrices
    pub fn cholesky_decomposition(matrix: &Array2<f64>) -> Result<Array2<f64>, String> {
        Cholesky::cholesky(matrix, UPLO::Lower)
            .map_err(|_| "Matrix is not symmetric positive definite".to_string())
    }

    /// Eigenvalue decomposition for symmetric matrices
    pub fn eigenvalue_decomposition(matrix: &Array2<f64>) -> Result<(Array1<f64>, Array2<f64>), String> {
        if !Self::is_symmetric(matrix) {
            return Err("Matrix is not symmetric".to_string());
        }

        Eigh::eigh(matrix, UPLO::Lower)
            .map_err(|_| "Eigenvalue decomposition failed".to_string())
    }

    /// Parallel eigenvalue decomposition for large symmetric matrices
    /// ndarray-linalg automatically uses parallel BLAS/LAPACK when available
    pub fn parallel_eigenvalue_decomposition(matrix: &Array2<f64>) -> Result<(Array1<f64>, Array2<f64>), String> {
        // Rely on ndarray-linalg's BLAS/LAPACK backend to handle parallelism automatically
        Self::eigenvalue_decomposition(matrix)
    }

    /// Singular value decomposition
    pub fn svd(matrix: &Array2<f64>) -> Result<SvdResult, String> {
        // Using ndarray-linalg's SVD implementation with BLAS/LAPACK
        let svd = SVD::svd(matrix, true, true)
            .map_err(|_| "SVD decomposition failed".to_string())?;

        let u = svd.0.unwrap_or_else(|| Array2::eye(matrix.nrows()));
        let v_t = svd.2.unwrap_or_else(|| Array2::eye(matrix.ncols()));

        // Return thin SVD: U is m×min(m,n), V^T is min(m,n)×n
        let min_dim = matrix.nrows().min(matrix.ncols());
        let u_thin = u.slice(s![.., 0..min_dim]).to_owned();
        let vt_thin = v_t.slice(s![0..min_dim, ..]).to_owned();

        Ok((u_thin, svd.1, vt_thin))
    }

    /// Solve linear system A * x = b
    pub fn solve_linear_system(a: &Array2<f64>, b: &Array1<f64>) -> Result<Array1<f64>, String> {
        // Use ndarray-linalg's solve with BLAS/LAPACK backend
        Solve::solve(a, b)
            .map_err(|_| "Linear system is singular or ill-conditioned".to_string())
    }

    /// Check if matrix is symmetric
    pub fn is_symmetric(matrix: &Array2<f64>) -> bool {
        Self::is_symmetric_with_tolerance(matrix, 1e-12)
    }

    /// Check if matrix is symmetric with configurable tolerance
    pub fn is_symmetric_with_tolerance(matrix: &Array2<f64>, tolerance: f64) -> bool {
        let n = matrix.nrows();
        if matrix.ncols() != n {
            return false;
        }

        for i in 0..n {
            for j in (i + 1)..n {
                if (matrix[[i, j]] - matrix[[j, i]]).abs() > tolerance {
                    return false;
                }
            }
        }
        true
    }

    /// Check if matrix is symmetric positive definite
    pub fn is_symmetric_positive_definite(matrix: &Array2<f64>) -> bool {
        if !Self::is_symmetric(matrix) {
            return false;
        }

        // Try Cholesky decomposition - if it succeeds, matrix is positive definite
        // This is much faster than computing all eigenvalues
        Self::cholesky_decomposition(matrix).is_ok()
    }

    /// Matrix inverse
    pub fn matrix_inverse(matrix: &Array2<f64>) -> Result<Array2<f64>, String> {
        Inverse::inv(matrix)
            .map_err(|_| "Matrix is singular".to_string())
    }

    /// Matrix determinant
    pub fn determinant(matrix: &Array2<f64>) -> Result<f64, String> {
        Determinant::det(matrix)
            .map_err(|_| "Determinant calculation failed".to_string())
    }

    /// Matrix trace
    pub fn trace(matrix: &Array2<f64>) -> f64 {
        matrix.diag().sum()
    }
}