use symb_anafis::{Expr, eval_f64};

use super::state::BatchEvaluationResult;
use crate::scientific::curve_fitting::types::{OdrError, OdrResult};

/// Evaluates the model and all its gradients in a single batched call using `eval_f64`.
///
/// This leverages SIMD vectorization and parallel evaluation for maximum performance.
///
/// # Errors
/// Returns `OdrError::Numerical` if evaluation fails or produces non-finite values.
#[allow(
    clippy::too_many_arguments,
    reason = "All parameters needed for batch evaluation"
)]
pub fn evaluate_model_and_gradients_batch(
    model_expr: &Expr,
    independent_gradient_exprs: &[Expr],
    parameter_gradient_exprs: &[Expr],
    independent_names: &[String],
    parameter_names: &[String],
    columns: &[&[f64]],
    layer_idx: usize,
) -> OdrResult<BatchEvaluationResult> {
    let total_exprs = 1 + independent_gradient_exprs.len() + parameter_gradient_exprs.len();

    // Build expression list: [model, df/dx1, df/dx2, ..., df/dp1, df/dp2, ...]
    let mut exprs: Vec<&Expr> = Vec::with_capacity(total_exprs);
    exprs.push(model_expr);
    for expr in independent_gradient_exprs {
        exprs.push(expr);
    }
    for expr in parameter_gradient_exprs {
        exprs.push(expr);
    }

    // Build variable names for each expression (all use the same variable order)
    let mut all_var_names: Vec<&str> =
        Vec::with_capacity(independent_names.len() + parameter_names.len());
    for name in independent_names {
        all_var_names.push(name.as_str());
    }
    for name in parameter_names {
        all_var_names.push(name.as_str());
    }

    // Each expression uses the same variable order
    let var_names: Vec<&[&str]> = exprs.iter().map(|_| &all_var_names[..]).collect();

    // Build data: each expression gets the same columnar data
    // data[expr_idx][var_idx][point_idx]
    let data: Vec<&[&[f64]]> = exprs.iter().map(|_| columns).collect();

    // Call eval_f64 for SIMD+parallel batch evaluation
    let results = eval_f64(&exprs, &var_names, &data).map_err(|error| {
        OdrError::Numerical(format!("eval_f64 failed for layer {layer_idx}: {error:?}"))
    })?;

    // Validate and split results
    let mut offset = 0;

    // Model values
    let fitted_values = validate_evaluation_output(
        results[offset].clone(),
        &format!("model evaluator layer {layer_idx}"),
    )?;
    offset += 1;

    // Independent derivatives
    let mut independent_derivatives = Vec::with_capacity(independent_gradient_exprs.len());
    for (idx, _) in independent_gradient_exprs.iter().enumerate() {
        let deriv = validate_evaluation_output(
            results[offset].clone(),
            &format!("independent gradient evaluator {idx} layer {layer_idx}"),
        )?;
        independent_derivatives.push(deriv);
        offset += 1;
    }

    // Parameter derivatives
    let mut parameter_derivatives = Vec::with_capacity(parameter_gradient_exprs.len());
    for (idx, _) in parameter_gradient_exprs.iter().enumerate() {
        let deriv = validate_evaluation_output(
            results[offset].clone(),
            &format!("parameter gradient evaluator {idx} layer {layer_idx}"),
        )?;
        parameter_derivatives.push(deriv);
        offset += 1;
    }

    Ok(BatchEvaluationResult {
        fitted_values,
        independent_derivatives,
        parameter_derivatives,
    })
}

/// Evaluates only the model expression in strict `eval_f64` batch mode.
///
/// # Errors
/// Returns `OdrError::Numerical` if evaluation fails or produces non-finite values.
pub fn evaluate_model_expr_batch(
    model_expr: &Expr,
    independent_names: &[String],
    parameter_names: &[String],
    columns: &[&[f64]],
    _point_count: usize,
    evaluator_label: &str,
) -> OdrResult<Vec<f64>> {
    let exprs: Vec<&Expr> = vec![model_expr];

    let mut all_var_names: Vec<&str> =
        Vec::with_capacity(independent_names.len() + parameter_names.len());
    for name in independent_names {
        all_var_names.push(name.as_str());
    }
    for name in parameter_names {
        all_var_names.push(name.as_str());
    }
    let var_names: Vec<&[&str]> = vec![&all_var_names[..]];

    let data: Vec<&[&[f64]]> = vec![columns];

    let results = eval_f64(&exprs, &var_names, &data).map_err(|error| {
        OdrError::Numerical(format!("eval_f64 failed for {evaluator_label}: {error:?}"))
    })?;

    validate_evaluation_output(results[0].clone(), evaluator_label)
}

fn validate_evaluation_output(output: Vec<f64>, evaluator_label: &str) -> OdrResult<Vec<f64>> {
    for (idx, value) in output.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Numerical(format!(
                "{evaluator_label} produced non-finite output at point {idx}"
            )));
        }
    }

    Ok(output)
}
