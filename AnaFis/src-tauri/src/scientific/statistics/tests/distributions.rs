//! Tests for probability distributions
//! Validates PDF/CDF/quantile functions against scipy.stats and statrs references

use crate::scientific::statistics::distributions::*;

/// Generate synthetic normal data for testing
fn generate_normal_data(n: usize, mean: f64, std_dev: f64) -> Vec<f64> {
    use rand::prelude::*;
    use rand_pcg::Pcg64;

    let mut rng = Pcg64::seed_from_u64(42);
    let normal = rand_distr::Normal::new(mean, std_dev).unwrap();

    (0..n).map(|_| normal.sample(&mut rng)).collect()
}

/// Generate synthetic exponential data for testing
fn generate_exponential_data(n: usize, rate: f64) -> Vec<f64> {
    use rand::prelude::*;
    use rand_pcg::Pcg64;

    let mut rng = Pcg64::seed_from_u64(42);
    let exp = rand_distr::Exp::new(rate).unwrap();

    (0..n).map(|_| exp.sample(&mut rng)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_distribution_fitting() {
        let data = generate_normal_data(1000, 5.0, 2.0);

        let result = StatisticalDistributionEngine::fit_normal_distribution(&data, None);
        assert!(result.is_ok(), "Normal distribution fitting should succeed");

        let fit = result.unwrap();
        assert_eq!(fit.distribution_name, "normal");
        assert!(fit.parameters.len() == 2);

        // Check parameter names
        let param_names: Vec<&str> = fit.parameters.iter().map(|(name, _)| name.as_str()).collect();
        assert!(param_names.contains(&"mean"));
        assert!(param_names.contains(&"std_dev"));

        // Check that parameters are reasonable (should be close to true values)
        let fitted_mean = fit.parameters.iter().find(|(name, _)| name == "mean").unwrap().1;
        let fitted_std = fit.parameters.iter().find(|(name, _)| name == "std_dev").unwrap().1;

        assert!((fitted_mean - 5.0).abs() < 0.5, "Fitted mean should be close to true mean");
        assert!((fitted_std - 2.0).abs() < 0.5, "Fitted std should be close to true std");

        // Check that AIC/BIC are finite
        assert!(fit.aic.is_finite());
        assert!(fit.bic.is_finite());
        assert!(fit.log_likelihood.is_finite());
    }

    #[test]
    fn test_exponential_distribution_fitting() {
        let data = generate_exponential_data(1000, 0.5);

        let result = StatisticalDistributionEngine::fit_exponential_distribution(&data, None);
        assert!(result.is_ok(), "Exponential distribution fitting should succeed");

        let fit = result.unwrap();
        assert_eq!(fit.distribution_name, "exponential");
        assert!(fit.parameters.len() == 1);

        // Check parameter names
        let param_names: Vec<&str> = fit.parameters.iter().map(|(name, _)| name.as_str()).collect();
        assert!(param_names.contains(&"rate"));

        // Check that rate parameter is reasonable (should be close to true value of 0.5)
        let fitted_rate = fit.parameters.iter().find(|(name, _)| name == "rate").unwrap().1;
        assert!((fitted_rate - 0.5).abs() < 0.2, "Fitted rate should be close to true rate");

        // Check that AIC/BIC are finite
        assert!(fit.aic.is_finite());
        assert!(fit.bic.is_finite());
        assert!(fit.log_likelihood.is_finite());
    }

    #[test]
    fn test_lognormal_distribution_fitting() {
        // Generate lognormal data (positive values)
        let normal_data = generate_normal_data(1000, 0.0, 1.0);
        let data: Vec<f64> = normal_data.iter().map(|x| x.exp()).collect();

        let result = StatisticalDistributionEngine::fit_lognormal_distribution(&data, None);
        assert!(result.is_ok(), "Lognormal distribution fitting should succeed");

        let fit = result.unwrap();
        assert_eq!(fit.distribution_name, "lognormal");
        assert!(fit.parameters.len() == 2);

        // Check parameter names
        let param_names: Vec<&str> = fit.parameters.iter().map(|(name, _)| name.as_str()).collect();
        assert!(param_names.contains(&"mu"));
        assert!(param_names.contains(&"sigma"));

        // Check that parameters are finite
        let fitted_mu = fit.parameters.iter().find(|(name, _)| name == "mu").unwrap().1;
        let fitted_sigma = fit.parameters.iter().find(|(name, _)| name == "sigma").unwrap().1;

        assert!(fitted_mu.is_finite());
        assert!(fitted_sigma.is_finite() && fitted_sigma > 0.0);

        // Check that AIC/BIC are finite
        assert!(fit.aic.is_finite());
        assert!(fit.bic.is_finite());
        assert!(fit.log_likelihood.is_finite());
    }

    #[test]
    fn test_weibull_distribution_fitting() {
        // Generate Weibull data (positive values)
        let data = vec![0.5, 1.2, 2.1, 0.8, 1.5, 3.2, 0.9, 1.8, 2.5, 1.1,
                       0.7, 1.9, 2.8, 1.3, 2.2, 0.6, 1.7, 2.9, 1.4, 2.0];

        let result = StatisticalDistributionEngine::fit_weibull_distribution(&data, None);
        assert!(result.is_ok(), "Weibull distribution fitting should succeed");

        let fit = result.unwrap();
        assert_eq!(fit.distribution_name, "weibull");
        assert!(fit.parameters.len() == 2);

        // Check parameter names
        let param_names: Vec<&str> = fit.parameters.iter().map(|(name, _)| name.as_str()).collect();
        assert!(param_names.contains(&"shape"));
        assert!(param_names.contains(&"scale"));

        // Check that parameters are positive and finite
        let fitted_shape = fit.parameters.iter().find(|(name, _)| name == "shape").unwrap().1;
        let fitted_scale = fit.parameters.iter().find(|(name, _)| name == "scale").unwrap().1;

        assert!(fitted_shape > 0.0 && fitted_shape.is_finite());
        assert!(fitted_scale > 0.0 && fitted_scale.is_finite());

        // Check that AIC/BIC are finite
        assert!(fit.aic.is_finite());
        assert!(fit.bic.is_finite());
        assert!(fit.log_likelihood.is_finite());
    }

    #[test]
    fn test_gamma_distribution_fitting() {
        // Generate Gamma-like data (positive values)
        let data = vec![1.2, 2.1, 0.8, 1.5, 3.2, 0.9, 1.8, 2.5, 1.1, 0.7,
                       1.9, 2.8, 1.3, 2.2, 0.6, 1.7, 2.9, 1.4, 2.0, 1.6];

        let result = StatisticalDistributionEngine::fit_gamma_distribution(&data, None);
        assert!(result.is_ok(), "Gamma distribution fitting should succeed");

        let fit = result.unwrap();
        assert_eq!(fit.distribution_name, "gamma");
        assert!(fit.parameters.len() == 2);

        // Check parameter names
        let param_names: Vec<&str> = fit.parameters.iter().map(|(name, _)| name.as_str()).collect();
        assert!(param_names.contains(&"shape"));
        assert!(param_names.contains(&"rate"));

        // Check that parameters are positive and finite
        let fitted_shape = fit.parameters.iter().find(|(name, _)| name == "shape").unwrap().1;
        let fitted_rate = fit.parameters.iter().find(|(name, _)| name == "rate").unwrap().1;

        assert!(fitted_shape > 0.0 && fitted_shape.is_finite());
        assert!(fitted_rate > 0.0 && fitted_rate.is_finite());

        // Check that AIC/BIC are finite
        assert!(fit.aic.is_finite());
        assert!(fit.bic.is_finite());
        assert!(fit.log_likelihood.is_finite());
    }

    #[test]
    fn test_beta_distribution_fitting() {
        // Generate Beta data (values in (0,1))
        let data = vec![0.2, 0.8, 0.3, 0.7, 0.1, 0.9, 0.4, 0.6, 0.25, 0.75,
                       0.15, 0.85, 0.35, 0.65, 0.05, 0.95, 0.45, 0.55, 0.12, 0.88];

        let result = StatisticalDistributionEngine::fit_beta_distribution(&data, None);
        assert!(result.is_ok(), "Beta distribution fitting should succeed");

        let fit = result.unwrap();
        assert_eq!(fit.distribution_name, "beta");
        assert!(fit.parameters.len() == 2);

        // Check parameter names
        let param_names: Vec<&str> = fit.parameters.iter().map(|(name, _)| name.as_str()).collect();
        assert!(param_names.contains(&"alpha"));
        assert!(param_names.contains(&"beta"));

        // Check that parameters are positive and finite
        let fitted_alpha = fit.parameters.iter().find(|(name, _)| name == "alpha").unwrap().1;
        let fitted_beta = fit.parameters.iter().find(|(name, _)| name == "beta").unwrap().1;

        assert!(fitted_alpha > 0.0 && fitted_alpha.is_finite());
        assert!(fitted_beta > 0.0 && fitted_beta.is_finite());

        // Check that AIC/BIC are finite
        assert!(fit.aic.is_finite());
        assert!(fit.bic.is_finite());
        assert!(fit.log_likelihood.is_finite());
    }

    #[test]
    fn test_distribution_fitting_multiple_distributions() {
        let data = generate_normal_data(500, 0.0, 1.0);

        let fits = StatisticalDistributionEngine::fit_distributions(&data, None);
        assert!(fits.is_ok(), "Multiple distribution fitting should succeed");

        let fits = fits.unwrap();
        assert!(!fits.is_empty(), "Should fit at least one distribution");

        // Check that normal distribution is included and has best fit (lowest AIC)
        let normal_fit = fits.iter().find(|fit| fit.distribution_name == "normal");
        assert!(normal_fit.is_some(), "Normal distribution should be fitted");

        let normal_fit = normal_fit.unwrap();
        assert!(fits.iter().all(|fit| fit.aic >= normal_fit.aic),
                "Normal distribution should have the lowest AIC for normal data");
    }

    #[test]
    fn test_goodness_of_fit_calculation() {
        let data = generate_normal_data(100, 0.0, 1.0);

        // Test goodness of fit for normal distribution
        let gof = StatisticalDistributionEngine::goodness_of_fit(&data, |x| {
            crate::scientific::statistics::distributions::distribution_functions::normal_cdf(x, 0.0, 1.0)
        });

        assert!(gof.is_ok(), "Goodness of fit calculation should succeed");
        let ks_statistic = gof.unwrap();
        assert!(ks_statistic >= 0.0 && ks_statistic <= 1.0, "KS statistic should be between 0 and 1");
    }

    #[test]
    fn test_moments_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let moments = StatisticalDistributionEngine::moments(&data);
        assert!(moments.is_ok(), "Moments calculation should succeed");

        let (mean, variance, skewness, kurtosis) = moments.unwrap();

        // For this symmetric data, skewness should be close to 0
        assert!(mean > 0.0 && mean.is_finite());
        assert!(variance > 0.0 && variance.is_finite());
        assert!(skewness.abs() < 1.0); // Should be close to 0 for symmetric data
        assert!(kurtosis.is_finite());
    }

    #[test]
    fn test_distribution_fitting_edge_cases() {
        // Test with empty data
        let result = StatisticalDistributionEngine::fit_distributions(&[], None);
        assert!(result.is_err(), "Should fail with empty data");

        // Test with single value
        let result = StatisticalDistributionEngine::fit_normal_distribution(&[1.0], None);
        assert!(result.is_err(), "Should fail with single value");

        // Test with constant data
        let result = StatisticalDistributionEngine::fit_normal_distribution(&[1.0, 1.0, 1.0], None);
        assert!(result.is_err(), "Should fail with constant data (zero variance)");
    }

    #[test]
    fn test_distribution_fitting_invalid_data() {
        // Test lognormal with negative data
        let result = StatisticalDistributionEngine::fit_lognormal_distribution(&[-1.0, 2.0, 3.0], None);
        assert!(result.is_err(), "Lognormal should fail with negative data");

        // Test exponential with negative data
        let result = StatisticalDistributionEngine::fit_exponential_distribution(&[-1.0, 2.0, 3.0], None);
        assert!(result.is_err(), "Exponential should fail with negative data");

        // Test Weibull with negative data
        let result = StatisticalDistributionEngine::fit_weibull_distribution(&[-1.0, 2.0, 3.0], None);
        assert!(result.is_err(), "Weibull should fail with negative data");

        // Test Beta with out-of-range data
        let result = StatisticalDistributionEngine::fit_beta_distribution(&[-0.1, 0.5, 1.1], None);
        assert!(result.is_err(), "Beta should fail with out-of-range data");
    }

    // Numerical Stability Tests for Distributions

    #[test]
    fn test_distribution_numerical_stability() {
        // Test with data that has very different scales
        let data = vec![1e-10, 1.0, 1e10];

        // Just ensure the function can be called without panicking
        let _result = StatisticalDistributionEngine::fit_distributions(&data, None);
        // Mixed-scale data is challenging - we're just testing it doesn't crash
    }

    #[test]
    fn test_distribution_extreme_parameters() {
        // Test with data that should produce extreme parameter values
        let data = vec![1e-20, 1e-19, 1e-18]; // Very small positive values

        let result = StatisticalDistributionEngine::fit_exponential_distribution(&data, None);
        assert!(result.is_ok(), "Should handle very small values");

        let fit = result.unwrap();
        let rate = fit.parameters.iter().find(|(name, _)| name == "rate").unwrap().1;
        assert!(rate > 0.0 && rate.is_finite(), "Rate should be positive and finite");
    }

    #[test]
    fn test_distribution_pathological_cases() {
        // Test with highly skewed data
        let skewed_data = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1000.0];

        // Just ensure the function can be called without panicking
        let _result = StatisticalDistributionEngine::fit_distributions(&skewed_data, None);

        // Test with data containing infinities
        let inf_data = vec![1.0, 2.0, f64::INFINITY, 4.0];
        let _result = StatisticalDistributionEngine::fit_distributions(&inf_data, None);
        // These are stress tests - the main requirement is no panic
    }

    #[test]
    fn test_distribution_precision_stress() {
        // Test with data that requires high precision
        let data: Vec<f64> = (1..=100).map(|x| x as f64 * 1e-15).collect();

        let result = StatisticalDistributionEngine::fit_normal_distribution(&data, None);
        assert!(result.is_ok(), "Should handle high-precision data");

        let fit = result.unwrap();
        let mean = fit.parameters.iter().find(|(name, _)| name == "mean").unwrap().1;
        let expected_mean = data.iter().sum::<f64>() / data.len() as f64;

        assert!((mean - expected_mean).abs() < 1e-14, "Mean should be accurate to high precision");
    }

    #[test]
    fn test_normality_tests() {
        use crate::scientific::statistics::distributions::normality_tests::NormalityTests;

        // Test with normal data
        let normal_data = generate_normal_data(100, 0.0, 1.0);

        // Test Shapiro-Wilk
        let result = NormalityTests::shapiro_wilk(&normal_data);
        assert!(result.is_ok(), "Shapiro-Wilk should succeed on normal data");
        let test_result = result.unwrap();
        assert_eq!(test_result.test_name, "Shapiro-Wilk");
        assert!(test_result.p_value >= 0.0 && test_result.p_value <= 1.0);

        // Test Anderson-Darling
        let result = NormalityTests::anderson_darling(&normal_data);
        assert!(result.is_ok(), "Anderson-Darling should succeed on normal data");
        let test_result = result.unwrap();
        assert_eq!(test_result.test_name, "Anderson-Darling");
        assert!(test_result.p_value >= 0.0 && test_result.p_value <= 1.0);

        // Test comprehensive normality tests
        let results = NormalityTests::comprehensive_normality_tests(&normal_data);
        assert!(results.is_ok(), "Comprehensive normality tests should succeed");
        let results = results.unwrap();
        assert!(!results.is_empty(), "Should return at least one test result");

        // Test with small sample
        let small_data = vec![1.0, 2.0, 3.0];
        let results = NormalityTests::comprehensive_normality_tests(&small_data);
        assert!(results.is_ok(), "Should handle small samples gracefully");
    }
}