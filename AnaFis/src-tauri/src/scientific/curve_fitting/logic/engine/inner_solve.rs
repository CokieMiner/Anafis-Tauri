//! Iterative inner solver for Orthogonal Distance Regression (ODR) stationarity.
//!
//! For a given set of model parameters (beta), the ODR objective requires finding
//! the optimal latent corrections (delta) that satisfy the stationarity condition:
//! `grad_delta(Chi^2)` = 0.
//!
//! This module implements a "Batched Newton" solver that solves these stationarity
//! equations for all data points simultaneously. This batching allows for
//! efficient SIMD vectorization of the model evaluations while maintaining
//! the theoretical rigor of the profiled ODR approach.

use super::{
    CORRECTION_VARIANCE_THRESHOLD, CompiledModel, INNER_CORRECTION_DAMPING,
    INNER_CORRECTION_MAX_ITERS, INNER_CORRECTION_TOLERANCE, OdrError, OdrResult, PreparedData,
    dependent_curvature_coefficient, evaluate_model_and_gradients_batch, extract_joint_covariance,
    invert_small_psd, solve_linear_system,
};
use nalgebra::{DMatrix, DVector};
use std::sync::Arc;

#[derive(Copy, Clone)]
/// Source of parameters for a batch solve.
pub enum ParameterSource<'data> {
    /// All points share the same local parameters (common for main evaluation).
    Shared(&'data [Vec<f64>]),
    /// Each point has its own local parameters (used for finite-difference perturbations).
    PerPoint(&'data [Vec<Vec<f64>>]),
}

/// Results for a batch of inner corrections.
pub struct MultiPointCorrectionResult {
    pub corrections: DMatrix<f64>, // (correction_count, batch_size)
    pub converged: Vec<bool>,
    pub covariance_regularization_count: usize,
    pub stationarity_norms: Vec<f64>,
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "Batched inner solve requires full context for lock-step iterations"
)]
pub fn solve_inner_corrections_multi_point(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    point_indices: &[usize],
    parameter_source: ParameterSource,
    dep_var_indices: &[usize],
    indep_var_indices: &[Vec<usize>],
    variable_to_correction_index: &[Option<usize>],
    layer_has_correctable_independent: &[bool],
    correction_count: usize,
) -> OdrResult<MultiPointCorrectionResult> {
    let batch_size = point_indices.len();
    if correction_count == 0 || batch_size == 0 {
        return Ok(MultiPointCorrectionResult {
            corrections: DMatrix::<f64>::zeros(correction_count, batch_size),
            converged: vec![true; batch_size],
            covariance_regularization_count: 0,
            stationarity_norms: vec![0.0; batch_size],
        });
    }

    let mut corrections = DMatrix::<f64>::zeros(correction_count, batch_size);
    let mut is_active = vec![true; batch_size];
    let mut converged = vec![false; batch_size];
    let mut stationarity_norms = vec![f64::INFINITY; batch_size];
    let mut covariance_regularization_count = 0_usize;

    // Pre-calculate joint weights for the entire batch.
    let mut layer_point_joint_weights: Vec<Vec<Option<DMatrix<f64>>>> =
        vec![vec![None; batch_size]; models.len()];

    for (layer_idx, _) in models.iter().enumerate() {
        if !layer_has_correctable_independent[layer_idx] {
            continue;
        }
        let dep_var_idx = dep_var_indices[layer_idx];
        let layer_indep_indices = &indep_var_indices[layer_idx];

        for (b_idx, &p_idx) in point_indices.iter().enumerate() {
            let has_correction = layer_indep_indices.iter().any(|&var_idx| {
                variable_to_correction_index[var_idx].is_some()
                    && data.point_covariances[p_idx][var_idx][var_idx]
                        > CORRECTION_VARIANCE_THRESHOLD
            });
            if !has_correction {
                continue;
            }

            let sigma_joint = extract_joint_covariance(
                &data.point_covariances[p_idx],
                layer_indep_indices,
                dep_var_idx,
            )?;
            if sigma_joint.was_regularized {
                covariance_regularization_count += 1;
            }
            layer_point_joint_weights[layer_idx][b_idx] =
                Some(invert_small_psd(&sigma_joint.matrix)?);
        }
    }

    for _iter in 0..INNER_CORRECTION_MAX_ITERS {
        let active_count = is_active.iter().filter(|&&a| a).count();
        if active_count == 0 {
            break;
        }

        let active_batch_indices: Vec<usize> = is_active
            .iter()
            .enumerate()
            .filter(|&(_, &a)| a)
            .map(|(i, _)| i)
            .collect();

        let mut layer_batch_eval_results = Vec::with_capacity(models.len());

        for (layer_idx, model) in models.iter().enumerate() {
            if !layer_has_correctable_independent[layer_idx] {
                layer_batch_eval_results.push(None);
                continue;
            }

            let layer_indep_indices = &indep_var_indices[layer_idx];
            let num_indep = layer_indep_indices.len();
            let num_params = model.parameter_names.len();

            let mut columns = vec![vec![0.0; active_count]; num_indep + num_params];
            for (sub_idx, &b_idx) in active_batch_indices.iter().enumerate() {
                let p_idx = point_indices[b_idx];
                for (col_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    let val = if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[p_idx][var_idx][var_idx]
                            > CORRECTION_VARIANCE_THRESHOLD
                    {
                        data.variable_values[var_idx][p_idx] + corrections[(corr_idx, b_idx)]
                    } else {
                        data.variable_values[var_idx][p_idx]
                    };
                    columns[col_idx][sub_idx] = val;
                }
                for p_local_idx in 0..num_params {
                    let val = match &parameter_source {
                        ParameterSource::Shared(s) => s[layer_idx][p_local_idx],
                        ParameterSource::PerPoint(p) => p[b_idx][layer_idx][p_local_idx],
                    };
                    columns[num_indep + p_local_idx][sub_idx] = val;
                }
            }

            let column_refs: Vec<&[f64]> = columns.iter().map(|c| &c[..]).collect();
            let batch_res = evaluate_model_and_gradients_batch(
                &model.model_expr,
                &model.independent_gradient_exprs,
                &model.parameter_gradient_exprs,
                &model.independent_names,
                &model.parameter_names,
                &column_refs,
                layer_idx,
            )?;
            layer_batch_eval_results.push(Some(batch_res));
        }

        for (sub_idx, &b_idx) in active_batch_indices.iter().enumerate() {
            let p_idx = point_indices[b_idx];
            let mut point_gradient = DVector::<f64>::zeros(correction_count);
            let mut point_hessian = DMatrix::<f64>::zeros(correction_count, correction_count);

            for (layer_idx, model) in models.iter().enumerate() {
                let Some(batch_res) = &layer_batch_eval_results[layer_idx] else {
                    continue;
                };
                let Some(weight_joint) = &layer_point_joint_weights[layer_idx][b_idx] else {
                    continue;
                };

                let layer_indep_indices = &indep_var_indices[layer_idx];
                let local_independent_count = layer_indep_indices.len();
                let block_dim = local_independent_count + 1;

                let fitted = batch_res.fitted_values[sub_idx];
                let residual = data.variable_values[dep_var_indices[layer_idx]][p_idx] - fitted;

                let mut joint_residual = DVector::<f64>::zeros(block_dim);
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[p_idx][var_idx][var_idx]
                            > CORRECTION_VARIANCE_THRESHOLD
                    {
                        joint_residual[local_idx] = -corrections[(corr_idx, b_idx)];
                    }
                }
                joint_residual[local_independent_count] = residual;

                let mut j_corrections = DMatrix::<f64>::zeros(block_dim, correction_count);
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[p_idx][var_idx][var_idx]
                            > CORRECTION_VARIANCE_THRESHOLD
                    {
                        j_corrections[(local_idx, corr_idx)] = -1.0;
                        j_corrections[(local_independent_count, corr_idx)] =
                            -batch_res.independent_derivatives[local_idx][sub_idx];
                    }
                }

                let weighted_residual = weight_joint * &joint_residual;
                point_gradient += j_corrections.transpose() * &weighted_residual;
                point_hessian += j_corrections.transpose() * weight_joint * &j_corrections;

                let second_order_coefficient = dependent_curvature_coefficient(
                    weight_joint,
                    &joint_residual,
                    local_independent_count,
                );
                if second_order_coefficient.abs() > 0.0 {
                    let local_args: Vec<f64> = (0..local_independent_count)
                        .map(|col_idx| {
                            let var_idx = layer_indep_indices[col_idx];
                            if let Some(corr_idx) = variable_to_correction_index[var_idx]
                                && data.point_covariances[p_idx][var_idx][var_idx]
                                    > CORRECTION_VARIANCE_THRESHOLD
                            {
                                data.variable_values[var_idx][p_idx]
                                    + corrections[(corr_idx, b_idx)]
                            } else {
                                data.variable_values[var_idx][p_idx]
                            }
                        })
                        .chain(match &parameter_source {
                            ParameterSource::Shared(s) => s[layer_idx].iter().copied(),
                            ParameterSource::PerPoint(p) => p[b_idx][layer_idx].iter().copied(),
                        })
                        .collect();

                    let local_hessian = evaluate_model_hessian_wrt_independents(
                        model,
                        &local_args,
                        local_independent_count,
                    )?;
                    for local_row in 0..local_independent_count {
                        let Some(global_row) =
                            variable_to_correction_index[layer_indep_indices[local_row]]
                        else {
                            continue;
                        };
                        if data.point_covariances[p_idx][layer_indep_indices[local_row]]
                            [layer_indep_indices[local_row]]
                            <= CORRECTION_VARIANCE_THRESHOLD
                        {
                            continue;
                        }
                        for local_col in 0..local_independent_count {
                            let Some(global_col) =
                                variable_to_correction_index[layer_indep_indices[local_col]]
                            else {
                                continue;
                            };
                            if data.point_covariances[p_idx][layer_indep_indices[local_col]]
                                [layer_indep_indices[local_col]]
                                <= CORRECTION_VARIANCE_THRESHOLD
                            {
                                continue;
                            }
                            point_hessian[(global_row, global_col)] -=
                                second_order_coefficient * local_hessian[(local_row, local_col)];
                        }
                    }
                }
            }

            let stationarity_norm = point_gradient.norm();
            stationarity_norms[b_idx] = stationarity_norm;

            if stationarity_norm < INNER_CORRECTION_TOLERANCE {
                converged[b_idx] = true;
                is_active[b_idx] = false;
                continue;
            }

            let neg_gradient = -&point_gradient;
            let step =
                if let Ok(solution) = solve_linear_system(point_hessian.clone(), &neg_gradient) {
                    solution
                } else {
                    let mut regularized = point_hessian;
                    let max_diag = (0..correction_count)
                        .map(|i| regularized[(i, i)].abs())
                        .fold(0.0, f64::max)
                        .max(1.0);
                    let damping = INNER_CORRECTION_DAMPING * max_diag;
                    for i in 0..correction_count {
                        regularized[(i, i)] += damping;
                    }
                    solve_linear_system(regularized, &neg_gradient)?
                };

            for c_idx in 0..correction_count {
                corrections[(c_idx, b_idx)] += step[c_idx];
            }
        }
    }

    Ok(MultiPointCorrectionResult {
        corrections,
        converged,
        covariance_regularization_count,
        stationarity_norms,
    })
}

pub fn evaluate_model_hessian_wrt_independents(
    model: &CompiledModel,
    args: &[f64],
    independent_count: usize,
) -> OdrResult<DMatrix<f64>> {
    let mut hessian = DMatrix::<f64>::zeros(independent_count, independent_count);
    if independent_count == 0 {
        return Ok(hessian);
    }

    for row in 0..independent_count {
        for col in 0..independent_count {
            let idx = row * independent_count + col;
            let value = model.independent_hessian_evaluators[idx].evaluate(args);
            if !value.is_finite() {
                return Err(OdrError::Numerical(
                    "Non-finite value while evaluating symbolic independent-variable Hessian"
                        .to_owned(),
                ));
            }
            hessian[(row, col)] = value;
        }
    }

    // Enforce symmetry of mixed partials for numerical robustness.
    for row in 0..independent_count {
        for col in (row + 1)..independent_count {
            let sym = 0.5 * (hessian[(row, col)] + hessian[(col, row)]);
            hessian[(row, col)] = sym;
            hessian[(col, row)] = sym;
        }
    }

    Ok(hessian)
}
