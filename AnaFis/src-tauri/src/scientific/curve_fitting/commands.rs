use super::logic::engine::{
    evaluate_model_expr_batch, get_or_compile_model, normalize_identifiers,
};
use super::run_fit_request;
use super::types::{
    CurveEvaluationRequest, CurveEvaluationResponse, GridEvaluationRequest, GridEvaluationResponse,
    OdrError, OdrFitRequest, OdrFitResponse, OdrResult,
};
use std::slice::from_ref;
use tauri;

const MAX_GRID_RESOLUTION: usize = 2_000;

/// Perform a custom ODR fit
///
/// # Errors
/// Returns an error if the data preparation fails, the model cannot be compiled,
/// or the ODR solver fails to converge.
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn fit_custom_odr(request: OdrFitRequest) -> Result<OdrFitResponse, String> {
    run_fit_request(&request).map_err(|error| error.to_string())
}

/// Evaluate a model on a 2D grid
///
/// # Errors
/// Returns an error if the model cannot be compiled, the resolution is invalid,
/// or numerical overflow occurs during grid generation.
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn evaluate_model_grid(
    request: GridEvaluationRequest,
) -> Result<GridEvaluationResponse, String> {
    evaluate_model_grid_inner(&request).map_err(|error| error.to_string())
}

/// Evaluate a model on a 1D curve.
///
/// # Errors
/// Returns an error if the model cannot be compiled, the resolution is invalid,
/// or numerical overflow occurs during curve generation.
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn evaluate_model_curve(
    request: CurveEvaluationRequest,
) -> Result<CurveEvaluationResponse, String> {
    evaluate_model_curve_inner(&request).map_err(|error| error.to_string())
}

fn evaluate_model_curve_inner(
    request: &CurveEvaluationRequest,
) -> OdrResult<CurveEvaluationResponse> {
    let normalized_parameter_names = normalize_identifiers(&request.parameter_names, "parameter")?;
    let normalized_independent_names =
        normalize_identifiers(from_ref(&request.independent_name), "independent variable")?;

    if request.parameter_values.len() != normalized_parameter_names.len() {
        return Err(OdrError::Validation(format!(
            "Parameter value length mismatch: expected {}, got {}",
            normalized_parameter_names.len(),
            request.parameter_values.len()
        )));
    }

    if request.resolution < 2 {
        return Err(OdrError::Validation(
            "Curve resolution must be at least 2".to_owned(),
        ));
    }
    if request.resolution > MAX_GRID_RESOLUTION {
        return Err(OdrError::Validation(format!(
            "Curve resolution too high: max supported is {MAX_GRID_RESOLUTION}"
        )));
    }

    let x_min = request.x_range.0;
    let x_max = request.x_range.1;
    if !x_min.is_finite() || !x_max.is_finite() {
        return Err(OdrError::Validation(
            "Curve range must contain finite values".to_owned(),
        ));
    }
    if (x_max - x_min).abs() <= f64::EPSILON {
        return Err(OdrError::Validation(
            "Curve range must span a non-zero interval".to_owned(),
        ));
    }

    let compiled_model = get_or_compile_model(
        &request.model_formula,
        "y", // dummy dependent name since curve evals the raw function
        &normalized_independent_names,
        &normalized_parameter_names,
    )?;

    let point_count = request.resolution;
    let mut curve_x = Vec::with_capacity(point_count);

    #[allow(
        clippy::cast_precision_loss,
        reason = "Precision loss in resolution cast is acceptable for visualization"
    )]
    let x_step = (x_max - x_min) / (point_count - 1) as f64;

    for i in 0..point_count {
        let x = if i == 0 {
            x_min
        } else if i == point_count - 1 {
            x_max
        } else {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Precision loss in index cast is acceptable for visualization"
            )]
            {
                (i as f64).mul_add(x_step, x_min)
            }
        };
        curve_x.push(x);
    }

    let mut columns: Vec<&[f64]> = Vec::with_capacity(1 + normalized_parameter_names.len());
    columns.push(&curve_x);

    let parameter_columns: Vec<Vec<f64>> = request
        .parameter_values
        .iter()
        .map(|&value| vec![value; point_count])
        .collect();
    for values in &parameter_columns {
        columns.push(values);
    }

    let curve_y = evaluate_model_expr_batch(
        &compiled_model.model_expr,
        &compiled_model.independent_names,
        &compiled_model.parameter_names,
        &columns,
        "curve evaluation",
    )?;

    Ok(CurveEvaluationResponse {
        x: curve_x,
        y: curve_y,
    })
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
            "Grid resolution must be at least 2".to_owned(),
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
        OdrError::Validation("Grid resolution overflow while computing point count".to_owned())
    })?;
    let mut grid_x = Vec::with_capacity(point_count);
    let mut grid_y = Vec::with_capacity(point_count);

    let x_min = request.x_range.0;
    let x_max = request.x_range.1;
    let y_min = request.y_range.0;
    let y_max = request.y_range.1;
    if !x_min.is_finite() || !x_max.is_finite() || !y_min.is_finite() || !y_max.is_finite() {
        return Err(OdrError::Validation(
            "Grid ranges must contain finite values".to_owned(),
        ));
    }
    if (x_max - x_min).abs() <= f64::EPSILON || (y_max - y_min).abs() <= f64::EPSILON {
        return Err(OdrError::Validation(
            "Grid ranges must span a non-zero interval".to_owned(),
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
        let y = if j == 0 {
            y_min
        } else if j == res - 1 {
            y_max
        } else {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Precision loss in grid index is acceptable for visualization"
            )]
            {
                (j as f64).mul_add(y_step, y_min)
            }
        };
        for i in 0..res {
            let x = if i == 0 {
                x_min
            } else if i == res - 1 {
                x_max
            } else {
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "Precision loss in grid index is acceptable for visualization"
                )]
                {
                    (i as f64).mul_add(x_step, x_min)
                }
            };
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

    let z = evaluate_model_expr_batch(
        &compiled_model.model_expr,
        &compiled_model.independent_names,
        &compiled_model.parameter_names,
        &columns,
        "grid evaluation",
    )?;

    Ok(GridEvaluationResponse {
        x: grid_x,
        y: grid_y,
        z,
    })
}
