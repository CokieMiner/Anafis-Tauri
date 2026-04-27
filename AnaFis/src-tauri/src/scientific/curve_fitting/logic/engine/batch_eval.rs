use std::iter::repeat_n;
use std::mem::take;
use symb_anafis::{Expr, eval_f64};

use super::{BatchEvaluationResult, OdrError, OdrResult};

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
    let var_names: Vec<&[&str]> = repeat_n(&all_var_names[..], total_exprs).collect();

    // Build data: each expression gets the same columnar data
    // data[expr_idx][var_idx][point_idx]
    let data: Vec<&[&[f64]]> = repeat_n(columns, total_exprs).collect();

    let expected_points = expected_point_count(columns, &format!("layer {layer_idx}"))?;

    // Call eval_f64 for SIMD+parallel batch evaluation
    let mut results = eval_f64(&exprs, &var_names, &data).map_err(|error| {
        OdrError::Numerical(format!("eval_f64 failed for layer {layer_idx}: {error:?}"))
    })?;

    if results.len() != total_exprs {
        return Err(OdrError::Numerical(format!(
            "eval_f64 returned {} outputs for layer {layer_idx}, expected {total_exprs}",
            results.len()
        )));
    }

    // Validate and split results
    let mut offset = 0;

    // Model values
    let fitted_values = validate_evaluation_output(
        take(&mut results[offset]),
        &format!("model evaluator layer {layer_idx}"),
        expected_points,
    )?;
    offset += 1;

    // Independent derivatives
    let mut independent_derivatives = Vec::with_capacity(independent_gradient_exprs.len());
    for (idx, _) in independent_gradient_exprs.iter().enumerate() {
        let deriv = validate_evaluation_output(
            take(&mut results[offset]),
            &format!("independent gradient evaluator {idx} layer {layer_idx}"),
            expected_points,
        )?;
        independent_derivatives.push(deriv);
        offset += 1;
    }

    // Parameter derivatives
    let mut parameter_derivatives = Vec::with_capacity(parameter_gradient_exprs.len());
    for (idx, _) in parameter_gradient_exprs.iter().enumerate() {
        let deriv = validate_evaluation_output(
            take(&mut results[offset]),
            &format!("parameter gradient evaluator {idx} layer {layer_idx}"),
            expected_points,
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

    let expected_points = expected_point_count(columns, evaluator_label)?;

    let results = eval_f64(&exprs, &var_names, &data).map_err(|error| {
        OdrError::Numerical(format!("eval_f64 failed for {evaluator_label}: {error:?}"))
    })?;

    if results.len() != 1 {
        return Err(OdrError::Numerical(format!(
            "evaluation returned {} outputs for {evaluator_label}, expected 1",
            results.len()
        )));
    }

    let output = results.into_iter().next().ok_or_else(|| {
        OdrError::Numerical(format!(
            "evaluation returned no outputs for {evaluator_label}"
        ))
    })?;

    validate_evaluation_output(output, evaluator_label, expected_points)
}

fn expected_point_count(columns: &[&[f64]], evaluator_label: &str) -> OdrResult<usize> {
    let Some(first) = columns.first() else {
        return Err(OdrError::Numerical(format!(
            "no input columns provided for {evaluator_label}"
        )));
    };
    let expected = first.len();
    for (idx, column) in columns.iter().enumerate() {
        if column.len() != expected {
            return Err(OdrError::Numerical(format!(
                "column length mismatch for {evaluator_label}: column {idx} has {}, expected {expected}",
                column.len()
            )));
        }
    }
    Ok(expected)
}

fn validate_evaluation_output(
    output: Vec<f64>,
    evaluator_label: &str,
    expected_len: usize,
) -> OdrResult<Vec<f64>> {
    if output.len() != expected_len {
        return Err(OdrError::Numerical(format!(
            "{evaluator_label} produced {} outputs, expected {expected_len}",
            output.len()
        )));
    }

    for (idx, value) in output.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Numerical(format!(
                "{evaluator_label} produced non-finite output at point {idx}"
            )));
        }
    }

    Ok(output)
}

/// Batch-evaluates a set of Hessian expressions across all data points in a single `eval_f64` call.
///
/// Returns a `Vec<Vec<f64>>` where `result[expr_idx][point_idx]` is the value of the
/// `expr_idx`-th Hessian element at point `point_idx`. The caller is responsible for
/// interpreting the row-major layout (e.g., `expr_idx = row * cols + col`).
///
/// # Errors
/// Returns `OdrError::Numerical` if evaluation fails or produces non-finite values.
pub fn evaluate_hessian_exprs_batch(
    hessian_exprs: &[Expr],
    independent_names: &[String],
    parameter_names: &[String],
    columns: &[&[f64]],
    label: &str,
) -> OdrResult<Vec<Vec<f64>>> {
    if hessian_exprs.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_var_names: Vec<&str> =
        Vec::with_capacity(independent_names.len() + parameter_names.len());
    for name in independent_names {
        all_var_names.push(name.as_str());
    }
    for name in parameter_names {
        all_var_names.push(name.as_str());
    }

    let exprs: Vec<&Expr> = hessian_exprs.iter().collect();
    let var_names: Vec<&[&str]> = repeat_n(&all_var_names[..], hessian_exprs.len()).collect();
    let data: Vec<&[&[f64]]> = repeat_n(columns, hessian_exprs.len()).collect();

    let expected_points = expected_point_count(columns, label)?;

    let results = eval_f64(&exprs, &var_names, &data)
        .map_err(|error| OdrError::Numerical(format!("eval_f64 failed for {label}: {error:?}")))?;

    if results.len() != hessian_exprs.len() {
        return Err(OdrError::Numerical(format!(
            "eval_f64 returned {} outputs for {label}, expected {}",
            results.len(),
            hessian_exprs.len()
        )));
    }

    let mut validated = Vec::with_capacity(results.len());
    for (idx, output) in results.into_iter().enumerate() {
        validated.push(validate_evaluation_output(
            output,
            &format!("{label} element {idx}"),
            expected_points,
        )?);
    }

    Ok(validated)
}
