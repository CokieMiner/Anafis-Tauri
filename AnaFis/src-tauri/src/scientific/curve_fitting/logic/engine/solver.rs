use nalgebra::{DMatrix, DVector};
use std::sync::Arc;

use super::batch_eval::evaluate_model_and_gradients_batch;
use super::diagnostics::{build_normal_equations, diagnose_matrix};
use super::linear_algebra::{invert_small_psd, solve_linear_system, sqrt_psd_matrix};
pub use super::super::cache::{CompiledModel, get_or_compile_model};
pub use super::super::constants::{
    CORRELATION_TOLERANCE, DEFAULT_DAMPING, DEFAULT_MAX_ITERATIONS, DEFAULT_TOLERANCE,
    INNER_CORRECTION_DAMPING, INNER_CORRECTION_MAX_ITERS, INNER_CORRECTION_TOLERANCE,
    MAX_DAMPING, MIN_DAMPING, MIN_VARIANCE, PSD_EIGEN_TOLERANCE,
};
pub use super::super::sanitization::{
    normalize_identifiers, validate_identifier, validate_symbol_sets,
};
use super::state::{
    EvaluationState, OdrTerminationReason, PreparedData,
};
use crate::scientific::curve_fitting::types::{OdrError, OdrFitRequest, OdrResult, VariableInput};

/// Prepares data for ODR fitting by combining all observed variables into a single unified space.
///
/// # Errors
/// Returns `OdrError::Validation` if data length or values are invalid.
pub fn prepare_data(request: &OdrFitRequest) -> OdrResult<PreparedData> {
    if request.layers.is_empty() {
        return Err(OdrError::Validation(
            "At least one model layer is required".to_string(),
        ));
    }

    if request.dependent_variables.is_empty() {
        return Err(OdrError::Validation(
            "At least one dependent variable observation is required".to_string(),
        ));
    }

    let point_count = request.dependent_variables[0].values.len();
    if point_count < 2 {
        return Err(OdrError::Validation(
            "At least two observations are required for fitting".to_string(),
        ));
    }

    let use_poisson = request.use_poisson_weighting.unwrap_or(false);

    let mut variable_names = Vec::new();
    let mut variable_values = Vec::new();
    let mut variable_sigmas = Vec::new();
    let mut had_uncertainty_clamp = false;

    let mut process_variable = |var: &VariableInput,
                                is_dependent: bool|
     -> OdrResult<()> {
        if var.values.len() != point_count {
            return Err(OdrError::Validation(format!(
                "Variable '{}' length mismatch: expected {}, got {}",
                var.name,
                point_count,
                var.values.len()
            )));
        }

        let name = var.name.trim().to_lowercase();
        validate_identifier(&name, "variable")?;

        if variable_names.contains(&name) {
            return Err(OdrError::Validation(format!(
                "Duplicate expected variable name mapping: {name}"
            )));
        }

        variable_names.push(name);
        variable_values.push(sanitize_values(&var.values, &var.name)?);

        if let Some(uncertainties) = &var.uncertainties {
            if uncertainties.len() != point_count {
                return Err(OdrError::Validation(format!(
                    "Uncertainty length mismatch for '{}': expected {}, got {}",
                    var.name,
                    point_count,
                    uncertainties.len()
                )));
            }
            let (sigma, clamped) = sanitize_uncertainties(uncertainties, &var.name)?;
            had_uncertainty_clamp |= clamped;
            variable_sigmas.push(sigma);
        } else if is_dependent && use_poisson {
            let mut sigma = Vec::with_capacity(point_count);
            for (idx, val) in var.values.iter().enumerate() {
                if *val < 0.0 {
                    return Err(OdrError::Validation(format!(
                        "Poisson weighting requires non-negative counts for '{}' at index {idx}",
                        var.name
                    )));
                }

                let variance = (*val).max(MIN_VARIANCE);
                if *val <= 0.0 {
                    had_uncertainty_clamp = true;
                }
                sigma.push(variance.sqrt());
            }
            variable_sigmas.push(sigma);
        } else {
            variable_sigmas.push(vec![0.0; point_count]);
        }

        Ok(())
    };

    for var in &request.independent_variables {
        process_variable(var, false)?;
    }
    for var in &request.dependent_variables {
        process_variable(var, true)?;
    }

    let point_covariances = build_point_covariances(
        point_count,
        &variable_sigmas,
        request.point_correlations.as_deref(),
    )?;

    Ok(PreparedData {
        variable_names,
        variable_values,
        point_covariances,
        point_count,
        had_uncertainty_clamp,
    })
}

/// Validates and ensures all values are finite.
fn sanitize_values(values: &[f64], label: &str) -> OdrResult<Vec<f64>> {
    let mut sanitized = Vec::with_capacity(values.len());
    for (idx, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Validation(format!(
                "Non-finite value in {label} at index {idx}"
            )));
        }
        sanitized.push(*value);
    }
    Ok(sanitized)
}

/// Validates uncertainties and clamps near-zero values.
fn sanitize_uncertainties(values: &[f64], label: &str) -> OdrResult<(Vec<f64>, bool)> {
    let mut sanitized = Vec::with_capacity(values.len());
    let mut had_clamp = false;
    let sigma_min = MIN_VARIANCE.sqrt();

    for (idx, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Validation(format!(
                "Non-finite uncertainty in {label} at index {idx}"
            )));
        }

        let positive = value.abs();
        let clamped = positive.max(sigma_min);
        if clamped > positive {
            had_clamp = true;
        }
        sanitized.push(clamped);
    }

    Ok((sanitized, had_clamp))
}

/// Constructs the full covariance matrix for each measurement point in the unified variable space.
fn build_point_covariances(
    point_count: usize,
    variable_sigmas: &[Vec<f64>],
    point_correlations: Option<&[Vec<Vec<f64>>]>,
) -> OdrResult<Vec<Vec<Vec<f64>>>> {
    let dim = variable_sigmas.len();

    if let Some(correlations) = point_correlations
        && correlations.len() != point_count
    {
        return Err(OdrError::Validation(format!(
            "point_correlations length mismatch: expected {}, got {}",
            point_count,
            correlations.len()
        )));
    }

    let mut covariances = Vec::with_capacity(point_count);

    for point in 0..point_count {
        let mut sigmas = vec![0.0; dim];
        for var_idx in 0..dim {
            sigmas[var_idx] = variable_sigmas[var_idx][point];
        }

        let covariance = if let Some(correlations) = point_correlations {
            let corr = &correlations[point];
            validate_point_correlation_matrix(corr, dim, point)?;

            let mut sigma = vec![vec![0.0; dim]; dim];
            for row in 0..dim {
                for col in 0..dim {
                    sigma[row][col] = corr[row][col] * sigmas[row] * sigmas[col];
                }
            }
            sigma
        } else {
            let mut sigma = vec![vec![0.0; dim]; dim];
            for idx in 0..dim {
                sigma[idx][idx] = sigmas[idx] * sigmas[idx];
            }
            sigma
        };

        covariances.push(covariance);
    }

    Ok(covariances)
}

/// Validates if a point correlation matrix is symmetric and has unit diagonal.
fn validate_point_correlation_matrix(
    matrix: &[Vec<f64>],
    dim: usize,
    point: usize,
) -> OdrResult<()> {
    if matrix.len() != dim {
        return Err(OdrError::Validation(format!(
            "point_correlations[{point}] has invalid shape: expected {dim} rows, got {}",
            matrix.len()
        )));
    }

    for row in matrix {
        if row.len() != dim {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}] has invalid shape: expected {dim} columns"
            )));
        }
    }

    for (row_idx, row_values) in matrix.iter().enumerate().take(dim) {
        let diagonal = row_values[row_idx];
        if !diagonal.is_finite() {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}][{row_idx}][{row_idx}] must be finite"
            )));
        }
        if (diagonal - 1.0).abs() > CORRELATION_TOLERANCE {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}][{row_idx}][{row_idx}] must be 1"
            )));
        }

        for (col_idx, value) in row_values.iter().copied().enumerate().take(dim) {
            if !value.is_finite() {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}][{row_idx}][{col_idx}] must be finite"
                )));
            }
            if !(-1.0 - CORRELATION_TOLERANCE..=1.0 + CORRELATION_TOLERANCE).contains(&value) {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}][{row_idx}][{col_idx}] must be in [-1, 1]"
                )));
            }

            let symmetric = matrix[col_idx][row_idx];
            if (value - symmetric).abs() > CORRELATION_TOLERANCE {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}] must be symmetric"
                )));
            }
        }
    }

    if !is_positive_semidefinite(matrix) {
        return Err(OdrError::Validation(format!(
            "point_correlations[{point}] must be positive semidefinite"
        )));
    }

    Ok(())
}

/// Checks if a matrix is Positive Semi-Definite using eigenvalue decomposition.
fn is_positive_semidefinite(matrix: &[Vec<f64>]) -> bool {
    let dim = matrix.len();
    if dim == 0 {
        return true;
    }

    let mut flat = Vec::with_capacity(dim * dim);
    for row in matrix {
        flat.extend(row.iter().copied());
    }

    let m = DMatrix::from_row_slice(dim, dim, &flat);
    let eigen = m.symmetric_eigen();
    eigen
        .eigenvalues
        .iter()
        .all(|value| value.is_finite() && *value >= -PSD_EIGEN_TOLERANCE)
}

/// Solves the Orthogonal Distance Regression (ODR) problem using Levenberg-Marquardt across all layers simultaneously.
///
/// # Errors
/// Returns `OdrError` if numerical convergence fails or fitting error occurs.
#[allow(
    clippy::too_many_lines,
    reason = "ODR loop keeps all acceptance/rejection and termination logic explicit for numerical safety"
)]
pub fn solve_odr(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    mut parameters: Vec<f64>,
    global_parameter_names: &[String],
    max_iterations: usize,
    tolerance: f64,
    initial_damping: f64,
) -> OdrResult<(Vec<f64>, EvaluationState, usize, OdrTerminationReason)> {
    let mut damping = initial_damping;
    let mut nu = 2.0;
    let mut current = evaluate_model(models, data, &parameters, global_parameter_names)?;
    let mut iterations = 0;
    let mut termination_reason = OdrTerminationReason::MaxIterations;
    let mut consecutive_rejections = 0usize;
    let parameter_count = parameters.len();
    let mut parameter_scales = vec![1.0f64; parameter_count];

    for iteration in 0..max_iterations {
        iterations = iteration + 1;

        let (normal_matrix, gradient_vector) = build_normal_equations(&current);
        let diagnostics = diagnose_matrix(&normal_matrix);
        if diagnostics.effective_rank == 0 {
            termination_reason = OdrTerminationReason::Singular;
            break;
        }
        for diagonal in 0..parameter_count {
            parameter_scales[diagonal] =
                parameter_scales[diagonal].max(normal_matrix[(diagonal, diagonal)].abs().sqrt());
        }

        let effective_scales: Vec<f64> = parameter_scales
            .iter()
            .map(|&scale| {
                if scale.is_finite() && scale >= 1e-30 {
                    scale
                } else {
                    1.0
                }
            })
            .collect();

        let scaled_gradient_norm = (0..parameter_count)
            .map(|i| (gradient_vector[i] / effective_scales[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        if scaled_gradient_norm <= tolerance {
            termination_reason = OdrTerminationReason::ScaledGradient;
            break;
        }

        let rhs = -&gradient_vector;

        let mut scaled_matrix = normal_matrix.clone();
        for row in 0..parameter_count {
            let row_scale = effective_scales[row];
            for column in 0..parameter_count {
                scaled_matrix[(row, column)] /= row_scale * effective_scales[column];
            }
        }

        for diagonal in 0..parameter_count {
            scaled_matrix[(diagonal, diagonal)] += damping;
        }

        let scaled_rhs = DVector::from_fn(parameter_count, |i, _| rhs[i] / effective_scales[i]);
        let Ok(scaled_delta) = solve_linear_system(scaled_matrix, &scaled_rhs) else {
            consecutive_rejections += 1;
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
            if damping >= MAX_DAMPING {
                termination_reason = OdrTerminationReason::DampingSaturated;
                break;
            }
            if consecutive_rejections >= 25 {
                termination_reason = OdrTerminationReason::Stagnated;
                break;
            }
            continue;
        };

        let delta = DVector::from_fn(parameter_count, |i, _| {
            scaled_delta[i] / effective_scales[i]
        });

        let scaled_delta_norm = (0..parameter_count)
            .map(|i| scaled_delta[i].powi(2))
            .sum::<f64>()
            .sqrt();
        let scaled_parameter_norm = (0..parameter_count)
            .map(|i| (parameters[i] * effective_scales[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        if scaled_delta_norm <= tolerance * (scaled_parameter_norm + tolerance) {
            termination_reason = OdrTerminationReason::ScaledStep;
            break;
        }

        let trial_parameters: Vec<f64> = parameters
            .iter()
            .zip(delta.iter())
            .map(|(parameter, step)| parameter + step)
            .collect();

        if trial_parameters.iter().any(|value| !value.is_finite()) {
            consecutive_rejections += 1;
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
            if damping >= MAX_DAMPING {
                termination_reason = OdrTerminationReason::DampingSaturated;
                break;
            }
            if consecutive_rejections >= 25 {
                termination_reason = OdrTerminationReason::Stagnated;
                break;
            }
            continue;
        }

        let trial = evaluate_model(models, data, &trial_parameters, global_parameter_names)?;
        let actual_reduction = current.chi_squared - trial.chi_squared;

        // Canonical LM/GN model reduction for chi_squared = r^T W r:
        // m(0)-m(delta) = -(2 * g^T delta + delta^T H delta)
        // where g = J^T W r and H = J^T W J.
        let h_delta = &normal_matrix * &delta;
        let mut predicted_reduction =
            -2.0f64.mul_add(gradient_vector.dot(&delta), -delta.dot(&h_delta));
        if !predicted_reduction.is_finite() || predicted_reduction <= MIN_VARIANCE {
            predicted_reduction = MIN_VARIANCE;
        }

        let rho = actual_reduction / predicted_reduction;

        if actual_reduction > 0.0 && rho.is_finite() && rho > 0.0 {
            let improvement = actual_reduction.abs();
            parameters = trial_parameters;
            current = trial;
            consecutive_rejections = 0;

            // Cubic LM update for accepted steps: lambda <- lambda * max(1/3, 1-(2*rho-1)^3)
            let cubic_factor = 1.0 - (2.0f64.mul_add(rho, -1.0)).powi(3);
            let accepted_factor = cubic_factor.max(1.0 / 3.0);
            damping = (damping * accepted_factor).clamp(MIN_DAMPING, MAX_DAMPING);
            nu = 2.0;

            if improvement <= tolerance {
                termination_reason = OdrTerminationReason::Improvement;
                break;
            }
        } else {
            consecutive_rejections += 1;
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
            if damping >= MAX_DAMPING {
                termination_reason = OdrTerminationReason::DampingSaturated;
                break;
            }
            if consecutive_rejections >= 25 {
                termination_reason = OdrTerminationReason::Stagnated;
                break;
            }
        }
    }

    Ok((parameters, current, iterations, termination_reason))
}

/// Evaluates the multi-layer model at the current global parameters.
///
/// # Errors
/// Returns `OdrError::Numerical` if models or gradients evaluate to non-finite values.
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
    let mut inner_correction_nonconverged_points = 0usize;
    let mut covariance_regularization_count = 0usize;
    let mut inner_stationarity_norm_max = 0.0f64;
    let mut inner_stationarity_norm_sum = 0.0f64;
    let mut inner_stationarity_samples = 0usize;

    let mut flat_weighted_residuals: Vec<f64> = Vec::new();
    let mut global_weighted_jacobian: Vec<f64> = Vec::new();

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
            let has_uncertainty = (0..point_count)
                .any(|point| data.point_covariances[point][idx][idx] > MIN_VARIANCE * 10.0);
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

    for point in 0..point_count {
        let point_has_active_corrections = correction_variable_indices
            .iter()
            .any(|&var_idx| data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0);

        let point_correction = if point_has_active_corrections {
            solve_coupled_point_correction(
                models,
                data,
                point,
                &dep_var_indices,
                &indep_var_indices,
                &local_parameters_per_layer,
                &variable_to_correction_index,
                &layer_has_correctable_independent,
                correction_variable_indices.len(),
            )?
        } else {
            PointCorrectionResult {
                corrections: vec![0.0; correction_variable_indices.len()],
                converged: true,
                covariance_regularization_count: 0,
                stationarity_norm: 0.0,
            }
        };
        covariance_regularization_count += point_correction.covariance_regularization_count;
        inner_stationarity_norm_max =
            inner_stationarity_norm_max.max(point_correction.stationarity_norm);
        inner_stationarity_norm_sum += point_correction.stationarity_norm;
        inner_stationarity_samples += 1;
        if !point_correction.converged {
            inner_correction_nonconverged_points += 1;
        }

        let mut point_fitted_values = Vec::with_capacity(models.len());
        let mut point_residuals = Vec::with_capacity(models.len());
        let mut point_args_per_layer: Vec<Vec<f64>> = Vec::with_capacity(models.len());
        let mut point_parameter_gradients: Vec<Vec<f64>> = Vec::with_capacity(models.len());
        let mut point_independent_gradients: Vec<Vec<f64>> = Vec::with_capacity(models.len());

        for (layer_idx, model) in models.iter().enumerate() {
            let layer_indep_indices = &indep_var_indices[layer_idx];
            let local_parameters = &local_parameters_per_layer[layer_idx];

            let mut args = Vec::with_capacity(layer_indep_indices.len() + local_parameters.len());
            for &var_idx in layer_indep_indices {
                let corrected = if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    data.variable_values[var_idx][point] + point_correction.corrections[corr_idx]
                } else {
                    data.variable_values[var_idx][point]
                };
                args.push(corrected);
            }
            args.extend(local_parameters.iter().copied());

            let (fitted, parameter_gradients, independent_gradients) =
                evaluate_point_model_and_gradients(model, &args, layer_idx).map_err(|error| {
                    OdrError::Numerical(format!(
                        "Failed to evaluate model/gradients at point {point} layer {layer_idx}: {error}"
                    ))
                })?;

            let dep_var_idx = dep_var_indices[layer_idx];
            let residual = data.variable_values[dep_var_idx][point] - fitted;

            point_fitted_values.push(fitted);
            point_residuals.push(residual);
            point_args_per_layer.push(args);
            point_parameter_gradients.push(parameter_gradients);
            point_independent_gradients.push(independent_gradients);
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

                let dep_var_idx = dep_var_indices[layer_idx];
                let layer_indep_indices = &indep_var_indices[layer_idx];
                let layer_has_point_correction = layer_indep_indices.iter().any(|&var_idx| {
                    variable_to_correction_index[var_idx].is_some()
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                });
                if !layer_has_point_correction {
                    continue;
                }
                let local_independent_count = layer_indep_indices.len();
                let block_dim = local_independent_count + 1;

                let sigma_joint = extract_joint_covariance(
                    &data.point_covariances[point],
                    layer_indep_indices,
                    dep_var_idx,
                )?;
                if sigma_joint.was_regularized {
                    covariance_regularization_count += 1;
                }
                let w_joint = invert_small_psd(&sigma_joint.matrix)?;

                let mut j_corrections =
                    DMatrix::<f64>::zeros(block_dim, correction_variable_indices.len());
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                    {
                        j_corrections[(local_idx, corr_idx)] = -1.0;
                        j_corrections[(local_independent_count, corr_idx)] =
                            -point_independent_gradients[layer_idx][local_idx];
                    }
                }

                let mut parameter_block = DMatrix::<f64>::zeros(block_dim, global_parameter_count);
                for (local_param_idx, &global_param_idx) in global_parameter_indices_per_layer
                    [layer_idx]
                    .iter()
                    .enumerate()
                {
                    parameter_block[(local_independent_count, global_param_idx)] =
                        -point_parameter_gradients[layer_idx][local_param_idx];
                }

                h_cc += j_corrections.transpose() * &w_joint * &j_corrections;
                h_cbeta += j_corrections.transpose() * &w_joint * parameter_block;

                let mut joint_residual = DVector::<f64>::zeros(block_dim);
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                    {
                        joint_residual[local_idx] = -point_correction.corrections[corr_idx];
                    }
                }
                joint_residual[local_independent_count] = point_residuals[layer_idx];

                let second_order_coefficient =
                    dependent_curvature_coefficient(&w_joint, &joint_residual, local_independent_count);
                if second_order_coefficient.abs() > 0.0 {
                    let local_hessian = evaluate_model_hessian_wrt_independents(
                        &models[layer_idx],
                        &point_args_per_layer[layer_idx],
                        local_independent_count,
                    )?;
                    let local_mixed_hessian =
                        evaluate_model_mixed_hessian_wrt_independents_parameters(
                            &models[layer_idx],
                            &point_args_per_layer[layer_idx],
                            local_independent_count,
                            global_parameter_indices_per_layer[layer_idx].len(),
                        )?;
                    for local_row in 0..local_independent_count {
                        let Some(global_row) =
                            variable_to_correction_index[layer_indep_indices[local_row]]
                        else {
                            continue;
                        };
                        if data.point_covariances[point][layer_indep_indices[local_row]]
                            [layer_indep_indices[local_row]]
                            <= MIN_VARIANCE * 10.0
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
                                <= MIN_VARIANCE * 10.0
                            {
                                continue;
                            }
                            h_cc[(global_row, global_col)] -=
                                second_order_coefficient * local_hessian[(local_row, local_col)];
                        }

                        for (local_param_idx, &global_param_idx) in
                            global_parameter_indices_per_layer[layer_idx].iter().enumerate()
                        {
                            // r_dep = y - f, so d2(r_dep)/(dc dβ) = -d2f/(dx dβ).
                            h_cbeta[(global_row, global_param_idx)] += second_order_coefficient
                                * local_mixed_hessian[(local_row, local_param_idx)];
                        }
                    }
                }
            }

            for col in 0..global_parameter_count {
                let rhs = -h_cbeta.column(col).into_owned();
                let solved_column = if let Ok(solution) = solve_linear_system(h_cc.clone(), &rhs) {
                    solution
                } else {
                    let mut regularized_h_cc = h_cc.clone();
                    let max_diag = (0..correction_variable_indices.len())
                        .map(|i| regularized_h_cc[(i, i)].abs())
                        .fold(0.0, f64::max)
                        .max(1.0);
                    let sensitivity_damping = INNER_CORRECTION_DAMPING * max_diag;
                    for diagonal in 0..correction_variable_indices.len() {
                        regularized_h_cc[(diagonal, diagonal)] += sensitivity_damping;
                    }
                    solve_linear_system(regularized_h_cc, &rhs)?
                };
                for row in 0..correction_variable_indices.len() {
                    d_corrections_d_beta[(row, col)] = solved_column[row];
                }
            }
        }

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

            let parameter_gradients = &point_parameter_gradients[layer_idx];
            let independent_gradients = &point_independent_gradients[layer_idx];
            let sigma_y2 =
                data.point_covariances[point][dep_var_idx][dep_var_idx].max(MIN_VARIANCE);
            chi_squared_observation += residual * residual / sigma_y2;

            let layer_has_point_correction = layer_has_correctable_independent[layer_idx]
                && layer_indep_indices.iter().any(|&var_idx| {
                    variable_to_correction_index[var_idx].is_some()
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                });

            if layer_has_point_correction {
                let sigma_joint = extract_joint_covariance(
                    &data.point_covariances[point],
                    layer_indep_indices,
                    dep_var_idx,
                )?;
                if sigma_joint.was_regularized {
                    covariance_regularization_count += 1;
                }
                let w_joint = invert_small_psd(&sigma_joint.matrix)?;

                let mut joint_residual = DVector::<f64>::zeros(block_dim);
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                    {
                        joint_residual[local_idx] = -point_correction.corrections[corr_idx];
                    }
                }
                joint_residual[local_independent_count] = residual;

                let mut parameter_block = DMatrix::<f64>::zeros(block_dim, global_parameter_count);
                for (local_param_idx, &global_param_idx) in param_global_indices.iter().enumerate()
                {
                    parameter_block[(local_independent_count, global_param_idx)] =
                        -parameter_gradients[local_param_idx];
                }

                let mut j_corrections =
                    DMatrix::<f64>::zeros(block_dim, correction_variable_indices.len());
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                    {
                        j_corrections[(local_idx, corr_idx)] = -1.0;
                        j_corrections[(local_independent_count, corr_idx)] =
                            -independent_gradients[local_idx];
                    }
                }

                parameter_block += &j_corrections * &d_corrections_d_beta;

                let whitening = sqrt_psd_matrix(&w_joint)?;
                let weighted_residual = &whitening * joint_residual;
                let weighted_parameter_jacobian = &whitening * parameter_block;

                chi_squared += weighted_residual.dot(&weighted_residual);
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
                flat_weighted_residuals.push(weighted_residual);
                let mut jacobian_row = vec![0.0; global_parameter_count];
                for (local_pos, &global_idx) in param_global_indices.iter().enumerate() {
                    jacobian_row[global_idx] = -parameter_gradients[local_pos] * weight;
                }
                global_weighted_jacobian.extend_from_slice(&jacobian_row);
            }
        }
    }

    let total_rows = flat_weighted_residuals.len();

    if flat_weighted_residuals.is_empty() || global_weighted_jacobian.is_empty() {
        return Err(OdrError::Numerical(
            "Internal weighted system is empty after model evaluation".to_string(),
        ));
    }

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
        correction_variable_count: correction_variable_indices.len(),
        inner_correction_nonconverged_points,
        covariance_regularization_count,
        inner_stationarity_norm_max,
        inner_stationarity_norm_mean: if inner_stationarity_samples > 0 {
            let sample_count_f64 = f64::from(
                u32::try_from(inner_stationarity_samples).unwrap_or(u32::MAX),
            );
            inner_stationarity_norm_sum / sample_count_f64
        } else {
            0.0
        },
    })
}

struct JointCovarianceBlock {
    matrix: Vec<Vec<f64>>,
    was_regularized: bool,
}

fn extract_joint_covariance(
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
        "Extracted joint covariance block is not PSD after regularization".to_string(),
    ))
}

struct PointCorrectionResult {
    corrections: Vec<f64>,
    converged: bool,
    covariance_regularization_count: usize,
    stationarity_norm: f64,
}

#[allow(
    clippy::too_many_lines,
    reason = "Coupled point correction keeps numerical update and diagnostics in one place"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "All parameters needed for per-point coupled correction solve"
)]
fn solve_coupled_point_correction(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    point_idx: usize,
    dep_var_indices: &[usize],
    indep_var_indices: &[Vec<usize>],
    local_parameters_per_layer: &[Vec<f64>],
    variable_to_correction_index: &[Option<usize>],
    layer_has_correctable_independent: &[bool],
    correction_count: usize,
) -> OdrResult<PointCorrectionResult> {
    if correction_count == 0 {
        return Ok(PointCorrectionResult {
            corrections: Vec::new(),
            converged: true,
            covariance_regularization_count: 0,
            stationarity_norm: 0.0,
        });
    }

    let mut corrections = vec![0.0; correction_count];
    let mut converged = false;
    let mut covariance_regularization_count = 0usize;
    let mut last_stationarity_norm = f64::INFINITY;

    for _ in 0..INNER_CORRECTION_MAX_ITERS {
        let mut gradient = DVector::<f64>::zeros(correction_count);
        let mut hessian = DMatrix::<f64>::zeros(correction_count, correction_count);

        for (layer_idx, model) in models.iter().enumerate() {
            if !layer_has_correctable_independent[layer_idx] {
                continue;
            }
            let dep_var_idx = dep_var_indices[layer_idx];
            let layer_indep_indices = &indep_var_indices[layer_idx];
            let local_parameters = &local_parameters_per_layer[layer_idx];
            let local_independent_count = layer_indep_indices.len();
            let block_dim = local_independent_count + 1;

            let mut args = Vec::with_capacity(local_independent_count + local_parameters.len());
            for &var_idx in layer_indep_indices {
                let corrected_value = if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    data.variable_values[var_idx][point_idx] + corrections[corr_idx]
                } else {
                    data.variable_values[var_idx][point_idx]
                };
                args.push(corrected_value);
            }
            args.extend(local_parameters.iter().copied());

            let (fitted, _parameter_gradients, gradient_x) =
                evaluate_point_model_and_gradients(model, &args, layer_idx).map_err(|error| {
                    OdrError::Numerical(format!(
                        "Failed coupled point evaluation at point {point_idx} layer {layer_idx}: {error}"
                    ))
                })?;

            let residual = data.variable_values[dep_var_idx][point_idx] - fitted;

            let mut joint_residual = DVector::<f64>::zeros(block_dim);
            for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    joint_residual[local_idx] = -corrections[corr_idx];
                }
            }
            joint_residual[local_independent_count] = residual;

            let sigma_joint = extract_joint_covariance(
                &data.point_covariances[point_idx],
                layer_indep_indices,
                dep_var_idx,
            )?;
            if sigma_joint.was_regularized {
                covariance_regularization_count += 1;
            }
            let weight_joint = invert_small_psd(&sigma_joint.matrix)?;

            let mut j_corrections = DMatrix::<f64>::zeros(block_dim, correction_count);
            for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    j_corrections[(local_idx, corr_idx)] = -1.0;
                    j_corrections[(local_independent_count, corr_idx)] = -gradient_x[local_idx];
                }
            }

            let weighted_residual = &weight_joint * &joint_residual;
            gradient += j_corrections.transpose() * &weighted_residual;
            hessian += j_corrections.transpose() * &weight_joint * &j_corrections;

            // Add the full second-order term sum_m W_{m,y} * r_m * d2(r_y)/dc2 to improve inner Newton fidelity
            // under x-y correlation (off-diagonal covariance terms).
            let second_order_coefficient =
                dependent_curvature_coefficient(&weight_joint, &joint_residual, local_independent_count);
            if second_order_coefficient.abs() > 0.0 {
                let local_hessian =
                    evaluate_model_hessian_wrt_independents(model, &args, local_independent_count)?;
                for local_row in 0..local_independent_count {
                    let Some(global_row) =
                        variable_to_correction_index[layer_indep_indices[local_row]]
                    else {
                        continue;
                    };
                    for local_col in 0..local_independent_count {
                        let Some(global_col) =
                            variable_to_correction_index[layer_indep_indices[local_col]]
                        else {
                            continue;
                        };
                        // r_dep = y - f, so d2(r_dep)/dc2 = -d2f/dx2.
                        hessian[(global_row, global_col)] -=
                            second_order_coefficient * local_hessian[(local_row, local_col)];
                    }
                }
            }
        }

        last_stationarity_norm = gradient.norm();

        let max_diag = (0..correction_count)
            .map(|diagonal| hessian[(diagonal, diagonal)].abs())
            .fold(0.0, f64::max)
            .max(1.0);
        let base_damping = INNER_CORRECTION_DAMPING * max_diag;
        let min_eigenvalue = hessian
            .clone()
            .symmetric_eigen()
            .eigenvalues
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let adaptive_damping = if min_eigenvalue.is_finite() && min_eigenvalue <= 0.0 {
            base_damping.max(-min_eigenvalue + base_damping)
        } else {
            base_damping
        };
        for diagonal in 0..correction_count {
            hessian[(diagonal, diagonal)] += adaptive_damping;
        }

        let rhs = -gradient;
        let solved = solve_linear_system(hessian, &rhs)?;
        let next_corrections: Vec<f64> = corrections
            .iter()
            .zip(solved.iter())
            .map(|(base, step)| base + step)
            .collect();
        if next_corrections.iter().any(|value| !value.is_finite()) {
            return Err(OdrError::Numerical(format!(
                "Coupled point correction produced non-finite values at point {point_idx}"
            )));
        }

        let delta_step_norm = next_corrections
            .iter()
            .zip(corrections.iter())
            .map(|(next, prev)| (next - prev).powi(2))
            .sum::<f64>()
            .sqrt();
        let correction_norm = next_corrections
            .iter()
            .map(|value| value.powi(2))
            .sum::<f64>()
            .sqrt();
        corrections = next_corrections;

        if delta_step_norm
            <= INNER_CORRECTION_TOLERANCE.mul_add(correction_norm, INNER_CORRECTION_TOLERANCE)
        {
            converged = true;
            break;
        }
    }

    Ok(PointCorrectionResult {
        corrections,
        converged,
        covariance_regularization_count,
        stationarity_norm: if last_stationarity_norm.is_finite() {
            last_stationarity_norm
        } else {
            f64::INFINITY
        },
    })
}

fn evaluate_model_hessian_wrt_independents(
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
                        .to_string(),
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

fn evaluate_model_mixed_hessian_wrt_independents_parameters(
    model: &CompiledModel,
    args: &[f64],
    independent_count: usize,
    parameter_count: usize,
) -> OdrResult<DMatrix<f64>> {
    let mut mixed_hessian = DMatrix::<f64>::zeros(independent_count, parameter_count);
    if independent_count == 0 || parameter_count == 0 {
        return Ok(mixed_hessian);
    }

    for row in 0..independent_count {
        for col in 0..parameter_count {
            let idx = row * parameter_count + col;
            let value = model.independent_parameter_mixed_hessian_evaluators[idx].evaluate(args);
            if !value.is_finite() {
                return Err(OdrError::Numerical(
                    "Non-finite value while evaluating symbolic independent-parameter mixed Hessian"
                        .to_string(),
                ));
            }
            mixed_hessian[(row, col)] = value;
        }
    }

    Ok(mixed_hessian)
}

fn dependent_curvature_coefficient(
    weight_joint: &DMatrix<f64>,
    joint_residual: &DVector<f64>,
    dependent_row: usize,
) -> f64 {
    (0..joint_residual.len())
        .map(|idx| weight_joint[(dependent_row, idx)] * joint_residual[idx])
        .sum()
}

fn evaluate_point_model_and_gradients(
    model: &CompiledModel,
    args: &[f64],
    layer_idx: usize,
) -> OdrResult<(f64, Vec<f64>, Vec<f64>)> {
    let column_storage: Vec<[f64; 1]> = args.iter().map(|&value| [value]).collect();
    let columns: Vec<&[f64]> = column_storage.iter().map(|value| &value[..]).collect();

    let batch = evaluate_model_and_gradients_batch(
        &model.model_expr,
        &model.independent_gradient_exprs,
        &model.parameter_gradient_exprs,
        &model.independent_names,
        &model.parameter_names,
        &columns,
        layer_idx,
    )?;

    let fitted = batch.fitted_values[0];
    let independent_gradients = batch
        .independent_derivatives
        .iter()
        .map(|values| values[0])
        .collect();
    let parameter_gradients = batch
        .parameter_derivatives
        .iter()
        .map(|values| values[0])
        .collect();

    Ok((fitted, parameter_gradients, independent_gradients))
}

