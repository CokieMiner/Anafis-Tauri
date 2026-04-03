use super::engine::{
    DEFAULT_DAMPING, DEFAULT_MAX_ITERATIONS, DEFAULT_TOLERANCE, OdrTerminationReason,
    build_normal_equations, diagnose_matrix, evaluate_compiled_batch_or_scalar,
    get_or_compile_model, invert_information_matrix, normalize_identifiers, prepare_data,
    solve_odr, validate_identifier, validate_symbol_sets,
};
use super::types::{
    GridEvaluationRequest, GridEvaluationResponse, OdrError, OdrFitRequest, OdrFitResponse,
    OdrResult,
};
use statrs::distribution::{ContinuousCDF, StudentsT};
use std::collections::{HashMap, HashSet};
use tauri::command;

const MAX_GRID_RESOLUTION: usize = 2_000;

/// Perform a custom ODR fit
///
/// # Errors
/// Returns an error if the data preparation fails, the model cannot be compiled,
/// or the ODR solver fails to converge.
#[command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn fit_custom_odr(request: OdrFitRequest) -> Result<OdrFitResponse, String> {
    fit_custom_odr_inner(&request).map_err(|error| error.to_string())
}

/// Evaluate a model on a 2D grid
///
/// # Errors
/// Returns an error if the model cannot be compiled, the resolution is invalid,
/// or numerical overflow occurs during grid generation.
#[command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn evaluate_model_grid(
    request: GridEvaluationRequest,
) -> Result<GridEvaluationResponse, String> {
    evaluate_model_grid_inner(&request).map_err(|error| error.to_string())
}

fn fit_custom_odr_inner(request: &OdrFitRequest) -> OdrResult<OdrFitResponse> {
    let prepared = prepare_data(request)?;
    let normalized_parameter_names = normalize_identifiers(&request.parameter_names, "parameter")?;

    validate_symbol_sets(&prepared.variable_names, &normalized_parameter_names)?;

    let mut compiled_models = Vec::with_capacity(request.layers.len());
    for layer in &request.layers {
        let dependent_trimmed = layer.dependent_variable.trim();
        validate_identifier(dependent_trimmed, "dependent variable")?;
        let normalized_dependent = dependent_trimmed.to_lowercase();
        let normalized_independent =
            normalize_identifiers(&layer.independent_variables, "independent variable")?;

        let compiled = get_or_compile_model(
            &layer.formula,
            &normalized_dependent,
            &normalized_independent,
            &normalized_parameter_names,
        )?;
        compiled_models.push(compiled);
    }

    let parameter_count = normalized_parameter_names.len();
    let initial_guess = if let Some(initial) = &request.initial_guess {
        if initial.len() != parameter_count {
            return Err(OdrError::Validation(format!(
                "Initial guess length mismatch: expected {}, got {}",
                parameter_count,
                initial.len()
            )));
        }
        for (idx, value) in initial.iter().enumerate() {
            if !value.is_finite() {
                return Err(OdrError::Validation(format!(
                    "Initial guess contains non-finite value at {idx}"
                )));
            }
        }
        initial.clone()
    } else {
        vec![1.0; parameter_count]
    };

    let max_iterations = request
        .max_iterations
        .unwrap_or(DEFAULT_MAX_ITERATIONS)
        .clamp(5, 5000);

    let confidence_level = request
        .confidence_level
        .unwrap_or(0.95)
        .clamp(0.5, 0.999_999);
    let tolerance = request.tolerance.unwrap_or(DEFAULT_TOLERANCE);
    let initial_damping = request.initial_damping.unwrap_or(DEFAULT_DAMPING);

    let (params, final_state, iterations, termination_reason) = solve_odr(
        &compiled_models,
        &prepared,
        initial_guess,
        &normalized_parameter_names,
        max_iterations,
        tolerance,
        initial_damping,
    )?;

    Ok(build_response(
        &compiled_models,
        &prepared,
        params,
        &final_state,
        iterations,
        termination_reason,
        confidence_level,
    ))
}

#[allow(
    clippy::too_many_lines,
    reason = "Grid evaluation requires validation and generation logic"
)]
fn evaluate_model_grid_inner(request: &GridEvaluationRequest) -> OdrResult<GridEvaluationResponse> {
    let normalized_parameter_names = normalize_identifiers(&request.parameter_names, "parameter")?;
    let normalized_independent_names =
        normalize_identifiers(&request.independent_names, "independent variable")?;

    if normalized_independent_names.len() != 2 {
        // TODO: Generalize this helper to N-D slicing/projection for higher-dimensional surfaces.
        return Err(OdrError::Validation(format!(
            "Grid evaluation currently supports exactly 2 independent variables; got {}",
            normalized_independent_names.len()
        )));
    }

    if request.parameter_values.len() != normalized_parameter_names.len() {
        return Err(OdrError::Validation(format!(
            "Parameter value length mismatch: expected {}, got {}",
            normalized_parameter_names.len(),
            request.parameter_values.len()
        )));
    }

    if request.resolution < 2 {
        return Err(OdrError::Validation(
            "Grid resolution must be at least 2".to_string(),
        ));
    }
    if request.resolution > MAX_GRID_RESOLUTION {
        return Err(OdrError::Validation(format!(
            "Grid resolution too high: max supported is {MAX_GRID_RESOLUTION}"
        )));
    }

    let compiled_model = get_or_compile_model(
        &request.model_formula,
        "z", // dummy dependent name since grid just evals the raw function
        &normalized_independent_names,
        &normalized_parameter_names,
    )?;

    let res = request.resolution;
    let point_count = res.checked_mul(res).ok_or_else(|| {
        OdrError::Validation("Grid resolution overflow while computing point count".to_string())
    })?;
    let mut grid_x = Vec::with_capacity(point_count);
    let mut grid_y = Vec::with_capacity(point_count);

    let x_min = request.x_range.0;
    let x_max = request.x_range.1;
    let y_min = request.y_range.0;
    let y_max = request.y_range.1;
    if !x_min.is_finite() || !x_max.is_finite() || !y_min.is_finite() || !y_max.is_finite() {
        return Err(OdrError::Validation(
            "Grid ranges must contain finite values".to_string(),
        ));
    }
    if (x_max - x_min).abs() <= f64::EPSILON || (y_max - y_min).abs() <= f64::EPSILON {
        return Err(OdrError::Validation(
            "Grid ranges must span a non-zero interval".to_string(),
        ));
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "Precision loss in grid resolution is acceptable for visualization"
    )]
    let x_step = (x_max - x_min) / (res - 1) as f64;
    #[allow(
        clippy::cast_precision_loss,
        reason = "Precision loss in grid resolution is acceptable for visualization"
    )]
    let y_step = (y_max - y_min) / (res - 1) as f64;

    for j in 0..res {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Precision loss in grid index is acceptable for visualization"
        )]
        let y = (j as f64).mul_add(y_step, y_min);
        for i in 0..res {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Precision loss in grid index is acceptable for visualization"
            )]
            let x = (i as f64).mul_add(x_step, x_min);
            grid_x.push(x);
            grid_y.push(y);
        }
    }

    let mut columns: Vec<&[f64]> =
        Vec::with_capacity(request.independent_names.len() + normalized_parameter_names.len());
    columns.push(&grid_x);
    columns.push(&grid_y);

    let parameter_columns: Vec<Vec<f64>> = request
        .parameter_values
        .iter()
        .map(|value| vec![*value; point_count])
        .collect();

    for values in &parameter_columns {
        columns.push(values);
    }

    let z = evaluate_compiled_batch_or_scalar(
        &compiled_model.model_evaluator,
        &columns,
        point_count,
        "grid evaluation",
    )?;

    Ok(GridEvaluationResponse {
        x: grid_x,
        y: grid_y,
        z,
    })
}

#[allow(clippy::too_many_lines, reason = "Building complex fit response")]
fn build_response(
    models: &[std::sync::Arc<super::engine::CompiledModel>],
    prepared: &super::engine::PreparedData,
    parameter_values: Vec<f64>,
    final_state: &super::engine::EvaluationState,
    iterations: usize,
    termination_reason: OdrTerminationReason,
    confidence_level: f64,
) -> OdrFitResponse {
    let parameter_count = parameter_values.len();
    let point_count = prepared.point_count;
    let total_residuals = point_count * models.len();

    let (normal_matrix, _) = build_normal_equations(final_state);
    let diagnostics = diagnose_matrix(&normal_matrix);

    // Use numerical rank as effective parameter count to avoid overconfident scaling
    // when parameters are non-identifiable.
    let effective_parameter_count = diagnostics.effective_rank.cast_signed();
    let degrees_of_freedom = total_residuals.cast_signed() - effective_parameter_count;
    let chi_squared_reduced = if degrees_of_freedom > 0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Degrees of freedom casting to f64 for division"
        )]
        let dof_f64 = degrees_of_freedom as f64;
        final_state.chi_squared_observation / dof_f64
    } else {
        f64::NAN
    };

    let coverage_factor = coverage_factor_from_confidence(confidence_level, degrees_of_freedom)
        .unwrap_or(1.959_963_984_540_054);

    let mut warnings: Vec<String> = Vec::new();
    if prepared.had_uncertainty_clamp {
        warnings.push(
            "Some zero/near-zero uncertainties were clamped to a minimum positive value"
                .to_string(),
        );
    }
    if degrees_of_freedom <= 0 {
        warnings.push(
            "Degrees of freedom <= 0: reduced chi-squared and coverage-factor interpretation may be unreliable".to_string(),
        );
    }
    if termination_reason == OdrTerminationReason::MaxIterations {
        warnings.push(
            "Maximum iterations reached before convergence; reporting best available estimate"
                .to_string(),
        );
    }
    if termination_reason == OdrTerminationReason::DampingSaturated {
        warnings.push(
            "Damping saturated before convergence; solution may be weakly constrained".to_string(),
        );
    }
    if diagnostics.effective_rank < parameter_count {
        warnings.push(format!(
            "Normal matrix is rank-deficient (effective rank {} / {}); parameter uncertainties are reported as infinite to avoid false precision",
            diagnostics.effective_rank, parameter_count
        ));
    }
    if has_shared_measured_variable_dependencies(models) {
        warnings.push(
            "At least one dependent variable is reused as an independent variable across layers; shared latent corrections improve consistency, but this is still not a full structural latent-state model across equations"
                .to_string(),
        );
    }
    if diagnostics.condition_number.is_finite() && diagnostics.condition_number > 1e12 {
        warnings.push(format!(
            "Normal matrix is ill-conditioned (condition number {:.3e}); parameter uncertainties may be unstable",
            diagnostics.condition_number
        ));
    }
    if final_state.inner_correction_nonconverged_points > 0 {
        warnings.push(format!(
            "Per-point inner correction did not converge for {} point/layer cases; results may be less reliable in strongly nonlinear regions",
            final_state.inner_correction_nonconverged_points
        ));
    }

    let (parameter_uncertainties, parameter_covariance) =
        match invert_information_matrix(normal_matrix) {
            Ok(covariance) => {
                let covariance_scale = if degrees_of_freedom > 0 && chi_squared_reduced.is_finite()
                {
                    chi_squared_reduced.max(0.0)
                } else {
                    1.0
                };

                let mut cov_matrix: Vec<Vec<f64>> = (0..parameter_count)
                    .map(|i| {
                        (0..parameter_count)
                            .map(|j| covariance[(i, j)] * covariance_scale)
                            .collect()
                    })
                    .collect();

                let mut uncertainties: Vec<f64> = (0..parameter_count)
                    .map(|idx| (covariance[(idx, idx)] * covariance_scale).max(0.0).sqrt())
                    .collect();

                if diagnostics.effective_rank < parameter_count {
                    uncertainties.fill(f64::INFINITY);
                    for (row_idx, row) in cov_matrix.iter_mut().enumerate() {
                        row.fill(f64::NAN);
                        row[row_idx] = f64::INFINITY;
                    }
                }

                (uncertainties, cov_matrix)
            }
            Err(error) => {
                warnings.push(format!(
                    "Fit converged, but parameter covariance could not be estimated: {error}"
                ));
                (
                    vec![f64::NAN; parameter_count],
                    vec![vec![f64::NAN; parameter_count]; parameter_count],
                )
            }
        };

    let parameter_expanded_uncertainties: Vec<f64> = parameter_uncertainties
        .iter()
        .map(|value| value * coverage_factor)
        .collect();

    let mut assumptions = vec![
        "Orthogonal Distance Regression (ODR) accounts for uncertainties in both independent and dependent variables".to_string(),
        "Parameter uncertainties are derived from the inverse normal matrix, scaled by reduced \u{03c7}\u{00b2} (GUM convention); expanded uncertainties use Student-t coverage at the requested confidence level".to_string(),
        "Degrees of freedom = N \u{00d7} L \u{2212} P, where N = data points, L = model layers, P = parameters".to_string(),
        "Covariance propagation and confidence intervals assume the model is approximately linear near the optimum".to_string(),
        "R\u{00b2} is a descriptive statistic only; it is not a rigorous goodness-of-fit measure when predictors have uncertainty".to_string(),
    ];
    if models.len() > 1 {
        assumptions.push("Per-layer R\u{00b2} values should be preferred over the global R\u{00b2}, which pools layers that may have different physical units or scales".to_string());
        assumptions.push("Shared measured variables across layers are coupled through corrections, but this is not a full structural state-space model".to_string());
    }

    let mut flat_residuals = Vec::with_capacity(total_residuals);
    let mut flat_fitted = Vec::with_capacity(total_residuals);

    for residuals in &final_state.layer_residuals {
        flat_residuals.extend_from_slice(residuals);
    }
    for fitted in &final_state.layer_fitted_values {
        flat_fitted.extend_from_slice(fitted);
    }

    let residual_sum_of_squares: f64 = flat_residuals.iter().map(|value| value * value).sum();

    let rmse_points = if degrees_of_freedom > 0 {
        degrees_of_freedom.unsigned_abs()
    } else {
        total_residuals.max(1)
    };
    #[allow(
        clippy::cast_precision_loss,
        reason = "Residual count casting to f64 for RMSE calculation"
    )]
    let rmse = (residual_sum_of_squares / rmse_points as f64).sqrt();

    let mut flat_targets = Vec::with_capacity(total_residuals);
    for model in models {
        if let Some(dep_idx) = prepared
            .variable_names
            .iter()
            .position(|name| name == &model.dependent_name)
        {
            flat_targets.extend_from_slice(&prepared.variable_values[dep_idx]);
        }
    }

    let r_squared = if flat_targets.is_empty() {
        1.0
    } else {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Point count casting to f64 for mean calculation"
        )]
        let mean_y = flat_targets.iter().sum::<f64>() / flat_targets.len() as f64;
        let total_sum_of_squares: f64 = flat_targets
            .iter()
            .map(|value| (value - mean_y).powi(2))
            .sum();

        if total_sum_of_squares > 0.0 {
            1.0 - residual_sum_of_squares / total_sum_of_squares
        } else {
            1.0
        }
    };

    let r_squared_per_layer: Vec<f64> = (0..models.len())
        .map(|layer_idx| {
            let model = &models[layer_idx];
            let dep_idx = prepared
                .variable_names
                .iter()
                .position(|name| name == &model.dependent_name);
            dep_idx.map_or(f64::NAN, |dep_idx| {
                let targets = &prepared.variable_values[dep_idx];
                let residuals = &final_state.layer_residuals[layer_idx];
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "Point count casting to f64 for mean calculation"
                )]
                let mean_y = targets.iter().sum::<f64>() / targets.len() as f64;
                let ss_tot: f64 = targets.iter().map(|v| (v - mean_y).powi(2)).sum();
                let ss_res: f64 = residuals.iter().map(|v| v.powi(2)).sum();
                if ss_tot > 0.0 {
                    1.0 - ss_res / ss_tot
                } else {
                    1.0
                }
            })
        })
        .collect();

    let primary_model = models
        .last()
        .expect("models should have at least one layer for response building");

    OdrFitResponse {
        success: matches!(
            termination_reason,
            OdrTerminationReason::ScaledGradient
                | OdrTerminationReason::ScaledStep
                | OdrTerminationReason::Improvement
        ),
        termination_reason: termination_reason_label(termination_reason).to_string(),
        message: if warnings.is_empty() {
            None
        } else {
            Some(warnings.join(" | "))
        },
        iterations,
        formula: primary_model.formula.clone(),
        dependent_variable: primary_model.dependent_name.clone(),
        independent_variables: primary_model.independent_names.clone(),
        parameter_names: primary_model.parameter_names.clone(),
        parameter_values,
        parameter_uncertainties,
        parameter_expanded_uncertainties,
        coverage_factor,
        parameter_covariance,
        residuals: flat_residuals,
        fitted_values: flat_fitted,
        chi_squared: final_state.chi_squared_observation,
        chi_squared_reduced,
        rmse,
        r_squared,
        r_squared_per_layer,
        effective_rank: diagnostics.effective_rank,
        condition_number: diagnostics.condition_number,
        assumptions,
    }
}

fn has_shared_measured_variable_dependencies(
    models: &[std::sync::Arc<super::engine::CompiledModel>],
) -> bool {
    let dependent_set: HashSet<&str> = models
        .iter()
        .map(|model| model.dependent_name.as_str())
        .collect();

    let mut dependent_counts: HashMap<&str, usize> = HashMap::new();
    for model in models {
        *dependent_counts
            .entry(model.dependent_name.as_str())
            .or_insert(0) += 1;
    }

    if dependent_counts.values().any(|count| *count > 1) {
        return true;
    }

    models.iter().any(|model| {
        model
            .independent_names
            .iter()
            .any(|name| dependent_set.contains(name.as_str()))
    })
}

fn coverage_factor_from_confidence(confidence_level: f64, dof: isize) -> Option<f64> {
    if !(0.5..1.0).contains(&confidence_level) {
        return None;
    }

    if dof <= 0 {
        return None;
    }

    let dof_i32 = i32::try_from(dof).ok()?;
    let degrees_of_freedom = f64::from(dof_i32);
    let students_t = StudentsT::new(0.0, 1.0, degrees_of_freedom).ok()?;
    let probability = (1.0 + confidence_level) * 0.5;
    Some(students_t.inverse_cdf(probability))
}

const fn termination_reason_label(reason: OdrTerminationReason) -> &'static str {
    match reason {
        OdrTerminationReason::ScaledGradient => "scaledGradient",
        OdrTerminationReason::ScaledStep => "scaledStep",
        OdrTerminationReason::Improvement => "improvement",
        OdrTerminationReason::Stagnated => "stagnated",
        OdrTerminationReason::Singular => "singular",
        OdrTerminationReason::DampingSaturated => "dampingSaturated",
        OdrTerminationReason::MaxIterations => "maxIterations",
    }
}
