use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::uncertainty::UncertaintyPropagationEngine;
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats;

#[derive(Debug, Clone)]
pub struct UncertaintyAnalysis {
    pub uncertainty_contributions: UncertaintyContributions,
    pub propagated_results: PropagatedResults,
    pub confidence_level: f64,
}

#[derive(Debug, Clone)]
pub struct UncertaintyContributions {
    pub measurement_error: f64,
    pub sampling_error: f64,
    pub total_uncertainty: f64,
    pub measurement_contribution_percent: f64,
    pub sampling_contribution_percent: f64,
}

#[derive(Debug, Clone)]
pub struct PropagatedResults {
    pub mean_with_uncertainty: (f64, f64),
    pub correlation_with_uncertainty: (f64, f64),
}

/// Uncertainty Propagation Coordinator
/// Coordinates measurement uncertainty analysis
pub struct UncertaintyPropagationCoordinator;

impl UncertaintyPropagationCoordinator {
    /// Analyze how measurement uncertainties affect results
    pub fn analyze(
        data: &[f64],
        measurement_uncertainties: Option<&[f64]>,
        measurement_confidence_levels: Option<&[f64]>,
        uncertainty_confidence: Option<f64>,
    ) -> Result<UncertaintyAnalysis, String> {
        if data.is_empty() {
            return Err("Cannot analyze uncertainty for empty dataset".to_string());
        }

        // If no measurement uncertainties provided, estimate from data
        let uncertainties = if let Some(meas_unc) = measurement_uncertainties {
            if meas_unc.len() != data.len() {
                return Err("Measurement uncertainties must match data length".to_string());
            }
            meas_unc.to_vec()
        } else {
            // Estimate uncertainties from data variability
            Self::estimate_uncertainties_from_data(data)?
        };

        // Get confidence levels for each measurement
        let confidence_levels = if let Some(conf_levels) = measurement_confidence_levels {
            if conf_levels.len() != data.len() {
                return Err("Confidence levels must match data length".to_string());
            }
            conf_levels.to_vec()
        } else {
            // Use default confidence level for all points
            let default_confidence = uncertainty_confidence.unwrap_or(0.95);
            vec![default_confidence; data.len()]
        };

        // Compute uncertainty contributions
        let uncertainty_contributions = Self::compute_uncertainty_contributions(data, &uncertainties)?;

        // Propagate uncertainties to derived quantities
        let propagated_results = Self::propagate_uncertainties(data, &uncertainties, &confidence_levels)?;

        Ok(UncertaintyAnalysis {
            uncertainty_contributions,
            propagated_results,
            confidence_level: uncertainty_confidence.unwrap_or(0.95),
        })
    }

    /// Estimate uncertainties from data variability
    fn estimate_uncertainties_from_data(data: &[f64]) -> Result<Vec<f64>, String> {
        if data.len() < 2 {
            return Err("Need at least 2 observations to estimate uncertainties".to_string());
        }

        // Use standard deviation as uncertainty estimate
        let std_dev = UnifiedStats::variance(data).sqrt();

        // Assume constant relative uncertainty
        let relative_uncertainty = std_dev / data.iter().sum::<f64>() * data.len() as f64;

        Ok(data.iter().map(|&x| relative_uncertainty * x.abs()).collect())
    }

    /// Compute uncertainty contributions
    fn compute_uncertainty_contributions(data: &[f64], uncertainties: &[f64]) -> Result<UncertaintyContributions, String> {
        // Use proper uncertainty propagation for measurement error
        let measurement_error = uncertainties.iter().sum::<f64>() / uncertainties.len() as f64;

        // Sampling uncertainty (standard error of mean) using proper propagation
        let sampling_error = UncertaintyPropagationEngine::propagate_mean_uncertainty(
            data,
            &vec![UnifiedStats::variance(data).sqrt(); data.len()],
            None, // Assume uncorrelated sampling errors
        )?;

        let total_uncertainty = (measurement_error.powi(2) + sampling_error.powi(2)).sqrt();

        let measurement_contribution = if total_uncertainty > 0.0 {
            measurement_error.powi(2) / total_uncertainty.powi(2) * 100.0
        } else {
            0.0
        };

        let sampling_contribution = if total_uncertainty > 0.0 {
            sampling_error.powi(2) / total_uncertainty.powi(2) * 100.0
        } else {
            0.0
        };

        Ok(UncertaintyContributions {
            measurement_error,
            sampling_error,
            total_uncertainty,
            measurement_contribution_percent: measurement_contribution,
            sampling_contribution_percent: sampling_contribution,
        })
    }

    /// Propagate uncertainties to derived quantities
    fn propagate_uncertainties(
        data: &[f64],
        uncertainties: &[f64],
        confidence_levels: &[f64],
    ) -> Result<PropagatedResults, String> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;

        // Uncertainty in mean using proper propagation
        let mean_uncertainty = UncertaintyPropagationEngine::propagate_mean_uncertainty(
            data,
            uncertainties,
            None, // Assume uncorrelated measurement uncertainties
        )?;

        // Use the first confidence level for mean confidence interval (simplified)
        let mean_confidence = confidence_levels.first().copied().unwrap_or(0.95);
        let z_score = UnifiedStats::normal_quantile((1.0 + mean_confidence) / 2.0);
        let mean_with_uncertainty = (
            mean - z_score * mean_uncertainty,
            mean + z_score * mean_uncertainty,
        );

        // Uncertainty in correlation - for now, use simplified approach
        // In a full implementation, this would require correlation between x and y variables
        let correlation_uncertainty = if data.len() >= 4 {
            // Simplified: assume correlation uncertainty based on sample size
            1.0 / (data.len() as f64).sqrt()
        } else {
            0.1 // Fallback for small samples
        };

        let correlation_with_uncertainty = (
            (-1.0 + correlation_uncertainty).max(-1.0),
            (1.0 - correlation_uncertainty).min(1.0)
        );

        Ok(PropagatedResults {
            mean_with_uncertainty,
            correlation_with_uncertainty,
        })
    }
}
