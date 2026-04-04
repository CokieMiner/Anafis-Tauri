use statrs::distribution::{ContinuousCDF, StudentsT};
use std::sync::Arc;

use super::engine::{
    CompiledModel, EvaluationState, OdrTerminationReason, build_normal_equations, diagnose_matrix,
    invert_information_matrix,
};
use super::fit_metrics::{
    compute_global_r_squared, compute_per_layer_r_squared, compute_rmse, flatten_residuals_and_fitted,
};
use super::fit_notes::{WarningContext, build_assumptions, build_warnings};
use super::super::types::OdrFitResponse;

#[allow(clippy::too_many_lines, reason = "Building complex fit response")]
pub fn build_response(
    models: &[Arc<CompiledModel>],
    prepared: &super::engine::PreparedData,
    parameter_values: Vec<f64>,
    final_state: &EvaluationState,
    iterations: usize,
    termination_reason: OdrTerminationReason,
    confidence_level: f64,
) -> OdrFitResponse {
    let parameter_count = parameter_values.len();
    let point_count = prepared.point_count;
    let total_observation_residuals = point_count * models.len();
    let total_weighted_rows = final_state.flat_weighted_residuals.len();

    let (normal_matrix, _) = build_normal_equations(final_state);
    let diagnostics = diagnose_matrix(&normal_matrix);

    // Use numerical rank as effective parameter count to avoid overconfident scaling
    // when parameters are non-identifiable.
    let effective_parameter_count = diagnostics.effective_rank.cast_signed();
    let profiled_degrees_of_freedom = total_weighted_rows.cast_signed() - effective_parameter_count;
    let observation_degrees_of_freedom =
        total_observation_residuals.cast_signed() - effective_parameter_count;
    let chi_squared_reduced = if profiled_degrees_of_freedom > 0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Degrees of freedom casting to f64 for division"
        )]
        let dof_f64 = profiled_degrees_of_freedom as f64;
        final_state.chi_squared / dof_f64
    } else {
        f64::NAN
    };
    let chi_squared_observation_reduced = if observation_degrees_of_freedom > 0 {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Degrees of freedom casting to f64 for division"
        )]
        let dof_f64 = observation_degrees_of_freedom as f64;
        final_state.chi_squared_observation / dof_f64
    } else {
        f64::NAN
    };

    let coverage_factor =
        coverage_factor_from_confidence(confidence_level, profiled_degrees_of_freedom)
            .unwrap_or(1.959_963_984_540_054);

    let mut warnings = build_warnings(&WarningContext {
        had_uncertainty_clamp: prepared.had_uncertainty_clamp,
        degrees_of_freedom: profiled_degrees_of_freedom,
        termination_reason,
        effective_rank: diagnostics.effective_rank,
        parameter_count,
        condition_number: diagnostics.condition_number,
        inner_correction_nonconverged_points: final_state.inner_correction_nonconverged_points,
        inner_stationarity_norm_max: final_state.inner_stationarity_norm_max,
        inner_stationarity_norm_mean: final_state.inner_stationarity_norm_mean,
        covariance_regularization_count: final_state.covariance_regularization_count,
        models,
    });

    let (
        parameter_uncertainties,
        parameter_covariance,
        parameter_uncertainties_raw,
        parameter_covariance_raw,
    ) =
        match invert_information_matrix(normal_matrix) {
            Ok(covariance) => {
                let covariance_scale =
                    if profiled_degrees_of_freedom > 0 && chi_squared_reduced.is_finite() {
                    chi_squared_reduced.max(0.0)
                } else {
                    1.0
                };

                let mut cov_matrix_scaled: Vec<Vec<f64>> = (0..parameter_count)
                    .map(|i| {
                        (0..parameter_count)
                            .map(|j| covariance[(i, j)] * covariance_scale)
                            .collect()
                    })
                    .collect();

                let mut cov_matrix_raw: Vec<Vec<f64>> = (0..parameter_count)
                    .map(|i| {
                        (0..parameter_count)
                            .map(|j| covariance[(i, j)])
                            .collect()
                    })
                    .collect();

                let mut uncertainties_scaled: Vec<f64> = (0..parameter_count)
                    .map(|idx| (covariance[(idx, idx)] * covariance_scale).max(0.0).sqrt())
                    .collect();

                let mut uncertainties_raw: Vec<f64> = (0..parameter_count)
                    .map(|idx| covariance[(idx, idx)].max(0.0).sqrt())
                    .collect();

                if diagnostics.effective_rank < parameter_count {
                    uncertainties_scaled.fill(f64::INFINITY);
                    uncertainties_raw.fill(f64::INFINITY);
                    for (row_idx, row) in cov_matrix_scaled.iter_mut().enumerate() {
                        row.fill(f64::NAN);
                        row[row_idx] = f64::INFINITY;
                    }
                    for (row_idx, row) in cov_matrix_raw.iter_mut().enumerate() {
                        row.fill(f64::NAN);
                        row[row_idx] = f64::INFINITY;
                    }
                }

                (
                    uncertainties_scaled,
                    cov_matrix_scaled,
                    uncertainties_raw,
                    cov_matrix_raw,
                )
            }
            Err(error) => {
                warnings.push(format!(
                    "Fit converged, but parameter covariance could not be estimated: {error}"
                ));
                (
                    vec![f64::NAN; parameter_count],
                    vec![vec![f64::NAN; parameter_count]; parameter_count],
                    vec![f64::NAN; parameter_count],
                    vec![vec![f64::NAN; parameter_count]; parameter_count],
                )
            }
        };

    let parameter_expanded_uncertainties: Vec<f64> = parameter_uncertainties
        .iter()
        .map(|value| value * coverage_factor)
        .collect();

    let assumptions = build_assumptions(models.len());

    let (flat_residuals, flat_fitted) = flatten_residuals_and_fitted(
        &final_state.layer_residuals,
        &final_state.layer_fitted_values,
        total_observation_residuals,
    );

    let rmse = compute_rmse(&flat_residuals, total_observation_residuals.max(1));
    let residual_standard_error_points = if observation_degrees_of_freedom > 0 {
        observation_degrees_of_freedom.unsigned_abs()
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
        parameter_uncertainties_raw,
        parameter_expanded_uncertainties,
        coverage_factor,
        parameter_covariance,
        parameter_covariance_raw,
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
        effective_rank: diagnostics.effective_rank,
        condition_number: diagnostics.condition_number,
        inner_stationarity_norm_max: final_state.inner_stationarity_norm_max,
        inner_stationarity_norm_mean: final_state.inner_stationarity_norm_mean,
        assumptions,
    }
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
