use statrs::distribution::{ContinuousCDF, Normal};
use thiserror::Error;

/// Error type for confidence calculations
#[derive(Debug, Error)]
pub enum ConfidenceError {
    #[error("Confidence level must be between 0 and 100, got {0}")]
    InvalidLevel(f64),
    #[error("Sigma value must be positive, got {0}")]
    InvalidSigma(f64),
}

/// Convert confidence percentage to sigma value using the normal distribution
///
/// # Arguments
/// * `confidence_percent` - Confidence level as a percentage (e.g., 95.0 for 95%)
///
/// # Returns
/// The corresponding sigma value for a two-sided confidence interval
///
/// # Examples
/// ```
/// # use anafis_lib::scientific::uncertainty_propagation::confidence::confidence_to_sigma;
/// let sigma = confidence_to_sigma(95.0).unwrap();
/// assert!((sigma - 1.96).abs() < 0.01);
/// ```
pub fn confidence_to_sigma(confidence_percent: f64) -> Result<f64, ConfidenceError> {
    if !confidence_percent.is_finite() || !(0.0..=100.0).contains(&confidence_percent) {
        return Err(ConfidenceError::InvalidLevel(confidence_percent));
    }

    // Convert percentage to proportion (e.g., 95.0 -> 0.95)
    let p = confidence_percent / 100.0;

    // Calculate the target quantile for a two-sided interval
    // e.g., 95% confidence -> 2.5% tails -> 0.975 quantile
    let target_quantile = 1.0 - (1.0 - p) / 2.0;

    let normal = Normal::new(0.0, 1.0).expect("Failed to create standard normal distribution");

    Ok(normal.inverse_cdf(target_quantile))
}

/// Convert sigma value to confidence percentage
///
/// # Arguments
/// * `sigma` - The sigma value
///
/// # Returns
/// The corresponding confidence level as a percentage
///
/// # Examples
/// ```
/// # use anafis_lib::scientific::uncertainty_propagation::confidence::sigma_to_confidence;
/// let confidence = sigma_to_confidence(1.96).unwrap();
/// assert!((confidence - 95.0).abs() < 0.1);
/// ```
pub fn sigma_to_confidence(sigma: f64) -> Result<f64, ConfidenceError> {
    if !sigma.is_finite() || sigma <= 0.0 {
        return Err(ConfidenceError::InvalidSigma(sigma));
    }

    let normal = Normal::new(0.0, 1.0).expect("Failed to create standard normal distribution");

    // Calculate one-sided tail probability
    let tail_prob = 1.0 - normal.cdf(sigma);

    // Convert to two-sided confidence level
    let confidence = (1.0 - 2.0 * tail_prob) * 100.0;

    Ok(confidence)
}

/// Validate that a confidence level is within valid range
pub fn validate_confidence_level(confidence: f64) -> Result<(), ConfidenceError> {
    if !confidence.is_finite() || !(0.0..=100.0).contains(&confidence) {
        Err(ConfidenceError::InvalidLevel(confidence))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_to_sigma_95() {
        let sigma = confidence_to_sigma(95.0).unwrap();
        assert!((sigma - 1.96).abs() < 0.01);
    }

    #[test]
    fn test_confidence_to_sigma_68() {
        let sigma = confidence_to_sigma(68.0).unwrap();
        assert!((sigma - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_confidence_to_sigma_99() {
        let sigma = confidence_to_sigma(99.0).unwrap();
        assert!((sigma - 2.576).abs() < 0.01);
    }

    #[test]
    fn test_sigma_to_confidence() {
        let confidence = sigma_to_confidence(1.96).unwrap();
        assert!((confidence - 95.0).abs() < 0.1);
    }

    #[test]
    fn test_round_trip() {
        let original_confidence = 95.0;
        let sigma = confidence_to_sigma(original_confidence).unwrap();
        let recovered_confidence = sigma_to_confidence(sigma).unwrap();
        assert!((original_confidence - recovered_confidence).abs() < 0.01);
    }

    #[test]
    fn test_invalid_confidence() {
        assert!(confidence_to_sigma(150.0).is_err());
        assert!(confidence_to_sigma(-10.0).is_err());
        assert!(confidence_to_sigma(f64::NAN).is_err());
    }

    #[test]
    fn test_invalid_sigma() {
        assert!(sigma_to_confidence(-1.0).is_err());
        assert!(sigma_to_confidence(0.0).is_err());
        assert!(sigma_to_confidence(f64::NAN).is_err());
    }
}
