use statrs::distribution::{ContinuousCDF, StudentsT};
use std::sync::Arc;

use super::super::types::OdrFitResponse;
use super::engine::{
    build_normal_equations, diagnose_matrix, invert_information_matrix, CompiledModel,
    EvaluationState, OdrTerminationReason,
};
use super::fit_metrics::{
    compute_global_r_squared, compute_per_layer_r_squared, compute_rmse,
    flatten_residuals_and_fitted,
};
use super::fit_notes::{build_assumptions, build_warnings, WarningContext};

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

    let coverage_factor_option =
        coverage_dof.and_then(|dof| coverage_factor_from_confidence(confidence_level, dof));
    let coverage_factor = coverage_factor_option.unwrap_or(1.959_963_984_540_054);
    let used_normal_coverage_fallback = coverage_factor_option.is_none();

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
        models,
    });

    let (
        parameter_uncertainties,
        parameter_covariance,
        parameter_uncertainties_raw,
        parameter_covariance_raw,
    ) = match invert_information_matrix(normal_matrix) {
        Ok(covariance) => {
            let covariance_scale = if residual_degrees_of_freedom > 0
                && chi_squared_observation_reduced.is_finite()
            {
                // Pure GUM scaling uses observation-only reduced chi-squared.
                chi_squared_observation_reduced.max(0.0)
            } else {
                1.0
            };

            if chi_squared_observation_reduced.is_finite() && chi_squared_observation_reduced < 0.5
            {
                warnings.push(
                    "Observation-only reduced chi-squared is below 0.5; pure GUM scaling reduces reported uncertainties, so raw and scaled values should both be inspected"
                        .to_string(),
                );
            }

            if diagnostics.effective_rank < parameter_count {
                let mut cov_matrix_scaled = vec![vec![f64::NAN; parameter_count]; parameter_count];
                let mut cov_matrix_raw = vec![vec![f64::NAN; parameter_count]; parameter_count];
                for idx in 0..parameter_count {
                    cov_matrix_scaled[idx][idx] = f64::INFINITY;
                    cov_matrix_raw[idx][idx] = f64::INFINITY;
                }

                (
                    vec![f64::INFINITY; parameter_count],
                    cov_matrix_scaled,
                    vec![f64::INFINITY; parameter_count],
                    cov_matrix_raw,
                )
            } else {
                let cov_matrix_scaled: Vec<Vec<f64>> = (0..parameter_count)
                    .map(|i| {
                        (0..parameter_count)
                            .map(|j| covariance[(i, j)] * covariance_scale)
                            .collect()
                    })
                    .collect();

                let cov_matrix_raw: Vec<Vec<f64>> = (0..parameter_count)
                    .map(|i| (0..parameter_count).map(|j| covariance[(i, j)]).collect())
                    .collect();

                let uncertainties_scaled: Vec<f64> = (0..parameter_count)
                    .map(|idx| (covariance[(idx, idx)] * covariance_scale).max(0.0).sqrt())
                    .collect();

                let uncertainties_raw: Vec<f64> = (0..parameter_count)
                    .map(|idx| covariance[(idx, idx)].max(0.0).sqrt())
                    .collect();

                (
                    uncertainties_scaled,
                    cov_matrix_scaled,
                    uncertainties_raw,
                    cov_matrix_raw,
                )
            }
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

    let (parameter_correlations, scaled_correlation_clamped) = covariance_to_correlation(
        &parameter_covariance,
        &parameter_uncertainties,
    );
    let (parameter_correlations_raw, raw_correlation_clamped) = covariance_to_correlation(
        &parameter_covariance_raw,
        &parameter_uncertainties_raw,
    );
    if scaled_correlation_clamped || raw_correlation_clamped {
        warnings.push(
            "At least one parameter correlation exceeded [-1, 1] before clamping; covariance may be numerically non-PSD and correlation diagnostics should be interpreted with caution"
                .to_string(),
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
        parameter_correlations,
        parameter_correlations_raw,
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
        welch_satterthwaite_dof: ws_dof,
        coverage_degrees_of_freedom: coverage_dof,
        assumptions,
    }
}

fn coverage_factor_from_confidence(confidence_level: f64, dof: f64) -> Option<f64> {
    if !(0.5..1.0).contains(&confidence_level) {
        return None;
    }

    if !dof.is_finite() || dof <= 0.0 {
        return None;
    }

    let students_t = StudentsT::new(0.0, 1.0, dof).ok()?;
    let probability = (1.0 + confidence_level) * 0.5;
    Some(students_t.inverse_cdf(probability))
}

fn covariance_to_correlation(covariance: &[Vec<f64>], stddev: &[f64]) -> (Vec<Vec<f64>>, bool) {
    let n = covariance.len();
    let mut correlation = vec![vec![f64::NAN; n]; n];
    let mut had_clamp = false;
    for (row, corr_row) in correlation.iter_mut().enumerate().take(n) {
        for (col, corr_value) in corr_row.iter_mut().enumerate().take(n) {
            if row == col {
                *corr_value = if stddev.get(row).copied().unwrap_or(f64::NAN).is_finite() {
                    1.0
                } else {
                    f64::NAN
                };
                continue;
            }

            let sigma_row = stddev.get(row).copied().unwrap_or(f64::NAN);
            let sigma_col = stddev.get(col).copied().unwrap_or(f64::NAN);
            let denom = sigma_row * sigma_col;
            let cov = covariance
                .get(row)
                .and_then(|line| line.get(col))
                .copied()
                .unwrap_or(f64::NAN);
            if denom.is_finite() && denom > 0.0 && cov.is_finite() {
                let raw_corr = cov / denom;
                if !(-1.0..=1.0).contains(&raw_corr) {
                    had_clamp = true;
                }
                *corr_value = raw_corr.clamp(-1.0, 1.0);
            }
        }
    }
    (correlation, had_clamp)
}

fn combine_coverage_dof_welch_satterthwaite(
    ws_dof: Option<f64>,
    fit_dof: Option<f64>,
    chi_squared_observation_reduced: f64,
) -> Option<f64> {
    let ws = ws_dof.filter(|value| value.is_finite() && *value > 0.0);
    let fit = fit_dof.filter(|value| value.is_finite() && *value > 0.0);

    match (ws, fit) {
        (Some(ws_value), Some(fit_value)) => {
            // Approximate two-component Welch-Satterthwaite combination.
            // Input-propagation component has normalized weight 1.0.
            // Fit-residual component weight uses observation-only reduced chi-squared as a
            // practical proxy for relative contribution.
            let input_component = 1.0;
            let fit_component = if chi_squared_observation_reduced.is_finite() {
                chi_squared_observation_reduced.max(0.0)
            } else {
                0.0
            };
            let numerator = (input_component + fit_component).powi(2);
            let denominator = input_component.powi(2) / ws_value + fit_component.powi(2) / fit_value;
            if denominator > 0.0 {
                Some(numerator / denominator)
            } else {
                Some(ws_value)
            }
        }
        (Some(ws_value), None) => Some(ws_value),
        (None, Some(fit_value)) => Some(fit_value),
        (None, None) => None,
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
