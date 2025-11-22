//! Matrix Operations Module
//!
//! This module provides essential matrix operations for statistical analysis:
//! - Covariance matrix computation
//! - Principal Component Analysis (PCA)
//! - Matrix decomposition methods (delegated to primitives)
//!
//! TODO: Consider adding uncertainty support for PCA and covariance estimation:
//! - Bootstrap resampling for confidence intervals on PCA eigenvalues and loadings
//! - Robust PCA methods for handling measurement errors and outliers
//! - Probabilistic PCA for Bayesian uncertainty quantification
//! - Bootstrap covariance matrix estimation for small sample sizes
//! This would enable uncertainty-aware dimensionality reduction and covariance analysis.

pub mod types;
pub mod covariance;
pub mod pca;
pub mod decomposition;

// Re-export main API
pub use types::*;
pub use covariance::CovarianceOps;
pub use pca::PcaOps;
pub use decomposition::DecompositionOps;

use ndarray::{Array1, Array2};

/// Type alias for SVD result: (U, s, V^T) where A = U S V^T
type SvdResult = Result<(Array2<f64>, Array1<f64>, Array2<f64>), String>;

/// Matrix operations engine (delegates to submodules)
pub struct MatrixOpsEngine;

impl MatrixOpsEngine {
    /// Compute covariance matrix from data matrix
    /// Data should be (n_samples x n_features)
    pub fn covariance_matrix(
        data: &[Vec<f64>],
        ddof: usize,
    ) -> Result<Array2<f64>, String> {
        CovarianceOps::covariance_matrix(data, ddof)
    }

    /// Compute correlation matrix from covariance matrix
    pub fn correlation_matrix_from_covariance(cov: &Array2<f64>) -> Result<Array2<f64>, String> {
        CovarianceOps::correlation_matrix_from_covariance(cov)
    }

    /// Principal Component Analysis
    pub fn pca(
        data: &[Vec<f64>],
        n_components: Option<usize>,
    ) -> Result<PcaResult, String> {
        PcaOps::pca(data, n_components)
    }

    /// Eigenvalue decomposition using BLAS/LAPACK
    /// Returns (eigenvalues, eigenvectors) where eigenvectors are column-major
    pub fn eigenvalue_decomposition(matrix: &Array2<f64>) -> Result<(Vec<f64>, Array2<f64>), String> {
        DecompositionOps::eigenvalue_decomposition(matrix)
    }

    /// Singular Value Decomposition (SVD) using BLAS/LAPACK
    /// Returns (U, s, V^T) where A = U S V^T
    pub fn svd(matrix: &Array2<f64>) -> SvdResult {
        DecompositionOps::svd(matrix)
    }

    /// Matrix inversion using BLAS/LAPACK
    pub fn matrix_inverse(matrix: &Array2<f64>) -> Result<Array2<f64>, String> {
        DecompositionOps::matrix_inverse(matrix)
    }

    /// Cholesky decomposition using BLAS/LAPACK
    pub fn cholesky_decomposition(matrix: &Array2<f64>) -> Result<Array2<f64>, String> {
        DecompositionOps::cholesky_decomposition(matrix)
    }
}