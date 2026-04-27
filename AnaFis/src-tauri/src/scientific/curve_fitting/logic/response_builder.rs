use std::sync::Arc;

use super::OdrFitResponse;
use super::dof_logic::combine_coverage_dof_welch_satterthwaite;
use super::engine::{
    CompiledModel, EvaluationState, OdrTerminationReason, ParameterInference, PreparedData,
    build_normal_equations, compute_parameter_inference, diagnose_matrix,
};
use super::fit_metrics::{
    compute_global_r_squared, compute_per_layer_r_squared, compute_rmse,
    flatten_residuals_and_fitted,
};
use super::fit_notes::{WarningContext, build_assumptions, build_warnings};

#[allow(clippy::too_many_lines, reason = "Building complex fit response")]
pub fn build_response(
    models: &[Arc<CompiledModel>],
    prepared: &PreparedData,
    parameter_values: Vec<f64>,
    final_state: &EvaluationState,
    iterations: usize,
    termination_reason: OdrTerminationReason,
    confidence_level: f64,
) -> OdrFitResponse {
    let parameter_count = parameter_values.len();
    let point_count = prepared.point_count;
    let total_observation_residuals = point_count * models.len();

    let (normal_matrix, _) = build_normal_equations(final_state);
    let diagnostics = diagnose_matrix(&normal_matrix);

    // Use numerical rank as effective parameter count to avoid overconfident scaling
    // when parameters are non-identifiable.
    let effective_parameter_count = diagnostics.effective_rank.cast_signed();
    // For profiled ODR, latent corrections are eliminated by inner stationarity,
    // so both profiled and observation residual DOF use N * L - P.
    let residual_degrees_of_freedom =
        total_observation_residuals.cast_signed() - effective_parameter_count;
    let chi_squared_reduced = if residual_degrees_of_freedom > 0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Degrees of freedom casting to f64 for division"
        )]
        let dof_f64 = residual_degrees_of_freedom as f64;
        final_state.chi_squared / dof_f64
    } else {
        f64::NAN
    };
    let chi_squared_observation_reduced = if residual_degrees_of_freedom > 0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Degrees of freedom casting to f64 for division"
        )]
        let dof_f64 = residual_degrees_of_freedom as f64;
        final_state.chi_squared_observation / dof_f64
    } else {
        f64::NAN
    };

    let fit_coverage_dof = if residual_degrees_of_freedom > 0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Profiled dof cast to f64 for Student-t"
        )]
        {
            Some(residual_degrees_of_freedom as f64)
        }
    } else {
        None
    };

    let ws_dof = final_state
        .welch_satterthwaite_dof
        .or(prepared.welch_satterthwaite_dof);

    let coverage_dof = combine_coverage_dof_welch_satterthwaite(
        ws_dof,
        fit_coverage_dof,
        chi_squared_observation_reduced,
    );

    let used_normal_coverage_fallback = coverage_dof.is_none();

    let mut warnings = build_warnings(&WarningContext {
        had_uncertainty_clamp: prepared.had_uncertainty_clamp,
        had_low_count_poisson: prepared.had_low_count_poisson,
        inferred_type_a_dof_count: prepared.inferred_type_a_dof_count,
        degrees_of_freedom: residual_degrees_of_freedom,
        used_normal_coverage_fallback,
        termination_reason,
        effective_rank: diagnostics.effective_rank,
        parameter_count,
        condition_number: diagnostics.condition_number,
        inner_correction_nonconverged_points: final_state.inner_correction_nonconverged_points,
        inner_stationarity_norm_max: final_state.inner_stationarity_norm_max,
        inner_stationarity_norm_mean: final_state.inner_stationarity_norm_mean,
        covariance_regularization_count: final_state.covariance_regularization_count,
        had_zero_count_poisson: prepared.had_zero_count_poisson,
        clamped_variance_count: prepared.clamped_variance_count,
        suppressed_correction_count: final_state.suppressed_correction_count,
        fd_tensor_unconverged_perturbations: final_state.fd_tensor_unconverged_perturbations,
        observation_reduced_chi_squared: chi_squared_observation_reduced,
        models,
    });

    let inference = match compute_parameter_inference(
        normal_matrix,
        chi_squared_observation_reduced,
        confidence_level,
        coverage_dof.unwrap_or(0.0),
    ) {
        Ok(inf) => inf,
        Err(error) => {
            warnings.push(format!("Inference computation failed: {error}"));
            ParameterInference {
                uncertainties_raw: vec![f64::NAN; parameter_count],
                uncertainties_scaled: vec![f64::NAN; parameter_count],
                covariance_raw: vec![vec![f64::NAN; parameter_count]; parameter_count],
                covariance_scaled: vec![vec![f64::NAN; parameter_count]; parameter_count],
                correlations_raw: vec![vec![f64::NAN; parameter_count]; parameter_count],
                correlations_scaled: vec![vec![f64::NAN; parameter_count]; parameter_count],
                expanded_uncertainties: vec![f64::NAN; parameter_count],
                coverage_factor: 1.96,
                diagnostics,
                correlation_clamped: false,
            }
        }
    };

    if inference.correlation_clamped {
        warnings.push(
            "At least one parameter correlation exceeded [-1, 1] before clamping; check for ill-conditioning".to_owned(),
        );
    }

    let assumptions = build_assumptions(models.len());

    let (flat_residuals, flat_fitted) = flatten_residuals_and_fitted(
        &final_state.layer_residuals,
        &final_state.layer_fitted_values,
        total_observation_residuals,
    );

    let rmse = compute_rmse(&flat_residuals, total_observation_residuals.max(1));
    let residual_standard_error_points = if residual_degrees_of_freedom > 0 {
        residual_degrees_of_freedom.unsigned_abs()
    } else {
        total_observation_residuals.max(1)
    };
    let residual_standard_error = compute_rmse(&flat_residuals, residual_standard_error_points);
    let r_squared = compute_global_r_squared(
        models,
        &prepared.variable_names,
        &prepared.variable_values,
        &flat_residuals,
        total_observation_residuals,
    );
    let r_squared_per_layer = compute_per_layer_r_squared(
        models,
        &prepared.variable_names,
        &prepared.variable_values,
        &final_state.layer_residuals,
    );

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
        termination_reason: termination_reason_label(termination_reason).to_owned(),
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
        parameter_uncertainties: inference.uncertainties_scaled,
        parameter_uncertainties_raw: inference.uncertainties_raw,
        parameter_expanded_uncertainties: inference.expanded_uncertainties,
        coverage_factor: inference.coverage_factor,
        parameter_covariance: inference.covariance_scaled,
        parameter_covariance_raw: inference.covariance_raw,
        parameter_correlations: inference.correlations_scaled,
        parameter_correlations_raw: inference.correlations_raw,
        residuals: flat_residuals,
        fitted_values: flat_fitted,
        chi_squared: final_state.chi_squared,
        chi_squared_observation: final_state.chi_squared_observation,
        chi_squared_observation_reduced,
        chi_squared_reduced,
        rmse,
        residual_standard_error,
        r_squared,
        r_squared_per_layer,
        effective_rank: inference.diagnostics.effective_rank,
        condition_number: inference.diagnostics.condition_number,
        inner_stationarity_norm_max: final_state.inner_stationarity_norm_max,
        inner_stationarity_norm_mean: final_state.inner_stationarity_norm_mean,
        welch_satterthwaite_dof: ws_dof,
        coverage_degrees_of_freedom: coverage_dof,
        assumptions,
    }
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
