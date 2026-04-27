//! Core Orthogonal Distance Regression (ODR) solver.
//!
//! This module implements the global Levenberg-Marquardt optimization loop
//! for ODR problems. It orchestrates the evaluation of multiple model layers,
//! handles parameter scaling and damping, and enforces termination criteria
//! based on gradient norm and step size.
//!
//! The solver is designed to be GUM-compliant, tracking second-order corrections
//! and effective degrees-of-freedom throughout the optimization process.

use nalgebra::DVector;
use std::sync::Arc;

use super::{
    CompiledModel, EvaluationState, MAX_DAMPING, MIN_DAMPING, OdrResult, OdrTerminationReason,
    PreparedData, build_normal_equations, diagnose_matrix, evaluate_model, solve_linear_system,
};

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
    let mut consecutive_rejections = 0_usize;
    let parameter_count = parameters.len();
    let mut parameter_scales = vec![1.0_f64; parameter_count];
    let mut effective_scales = vec![1.0_f64; parameter_count];
    let mut trial_parameters = vec![0.0_f64; parameter_count];

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
        let predicted_reduction =
            -2.0_f64.mul_add(gradient_vector.dot(&delta), -delta.dot(&h_delta));

        // If the quadratic model predicts non-positive reduction, the step
        // direction is unreliable (curvature dominated by second-order terms
        // or numerical noise). Reject by setting ρ = −1 rather than clamping
        // predicted_reduction to a tiny positive value, which would make ρ
        // artificially huge and aggressively reduce λ.
        let rho = if predicted_reduction.is_finite() && predicted_reduction > 0.0 {
            actual_reduction / predicted_reduction
        } else {
            -1.0
        };

        if actual_reduction > 0.0 && rho.is_finite() && rho > 0.0 {
            let improvement = actual_reduction.abs();
            parameters.clone_from(&trial_parameters);
            current = trial;
            consecutive_rejections = 0;

            // Cubic LM update for accepted steps: lambda <- lambda * max(1/3, 1-(2*rho-1)^3)
            let cubic_factor = 1.0 - (2.0_f64.mul_add(rho, -1.0)).powi(3);
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
