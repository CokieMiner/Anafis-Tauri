use super::engine::{
    get_or_compile_model, normalize_identifiers, prepare_data, solve_odr, validate_identifier,
    validate_symbol_sets, DEFAULT_DAMPING, DEFAULT_MAX_ITERATIONS, DEFAULT_TOLERANCE,
};
use super::response_builder::build_response;
use crate::scientific::curve_fitting::types::{OdrError, OdrFitRequest, OdrFitResponse, OdrResult};

pub fn run_fit_request(request: &OdrFitRequest) -> OdrResult<OdrFitResponse> {
    // Future extension point: route by solver mode (profiled vs. simultaneous augmented-state)
    // once a full ODRPACK-style backend is introduced.
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
