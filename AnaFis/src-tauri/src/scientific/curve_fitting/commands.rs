use super::engine::{
    build_normal_equations, evaluate_compiled_batch_or_scalar, get_or_compile_model,
    invert_information_matrix, normalize_identifiers, prepare_data, solve_odr,
    validate_symbol_sets, DEFAULT_DAMPING, DEFAULT_MAX_ITERATIONS, DEFAULT_TOLERANCE,
};
use super::types::{
    GridEvaluationRequest, GridEvaluationResponse, OdrError, OdrFitRequest, OdrFitResponse,
    OdrResult,
};
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
        let compiled = get_or_compile_model(
            &layer.formula,
            &layer.dependent_variable,
            &layer.independent_variables,
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

    let (params, final_state, iterations) = solve_odr(
        &compiled_models,
        &prepared,
        initial_guess,
        &normalized_parameter_names,
        max_iterations,
        DEFAULT_TOLERANCE,
        DEFAULT_DAMPING,
    )?;

    Ok(build_response(
        &compiled_models,
        &prepared,
        params,
        &final_state,
        iterations,
    ))
}

#[allow(clippy::too_many_lines, reason = "Grid evaluation requires validation and generation logic")]
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
) -> OdrFitResponse {
    let parameter_count = parameter_values.len();
    let point_count = prepared.point_count;
    let total_residuals = point_count * models.len();

    let (normal_matrix, _) = build_normal_equations(final_state);

    let degrees_of_freedom = total_residuals.cast_signed() - parameter_count.cast_signed();
    let chi_squared_reduced = if degrees_of_freedom > 0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Degrees of freedom casting to f64 for division"
        )]
        let dof_f64 = degrees_of_freedom as f64;
        final_state.chi_squared / dof_f64
    } else {
        f64::NAN
    };

    let covariance_scale = if degrees_of_freedom > 0 && chi_squared_reduced.is_finite() {
        chi_squared_reduced.max(0.0)
    } else {
        1.0
    };

    let mut warnings: Vec<String> = Vec::new();
    if prepared.had_uncertainty_clamp {
        warnings.push(
            "Some zero/near-zero uncertainties were clamped to a minimum positive value"
                .to_string(),
        );
    }
    if degrees_of_freedom <= 0 {
        warnings.push(
            "Degrees of freedom <= 0: reduced chi-squared and parameter uncertainty scaling may be unreliable".to_string(),
        );
    }

    let (parameter_uncertainties, parameter_covariance) =
        match invert_information_matrix(normal_matrix) {
            Ok(covariance) => {
                let cov_matrix: Vec<Vec<f64>> = (0..parameter_count)
                    .map(|i| {
                        (0..parameter_count)
                            .map(|j| covariance[(i, j)].max(0.0) * covariance_scale)
                            .collect()
                    })
                    .collect();

                let uncertainties: Vec<f64> = (0..parameter_count)
                    .map(|idx| (covariance[(idx, idx)].max(0.0) * covariance_scale).sqrt())
                    .collect();

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

    let mut flat_residuals = Vec::with_capacity(total_residuals);
    let mut flat_fitted = Vec::with_capacity(total_residuals);

    for residuals in &final_state.layer_residuals {
        flat_residuals.extend_from_slice(residuals);
    }
    for fitted in &final_state.layer_fitted_values {
        flat_fitted.extend_from_slice(fitted);
    }

    let residual_sum_of_squares: f64 = flat_residuals.iter().map(|value| value * value).sum();
    #[allow(
        clippy::cast_precision_loss,
        reason = "Point count casting to f64 for RMSE calculation"
    )]
    let rmse = (residual_sum_of_squares / total_residuals as f64).sqrt();

    // Use only the primary/last layer for R-squared rendering if applicable,
    // or compute a global Rsquared across the combined data variance.
    let r_squared = models.last().map_or(1.0, |last_model| {
        // Find dependent index for last layer
        let dep_idx_tgt = prepared
            .variable_names
            .iter()
            .position(|name| name == &last_model.dependent_name)
            .unwrap_or(0);

        let target_data = &prepared.variable_values[dep_idx_tgt];
        #[allow(
            clippy::cast_precision_loss,
            reason = "Point count casting to f64 for mean calculation"
        )]
        let mean_y = target_data.iter().sum::<f64>() / point_count as f64;
        let total_sum_of_squares: f64 = target_data
            .iter()
            .map(|value| (value - mean_y).powi(2))
            .sum();

        // Sum only the residuals of the last layer
        let last_layer_residuals = final_state
            .layer_residuals
            .last()
            .expect("layer_residuals should have at least one layer");
        let layer_rss: f64 = last_layer_residuals.iter().map(|val| val * val).sum();

        if total_sum_of_squares > 0.0 {
            1.0 - layer_rss / total_sum_of_squares
        } else {
            1.0
        }
    });

    let primary_model = models
        .last()
        .expect("models should have at least one layer for response building");

    OdrFitResponse {
        success: true,
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
        parameter_covariance,
        residuals: flat_residuals,
        fitted_values: flat_fitted,
        chi_squared: final_state.chi_squared,
        chi_squared_reduced,
        rmse,
        r_squared,
    }
}
