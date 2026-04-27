use std::collections::HashSet;
use std::sync::Arc;

use super::engine::{CORRECTION_VARIANCE_THRESHOLD, CompiledModel, OdrTerminationReason};

#[allow(
    clippy::struct_excessive_bools,
    reason = "Warning context pools many status flags for report generation"
)]
pub struct WarningContext<'model> {
    pub had_uncertainty_clamp: bool,
    pub had_low_count_poisson: bool,
    pub inferred_type_a_dof_count: usize,
    pub degrees_of_freedom: isize,
    pub used_normal_coverage_fallback: bool,
    pub termination_reason: OdrTerminationReason,
    pub effective_rank: usize,
    pub parameter_count: usize,
    pub condition_number: f64,
    pub inner_correction_nonconverged_points: usize,
    pub inner_stationarity_norm_max: f64,
    pub inner_stationarity_norm_mean: f64,
    pub covariance_regularization_count: usize,
    pub fd_tensor_unconverged_perturbations: usize,
    pub had_zero_count_poisson: bool,
    pub clamped_variance_count: usize,
    pub suppressed_correction_count: usize,
    pub observation_reduced_chi_squared: f64,
    pub models: &'model [Arc<CompiledModel>],
}

#[allow(
    clippy::too_many_lines,
    reason = "Comprehensive warning logic exceeds default line limit"
)]
pub fn build_warnings(context: &WarningContext<'_>) -> Vec<String> {
    let WarningContext {
        had_uncertainty_clamp,
        had_low_count_poisson,
        inferred_type_a_dof_count,
        degrees_of_freedom,
        used_normal_coverage_fallback,
        termination_reason,
        effective_rank,
        parameter_count,
        condition_number,
        inner_correction_nonconverged_points,
        inner_stationarity_norm_max,
        inner_stationarity_norm_mean,
        covariance_regularization_count,
        fd_tensor_unconverged_perturbations,
        had_zero_count_poisson,
        clamped_variance_count,
        suppressed_correction_count,
        observation_reduced_chi_squared,
        models,
    } = *context;

    let mut warnings: Vec<String> = Vec::with_capacity(8);

    if had_uncertainty_clamp {
        warnings.push(format!(
            "Some zero/near-zero uncertainties were clamped to a minimum positive value ({clamped_variance_count} individual variance entries affected)"
        ));
    }
    if had_low_count_poisson {
        let msg = if had_zero_count_poisson {
            "Poisson weighting was applied to one or more zero counts (variance clamped to MIN_VARIANCE). Plug-in sigma=sqrt(n) may significantly underestimate uncertainty for low counts"
        } else {
            "Poisson weighting observed low counts (<20); plug-in sigma=sqrt(n) may underestimate uncertainty in this regime"
        };
        warnings.push(msg.to_owned());
    }
    if inferred_type_a_dof_count > 0 {
        warnings.push(format!(
            "Type A uncertainty DOF was auto-inferred as n-1 for {inferred_type_a_dof_count} variable(s); provide explicit uncertainty DOF to override"
        ));
    }
    if degrees_of_freedom <= 0 {
        warnings.push(
            "Degrees of freedom <= 0: reduced chi-squared and coverage-factor interpretation may be unreliable".to_owned(),
        );
    }
    if termination_reason == OdrTerminationReason::MaxIterations {
        warnings.push(
            "Maximum iterations reached before convergence; reporting best available estimate"
                .to_owned(),
        );
    }
    if termination_reason == OdrTerminationReason::DampingSaturated {
        warnings.push(
            "Damping saturated before convergence; solution may be weakly constrained".to_owned(),
        );
    }
    if effective_rank < parameter_count {
        warnings.push(format!(
            "Normal matrix is rank-deficient (effective rank {effective_rank} / {parameter_count}); parameter uncertainties are reported as infinite to avoid false precision"
        ));
    }
    if has_shared_measured_variable_dependencies(models) {
        warnings.push(
            "At least one dependent variable is reused as an independent variable across layers; latent corrections are jointly coupled across layers, while the outer optimizer still uses a Gauss-Newton approximation of the profiled objective curvature".to_owned(),
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
    if used_normal_coverage_fallback {
        warnings.push(
            "Coverage factor fell back to normal approximation (k\u{2248}1.96); effective degrees of freedom for Student-t were unavailable or non-positive".to_owned(),
        );
    }
    if fd_tensor_unconverged_perturbations > 0 {
        warnings.push(format!(
            "Finite-difference d²c*/dβ² tensor had {fd_tensor_unconverged_perturbations} unconverged perturbation(s); those entries were zeroed to prevent noise in the outer curvature correction"
        ));
    }
    if suppressed_correction_count > 0 {
        warnings.push(format!(
            "{suppressed_correction_count} independent variable corrections were suppressed because their measurement variance was below the correction threshold ({CORRECTION_VARIANCE_THRESHOLD:.3e}); these points are effectively treated as fixed"
        ));
    }

    // Quality-of-fit assessments
    if observation_reduced_chi_squared > 5.0 {
        warnings.push(format!(
            "Poor fit detected (reduced chi-squared {observation_reduced_chi_squared:.2}); the model does not explain the data within the given uncertainties. Check for model mismatch or underestimated variances"
        ));
    }
    if observation_reduced_chi_squared < 0.1 && degrees_of_freedom > 5 {
        warnings.push(format!(
            "Suspiciously low reduced chi-squared ({observation_reduced_chi_squared:.3}); uncertainties may be overestimated or the model is potentially over-fitting"
        ));
    }

    warnings
}

pub fn build_assumptions(models_len: usize) -> Vec<String> {
    let mut assumptions = vec![
        "Orthogonal Distance Regression (ODR) accounts for uncertainties in both independent and dependent variables".to_owned(),
        "Parameter uncertainties are obtained from the inverse of the full (Gauss-Newton + second-order) normal matrix. Two sets are reported: \n  \u{2022} Raw (unscaled) \u{2013} direct inverse matrix. \n  \u{2022} Scaled \u{2013} multiplied by the observation-only reduced chi-squared (\u{3c7}\u{b2}obs / \u{3bd}). \nThe scaling uses only the dependent-variable part of the residuals, not the full profiled chi-squared. This choice follows the ODRPACK convention and avoids double-counting the independent-variable penalty when input uncertainties are well known. If the full profiled \u{3c7}\u{b2} is desired for scaling, it can be computed from chi_squared_reduced".to_owned(),
        "For profiled ODR (latent x-corrections eliminated in the inner solve), profiled and observation-based degrees of freedom both use N \u{d7} L \u{2212} Peff, where N is the number of data points, L the number of layers, and Peff the numerical rank of the final normal matrix (i.e. the effective number of identifiable parameters). When the matrix is rank-deficient, this avoids over-optimistic DOF values".to_owned(),
        "The outer curvature model augments Gauss-Newton with second-order terms (including profiled implicit-correction coupling) to improve uncertainty fidelity for nonlinear models".to_owned(),
        "The outer Hessian includes only the dependent-residual curvature contribution (Wr\u{303})_dep \u{b7} \u{2202}\u{b2}r_dep/\u{2202}\u{3b2}\u{b2}; independent-correction-row curvature terms (Wr\u{303})_indep \u{b7} (\u{2212}\u{2202}\u{b2}\u{3b4}*/\u{2202}\u{3b2}\u{b2}) are omitted, consistent with the ODRPACK / NL2SOL standard approximation. Near the solution these terms are typically small because the weighted independent residuals are near zero. This approximation is usually excellent when the independent-variable uncertainties are moderate and the fit is good. If independent uncertainties dominate and the model is severely nonlinear, the omitted terms could modestly affect parameter covariance; the reported inner stationarity norms help assess this risk".to_owned(),
        "Numerical stability safeguards include minimum positive uncertainty clamping, PSD covariance-block regularization when needed, and bounded correlation reporting".to_owned(),
        "Covariance propagation and confidence intervals assume the model is approximately linear near the optimum".to_owned(),
        "The effective DOF reported for coverage factors is obtained by combining the input-side DOF (via model-sensitivity-weighted Welch-Satterthwaite) with the residual DOF using a two-component decomposition; see GUM F.1.1.3 for the underlying principle. Correlations among input quantities are accounted for in the weighting matrix but not in this DOF combination".to_owned(),
        "R\u{b2} is a descriptive statistic only; it is not a rigorous goodness-of-fit measure when predictors have uncertainty".to_owned(),
    ];
    if models_len > 1 {
        assumptions.push("Per-layer R\u{b2} values should be preferred over the global R\u{b2}, which pools layers that may have different physical units or scales".to_owned());
        assumptions.push("Shared measured variables are coupled through the inner latent corrections; the outer curvature model uses a Gauss-Newton approximation of the profiled objective, which ignores the implicit curvature of the coupling map".to_owned());
    }
    assumptions
}

fn has_shared_measured_variable_dependencies(models: &[Arc<CompiledModel>]) -> bool {
    let mut dependent_set: HashSet<&str> = HashSet::with_capacity(models.len());
    for model in models {
        if !dependent_set.insert(model.dependent_name.as_str()) {
            return true;
        }
    }

    models.iter().any(|model| {
        model
            .independent_names
            .iter()
            .any(|name| dependent_set.contains(name.as_str()))
    })
}
