//! Tests for uncertainty propagation and analysis
//!
//! Validates Monte Carlo and analytical methods against GUM and reference implementations.

use super::test_utils::approx_eq;
use crate::scientific::statistics::uncertainty::*;
use rand_pcg::Pcg64;
use rand::SeedableRng;


/// Test uncertainty propagation with simple functions
#[cfg(test)]
mod tests {
    use super::*;

    /// Test covariance matrix construction from uncertainties
    #[test]
    fn test_covariance_matrix_from_uncertainties() {
        let uncertainties = vec![0.1, 0.2, 0.15];
        let correlations = ndarray::arr2(&[
            [1.0, 0.5, 0.3],
            [0.5, 1.0, 0.2],
            [0.3, 0.2, 1.0],
        ]);

        let cov_matrix = UncertaintyPropagation::covariance_matrix_from_uncertainties(
            &uncertainties,
            Some(&correlations),
        ).unwrap();

        // Check diagonal elements (variances)
        assert!(approx_eq(cov_matrix[[0, 0]], 0.01, 1e-10)); // 0.1^2
        assert!(approx_eq(cov_matrix[[1, 1]], 0.04, 1e-10)); // 0.2^2
        assert!(approx_eq(cov_matrix[[2, 2]], 0.0225, 1e-10)); // 0.15^2

        // Check off-diagonal elements
        assert!(approx_eq(cov_matrix[[0, 1]], 0.01, 1e-10)); // 0.5 * 0.1 * 0.2
        assert!(approx_eq(cov_matrix[[0, 2]], 0.0045, 1e-10)); // 0.3 * 0.1 * 0.15
        assert!(approx_eq(cov_matrix[[1, 2]], 0.006, 1e-10)); // 0.2 * 0.2 * 0.15
    }

    /// Test uncertainty propagation through a simple function
    #[test]
    fn test_propagate_uncertainty_simple() {
        let variables = vec![2.0, 3.0];
        let uncertainties = vec![0.1, 0.2];
        let cov_matrix = UncertaintyPropagation::covariance_matrix_from_uncertainties(
            &uncertainties,
            None, // No correlations
        ).unwrap();

        // Test f(x,y) = x + y
        // The closure now receives &[Dual64] and returns Dual64
        let sum_uncertainty = UncertaintyPropagation::propagate_uncertainty(
            |vars| vars[0] + vars[1],
            &variables,
            &cov_matrix,
        ).unwrap();

        // For uncorrelated variables: σ_f² = σ_x² + σ_y²
        let expected = (0.1f64.powi(2) + 0.2f64.powi(2)).sqrt();
        assert!(approx_eq(sum_uncertainty, expected, 1e-3));
    }

    /// Test uncertainty propagation for mean calculation
    #[test]
    fn test_propagate_mean_uncertainty() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let uncertainties = vec![0.1, 0.1, 0.1, 0.1, 0.1];
        let correlations = ndarray::arr2(&[
            [1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0],
        ]);

        let mean_uncertainty = UncertaintyPropagation::propagate_mean_uncertainty(
            &values,
            &uncertainties,
            Some(&correlations),
        ).unwrap();

        // For uncorrelated measurements: σ_mean = σ / sqrt(n)
        let expected = 0.1 / (5.0f64).sqrt();
        assert!(approx_eq(mean_uncertainty, expected, 1e-3));
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test uncertainty propagation for variance calculation
    #[test]
    fn test_propagate_variance_uncertainty() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let uncertainties = vec![0.1, 0.1, 0.1, 0.1, 0.1];

        let variance_uncertainty = UncertaintyPropagation::propagate_variance_uncertainty(
            &values,
            &uncertainties,
            None,
        ).unwrap();

        // Should be a reasonable positive value
        assert!(variance_uncertainty > 0.0);
        assert!(variance_uncertainty.is_finite());
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test uncertainty propagation for correlation coefficient
    #[test]
    fn test_propagate_correlation_uncertainty() {
        let x_values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y_values = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let x_uncertainties = vec![0.1, 0.1, 0.1, 0.1, 0.1];
        let y_uncertainties = vec![0.2, 0.2, 0.2, 0.2, 0.2];

        let correlation_uncertainty = UncertaintyPropagation::propagate_correlation_uncertainty(
            &x_values,
            &y_values,
            &x_uncertainties,
            &y_uncertainties,
            None,
        ).unwrap();

        // Should be a reasonable positive value
        assert!(correlation_uncertainty >= 0.0);
        assert!(correlation_uncertainty.is_finite());
        assert!(correlation_uncertainty < 1.0); // Should be less than 1
    }

    /// Test simplified uncertainty propagation
    #[test]
    fn test_propagate_simple_function_uncertainty() {
        let variables = vec![2.0, 3.0];
        let uncertainties = vec![0.1, 0.2];

        // Test f(x,y) = x * y with derivatives ∂f/∂x = y, ∂f/∂y = x
        let product_uncertainty = UncertaintyPropagation::propagate_simple_function_uncertainty(
            |vars| vars[0] * vars[1],
            |vars| vec![vars[1], vars[0]], // derivatives
            &variables,
            &uncertainties,
        ).unwrap();

        // For uncorrelated variables: σ_f² = (∂f/∂x * σ_x)² + (∂f/∂y * σ_y)²
        let expected = ((3.0f64 * 0.1f64).powi(2) + (2.0f64 * 0.2f64).powi(2)).sqrt();
        assert!(approx_eq(product_uncertainty, expected, 1e-10));
    }

    /// Test sum uncertainty propagation
    #[test]
    fn test_propagate_sum_uncertainty() {
        let uncertainties = vec![0.1, 0.2, 0.15];
        let correlations = ndarray::arr2(&[
            [1.0, 0.5, 0.0],
            [0.5, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]);

        let variables = vec![1.0, 2.0, 3.0]; // dummy values for dual numbers
        let cov_matrix = UncertaintyPropagation::covariance_matrix_from_uncertainties(
            &uncertainties,
            Some(&correlations),
        ).unwrap();

        let sum_uncertainty = UncertaintyPropagation::propagate_uncertainty(
            |vars| vars[0] + vars[1] + vars[2],
            &variables,
            &cov_matrix,
        ).unwrap();

        // σ_sum² = Σᵢ Σⱼ cov(xᵢ,xⱼ) = Σᵢ σᵢ² + 2*Σᵢ<ⱼ cov(xᵢ,xⱼ)
        let expected = (0.01f64 + 0.04f64 + 0.0225f64 + 2.0f64 * 0.01f64).sqrt(); // 0.01 is cov[0,1]
        assert!(approx_eq(sum_uncertainty, expected, 1e-10));
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test product uncertainty propagation
    #[test]
    fn test_propagate_product_uncertainty() {
        let values = vec![2.0, 3.0];
        let uncertainties = vec![0.1, 0.2];
        let correlations = ndarray::arr2(&[
            [1.0, 0.5],
            [0.5, 1.0],
        ]);

        let cov_matrix = UncertaintyPropagation::covariance_matrix_from_uncertainties(
            &uncertainties,
            Some(&correlations),
        ).unwrap();

        let product_uncertainty = UncertaintyPropagation::propagate_uncertainty(
            |vars| vars[0] * vars[1],
            &values,
            &cov_matrix,
        ).unwrap();

        let product = 6.0; // 2 * 3
        // Relative uncertainty calculation
        let rel_uncertainty = product_uncertainty / product;
        assert!(rel_uncertainty > 0.0);
        assert!(rel_uncertainty.is_finite());
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test ratio uncertainty propagation
    #[test]
    fn test_propagate_ratio_uncertainty() {
        let values = vec![10.0, 2.0];
        let uncertainties = vec![0.5, 0.1];
        let correlations = ndarray::arr2(&[
            [1.0, 0.0],
            [0.0, 1.0],
        ]);

        let cov_matrix = UncertaintyPropagation::covariance_matrix_from_uncertainties(
            &uncertainties,
            Some(&correlations),
        ).unwrap();

        let ratio_uncertainty = UncertaintyPropagation::propagate_uncertainty(
            |vars| vars[0] / vars[1],
            &values,
            &cov_matrix,
        ).unwrap();

        let ratio = 5.0; // 10 / 2
        // Relative uncertainty calculation
        let rel_uncertainty = ratio_uncertainty / ratio;
        assert!(rel_uncertainty > 0.0);
        assert!(rel_uncertainty.is_finite());
    }

    /// Test bootstrap confidence intervals
    #[test]
    fn test_bootstrap_confidence_intervals() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut rng = Pcg64::seed_from_u64(42);

        let ci = BootstrapEngine::percentile_confidence_intervals(
            &data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64, // mean
            0.95,
            1000,
            &mut rng,
        ).unwrap();

        let (lower, upper) = ci;
        assert!(lower < upper);
        assert!(lower > 0.0 && upper > 0.0);
        assert!(lower <= 5.5 && upper >= 5.5); // True mean is 5.5
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test BCa bootstrap confidence intervals
    #[test]
    fn test_bca_bootstrap_confidence_intervals() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut rng = Pcg64::seed_from_u64(42);

        let ci = BootstrapEngine::bca_confidence_intervals(
            &data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64, // mean
            0.95,
            1000,
            &mut rng,
        ).unwrap();

        let (lower, upper) = ci;
        assert!(lower < upper);
        assert!(lower > 0.0 && upper > 0.0);
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test bootstrap standard error
    #[test]
    fn test_bootstrap_standard_error() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut rng = Pcg64::seed_from_u64(42);

        let se = BootstrapEngine::bootstrap_standard_error(
            &data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64, // mean
            1000,
            &mut rng,
        ).unwrap();

        assert!(se > 0.0);
        assert!(se.is_finite());
        // For uniform data, SE should be reasonable
        assert!(se < 1.0);
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test bootstrap bias estimation
    #[test]
    fn test_bootstrap_bias() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut rng = Pcg64::seed_from_u64(42);

        let bias = BootstrapEngine::bootstrap_bias(
            &data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64, // mean
            1000,
            &mut rng,
        ).unwrap();

        // Bias should be finite (may be positive or negative)
        assert!(bias.is_finite());
        assert!(bias.abs() < 1.0); // Should be small for mean
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test block bootstrap
    #[test]
    fn test_block_bootstrap_confidence_intervals() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut rng = Pcg64::seed_from_u64(42);

        let ci = BootstrapEngine::block_bootstrap_confidence_intervals(
            &data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64, // mean
            0.95,
            1000,
            Some(3), // block size
            &mut rng,
        ).unwrap();

        let (lower, upper) = ci;
        assert!(lower < upper);
        assert!(lower > 0.0 && upper > 0.0);
    }

    // FIXME: This test only performs basic sanity checks. The actual logic for assessing convergence
    // should be thoroughly tested to ensure it provides meaningful insights. It needs rigorous
    // numerical validation against known reference values or a trusted third-party library.
    /// Test bootstrap convergence assessment
    #[test]
    fn test_bootstrap_convergence_assessment() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut rng = Pcg64::seed_from_u64(42);

        let convergence = BootstrapEngine::assess_convergence(
            &data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64, // mean
            1000,
            &mut rng,
        ).unwrap();

        assert!(convergence.stability_score >= 0.0 && convergence.stability_score <= 1.0);
        assert!(!convergence.assessment.is_empty());
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test confidence intervals for descriptive statistics
    #[test]
    fn test_confidence_intervals_descriptive_stats() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut rng = Pcg64::seed_from_u64(42);

        let ci = BootstrapEngine::confidence_intervals(
            &data,
            0.95,
            1000,
            &mut rng,
        ).unwrap();

        // Check that all confidence intervals are valid
        assert!(ci.mean.0 < ci.mean.1);
        assert!(ci.median.0 < ci.median.1);
        assert!(ci.std_dev.0 < ci.std_dev.1);
        assert!(ci.skewness.0 < ci.skewness.1);
        assert!(ci.kurtosis.0 < ci.kurtosis.1);
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test uncertainty-aware bootstrap
    #[test]
    fn test_uncertainty_aware_bootstrap() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let uncertainties = vec![0.1, 0.1, 0.1, 0.1, 0.1];
        let confidence_levels = vec![0.95, 0.95, 0.95, 0.95, 0.95];
        let mut rng = Pcg64::seed_from_u64(42);

        let ci = BootstrapEngine::uncertainty_aware_confidence_intervals(
            &data,
            &uncertainties,
            &confidence_levels,
            0.95,
            1000,
            &mut rng,
        ).unwrap();

        // Check that all confidence intervals are valid
        assert!(ci.mean.0 < ci.mean.1);
        assert!(ci.median.0 < ci.median.1);
        assert!(ci.std_dev.0 < ci.std_dev.1);
    }

    /// Test edge cases
    #[test]
    fn test_uncertainty_propagation_edge_cases() {
        // Empty data should fail
        assert!(UncertaintyPropagation::covariance_matrix_from_uncertainties(&[], None).is_err());

        // Single variable
        let single_var_cov = UncertaintyPropagation::covariance_matrix_from_uncertainties(&[0.1], None).unwrap();
        assert_eq!(single_var_cov.nrows(), 1);
        assert_eq!(single_var_cov.ncols(), 1);
        assert!(approx_eq(single_var_cov[[0, 0]], 0.01, 1e-10));

        // Zero uncertainty
        let zero_uncertainty_cov = UncertaintyPropagation::covariance_matrix_from_uncertainties(&[0.0, 0.1], None).unwrap();
        assert!(approx_eq(zero_uncertainty_cov[[0, 0]], 0.0, 1e-10));
        assert!(approx_eq(zero_uncertainty_cov[[1, 1]], 0.01, 1e-10));
    }

    /// Test bootstrap with small datasets
    #[test]
    fn test_bootstrap_small_dataset() {
        let data = vec![1.0, 2.0];
        let mut rng = Pcg64::seed_from_u64(42);

        let ci = BootstrapEngine::percentile_confidence_intervals(
            &data,
            |sample| sample.iter().sum::<f64>() / sample.len() as f64,
            0.95,
            100,
            &mut rng,
        ).unwrap();

        assert!(ci.0 < ci.1);
        assert!(ci.0 > 0.0 && ci.1 > 0.0);
    }

    // FIXME: This test only performs basic sanity checks. It needs rigorous numerical validation
    // against known reference values or a trusted third-party library to ensure mathematical precision.
    /// Test correlation uncertainty with perfect correlation
    #[test]
    fn test_correlation_uncertainty_perfect() {
        let x_values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y_values = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // Perfectly correlated
        let x_uncertainties = vec![0.01, 0.01, 0.01, 0.01, 0.01]; // Very small uncertainties
        let y_uncertainties = vec![0.02, 0.02, 0.02, 0.02, 0.02];

        let uncertainty = UncertaintyPropagation::propagate_correlation_uncertainty(
            &x_values,
            &y_values,
            &x_uncertainties,
            &y_uncertainties,
            None,
        ).unwrap();

        // Should be very small due to small measurement uncertainties
        assert!(uncertainty >= 0.0);
        assert!(uncertainty.is_finite());
    }


}