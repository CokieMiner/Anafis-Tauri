//! Multi-layer ODR model evaluation and Jacobian assembly.
//!
//! This module orchestrates the evaluation of complex models consisting of multiple
//! linked layers. It performs the following roles:
//! 1. Coordinates the inner stationarity solve for latent corrections.
//! 2. Assembles the "augmented" Jacobian matrix (`J_beta` + `J_delta` * `dc/dbeta`).
//! 3. Computes the second-order corrections to the Hessian for GUM compliance.
//! 4. Aggregates chi-squared metrics and effective degrees-of-freedom using
//!    the Welch-Satterthwaite formula.

use super::{
    CORRECTION_VARIANCE_THRESHOLD, CompiledModel, EvaluationState, INNER_CORRECTION_DAMPING,
    MIN_VARIANCE, OdrError, OdrResult, ParameterSource, PreparedData,
    compute_second_derivative_corrections_numerical, dependent_curvature_coefficient,
    evaluate_hessian_exprs_batch, evaluate_model_and_gradients_batch, extract_joint_covariance,
    invert_small_psd, solve_inner_corrections_multi_point, solve_linear_system_matrix,
    sqrt_psd_matrix,
};
use nalgebra::{DMatrix, DVector};
use std::sync::Arc;

/// Evaluates the multi-layer model at the current global parameters.
#[allow(
    clippy::too_many_lines,
    reason = "Multi-layer ODR evaluation requires comprehensive logic"
)]
pub fn evaluate_model(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    global_parameters: &[f64],
    global_parameter_names: &[String],
) -> OdrResult<EvaluationState> {
    let point_count = data.point_count;
    let global_parameter_count = global_parameters.len();

    let mut chi_squared = 0.0;
    let mut chi_squared_observation = 0.0;

    let mut layer_residuals: Vec<Vec<f64>> = (0..models.len())
        .map(|_| Vec::with_capacity(point_count))
        .collect();
    let mut layer_fitted_values: Vec<Vec<f64>> = (0..models.len())
        .map(|_| Vec::with_capacity(point_count))
        .collect();
    let mut inner_correction_nonconverged_points = 0_usize;
    let mut covariance_regularization_count = 0_usize;
    let mut inner_stationarity_norm_max = 0.0_f64;
    let mut inner_stationarity_norm_sum = 0.0_f64;
    let mut inner_stationarity_samples = 0_usize;
    let mut fd_tensor_unconverged_perturbations = 0_usize;

    let mut flat_weighted_residuals: Vec<f64> = Vec::with_capacity(point_count * models.len() * 4);
    let mut global_weighted_jacobian: Vec<f64> =
        Vec::with_capacity(point_count * models.len() * 4 * global_parameter_count);
    let mut outer_second_order_normal =
        DMatrix::<f64>::zeros(global_parameter_count, global_parameter_count);
    let mut ws_uc2 = 0.0_f64;
    let mut ws_denominator = 0.0_f64;

    let mut dep_var_indices = Vec::with_capacity(models.len());
    let mut indep_var_indices = Vec::with_capacity(models.len());
    let mut local_parameters_per_layer = Vec::with_capacity(models.len());
    let mut global_parameter_indices_per_layer = Vec::with_capacity(models.len());
    let mut variable_to_correction_index: Vec<Option<usize>> =
        vec![None; data.variable_names.len()];
    let mut correction_variable_indices: Vec<usize> = Vec::new();
    let mut layer_has_correctable_independent = Vec::with_capacity(models.len());

    for model in models {
        let dep_var_idx = data
            .variable_names
            .iter()
            .position(|name| name == &model.dependent_name)
            .ok_or_else(|| {
                OdrError::Validation(format!(
                    "Dependent variable {} not found in data",
                    model.dependent_name
                ))
            })?;
        dep_var_indices.push(dep_var_idx);

        let mut indep_indices = Vec::with_capacity(model.independent_names.len());
        let mut has_correctable_independent = false;
        for name in &model.independent_names {
            let idx = data
                .variable_names
                .iter()
                .position(|n| n == name)
                .ok_or_else(|| {
                    OdrError::Validation(format!("Independent variable {name} not found in data"))
                })?;
            indep_indices.push(idx);
            let has_uncertainty = (0..point_count).any(|point| {
                data.point_covariances[point][idx][idx] > CORRECTION_VARIANCE_THRESHOLD
            });
            if has_uncertainty {
                has_correctable_independent = true;
            }
            if has_uncertainty && variable_to_correction_index[idx].is_none() {
                let next = correction_variable_indices.len();
                variable_to_correction_index[idx] = Some(next);
                correction_variable_indices.push(idx);
            }
        }
        indep_var_indices.push(indep_indices);
        layer_has_correctable_independent.push(has_correctable_independent);

        let mut local_parameters = Vec::with_capacity(model.parameter_names.len());
        let mut param_global_indices = Vec::with_capacity(model.parameter_names.len());

        for local_name in &model.parameter_names {
            let global_idx = global_parameter_names
                .iter()
                .position(|name| name == local_name)
                .ok_or_else(|| {
                    OdrError::Validation(format!(
                        "Parameter {local_name} not found in global parameters"
                    ))
                })?;
            local_parameters.push(global_parameters[global_idx]);
            param_global_indices.push(global_idx);
        }
        local_parameters_per_layer.push(local_parameters);
        global_parameter_indices_per_layer.push(param_global_indices);
    }

    let point_indices: Vec<usize> = (0..point_count).collect();

    let multi_correction_result = solve_inner_corrections_multi_point(
        models,
        data,
        &point_indices,
        ParameterSource::Shared(&local_parameters_per_layer),
        &dep_var_indices,
        &indep_var_indices,
        &variable_to_correction_index,
        &layer_has_correctable_independent,
        correction_variable_indices.len(),
    )?;

    covariance_regularization_count += multi_correction_result.covariance_regularization_count;
    for &norm in &multi_correction_result.stationarity_norms {
        inner_stationarity_norm_max = inner_stationarity_norm_max.max(norm);
        inner_stationarity_norm_sum += norm;
        inner_stationarity_samples += 1;
    }
    for &is_converged in &multi_correction_result.converged {
        if !is_converged {
            inner_correction_nonconverged_points += 1;
        }
    }

    let mut layer_batch_results = Vec::with_capacity(models.len());
    let mut layer_independent_hessian_results = Vec::with_capacity(models.len());
    let mut layer_mixed_hessian_results = Vec::with_capacity(models.len());
    let mut layer_parameter_hessian_results = Vec::with_capacity(models.len());

    for (layer_idx, model) in models.iter().enumerate() {
        let layer_indep_indices = &indep_var_indices[layer_idx];
        let local_parameters = &local_parameters_per_layer[layer_idx];

        let mut columns =
            vec![vec![0.0; point_count]; layer_indep_indices.len() + local_parameters.len()];
        #[allow(
            clippy::needless_range_loop,
            reason = "Explicit point indexing aligns correctly with multiple data vectors"
        )]
        for point in 0..point_count {
            for (col_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                let corrected = if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point][var_idx][var_idx]
                        > CORRECTION_VARIANCE_THRESHOLD
                {
                    data.variable_values[var_idx][point]
                        + multi_correction_result.corrections[(corr_idx, point)]
                } else {
                    data.variable_values[var_idx][point]
                };
                columns[col_idx][point] = corrected;
            }
            for (p_idx, &val) in local_parameters.iter().enumerate() {
                columns[layer_indep_indices.len() + p_idx][point] = val;
            }
        }

        let column_refs: Vec<&[f64]> = columns.iter().map(|c| &c[..]).collect();

        layer_batch_results.push(evaluate_model_and_gradients_batch(
            &model.model_expr,
            &model.independent_gradient_exprs,
            &model.parameter_gradient_exprs,
            &model.independent_names,
            &model.parameter_names,
            &column_refs,
            layer_idx,
        )?);

        layer_independent_hessian_results.push(evaluate_hessian_exprs_batch(
            &model.independent_hessian_exprs,
            &model.independent_names,
            &model.parameter_names,
            &column_refs,
            &format!("layer {layer_idx} independent Hessian"),
        )?);

        layer_mixed_hessian_results.push(evaluate_hessian_exprs_batch(
            &model.independent_parameter_mixed_hessian_exprs,
            &model.independent_names,
            &model.parameter_names,
            &column_refs,
            &format!("layer {layer_idx} mixed Hessian"),
        )?);

        layer_parameter_hessian_results.push(evaluate_hessian_exprs_batch(
            &model.parameter_hessian_exprs,
            &model.independent_names,
            &model.parameter_names,
            &column_refs,
            &format!("layer {layer_idx} parameter Hessian"),
        )?);
    }

    for point in 0..point_count {
        let mut point_fitted_values = Vec::with_capacity(models.len());
        let mut point_residuals = Vec::with_capacity(models.len());
        let mut layer_has_point_correction = vec![false; models.len()];
        let mut point_joint_weights = vec![None; models.len()];

        for layer_idx in 0..models.len() {
            let fitted = layer_batch_results[layer_idx].fitted_values[point];
            let dep_var_idx = dep_var_indices[layer_idx];
            let residual = data.variable_values[dep_var_idx][point] - fitted;

            point_fitted_values.push(fitted);
            point_residuals.push(residual);
        }

        for layer_idx in 0..models.len() {
            if !layer_has_correctable_independent[layer_idx] {
                continue;
            }
            let dep_var_idx = dep_var_indices[layer_idx];
            let layer_indep_indices = &indep_var_indices[layer_idx];
            layer_has_point_correction[layer_idx] = layer_indep_indices.iter().any(|&var_idx| {
                variable_to_correction_index[var_idx].is_some()
                    && data.point_covariances[point][var_idx][var_idx]
                        > CORRECTION_VARIANCE_THRESHOLD
            });

            if !layer_has_point_correction[layer_idx] {
                continue;
            }

            let sigma_joint = extract_joint_covariance(
                &data.point_covariances[point],
                layer_indep_indices,
                dep_var_idx,
            )?;
            if sigma_joint.was_regularized {
                covariance_regularization_count += 1;
            }
            point_joint_weights[layer_idx] = Some(invert_small_psd(&sigma_joint.matrix)?);
        }

        let mut d_corrections_d_beta =
            DMatrix::<f64>::zeros(correction_variable_indices.len(), global_parameter_count);
        if !correction_variable_indices.is_empty() {
            let mut h_cc = DMatrix::<f64>::zeros(
                correction_variable_indices.len(),
                correction_variable_indices.len(),
            );
            let mut h_cbeta =
                DMatrix::<f64>::zeros(correction_variable_indices.len(), global_parameter_count);

            for layer_idx in 0..models.len() {
                if !layer_has_correctable_independent[layer_idx] {
                    continue;
                }

                let layer_indep_indices = &indep_var_indices[layer_idx];
                if !layer_has_point_correction[layer_idx] {
                    continue;
                }
                let local_independent_count = layer_indep_indices.len();
                let block_dim = local_independent_count + 1;

                let w_joint = point_joint_weights[layer_idx]
                    .as_ref()
                    .expect("joint weight must be precomputed for corrected layer");

                let mut j_corrections =
                    DMatrix::<f64>::zeros(block_dim, correction_variable_indices.len());
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx]
                            > CORRECTION_VARIANCE_THRESHOLD
                    {
                        j_corrections[(local_idx, corr_idx)] = -1.0;
                        j_corrections[(local_independent_count, corr_idx)] = -layer_batch_results
                            [layer_idx]
                            .independent_derivatives[local_idx][point];
                    }
                }

                let mut parameter_block = DMatrix::<f64>::zeros(block_dim, global_parameter_count);
                for (local_param_idx, &global_param_idx) in global_parameter_indices_per_layer
                    [layer_idx]
                    .iter()
                    .enumerate()
                {
                    parameter_block[(local_independent_count, global_param_idx)] =
                        -layer_batch_results[layer_idx].parameter_derivatives[local_param_idx]
                            [point];
                }

                h_cc += j_corrections.transpose() * w_joint * &j_corrections;
                h_cbeta += j_corrections.transpose() * w_joint * parameter_block;

                let mut joint_residual = DVector::<f64>::zeros(block_dim);
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx]
                            > CORRECTION_VARIANCE_THRESHOLD
                    {
                        joint_residual[local_idx] =
                            -multi_correction_result.corrections[(corr_idx, point)];
                    }
                }
                joint_residual[local_independent_count] = point_residuals[layer_idx];

                // For the inner Hessian (h_cc, h_cbeta), the sign logic matches
                // the outer curvature: −(W·r̃)_dep · ∂²f/∂β².
                let second_order_coefficient = dependent_curvature_coefficient(
                    w_joint,
                    &joint_residual,
                    local_independent_count,
                );
                if second_order_coefficient.abs() > 0.0 {
                    let local_hessian_flat = &layer_independent_hessian_results[layer_idx];
                    let local_mixed_hessian_flat = &layer_mixed_hessian_results[layer_idx];

                    let num_params = global_parameter_indices_per_layer[layer_idx].len();

                    for local_row in 0..local_independent_count {
                        let Some(global_row) =
                            variable_to_correction_index[layer_indep_indices[local_row]]
                        else {
                            continue;
                        };
                        if data.point_covariances[point][layer_indep_indices[local_row]]
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
                            if data.point_covariances[point][layer_indep_indices[local_col]]
                                [layer_indep_indices[local_col]]
                                <= CORRECTION_VARIANCE_THRESHOLD
                            {
                                continue;
                            }
                            let hess_idx = local_row * local_independent_count + local_col;
                            let val = local_hessian_flat[hess_idx][point];
                            h_cc[(global_row, global_col)] -= second_order_coefficient * val;
                        }

                        for (local_param_idx, &global_param_idx) in
                            global_parameter_indices_per_layer[layer_idx]
                                .iter()
                                .enumerate()
                        {
                            let mixed_idx = local_row * num_params + local_param_idx;
                            let val = local_mixed_hessian_flat[mixed_idx][point];
                            h_cbeta[(global_row, global_param_idx)] +=
                                second_order_coefficient * val;
                        }
                    }
                }
            }

            let neg_h_cbeta = -h_cbeta;
            let d_corrections = if let Ok(solution) =
                solve_linear_system_matrix(h_cc.clone(), &neg_h_cbeta)
            {
                solution
            } else {
                let mut regularized_h_cc = h_cc;
                let max_diag = (0..correction_variable_indices.len())
                    .map(|i| regularized_h_cc[(i, i)].abs())
                    .fold(0.0, f64::max)
                    .max(1.0);
                let sensitivity_damping = INNER_CORRECTION_DAMPING * max_diag;
                for diagonal in 0..correction_variable_indices.len() {
                    regularized_h_cc[(diagonal, diagonal)] += sensitivity_damping;
                }
                crate::scientific::curve_fitting::logic::engine::linear_algebra::solve_linear_system_matrix(regularized_h_cc, &neg_h_cbeta)?
            };
            for row in 0..correction_variable_indices.len() {
                for col in 0..global_parameter_count {
                    d_corrections_d_beta[(row, col)] = d_corrections[(row, col)];
                }
            }
        }

        let d2_corrections_d_beta2 = if correction_variable_indices.is_empty() {
            Vec::new()
        } else {
            let (tensor, unconverged) = compute_second_derivative_corrections_numerical(
                models,
                data,
                point,
                &dep_var_indices,
                &indep_var_indices,
                &local_parameters_per_layer,
                &global_parameter_indices_per_layer,
                &variable_to_correction_index,
                &layer_has_correctable_independent,
                correction_variable_indices.len(),
                global_parameters,
            )?;
            fd_tensor_unconverged_perturbations += unconverged;
            tensor
        };

        chi_squared_observation += observation_chi_squared_for_point(
            &data.point_covariances[point],
            &dep_var_indices,
            &point_residuals,
        )?;

        for layer_idx in 0..models.len() {
            let dep_var_idx = dep_var_indices[layer_idx];
            let layer_indep_indices = &indep_var_indices[layer_idx];
            let param_global_indices = &global_parameter_indices_per_layer[layer_idx];
            let local_independent_count = layer_indep_indices.len();
            let block_dim = local_independent_count + 1;

            let fitted = point_fitted_values[layer_idx];
            let residual = point_residuals[layer_idx];
            layer_residuals[layer_idx].push(residual);
            layer_fitted_values[layer_idx].push(fitted);

            let parameter_gradients = &layer_batch_results[layer_idx].parameter_derivatives;
            let independent_gradients = &layer_batch_results[layer_idx].independent_derivatives;
            let sigma_y2 =
                data.point_covariances[point][dep_var_idx][dep_var_idx].max(MIN_VARIANCE);

            if let Some(dof) = data.variable_uncertainty_dofs[dep_var_idx]
                && dof.is_finite()
                && dof > 0.0
            {
                let contribution = sigma_y2;
                ws_uc2 += contribution;
                ws_denominator += (contribution * contribution) / dof;
            }
            for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                if let Some(dof) = data.variable_uncertainty_dofs[var_idx]
                    && dof.is_finite()
                    && dof > 0.0
                {
                    let input_variance =
                        data.point_covariances[point][var_idx][var_idx].max(MIN_VARIANCE);
                    let sensitivity = independent_gradients[local_idx][point];
                    let contribution = sensitivity * sensitivity * input_variance;
                    if contribution.is_finite() && contribution > 0.0 {
                        ws_uc2 += contribution;
                        ws_denominator += (contribution * contribution) / dof;
                    }
                }
            }

            if layer_has_point_correction[layer_idx] {
                let w_joint = point_joint_weights[layer_idx]
                    .as_ref()
                    .expect("joint weight must be precomputed for corrected layer");

                let mut joint_residual = DVector::<f64>::zeros(block_dim);
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx]
                            > CORRECTION_VARIANCE_THRESHOLD
                    {
                        joint_residual[local_idx] =
                            -multi_correction_result.corrections[(corr_idx, point)];
                    }
                }
                joint_residual[local_independent_count] = residual;

                let mut parameter_block = DMatrix::<f64>::zeros(block_dim, global_parameter_count);
                for (local_param_idx, &global_param_idx) in param_global_indices.iter().enumerate()
                {
                    parameter_block[(local_independent_count, global_param_idx)] =
                        -parameter_gradients[local_param_idx][point];
                }

                let mut j_corrections =
                    DMatrix::<f64>::zeros(block_dim, correction_variable_indices.len());
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx]
                            > CORRECTION_VARIANCE_THRESHOLD
                    {
                        j_corrections[(local_idx, corr_idx)] = -1.0;
                        j_corrections[(local_independent_count, corr_idx)] =
                            -independent_gradients[local_idx][point];
                    }
                }

                parameter_block += &j_corrections * &d_corrections_d_beta;

                let whitening = sqrt_psd_matrix(w_joint)?;
                let weighted_residual = &whitening * &joint_residual;
                let weighted_parameter_jacobian = &whitening * parameter_block;

                chi_squared += weighted_residual.dot(&weighted_residual);

                // ──────────────────────────────────────────────────────────────────
                // Second-order curvature correction for the outer normal matrix.
                //
                // The profiled objective is ½χ² = ½ r̃ᵀ W r̃, where r̃ includes both
                // correction-penalty and model-residual components.
                //
                // ∂²(½χ²)/∂β² = JᵀWJ + ∑ₖ (W·r̃)ₖ · ∂²r̃ₖ/∂β²
                //
                // Since the model residual is r_dep = y − f(x + c*(β), β):
                //   ∂²r_dep/∂β² = −∂²f/∂β²   (the Hessian of f, negated)
                //
                // `second_order_coefficient` = (W · r̃) evaluated at the dependent
                // row, i.e. the weight of the model-residual curvature contribution.
                //
                // The subtraction `outer_second_order_normal -= coeff * implicit_curvature`
                // implements the negative sign: −(W·r̃)_dep · ∂²f/∂β², which equals
                // +(W·r̃)_dep · ∂²r_dep/∂β² as required by the true Hessian.
                //
                // The same sign logic applies to the inner Hessian (h_cc, h_cbeta)
                // and the scalar-weight branch below.
                // ──────────────────────────────────────────────────────────────────
                let second_order_coefficient = dependent_curvature_coefficient(
                    w_joint,
                    &joint_residual,
                    local_independent_count,
                );
                if second_order_coefficient.abs() > 0.0 {
                    let local_parameter_hessian_flat = &layer_parameter_hessian_results[layer_idx];
                    let local_mixed_hessian_flat = &layer_mixed_hessian_results[layer_idx];
                    let local_independent_hessian_flat =
                        &layer_independent_hessian_results[layer_idx];

                    let num_params = param_global_indices.len();

                    for (local_row, &global_row) in param_global_indices.iter().enumerate() {
                        for (local_col, &global_col) in param_global_indices.iter().enumerate() {
                            let hess_idx = local_row * num_params + local_col;
                            let mut implicit_curvature =
                                local_parameter_hessian_flat[hess_idx][point];

                            for (local_k, &var_idx_k) in layer_indep_indices.iter().enumerate() {
                                let Some(corr_idx_k) = variable_to_correction_index[var_idx_k]
                                else {
                                    continue;
                                };
                                if data.point_covariances[point][var_idx_k][var_idx_k]
                                    <= CORRECTION_VARIANCE_THRESHOLD
                                {
                                    continue;
                                }

                                let dc_k_row = d_corrections_d_beta[(corr_idx_k, global_row)];
                                let dc_k_col = d_corrections_d_beta[(corr_idx_k, global_col)];

                                let mixed_idx_col = local_k * num_params + local_col;
                                let mixed_idx_row = local_k * num_params + local_row;

                                implicit_curvature = local_mixed_hessian_flat[mixed_idx_col][point]
                                    .mul_add(dc_k_row, implicit_curvature);
                                implicit_curvature = local_mixed_hessian_flat[mixed_idx_row][point]
                                    .mul_add(dc_k_col, implicit_curvature);

                                if let Some(d2_corr_k) = d2_corrections_d_beta2.get(corr_idx_k) {
                                    implicit_curvature = independent_gradients[local_k][point]
                                        .mul_add(
                                            d2_corr_k[(global_row, global_col)],
                                            implicit_curvature,
                                        );
                                }
                            }

                            for (local_k, &var_idx_k) in layer_indep_indices.iter().enumerate() {
                                let Some(corr_idx_k) = variable_to_correction_index[var_idx_k]
                                else {
                                    continue;
                                };
                                if data.point_covariances[point][var_idx_k][var_idx_k]
                                    <= CORRECTION_VARIANCE_THRESHOLD
                                {
                                    continue;
                                }
                                let dc_k_row = d_corrections_d_beta[(corr_idx_k, global_row)];
                                for (local_l, &var_idx_l) in layer_indep_indices.iter().enumerate()
                                {
                                    let Some(corr_idx_l) = variable_to_correction_index[var_idx_l]
                                    else {
                                        continue;
                                    };
                                    if data.point_covariances[point][var_idx_l][var_idx_l]
                                        <= CORRECTION_VARIANCE_THRESHOLD
                                    {
                                        continue;
                                    }
                                    let dc_l_col = d_corrections_d_beta[(corr_idx_l, global_col)];

                                    let hess_idx_kl = local_k * local_independent_count + local_l;
                                    implicit_curvature = (local_independent_hessian_flat
                                        [hess_idx_kl][point]
                                        * dc_k_row)
                                        .mul_add(dc_l_col, implicit_curvature);
                                }
                            }

                            outer_second_order_normal[(global_row, global_col)] -=
                                second_order_coefficient * implicit_curvature;
                        }
                    }
                }

                for row in 0..block_dim {
                    flat_weighted_residuals.push(weighted_residual[row]);
                    for col in 0..global_parameter_count {
                        global_weighted_jacobian.push(weighted_parameter_jacobian[(row, col)]);
                    }
                }
            } else {
                let weight = 1.0 / sigma_y2.sqrt();
                let weighted_residual = residual * weight;
                chi_squared += weighted_residual * weighted_residual;

                // For the scalar branch, the curvature correction follows the same
                // sign logic as the corrected branch above: −(r/sigma^2) · ∂²f/∂β².
                let second_order_coefficient = residual / sigma_y2;
                if second_order_coefficient.abs() > 0.0 {
                    let local_parameter_hessian_flat = &layer_parameter_hessian_results[layer_idx];
                    let num_params = param_global_indices.len();
                    for (local_row, &global_row) in param_global_indices.iter().enumerate() {
                        for (local_col, &global_col) in param_global_indices.iter().enumerate() {
                            let hess_idx = local_row * num_params + local_col;
                            let val = local_parameter_hessian_flat[hess_idx][point];
                            outer_second_order_normal[(global_row, global_col)] =
                                second_order_coefficient.mul_add(
                                    -val,
                                    outer_second_order_normal[(global_row, global_col)],
                                );
                        }
                    }
                }

                flat_weighted_residuals.push(weighted_residual);
                let mut jacobian_row_buf = vec![0.0; global_parameter_count];
                for (local_pos, &global_idx) in param_global_indices.iter().enumerate() {
                    jacobian_row_buf[global_idx] = -parameter_gradients[local_pos][point] * weight;
                }
                global_weighted_jacobian.extend_from_slice(&jacobian_row_buf);
            }
        }
    }

    let total_rows = flat_weighted_residuals.len();

    if flat_weighted_residuals.is_empty() || global_weighted_jacobian.is_empty() {
        return Err(OdrError::Numerical(
            "Internal weighted system is empty after model evaluation".to_owned(),
        ));
    }

    let welch_satterthwaite_dof = if ws_uc2 > 0.0 && ws_denominator > 0.0 {
        let nu_eff = (ws_uc2 * ws_uc2) / ws_denominator;
        (nu_eff.is_finite() && nu_eff > 0.0).then_some(nu_eff)
    } else {
        None
    };

    let mut suppressed_correction_count = 0_usize;
    for point in 0..point_count {
        for &var_idx in &correction_variable_indices {
            if data.point_covariances[point][var_idx][var_idx] <= CORRECTION_VARIANCE_THRESHOLD {
                suppressed_correction_count += 1;
            }
        }
    }

    outer_second_order_normal =
        (&outer_second_order_normal + outer_second_order_normal.transpose()) * 0.5;

    Ok(EvaluationState {
        chi_squared,
        chi_squared_observation,
        layer_residuals,
        layer_fitted_values,
        flat_weighted_residuals: DVector::from_vec(flat_weighted_residuals),
        global_weighted_jacobian: DMatrix::from_row_slice(
            total_rows,
            global_parameter_count,
            &global_weighted_jacobian,
        ),
        outer_second_order_normal,
        inner_correction_nonconverged_points,
        covariance_regularization_count,
        inner_stationarity_norm_max,
        inner_stationarity_norm_mean: if inner_stationarity_samples > 0 {
            let sample_count_f64 = u32::try_from(inner_stationarity_samples)
                .map_or_else(|_| f64::from(u32::MAX), f64::from);
            inner_stationarity_norm_sum / sample_count_f64
        } else {
            0.0
        },
        suppressed_correction_count,
        welch_satterthwaite_dof,
        fd_tensor_unconverged_perturbations,
    })
}

pub fn observation_chi_squared_for_point(
    point_covariance: &[Vec<f64>],
    dependent_indices: &[usize],
    residuals: &[f64],
) -> OdrResult<f64> {
    if dependent_indices.is_empty() {
        return Ok(0.0);
    }

    let dim = dependent_indices.len();
    if residuals.len() != dim {
        return Err(OdrError::Numerical(format!(
            "Observation residual length mismatch: got {}, expected {dim}",
            residuals.len()
        )));
    }

    let mut sigma_y = vec![vec![0.0; dim]; dim];
    for (row_local, &row_global) in dependent_indices.iter().enumerate() {
        for (col_local, &col_global) in dependent_indices.iter().enumerate() {
            let value = point_covariance[row_global][col_global];
            if !value.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Non-finite dependent covariance entry at ({row_global}, {col_global})"
                )));
            }
            sigma_y[row_local][col_local] = value;
        }
        sigma_y[row_local][row_local] = sigma_y[row_local][row_local].max(MIN_VARIANCE);
    }

    let weight_y = invert_small_psd(&sigma_y)?;
    let residual_vec = DVector::from_iterator(dim, residuals.iter().copied());
    let weighted = &weight_y * &residual_vec;
    Ok(residual_vec.dot(&weighted))
}
