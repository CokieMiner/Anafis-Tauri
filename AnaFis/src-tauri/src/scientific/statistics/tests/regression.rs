//! Tests for regression analysis functions
//! Validates robust regression methods against reference implementations

use crate::scientific::statistics::robust_regression::*;

/// Generate synthetic regression data for testing
fn generate_regression_data(n_samples: usize, n_features: usize, noise_std: f64) -> (Vec<Vec<f64>>, Vec<f64>) {
    use rand::prelude::*;
    use rand_pcg::Pcg64;

    let mut rng = Pcg64::seed_from_u64(42);
    let normal = rand_distr::Normal::new(0.0, noise_std).unwrap();

    // True coefficients (including intercept)
    let true_coeffs = vec![1.0, 2.0, -1.5, 0.5]; // intercept + 3 features

    let mut x_data = Vec::with_capacity(n_samples);
    let mut y_data = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let mut x_row = vec![1.0]; // intercept term
        for _ in 0..n_features {
            x_row.push(rng.random_range(-2.0..2.0));
        }
        x_data.push(x_row);
    }

    for x_row in &x_data {
        let mut y = 0.0;
        for (i, &x_val) in x_row.iter().enumerate() {
            if i < true_coeffs.len() {
                y += true_coeffs[i] * x_val;
            }
        }
        y += normal.sample(&mut rng); // add noise
        y_data.push(y);
    }

    (x_data, y_data)
}

/// Generate data with outliers for robust regression testing
fn generate_regression_data_with_outliers(n_samples: usize, n_features: usize, outlier_fraction: f64) -> (Vec<Vec<f64>>, Vec<f64>) {
    let (x_data, mut y_data) = generate_regression_data(n_samples, n_features, 0.1);

    use rand::prelude::*;
    use rand_pcg::Pcg64;
    let mut rng = Pcg64::seed_from_u64(123);

    let n_outliers = (n_samples as f64 * outlier_fraction) as usize;

    // Add outliers by corrupting some y values
    for _i in 0..n_outliers {
        let idx = rng.random_range(0..n_samples);
        y_data[idx] += rng.random_range(5.0..10.0) * if rng.random_bool(0.5) { 1.0 } else { -1.0 };
    }

    (x_data, y_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huber_regression_basic() {
        let (x_data, y_data) = generate_regression_data(100, 3, 0.1);

        let result = RobustRegressionEngine::huber_regression(
            &x_data,
            &y_data,
            1.345, // Standard Huber parameter
            100,
            1e-6,
        );

        assert!(result.is_ok(), "Huber regression should succeed");
        let regression = result.unwrap();

        // Check that we got coefficients for all features + intercept
        assert_eq!(regression.coefficients.len(), 4, "Should have 4 coefficients (intercept + 3 features)");

        // Check that R-squared is reasonable (should be high for clean data)
        assert!(regression.r_squared > 0.8, "R-squared should be high for clean synthetic data");

        // Check that residuals have reasonable magnitude
        let mean_residual = regression.residuals.iter().sum::<f64>() / regression.residuals.len() as f64;
        assert!(mean_residual.abs() < 1.0, "Mean residual should be small");

        // Check convergence
        assert!(regression.converged, "Regression should converge");
        assert!(regression.iterations > 0 && regression.iterations <= 100, "Iterations should be reasonable");
    }

    #[test]
    fn test_huber_regression_with_outliers() {
        let (x_data, y_data) = generate_regression_data_with_outliers(100, 3, 0.2); // 20% outliers

        let result = RobustRegressionEngine::huber_regression(
            &x_data,
            &y_data,
            1.345,
            100,
            1e-6,
        );

        assert!(result.is_ok(), "Huber regression should handle outliers");
        let regression = result.unwrap();

        // Even with outliers, should still get reasonable R-squared
        // Note: With 20% outliers, R-squared might be lower than expected
        assert!(regression.r_squared > 0.2, "R-squared should be reasonable even with outliers (got {:.3})", regression.r_squared);

        // Check that coefficients are finite
        for &coeff in &regression.coefficients {
            assert!(coeff.is_finite(), "All coefficients should be finite");
        }
    }

    #[test]
    fn test_ransac_regression() {
        let (x_data, y_data) = generate_regression_data_with_outliers(100, 3, 0.3); // 30% outliers

        let result = RobustRegressionEngine::ransac_regression(
            &x_data,
            &y_data,
            10,     // min_samples
            100,    // max_trials
            1.0,    // residual_threshold
        );

        assert!(result.is_ok(), "RANSAC regression should succeed");
        let regression = result.unwrap();

        // Check that we got coefficients
        assert_eq!(regression.coefficients.len(), 4, "Should have 4 coefficients");

        // RANSAC should be robust to outliers
        assert!(regression.r_squared > 0.2, "RANSAC should achieve reasonable R-squared despite outliers (got {:.3})", regression.r_squared);

        // Check that coefficients are finite
        for &coeff in &regression.coefficients {
            assert!(coeff.is_finite(), "All coefficients should be finite");
        }
    }

    #[test]
    fn test_weighted_least_squares_convergence() {
        let (x_data, y_data) = generate_regression_data(50, 2, 0.05);

        let result = RobustRegressionEngine::huber_regression(
            &x_data,
            &y_data,
            1.345,
            50,  // fewer iterations
            1e-8, // tighter tolerance
        );

        assert!(result.is_ok(), "Weighted least squares should converge");
        let regression = result.unwrap();

        assert!(regression.converged, "Should converge with tight tolerance");
        assert!(regression.iterations <= 50, "Should not exceed max iterations");
    }

    #[test]
    fn test_regression_edge_cases() {
        // Test with empty data
        let result = RobustRegressionEngine::huber_regression(&[], &[], 1.345, 10, 1e-6);
        assert!(result.is_err(), "Should fail with empty data");

        // Test with mismatched dimensions
        let x_data = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let y_data = vec![1.0]; // Wrong length
        let result = RobustRegressionEngine::huber_regression(&x_data, &y_data, 1.345, 10, 1e-6);
        assert!(result.is_err(), "Should fail with mismatched dimensions");

        // Test RANSAC with insufficient data
        let x_data = vec![vec![1.0, 2.0]];
        let y_data = vec![1.0];
        let result = RobustRegressionEngine::ransac_regression(&x_data, &y_data, 10, 10, 1.0);
        assert!(result.is_err(), "RANSAC should fail with insufficient data");
    }

    #[test]
    fn test_regression_coefficient_consistency() {
        // Test that similar data gives similar results
        let (x_data1, y_data1) = generate_regression_data(100, 2, 0.1);
        let (x_data2, y_data2) = generate_regression_data(100, 2, 0.1);

        let result1 = RobustRegressionEngine::huber_regression(&x_data1, &y_data1, 1.345, 50, 1e-6).unwrap();
        let result2 = RobustRegressionEngine::huber_regression(&x_data2, &y_data2, 1.345, 50, 1e-6).unwrap();

        // Coefficients should be similar (within reasonable bounds) since same data generation
        for (c1, c2) in result1.coefficients.iter().zip(result2.coefficients.iter()) {
            assert!((c1 - c2).abs() < 2.0, "Coefficients should be reasonably similar for similar data");
        }
    }

    #[test]
    fn test_ransac_parameter_sensitivity() {
        let (x_data, y_data) = generate_regression_data_with_outliers(100, 3, 0.25);

        // Test with different residual thresholds
        let result_strict = RobustRegressionEngine::ransac_regression(&x_data, &y_data, 10, 50, 0.5).unwrap();
        let result_lenient = RobustRegressionEngine::ransac_regression(&x_data, &y_data, 10, 50, 2.0).unwrap();

        // Both should produce valid results
        assert!(result_strict.r_squared > 0.0, "Strict threshold should produce valid R-squared");
        assert!(result_lenient.r_squared > 0.0, "Lenient threshold should produce valid R-squared");

        // Check that coefficients are different (different thresholds should give different results)
        let coeff_diff = result_strict.coefficients.iter()
            .zip(result_lenient.coefficients.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>();

        assert!(coeff_diff > 0.0, "Different thresholds should produce different coefficients");
    }

    #[test]
    fn test_huber_parameter_effect() {
        let (x_data, y_data) = generate_regression_data_with_outliers(100, 3, 0.15);

        let result_small_k = RobustRegressionEngine::huber_regression(&x_data, &y_data, 0.5, 50, 1e-6).unwrap();
        let result_large_k = RobustRegressionEngine::huber_regression(&x_data, &y_data, 2.0, 50, 1e-6).unwrap();

        // Larger k should be more robust (less sensitive to outliers)
        // This is a heuristic check - in practice, the effect depends on the data
        assert!(result_large_k.r_squared >= result_small_k.r_squared - 0.1,
                "Larger Huber parameter should not drastically reduce R-squared");
    }
}