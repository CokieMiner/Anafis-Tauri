use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::engine::{CompiledModel, OdrTerminationReason};

pub struct WarningContext<'a> {
    pub had_uncertainty_clamp: bool,
    pub degrees_of_freedom: isize,
    pub termination_reason: OdrTerminationReason,
    pub effective_rank: usize,
    pub parameter_count: usize,
    pub condition_number: f64,
    pub inner_correction_nonconverged_points: usize,
    pub inner_stationarity_norm_max: f64,
    pub inner_stationarity_norm_mean: f64,
    pub covariance_regularization_count: usize,
    pub models: &'a [Arc<CompiledModel>],
}

pub fn build_warnings(context: &WarningContext<'_>) -> Vec<String> {
    let WarningContext {
        had_uncertainty_clamp,
        degrees_of_freedom,
        termination_reason,
        effective_rank,
        parameter_count,
        condition_number,
        inner_correction_nonconverged_points,
        inner_stationarity_norm_max,
        inner_stationarity_norm_mean,
        covariance_regularization_count,
        models,
    } = *context;

    let mut warnings: Vec<String> = Vec::new();

    if had_uncertainty_clamp {
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
    if effective_rank < parameter_count {
        warnings.push(format!(
            "Normal matrix is rank-deficient (effective rank {effective_rank} / {parameter_count}); parameter uncertainties are reported as infinite to avoid false precision"
        ));
    }
    if has_shared_measured_variable_dependencies(models) {
        warnings.push(
            "At least one dependent variable is reused as an independent variable across layers; shared latent corrections improve consistency, but this is still not a full structural latent-state model across equations"
                .to_string(),
        );
    }
    if condition_number.is_finite() && condition_number > 1e12 {
        warnings.push(format!(
            "Normal matrix is ill-conditioned (condition number {condition_number:.3e}); parameter uncertainties may be unstable"
        ));
    }
    if inner_correction_nonconverged_points > 0 {
        warnings.push(format!(
            "Per-point inner correction did not converge for {inner_correction_nonconverged_points} point/layer cases; results may be less reliable in strongly nonlinear regions"
        ));
    }
    if inner_stationarity_norm_max.is_finite() {
        if inner_stationarity_norm_max > 1e-3 {
            warnings.push(format!(
                "Inner profiled correction stationarity is weak (max {inner_stationarity_norm_max:.3e}, mean {inner_stationarity_norm_mean:.3e}); reduced-objective linearization may be inaccurate"
            ));
        } else if inner_stationarity_norm_max > 1e-6 {
            warnings.push(format!(
                "Inner profiled correction stationarity is moderate (max {inner_stationarity_norm_max:.3e}, mean {inner_stationarity_norm_mean:.3e}); verify sensitivity on strongly nonlinear datasets"
            ));
        }
    }
    if covariance_regularization_count > 0 {
        warnings.push(format!(
            "Joint covariance blocks required PSD diagonal regularization {covariance_regularization_count} times; correlated weighting was stabilized and may slightly alter uncertainty propagation"
        ));
    }

    warnings
}

pub fn build_assumptions(models_len: usize) -> Vec<String> {
    let mut assumptions = vec![
        "Orthogonal Distance Regression (ODR) accounts for uncertainties in both independent and dependent variables".to_string(),
        "Parameter uncertainties are derived from the inverse normal matrix; both raw (unscaled) and reduced-χ²-scaled values are reported (GUM convention for scaled values). Expanded uncertainties use Student-t coverage at the requested confidence level".to_string(),
        "For profiled ODR (latent x-corrections eliminated in the inner solve), profiled degrees of freedom use weighted residual-row count minus effective parameter count; observation-only degrees of freedom use N × L − P".to_string(),
        "Covariance propagation and confidence intervals assume the model is approximately linear near the optimum".to_string(),
        "R² is a descriptive statistic only; it is not a rigorous goodness-of-fit measure when predictors have uncertainty".to_string(),
    ];
    if models_len > 1 {
        assumptions.push("Per-layer R² values should be preferred over the global R², which pools layers that may have different physical units or scales".to_string());
        assumptions.push("Shared measured variables across layers are coupled through corrections, but this is not a full structural state-space model".to_string());
    }
    assumptions
}

fn has_shared_measured_variable_dependencies(models: &[Arc<CompiledModel>]) -> bool {
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
