//! Matrix Operations Types
//!
//! Type definitions for matrix operations and linear algebra results.

use serde::{Deserialize, Serialize};

/// Result of PCA analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcaResult {
    pub eigenvalues: Vec<f64>,
    pub eigenvectors: Vec<Vec<f64>>, // Each inner vec is an eigenvector
    pub explained_variance_ratio: Vec<f64>,
    pub projected_data: Vec<Vec<f64>>, // Projected data points
    pub n_components: usize,
}

/// Singular Value Decomposition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvdResult {
    pub u: Vec<Vec<f64>>, // Left singular vectors
    pub s: Vec<f64>,      // Singular values
    pub vt: Vec<Vec<f64>>, // Right singular vectors (transposed)
}

/// Matrix inversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixInverseResult {
    pub inverse: Vec<Vec<f64>>,
    pub condition_number: Option<f64>,
}

/// Cholesky decomposition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CholeskyResult {
    pub l: Vec<Vec<f64>>, // Lower triangular matrix
}

/// Eigenvalue decomposition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EigenDecompositionResult {
    pub eigenvalues: Vec<f64>,
    pub eigenvectors: Vec<Vec<f64>>,
}

/// Linear system solution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearSystemSolution {
    pub solution: Vec<f64>,
    pub residual_norm: Option<f64>,
}