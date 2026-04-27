//! GUM-compliant degrees-of-freedom logic for curve fitting.
//!
//! This module implements the statistical logic for combining uncertainty sources
//! with varying degrees of freedom. It uses the Welch-Satterthwaite equation
//! to estimate the effective degrees of freedom for the final fit result,
//! allowing for proper Student-t based coverage factor computation even when
//! some variables have small sample sizes or Type B uncertainties.

/// Logic for combining different sources of degrees-of-freedom in GUM-compliant fitting.
pub fn combine_coverage_dof_welch_satterthwaite(
    ws_dof: Option<f64>,
    fit_dof: Option<f64>,
    chi_squared_observation_reduced: f64,
) -> Option<f64> {
    let ws = ws_dof.filter(|value| value.is_finite() && *value > 0.0);
    let fit = fit_dof.filter(|value| value.is_finite() && *value > 0.0);

    match (ws, fit) {
        (Some(ws_value), Some(fit_value)) => {
            // Proper two-component Welch-Satterthwaite decomposition.
            //
            // The scaled parameter variance is:  u² = χ²_red · σ²_raw
            // where σ²_raw = (JᵀWJ)⁻¹ already propagates input uncertainties.
            //
            // We decompose into two independent components:
            //   u₁² = σ²_raw          (input-propagation, DOF = ν_ws)
            //   u₂² = max(0, χ²_red − 1) · σ²_raw  (excess residual scatter, DOF = ν_fit)
            //
            // Normalizing by σ²_raw:
            //   u₁ = 1,  u₂ = max(0, χ²_red − 1)
            //
            // W-S formula:  ν_eff = (u₁² + u₂²)² / (u₁⁴/ν₁ + u₂⁴/ν₂)
            //
            // When χ²_red ≤ 1: u₂ = 0, ν_eff = ν_ws (inputs fully explain scatter).
            // When χ²_red >> 1: ν_eff → ν_fit (residual DOF dominates).
            let s2 = if chi_squared_observation_reduced.is_finite() {
                chi_squared_observation_reduced.max(0.0)
            } else {
                1.0
            };
            let u_residual_sq = (s2 - 1.0).max(0.0);

            if u_residual_sq <= 0.0 {
                // Input uncertainties fully explain the observed scatter;
                // residual component contributes nothing.
                Some(ws_value)
            } else {
                let u_input_sq = 1.0;
                let total_sq = u_input_sq + u_residual_sq;
                let numerator = total_sq * total_sq;
                let denominator = (u_input_sq * u_input_sq) / ws_value
                    + (u_residual_sq * u_residual_sq) / fit_value;
                if denominator > 0.0 && numerator.is_finite() {
                    Some(numerator / denominator)
                } else {
                    Some(ws_value.min(fit_value))
                }
            }
        }
        (Some(ws_value), None) => Some(ws_value),
        (None, Some(fit_value)) => Some(fit_value),
        (None, None) => None,
    }
}
