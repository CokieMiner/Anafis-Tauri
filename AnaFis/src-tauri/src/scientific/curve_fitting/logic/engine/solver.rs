use nalgebra::{DMatrix, DVector};
use std::sync::Arc;

use super::batch_eval::evaluate_model_and_gradients_batch;
use super::diagnostics::{build_normal_equations, diagnose_matrix};
use super::linear_algebra::{
    invert_small_psd, solve_linear_system, solve_linear_system_matrix, sqrt_psd_matrix,
};
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
use crate::scientific::curve_fitting::types::{
    OdrError, OdrFitRequest, OdrResult, UncertaintyType, VariableInput,
};

/// Prepares data for ODR fitting by combining all observed variables into a single unified space.
///
/// # Errors
/// Returns `OdrError::Validation` if data length or values are invalid.
#[allow(clippy::too_many_lines, reason = "Validation and metadata propagation are kept in one pass")]
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

    let mut variable_names = Vec::with_capacity(
        request.independent_variables.len() + request.dependent_variables.len(),
    );
    let mut variable_values = Vec::with_capacity(
        request.independent_variables.len() + request.dependent_variables.len(),
    );
    let mut variable_sigmas = Vec::with_capacity(
        request.independent_variables.len() + request.dependent_variables.len(),
    );
    let mut had_uncertainty_clamp = false;
    let mut had_low_count_poisson = false;
    let mut inferred_type_a_dof_count = 0usize;
    let mut variable_uncertainty_dofs: Vec<Option<f64>> = Vec::with_capacity(
        request.independent_variables.len() + request.dependent_variables.len(),
    );

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

        let resolved_dof = match (var.uncertainty_type, var.uncertainty_degrees_of_freedom) {
            (_, Some(dof)) if !dof.is_finite() || dof <= 0.0 => {
                return Err(OdrError::Validation(format!(
                    "Uncertainty degrees of freedom for '{}' must be finite and > 0",
                    var.name
                )));
            }
            (_, Some(dof)) => Some(dof),
            // Strict GUM default: Type A uncertainty inferred from repeated
            // observations uses nu = n - 1 when not explicitly provided.
            (Some(UncertaintyType::TypeA), None) => {
                inferred_type_a_dof_count += 1;
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "point_count converted to f64 for degrees-of-freedom metadata"
                )]
                {
                    Some((point_count.saturating_sub(1)) as f64)
                }
            }
            _ => None,
        };

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
                if *val < 20.0 {
                    had_low_count_poisson = true;
                }
                if *val <= 0.0 {
                    had_uncertainty_clamp = true;
                }
                sigma.push(variance.sqrt());
            }
            variable_sigmas.push(sigma);
        } else {
            variable_sigmas.push(vec![0.0; point_count]);
        }
        variable_uncertainty_dofs.push(resolved_dof);

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
        had_low_count_poisson,
        inferred_type_a_dof_count,
        variable_uncertainty_dofs,
        welch_satterthwaite_dof: None,
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

    if dim == 1 {
        let v = matrix[0][0];
        return v.is_finite() && v >= -PSD_EIGEN_TOLERANCE;
    }

    if dim == 2 {
        let a = matrix[0][0];
        let b = matrix[0][1];
        let c = matrix[1][0];
        let d = matrix[1][1];
        if !(a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()) {
            return false;
        }
        if (b - c).abs() > CORRELATION_TOLERANCE {
            return false;
        }
        let det = a.mul_add(d, -(b * c));
        return a >= -PSD_EIGEN_TOLERANCE
            && d >= -PSD_EIGEN_TOLERANCE
            && det >= -PSD_EIGEN_TOLERANCE;
    }

    if dim == 3 {
        let m00 = matrix[0][0];
        let m01 = matrix[0][1];
        let m02 = matrix[0][2];
        let m11 = matrix[1][1];
        let m12 = matrix[1][2];
        let m22 = matrix[2][2];
        if !(m00.is_finite()
            && m01.is_finite()
            && m02.is_finite()
            && m11.is_finite()
            && m12.is_finite()
            && m22.is_finite())
        {
            return false;
        }
        if (matrix[0][1] - matrix[1][0]).abs() > CORRELATION_TOLERANCE
            || (matrix[0][2] - matrix[2][0]).abs() > CORRELATION_TOLERANCE
            || (matrix[1][2] - matrix[2][1]).abs() > CORRELATION_TOLERANCE
        {
            return false;
        }

        let principal_01 = m00.mul_add(m11, -(m01 * m01));
        let principal_02 = m00.mul_add(m22, -(m02 * m02));
        let principal_12 = m11.mul_add(m22, -(m12 * m12));
        #[allow(
            clippy::suspicious_operation_groupings,
            reason = "Expanded 3x3 symmetric determinant expression"
        )]
        let det3 = m02.mul_add(
            m01.mul_add(m12, -(m02 * m11)),
            m00.mul_add(m11.mul_add(m22, -(m12 * m12)), -(m01 * m01.mul_add(m22, -(m02 * m12)))),
        );

        return m00 >= -PSD_EIGEN_TOLERANCE
            && m11 >= -PSD_EIGEN_TOLERANCE
            && m22 >= -PSD_EIGEN_TOLERANCE
            && principal_01 >= -PSD_EIGEN_TOLERANCE
            && principal_02 >= -PSD_EIGEN_TOLERANCE
            && principal_12 >= -PSD_EIGEN_TOLERANCE
            && det3 >= -PSD_EIGEN_TOLERANCE;
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

fn min_eigenvalue_symmetric(matrix: &DMatrix<f64>) -> f64 {
    if matrix.nrows() != matrix.ncols() || matrix.nrows() == 0 {
        return f64::INFINITY;
    }

    if matrix.nrows() == 1 {
        return matrix[(0, 0)];
    }

    if matrix.nrows() == 2 {
        let a = matrix[(0, 0)];
        let b = matrix[(0, 1)];
        let d = matrix[(1, 1)];
        let trace = a + d;
        let det = a.mul_add(d, -(b * b));
        let discriminant = trace.mul_add(trace, -(4.0 * det)).max(0.0).sqrt();
        return 0.5 * (trace - discriminant);
    }

    matrix
        .clone()
        .symmetric_eigen()
        .eigenvalues
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min)
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
    let mut effective_scales = vec![1.0f64; parameter_count];
    let mut trial_parameters = vec![0.0f64; parameter_count];

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

        for diagonal in 0..parameter_count {
            let scale = parameter_scales[diagonal];
            effective_scales[diagonal] = if scale.is_finite() && scale >= 1e-30 {
                scale
            } else {
                1.0
            };
        }

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

        for i in 0..parameter_count {
            trial_parameters[i] = parameters[i] + delta[i];
        }

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
            parameters.clone_from(&trial_parameters);
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

    let mut flat_weighted_residuals: Vec<f64> = Vec::with_capacity(
        point_count * models.len() * 4,
    );
    let mut global_weighted_jacobian: Vec<f64> = Vec::with_capacity(
        point_count * models.len() * 4 * global_parameter_count,
    );
    let mut outer_second_order_normal = DMatrix::<f64>::zeros(
        global_parameter_count,
        global_parameter_count,
    );
    let mut ws_uc2 = 0.0_f64;
    let mut ws_denominator = 0.0_f64;

    // Hoisted per-point buffers to avoid reallocation in the point loop
    let mut point_fitted_values: Vec<f64> = Vec::with_capacity(models.len());
    let mut point_residuals: Vec<f64> = Vec::with_capacity(models.len());
    let mut point_args_per_layer: Vec<Vec<f64>> = (0..models.len())
        .map(|_| Vec::new())
        .collect();
    let mut point_parameter_gradients: Vec<Vec<f64>> = (0..models.len())
        .map(|_| Vec::new())
        .collect();
    let mut point_independent_gradients: Vec<Vec<f64>> = (0..models.len())
        .map(|_| Vec::new())
        .collect();
    let mut jacobian_row_buf: Vec<f64> = Vec::new();
    let mut layer_has_point_correction: Vec<bool> = vec![false; models.len()];
    let mut point_joint_weights: Vec<Option<DMatrix<f64>>> = vec![None; models.len()];

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

        point_fitted_values.clear();
        point_residuals.clear();
        for args in &mut point_args_per_layer {
            args.clear();
        }
        for grads in &mut point_parameter_gradients {
            grads.clear();
        }
        for grads in &mut point_independent_gradients {
            grads.clear();
        }
        for layer_idx in 0..models.len() {
            layer_has_point_correction[layer_idx] = false;
            point_joint_weights[layer_idx] = None;
        }

        for (layer_idx, model) in models.iter().enumerate() {
            let layer_indep_indices = &indep_var_indices[layer_idx];
            let local_parameters = &local_parameters_per_layer[layer_idx];

            let point_args = &mut point_args_per_layer[layer_idx];
            point_args.clear();
            point_args.reserve(layer_indep_indices.len() + local_parameters.len());
            for &var_idx in layer_indep_indices {
                let corrected = if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    data.variable_values[var_idx][point] + point_correction.corrections[corr_idx]
                } else {
                    data.variable_values[var_idx][point]
                };
                point_args.push(corrected);
            }
            point_args.extend(local_parameters.iter().copied());

            let (fitted, parameter_gradients, independent_gradients) =
                evaluate_point_model_and_gradients(
                    model,
                    point_args,
                    layer_idx,
                )
                .map_err(|error| {
                    OdrError::Numerical(format!(
                        "Failed to evaluate model/gradients at point {point} layer {layer_idx}: {error}"
                    ))
                })?;

            let dep_var_idx = dep_var_indices[layer_idx];
            let residual = data.variable_values[dep_var_idx][point] - fitted;

            point_fitted_values.push(fitted);
            point_residuals.push(residual);
            point_parameter_gradients[layer_idx] = parameter_gradients;
            point_independent_gradients[layer_idx] = independent_gradients;
        }

        for layer_idx in 0..models.len() {
            if !layer_has_correctable_independent[layer_idx] {
                continue;
            }
            let dep_var_idx = dep_var_indices[layer_idx];
            let layer_indep_indices = &indep_var_indices[layer_idx];
            layer_has_point_correction[layer_idx] = layer_indep_indices.iter().any(|&var_idx| {
                variable_to_correction_index[var_idx].is_some()
                    && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
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

                h_cc += j_corrections.transpose() * w_joint * &j_corrections;
                h_cbeta += j_corrections.transpose() * w_joint * parameter_block;

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
                    dependent_curvature_coefficient(w_joint, &joint_residual, local_independent_count);
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

            // Solve all columns at once: H_cc * X = -H_cbeta => X = H_cc^{-1} * (-H_cbeta)
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
                solve_linear_system_matrix(regularized_h_cc, &neg_h_cbeta)?
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
            compute_second_derivative_corrections_numerical(
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
            )?
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

            let parameter_gradients = &point_parameter_gradients[layer_idx];
            let independent_gradients = &point_independent_gradients[layer_idx];
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
                    let sensitivity = independent_gradients[local_idx];
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

                let whitening = sqrt_psd_matrix(w_joint)?;
                let weighted_residual = &whitening * &joint_residual;
                let weighted_parameter_jacobian = &whitening * parameter_block;

                chi_squared += weighted_residual.dot(&weighted_residual);

                let second_order_coefficient = dependent_curvature_coefficient(
                    w_joint,
                    &joint_residual,
                    local_independent_count,
                );
                if second_order_coefficient.abs() > 0.0 {
                    let local_parameter_hessian = evaluate_model_hessian_wrt_parameters(
                        &models[layer_idx],
                        &point_args_per_layer[layer_idx],
                        param_global_indices.len(),
                    )?;
                    let local_mixed_hessian = evaluate_model_mixed_hessian_wrt_independents_parameters(
                        &models[layer_idx],
                        &point_args_per_layer[layer_idx],
                        local_independent_count,
                        param_global_indices.len(),
                    )?;
                    let local_independent_hessian = evaluate_model_hessian_wrt_independents(
                        &models[layer_idx],
                        &point_args_per_layer[layer_idx],
                        local_independent_count,
                    )?;
                    for (local_row, &global_row) in param_global_indices.iter().enumerate() {
                        for (local_col, &global_col) in param_global_indices.iter().enumerate() {
                            let mut implicit_curvature = local_parameter_hessian
                                [(local_row, local_col)];

                            for (local_k, &var_idx_k) in layer_indep_indices.iter().enumerate() {
                                let Some(corr_idx_k) = variable_to_correction_index[var_idx_k]
                                else {
                                    continue;
                                };
                                if data.point_covariances[point][var_idx_k][var_idx_k]
                                    <= MIN_VARIANCE * 10.0
                                {
                                    continue;
                                }

                                let dc_k_row = d_corrections_d_beta[(corr_idx_k, global_row)];
                                let dc_k_col = d_corrections_d_beta[(corr_idx_k, global_col)];
                                implicit_curvature += local_mixed_hessian
                                    [(local_k, local_col)]
                                    * dc_k_row;
                                implicit_curvature += local_mixed_hessian
                                    [(local_k, local_row)]
                                    * dc_k_col;

                                if let Some(d2_corr_k) = d2_corrections_d_beta2.get(corr_idx_k) {
                                    implicit_curvature += independent_gradients[local_k]
                                        * d2_corr_k[(global_row, global_col)];
                                }
                            }

                            for (local_k, &var_idx_k) in layer_indep_indices.iter().enumerate() {
                                let Some(corr_idx_k) = variable_to_correction_index[var_idx_k]
                                else {
                                    continue;
                                };
                                if data.point_covariances[point][var_idx_k][var_idx_k]
                                    <= MIN_VARIANCE * 10.0
                                {
                                    continue;
                                }
                                let dc_k_row = d_corrections_d_beta[(corr_idx_k, global_row)];
                                for (local_l, &var_idx_l) in layer_indep_indices.iter().enumerate() {
                                    let Some(corr_idx_l) = variable_to_correction_index[var_idx_l]
                                    else {
                                        continue;
                                    };
                                    if data.point_covariances[point][var_idx_l][var_idx_l]
                                        <= MIN_VARIANCE * 10.0
                                    {
                                        continue;
                                    }
                                    let dc_l_col =
                                        d_corrections_d_beta[(corr_idx_l, global_col)];
                                    implicit_curvature += local_independent_hessian
                                        [(local_k, local_l)]
                                        * dc_k_row
                                        * dc_l_col;
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

                let second_order_coefficient = residual / sigma_y2;
                if second_order_coefficient.abs() > 0.0 {
                    let local_parameter_hessian = evaluate_model_hessian_wrt_parameters(
                        &models[layer_idx],
                        &point_args_per_layer[layer_idx],
                        param_global_indices.len(),
                    )?;
                    for (local_row, &global_row) in param_global_indices.iter().enumerate() {
                        for (local_col, &global_col) in param_global_indices.iter().enumerate() {
                            outer_second_order_normal[(global_row, global_col)] -=
                                second_order_coefficient
                                    * local_parameter_hessian[(local_row, local_col)];
                        }
                    }
                }

                flat_weighted_residuals.push(weighted_residual);
                jacobian_row_buf.clear();
                jacobian_row_buf.resize(global_parameter_count, 0.0);
                for (local_pos, &global_idx) in param_global_indices.iter().enumerate() {
                    jacobian_row_buf[global_idx] = -parameter_gradients[local_pos] * weight;
                }
                global_weighted_jacobian.extend_from_slice(&jacobian_row_buf);
            }
        }
    }

    let total_rows = flat_weighted_residuals.len();

    if flat_weighted_residuals.is_empty() || global_weighted_jacobian.is_empty() {
        return Err(OdrError::Numerical(
            "Internal weighted system is empty after model evaluation".to_string(),
        ));
    }

    let welch_satterthwaite_dof = if ws_uc2 > 0.0 && ws_denominator > 0.0 {
        let nu_eff = (ws_uc2 * ws_uc2) / ws_denominator;
        if nu_eff.is_finite() && nu_eff > 0.0 {
            Some(nu_eff)
        } else {
            None
        }
    } else {
        None
    };

    // Numerically enforce symmetry for curvature correction before normal-equation usage.
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
        welch_satterthwaite_dof,
    })
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "Finite-difference implicit correction tensor needs full local context"
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
fn compute_second_derivative_corrections_numerical(
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
) -> OdrResult<Vec<DMatrix<f64>>> {
    let p = global_parameters.len();
    let mut d2 = vec![DMatrix::<f64>::zeros(p, p); correction_count];
    if correction_count == 0 || p == 0 {
        return Ok(d2);
    }

    let step_for = |value: f64| (1e-5_f64 * value.abs().max(1.0)).max(1e-8);

    for i in 0..p {
        let hi = step_for(global_parameters[i]);
        for j in i..p {
            let hj = step_for(global_parameters[j]);

            let mut beta_plus_plus = global_parameters.to_vec();
            beta_plus_plus[i] += hi;
            beta_plus_plus[j] += hj;
            let local_plus_plus = local_parameters_from_global(
                &beta_plus_plus,
                local_parameters_per_layer,
                global_parameter_indices_per_layer,
            );
            let corr_plus_plus = solve_coupled_point_correction(
                models,
                data,
                point_idx,
                dep_var_indices,
                indep_var_indices,
                &local_plus_plus,
                variable_to_correction_index,
                layer_has_correctable_independent,
                correction_count,
            )?
            .corrections;

            let mut beta_plus_minus = global_parameters.to_vec();
            beta_plus_minus[i] += hi;
            beta_plus_minus[j] -= hj;
            let local_plus_minus = local_parameters_from_global(
                &beta_plus_minus,
                local_parameters_per_layer,
                global_parameter_indices_per_layer,
            );
            let corr_plus_minus = solve_coupled_point_correction(
                models,
                data,
                point_idx,
                dep_var_indices,
                indep_var_indices,
                &local_plus_minus,
                variable_to_correction_index,
                layer_has_correctable_independent,
                correction_count,
            )?
            .corrections;

            let mut beta_minus_plus = global_parameters.to_vec();
            beta_minus_plus[i] -= hi;
            beta_minus_plus[j] += hj;
            let local_minus_plus = local_parameters_from_global(
                &beta_minus_plus,
                local_parameters_per_layer,
                global_parameter_indices_per_layer,
            );
            let corr_minus_plus = solve_coupled_point_correction(
                models,
                data,
                point_idx,
                dep_var_indices,
                indep_var_indices,
                &local_minus_plus,
                variable_to_correction_index,
                layer_has_correctable_independent,
                correction_count,
            )?
            .corrections;

            let mut beta_minus_minus = global_parameters.to_vec();
            beta_minus_minus[i] -= hi;
            beta_minus_minus[j] -= hj;
            let local_minus_minus = local_parameters_from_global(
                &beta_minus_minus,
                local_parameters_per_layer,
                global_parameter_indices_per_layer,
            );
            let corr_minus_minus = solve_coupled_point_correction(
                models,
                data,
                point_idx,
                dep_var_indices,
                indep_var_indices,
                &local_minus_minus,
                variable_to_correction_index,
                layer_has_correctable_independent,
                correction_count,
            )?
            .corrections;

            let denom = 4.0 * hi * hj;
            for corr_idx in 0..correction_count {
                let value = (corr_plus_plus[corr_idx]
                    - corr_plus_minus[corr_idx]
                    - corr_minus_plus[corr_idx]
                    + corr_minus_minus[corr_idx])
                    / denom;
                d2[corr_idx][(i, j)] = value;
                d2[corr_idx][(j, i)] = value;
            }
        }
    }

    Ok(d2)
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

fn observation_chi_squared_for_point(
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
    let mut args_buf = Vec::new();
    let mut layer_joint_weights: Vec<Option<DMatrix<f64>>> = vec![None; models.len()];

    for (layer_idx, _) in models.iter().enumerate() {
        if !layer_has_correctable_independent[layer_idx] {
            continue;
        }
        let dep_var_idx = dep_var_indices[layer_idx];
        let layer_indep_indices = &indep_var_indices[layer_idx];
        let layer_has_point_correction = layer_indep_indices.iter().any(|&var_idx| {
            variable_to_correction_index[var_idx].is_some()
                && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
        });
        if !layer_has_point_correction {
            continue;
        }

        let sigma_joint = extract_joint_covariance(
            &data.point_covariances[point_idx],
            layer_indep_indices,
            dep_var_idx,
        )?;
        if sigma_joint.was_regularized {
            covariance_regularization_count += 1;
        }
        layer_joint_weights[layer_idx] = Some(invert_small_psd(&sigma_joint.matrix)?);
    }

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

            let Some(weight_joint) = layer_joint_weights[layer_idx].as_ref() else {
                continue;
            };

            args_buf.clear();
            args_buf.reserve(local_independent_count + local_parameters.len());
            for &var_idx in layer_indep_indices {
                let corrected_value = if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    data.variable_values[var_idx][point_idx] + corrections[corr_idx]
                } else {
                    data.variable_values[var_idx][point_idx]
                };
                args_buf.push(corrected_value);
            }
            args_buf.extend(local_parameters.iter().copied());

            let (fitted, _parameter_gradients, gradient_x) =
                evaluate_point_model_and_gradients(
                    model,
                    &args_buf,
                    layer_idx,
                )
                .map_err(|error| {
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

            let mut j_corrections = DMatrix::<f64>::zeros(block_dim, correction_count);
            for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    j_corrections[(local_idx, corr_idx)] = -1.0;
                    j_corrections[(local_independent_count, corr_idx)] = -gradient_x[local_idx];
                }
            }

            let weighted_residual = weight_joint * &joint_residual;
            gradient += j_corrections.transpose() * &weighted_residual;
            hessian += j_corrections.transpose() * weight_joint * &j_corrections;

            // Add the full second-order term sum_m W_{m,y} * r_m * d2(r_y)/dc2 to improve inner Newton fidelity
            // under x-y correlation (off-diagonal covariance terms).
            let second_order_coefficient =
                dependent_curvature_coefficient(weight_joint, &joint_residual, local_independent_count);
            if second_order_coefficient.abs() > 0.0 {
                let local_hessian =
                    evaluate_model_hessian_wrt_independents(model, &args_buf, local_independent_count)?;
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
        let min_eigenvalue = min_eigenvalue_symmetric(&hessian);
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
        let mut delta_sq = 0.0_f64;
        let mut correction_sq = 0.0_f64;
        for (correction, &step) in corrections.iter_mut().zip(solved.iter()) {
            *correction += step;
            if !correction.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Coupled point correction produced non-finite values at point {point_idx}"
                )));
            }
            delta_sq += step * step;
            correction_sq += *correction * *correction;
        }

        let delta_step_norm = delta_sq.sqrt();
        let correction_norm = correction_sq.sqrt();

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

fn evaluate_model_hessian_wrt_parameters(
    model: &CompiledModel,
    args: &[f64],
    parameter_count: usize,
) -> OdrResult<DMatrix<f64>> {
    let mut parameter_hessian = DMatrix::<f64>::zeros(parameter_count, parameter_count);
    if parameter_count == 0 {
        return Ok(parameter_hessian);
    }

    for row in 0..parameter_count {
        for col in 0..parameter_count {
            let idx = row * parameter_count + col;
            let value = model.parameter_hessian_evaluators[idx].evaluate(args);
            if !value.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Non-finite parameter Hessian value at ({row}, {col})"
                )));
            }
            parameter_hessian[(row, col)] = value;
        }
    }

    for row in 0..parameter_count {
        for col in (row + 1)..parameter_count {
            let sym = 0.5 * (parameter_hessian[(row, col)] + parameter_hessian[(col, row)]);
            parameter_hessian[(row, col)] = sym;
            parameter_hessian[(col, row)] = sym;
        }
    }

    Ok(parameter_hessian)
}

fn dependent_curvature_coefficient(
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

fn evaluate_point_model_and_gradients(
    model: &CompiledModel,
    args: &[f64],
    layer_idx: usize,
) -> OdrResult<(f64, Vec<f64>, Vec<f64>)> {
    const STACK_ARG_CAP: usize = 32;

    let batch = if args.len() <= STACK_ARG_CAP {
        let mut column_storage = [[0.0_f64; 1]; STACK_ARG_CAP];
        let mut columns: [&[f64]; STACK_ARG_CAP] = [&[]; STACK_ARG_CAP];

        for (idx, &value) in args.iter().enumerate() {
            column_storage[idx][0] = value;
        }
        for idx in 0..args.len() {
            columns[idx] = &column_storage[idx];
        }

        evaluate_model_and_gradients_batch(
            &model.model_expr,
            &model.independent_gradient_exprs,
            &model.parameter_gradient_exprs,
            &model.independent_names,
            &model.parameter_names,
            &columns[..args.len()],
            layer_idx,
        )?
    } else {
        let column_storage: Vec<[f64; 1]> = args.iter().map(|&value| [value]).collect();
        let columns: Vec<&[f64]> = column_storage.iter().map(|value| &value[..]).collect();
        evaluate_model_and_gradients_batch(
            &model.model_expr,
            &model.independent_gradient_exprs,
            &model.parameter_gradient_exprs,
            &model.independent_names,
            &model.parameter_names,
            &columns,
            layer_idx,
        )?
    };

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

