//! Curvature and implicit derivative logic for Orthogonal Distance Regression (ODR).
//!
//! This module implements the "implicit correction tensor" logic required for
//! second-order ODR. Because the latent corrections (delta) are determined by
//! an iterative inner stationarity solve, their second derivatives with respect
//! to the model parameters (beta) cannot be easily computed symbolically.
//!
//! We use a central finite-difference scheme to estimate the tensor d^2c*/dbeta^2,
//! which is essential for correcting the bias in the parameter covariance matrix
//! when the model is highly non-linear or the uncertainties are large.
//!
//! # Omissions
//! - **Independent-Residual Terms**: This implementation omits the Hessian terms arising from
//!   the $-δ$ part of the residual vector. This simplification is standard for many ODR
//!   engines and is typically valid when the residuals are small relative to the curvature
//!   of the model surface.

use super::{
    CompiledModel, MIN_VARIANCE, OdrError, OdrResult, ParameterSource, PreparedData,
    is_positive_semidefinite, solve_inner_corrections_multi_point,
};
use nalgebra::{DMatrix, DVector};
use std::sync::Arc;

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::similar_names,
    reason = "Finite-difference implicit correction tensor needs full local context; beta_pp/pm/mp/mm name the four stencil corners"
)]
/// Computes a finite-difference approximation of the implicit correction-map tensor
/// d2(c*) / d(beta)^2 for one data point.
///
/// Complexity: for p parameters this requires O(p^2) parameter-pair evaluations, and each
/// pair evaluates the inner correction solve four times (++, +-, -+, --). This is accurate
/// but expensive for large p.
///
/// Why not direct CAS here:
/// - Symbolic CAS derivatives are used for explicit model derivatives `f_x`, `f_p`,
///   `f_xx`, `f_xp`, `f_pp`.
/// - c*(beta) itself is implicit: it is defined as the root of the inner stationarity system,
///   which includes covariance-coupled solves and numerical regularization branches.
/// - A fully symbolic d2(c*)/d(beta)^2 path would require differentiating the implicit linear
///   solve (effectively third-order model derivatives plus derivative-of-inverse terms) and
///   carrying the same branch logic, which is considerably more complex and fragile.
pub fn compute_second_derivative_corrections_numerical(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    point_idx: usize,
    dep_var_indices: &[usize],
    indep_var_indices: &[Vec<usize>],
    local_parameters_per_layer: &[Vec<f64>],
    global_parameter_indices_per_layer: &[Vec<usize>],
    variable_to_correction_index: &[Option<usize>],
    layer_has_correctable_independent: &[bool],
    correction_count: usize,
    global_parameters: &[f64],
) -> OdrResult<(Vec<DMatrix<f64>>, usize)> {
    let p = global_parameters.len();
    let mut d2 = vec![DMatrix::<f64>::zeros(p, p); correction_count];
    if correction_count == 0 || p == 0 {
        return Ok((d2, 0));
    }

    let step_for = |value: f64| (1e-5_f64 * value.abs().max(1.0)).max(1e-8);
    let mut unconverged_count = 0_usize;

    for i in 0..p {
        let hi = step_for(global_parameters[i]);
        for j in i..p {
            let hj = step_for(global_parameters[j]);

            let mut beta_pp = global_parameters.to_vec();
            beta_pp[i] += hi;
            beta_pp[j] += hj;
            let mut beta_pm = global_parameters.to_vec();
            beta_pm[i] += hi;
            beta_pm[j] -= hj;
            let mut beta_mp = global_parameters.to_vec();
            beta_mp[i] -= hi;
            beta_mp[j] += hj;
            let mut beta_mm = global_parameters.to_vec();
            beta_mm[i] -= hi;
            beta_mm[j] -= hj;

            let stencils = [beta_pp, beta_pm, beta_mp, beta_mm];
            let stencil_params: Vec<Vec<Vec<f64>>> = stencils
                .iter()
                .map(|beta| {
                    local_parameters_from_global(
                        beta,
                        local_parameters_per_layer,
                        global_parameter_indices_per_layer,
                    )
                })
                .collect();

            let batch_res = solve_inner_corrections_multi_point(
                models,
                data,
                &[point_idx; 4],
                ParameterSource::PerPoint(&stencil_params),
                dep_var_indices,
                indep_var_indices,
                variable_to_correction_index,
                layer_has_correctable_independent,
                correction_count,
            )?;

            let all_converged = batch_res.converged.iter().all(|&c| c);

            if all_converged {
                let c_pp = batch_res.corrections.column(0);
                let c_pm = batch_res.corrections.column(1);
                let c_mp = batch_res.corrections.column(2);
                let c_mm = batch_res.corrections.column(3);

                let denom = 4.0 * hi * hj;
                for k in 0..correction_count {
                    let val = (c_pp[k] - c_pm[k] - c_mp[k] + c_mm[k]) / denom;
                    d2[k][(i, j)] = val;
                    if i != j {
                        d2[k][(j, i)] = val;
                    }
                }
            } else {
                // If any of the four perturbed inner solves failed to converge,
                // the FD quotient is unreliable — zero out this (i,j) entry to
                // avoid injecting noise into the outer curvature correction.
                unconverged_count += 1;
            }
        }
    }

    Ok((d2, unconverged_count))
}

fn local_parameters_from_global(
    global_parameters: &[f64],
    base_local_parameters: &[Vec<f64>],
    global_parameter_indices_per_layer: &[Vec<usize>],
) -> Vec<Vec<f64>> {
    let mut locals = base_local_parameters.to_vec();
    for (layer_idx, layer_global_indices) in global_parameter_indices_per_layer.iter().enumerate() {
        for (local_idx, &global_idx) in layer_global_indices.iter().enumerate() {
            locals[layer_idx][local_idx] = global_parameters[global_idx];
        }
    }
    locals
}

pub fn dependent_curvature_coefficient(
    weight_joint: &DMatrix<f64>,
    joint_residual: &DVector<f64>,
    dependent_row: usize,
) -> f64 {
    // For residual vector r = [delta_x..., r_dep], this returns
    // (W_joint * r) evaluated at the dependent-residual row, i.e. the
    // coefficient multiplying second-derivative terms in the local profiled
    // curvature correction.
    (0..joint_residual.len())
        .map(|idx| weight_joint[(dependent_row, idx)] * joint_residual[idx])
        .sum()
}

pub struct JointCovarianceBlock {
    pub matrix: Vec<Vec<f64>>,
    pub was_regularized: bool,
}

pub fn extract_joint_covariance(
    covariance: &[Vec<f64>],
    independent_indices: &[usize],
    dependent_index: usize,
) -> OdrResult<JointCovarianceBlock> {
    let dim = independent_indices.len() + 1;
    let mut block = vec![vec![0.0; dim]; dim];
    for (row_local, &row_global) in independent_indices.iter().enumerate() {
        for (col_local, &col_global) in independent_indices.iter().enumerate() {
            block[row_local][col_local] = covariance[row_global][col_global];
        }
        block[row_local][row_local] = block[row_local][row_local].max(MIN_VARIANCE);
        block[row_local][dim - 1] = covariance[row_global][dependent_index];
        block[dim - 1][row_local] = covariance[dependent_index][row_global];
    }
    block[dim - 1][dim - 1] = covariance[dependent_index][dependent_index].max(MIN_VARIANCE);

    if is_positive_semidefinite(&block) {
        return Ok(JointCovarianceBlock {
            matrix: block,
            was_regularized: false,
        });
    }

    let mut regularized = block;
    let mut jitter = MIN_VARIANCE;
    for _ in 0..8 {
        #[allow(
            clippy::needless_range_loop,
            reason = "Diagonal indexing requires row == col"
        )]
        for diagonal in 0..dim {
            regularized[diagonal][diagonal] += jitter;
        }
        if is_positive_semidefinite(&regularized) {
            return Ok(JointCovarianceBlock {
                matrix: regularized,
                was_regularized: true,
            });
        }
        jitter *= 10.0;
    }

    Err(OdrError::Numerical(
        "Extracted joint covariance block is not PSD after regularization".to_owned(),
    ))
}
