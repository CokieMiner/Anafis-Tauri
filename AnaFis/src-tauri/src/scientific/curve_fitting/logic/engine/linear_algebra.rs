use nalgebra::{DMatrix, DVector};

use crate::scientific::curve_fitting::logic::constants::MATRIX_SINGULAR_EPS;
use crate::scientific::curve_fitting::types::{OdrError, OdrResult};

pub fn solve_linear_system(matrix: DMatrix<f64>, rhs: &DVector<f64>) -> OdrResult<DVector<f64>> {
    let svd = matrix.svd(true, true);
    svd.solve(rhs, MATRIX_SINGULAR_EPS)
        .map_err(|error| OdrError::Numerical(format!("SVD solve failed: {error}")))
}

/// Inverts the information matrix to compute covariance.
///
/// # Errors
/// Returns `OdrError::Numerical` if the matrix is singular or inversion fails.
pub fn invert_information_matrix(matrix: DMatrix<f64>) -> OdrResult<DMatrix<f64>> {
    let svd = matrix.svd(true, true);
    svd.pseudo_inverse(MATRIX_SINGULAR_EPS)
        .map_err(|error| OdrError::Numerical(format!("Pseudo-inverse failed: {error}")))
}

pub fn invert_small_psd(covariance: &[Vec<f64>]) -> OdrResult<DMatrix<f64>> {
    let dim = covariance.len();
    let mut flat = Vec::with_capacity(dim * dim);
    for row in covariance {
        flat.extend(row.iter().copied());
    }
    let matrix = DMatrix::from_row_slice(dim, dim, &flat);
    let svd = matrix.svd(true, true);
    svd.pseudo_inverse(MATRIX_SINGULAR_EPS)
        .map_err(|error| OdrError::Numerical(format!("Point covariance inversion failed: {error}")))
}

pub fn sqrt_psd_matrix(matrix: &DMatrix<f64>) -> OdrResult<DMatrix<f64>> {
    let eigen = matrix.clone().symmetric_eigen();
    let dim = matrix.nrows();
    let mut sqrt_diag = DMatrix::<f64>::zeros(dim, dim);
    for idx in 0..dim {
        let lambda = eigen.eigenvalues[idx];
        if !lambda.is_finite() {
            return Err(OdrError::Numerical(
                "Non-finite eigenvalue found while building weighted residual blocks".to_string(),
            ));
        }
        sqrt_diag[(idx, idx)] = lambda.max(0.0).sqrt();
    }
    Ok(&eigen.eigenvectors * sqrt_diag * eigen.eigenvectors.transpose())
}
