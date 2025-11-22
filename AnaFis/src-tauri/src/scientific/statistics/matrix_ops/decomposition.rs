//! Matrix decomposition operations
//!
//! Wrappers for BLAS/LAPACK decomposition methods

use ndarray::{Array1, Array2};
use crate::scientific::statistics::primitives::LinearAlgebra;

/// Type alias for SVD result: (U, s, V^T) where A = U S V^T
type SvdResult = Result<(Array2<f64>, Array1<f64>, Array2<f64>), String>;

/// Matrix decomposition operations
pub struct DecompositionOps;

impl DecompositionOps {
    /// Eigenvalue decomposition using BLAS/LAPACK
    /// Returns (eigenvalues, eigenvectors) where eigenvectors are column-major
    pub fn eigenvalue_decomposition(matrix: &Array2<f64>) -> Result<(Vec<f64>, Array2<f64>), String> {
        let (eigenvalues, eigenvectors) = LinearAlgebra::eigenvalue_decomposition(matrix)?;
        Ok((eigenvalues.to_vec(), eigenvectors))
    }

    /// Singular Value Decomposition (SVD) using BLAS/LAPACK
    /// Returns (U, s, V^T) where A = U S V^T
    pub fn svd(matrix: &Array2<f64>) -> SvdResult {
        LinearAlgebra::svd(matrix)
    }

    /// Matrix inversion using BLAS/LAPACK
    pub fn matrix_inverse(matrix: &Array2<f64>) -> Result<Array2<f64>, String> {
        LinearAlgebra::matrix_inverse(matrix)
    }

    /// Cholesky decomposition using BLAS/LAPACK
    pub fn cholesky_decomposition(matrix: &Array2<f64>) -> Result<Array2<f64>, String> {
        LinearAlgebra::cholesky_decomposition(matrix)
    }
}
