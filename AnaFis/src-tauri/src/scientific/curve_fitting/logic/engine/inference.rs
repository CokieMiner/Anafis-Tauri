use nalgebra::DMatrix;
use statrs::distribution::{ContinuousCDF, StudentsT};

use super::diagnostics::diagnose_matrix;
use super::state::MatrixDiagnostics;
use crate::scientific::curve_fitting::logic::constants::MATRIX_SINGULAR_EPS;
use crate::scientific::curve_fitting::types::{OdrError, OdrResult};

/// Comprehensive parameter inference results (GUM compliant).
#[derive(Debug, Clone)]
pub struct ParameterInference {
    /// Standard uncertainties from the unscaled inverse information matrix.
    pub uncertainties_raw: Vec<f64>,
    /// Standard uncertainties scaled by reduced chi-squared (if applicable).
    pub uncertainties_scaled: Vec<f64>,
    /// Full unscaled covariance matrix.
    pub covariance_raw: Vec<Vec<f64>>,
    /// Full scaled covariance matrix.
    pub covariance_scaled: Vec<Vec<f64>>,
    /// Full unscaled correlation matrix.
    pub correlations_raw: Vec<Vec<f64>>,
    /// Full scaled correlation matrix.
    pub correlations_scaled: Vec<Vec<f64>>,
    /// Expanded uncertainties at the requested confidence level.
    pub expanded_uncertainties: Vec<f64>,
    /// Coverage factor (k) used for expanded uncertainties.
    pub coverage_factor: f64,
    /// Numerical diagnostics of the information matrix.
    pub diagnostics: MatrixDiagnostics,
    /// Whether any correlation entries were clamped to [-1, 1].
    pub correlation_clamped: bool,
}

/// A "turn-key" function for GUM-compliant parameter uncertainty propagation.
///
/// This handles inversion of the normal matrix, calculation of standard and expanded
/// uncertainties, scaling by reduced chi-squared, and Student-t coverage factors.
///
/// # Metrological Standards
/// - **Uncertainty Propagation**: Follows JCGM 100:2008 (GUM) for Type B evaluation.
/// - **DOF Combination**: Employs the Welch-Satterthwaite formula to determine effective degrees of freedom.
/// - **Scaling Assumption**: This implementation uses "observation-only" reduced chi-squared scaling
///   (NIST mode). This assumes prior uncertainties are reliable standard deviations and uses
///   the goodness-of-fit to scale the final parameter covariance, effectively treating the
///   input weights as relative rather than absolute.
pub fn compute_parameter_inference(
    information_matrix: DMatrix<f64>,
    observation_reduced_chi_squared: f64,
    confidence_level: f64,
    effective_dof: f64,
) -> OdrResult<ParameterInference> {
    let parameter_count = information_matrix.nrows();
    let diagnostics = diagnose_matrix(&information_matrix);

    // Invert the information matrix using pseudo-inverse for stability in rank-deficient cases.
    let svd = information_matrix.svd(true, true);
    let covariance = svd.pseudo_inverse(MATRIX_SINGULAR_EPS).map_err(|error| {
        OdrError::Numerical(format!("Inference covariance inversion failed: {error}"))
    })?;

    let is_rank_deficient = diagnostics.effective_rank < parameter_count;
    let covariance_scale =
        if observation_reduced_chi_squared.is_finite() && observation_reduced_chi_squared > 0.0 {
            observation_reduced_chi_squared
        } else {
            1.0
        };

    let mut uncertainties_raw = vec![0.0; parameter_count];
    let mut uncertainties_scaled = vec![0.0; parameter_count];
    let mut cov_raw = vec![vec![0.0; parameter_count]; parameter_count];
    let mut cov_scaled = vec![vec![0.0; parameter_count]; parameter_count];

    if is_rank_deficient {
        for i in 0..parameter_count {
            uncertainties_raw[i] = f64::INFINITY;
            uncertainties_scaled[i] = f64::INFINITY;
            for j in 0..parameter_count {
                if i == j {
                    cov_raw[i][j] = f64::INFINITY;
                    cov_scaled[i][j] = f64::INFINITY;
                } else {
                    cov_raw[i][j] = f64::NAN;
                    cov_scaled[i][j] = f64::NAN;
                }
            }
        }
    } else {
        for i in 0..parameter_count {
            let var_raw = covariance[(i, i)].max(0.0);
            uncertainties_raw[i] = var_raw.sqrt();
            uncertainties_scaled[i] = (var_raw * covariance_scale).sqrt();
            for j in 0..parameter_count {
                cov_raw[i][j] = covariance[(i, j)];
                cov_scaled[i][j] = covariance[(i, j)] * covariance_scale;
            }
        }
    }

    let (correlations_raw, clamped_raw) = covariance_to_correlation(&cov_raw, &uncertainties_raw);
    let (correlations_scaled, clamped_scaled) =
        covariance_to_correlation(&cov_scaled, &uncertainties_scaled);

    let coverage_factor = if effective_dof > 0.0 && effective_dof.is_finite() {
        StudentsT::new(0.0, 1.0, effective_dof)
            .ok()
            .and_then(|t| {
                let p = (1.0 + confidence_level) * 0.5;
                (p > 0.0 && p < 1.0).then(|| t.inverse_cdf(p))
            })
            .unwrap_or(1.959_963_984_540_054) // Fallback to k=1.96 (normal)
    } else {
        1.959_963_984_540_054
    };

    let expanded_uncertainties = uncertainties_scaled
        .iter()
        .map(|&u| u * coverage_factor)
        .collect();

    Ok(ParameterInference {
        uncertainties_raw,
        uncertainties_scaled,
        covariance_raw: cov_raw,
        covariance_scaled: cov_scaled,
        correlations_raw,
        correlations_scaled,
        expanded_uncertainties,
        coverage_factor,
        diagnostics,
        correlation_clamped: clamped_raw || clamped_scaled,
    })
}

fn covariance_to_correlation(covariance: &[Vec<f64>], stddev: &[f64]) -> (Vec<Vec<f64>>, bool) {
    let n = covariance.len();
    let mut correlation = vec![vec![0.0; n]; n];
    let mut had_clamp = false;
    for i in 0..n {
        for j in 0..n {
            if i == j {
                correlation[i][j] = 1.0;
            } else {
                let den = stddev[i] * stddev[j];
                if den > 0.0 && den.is_finite() {
                    let val = covariance[i][j] / den;
                    if val > 1.0 {
                        correlation[i][j] = 1.0;
                        had_clamp = true;
                    } else if val < -1.0 {
                        correlation[i][j] = -1.0;
                        had_clamp = true;
                    } else {
                        correlation[i][j] = val;
                    }
                } else {
                    correlation[i][j] = f64::NAN;
                }
            }
        }
    }
    (correlation, had_clamp)
}
