use nalgebra::{DMatrix, DVector};

use super::state::{EvaluationState, MatrixDiagnostics};
use crate::scientific::curve_fitting::logic::constants::MATRIX_SINGULAR_EPS;

#[must_use]
/// Constructs the normal equations (`AtA` and `Atb`) from the Jacobian and residuals.
pub fn build_normal_equations(state: &EvaluationState) -> (DMatrix<f64>, DVector<f64>) {
    let j_t = state.global_weighted_jacobian.transpose();
    let normal = &j_t * &state.global_weighted_jacobian;
    let gradient = &j_t * &state.flat_weighted_residuals;
    (normal, gradient)
}

#[must_use]
/// Estimates effective rank and condition number using singular values.
pub fn diagnose_matrix(matrix: &DMatrix<f64>) -> MatrixDiagnostics {
    let svd = matrix.clone().svd(false, false);
    let singular_values = svd.singular_values;

    if singular_values.is_empty() {
        return MatrixDiagnostics {
            effective_rank: 0,
            condition_number: f64::INFINITY,
        };
    }

    let sigma_max = singular_values.iter().copied().fold(0.0, f64::max);
    let threshold = MATRIX_SINGULAR_EPS * sigma_max;

    let mut effective_rank = 0usize;
    let mut sigma_min_nonzero = f64::INFINITY;
    for sigma in singular_values.iter().copied() {
        if sigma > threshold {
            effective_rank += 1;
            sigma_min_nonzero = sigma_min_nonzero.min(sigma);
        }
    }

    let condition_number = if effective_rank == 0 || !sigma_min_nonzero.is_finite() {
        f64::INFINITY
    } else {
        sigma_max / sigma_min_nonzero
    };

    MatrixDiagnostics {
        effective_rank,
        condition_number,
    }
}
