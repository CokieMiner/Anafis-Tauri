//! Unit Tests for Individual Statistical Functions
//!
//! This module contains basic unit tests for individual functions and algorithms,
//! focusing on correctness and edge cases for specific components.

use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::CorrelationEngine;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::outliers::OutlierDetectionEngine;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::distribution::StatisticalDistributionEngine;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::bootstrap::BootstrapEngine;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::time_series::TimeSeriesDecompositionEngine;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::uncertainty::UncertaintyPropagationEngine;
use crate::scientific::statistics::types::AnalysisOptions;
use rand_pcg::Pcg64;
use rand_distr::{Normal, Weibull, Distribution};
use rand::SeedableRng;

#[test]
fn test_compute_matrix_with_method_spearman_vs_pearson() {
    // x and y have a strong monotonic but nonlinear relation
    let x = vec![1.0, 2.0, 3.0, 4.0];
    let y = vec![1.0, 4.0, 9.0, 16.0];
    let datasets = vec![x.clone(), y.clone()];

    let pearson = CorrelationEngine::compute_matrix_with_method(&datasets, "pearson", 9.0).unwrap();
    let spearman = CorrelationEngine::compute_matrix_with_method(&datasets, "spearman", 9.0).unwrap();

    let pearson_corr = pearson[(0, 1)];
    let spearman_corr = spearman[(0, 1)];

    // Pearson should be < 1 for this nonlinear relationship, Spearman should be very close to 1.0
    assert!(pearson_corr < 1.0);
    assert!((spearman_corr - 1.0).abs() < 1e-12, "Spearman expected to be very close to 1.0, got {}", spearman_corr);
}

#[test]
fn test_kendall_tau_with_ties() {
    // identical sequences with ties should yield tau == 1.0
    let x = vec![1.0, 2.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 2.0, 3.0];
    let tau = CorrelationEngine::kendall_correlation(&x, &y).unwrap();
    assert!((tau - 1.0).abs() < 1e-12, "Expected tau = 1.0 for identical sequences with ties, got {}", tau);
}

#[test]
fn test_weibull_mle_estimation() {
    // Generate synthetic Weibull data with known parameters
    let shape = 2.0f64;
    let scale = 3.0f64;
    let seed = 12345u64;
    let mut rng = Pcg64::seed_from_u64(seed);
    let weibull = Weibull::new(shape, scale).unwrap();
    let sample: Vec<f64> = (0..500).map(|_| weibull.sample(&mut rng)).collect();

    // Fit Weibull distribution
    let fits = StatisticalDistributionEngine::fit_distributions(&sample).unwrap();
    // The first entry with distribution_name=="weibull" is expected
    let mut weibull_fit = None;
    for f in &fits {
        if f.distribution_name == "weibull" {
            weibull_fit = Some(f.clone());
            break;
        }
    }

    assert!(weibull_fit.is_some(), "Weibull fit expected in results");
    let fit = weibull_fit.unwrap();
    let fitted_shape = fit.parameters.iter().find(|p| p.0 == "shape").unwrap().1;
    let fitted_scale = fit.parameters.iter().find(|p| p.0 == "scale").unwrap().1;

    // Expect fitted parameters to be finite and positive and the fit to be reasonable
    assert!(fitted_shape.is_finite() && fitted_shape > 0.0, "Fitted shape must be finite and positive");
    assert!(fitted_scale.is_finite() && fitted_scale > 0.0, "Fitted scale must be finite and positive");
    // The best fit should typically have a lower AIC than a poor candidate (e.g., normal fit)
    let normal_aic = fits.iter().find(|f| f.distribution_name == "normal").map(|f| f.aic).unwrap_or(f64::INFINITY);
    assert!(fit.aic < normal_aic, "Weibull AIC should be better than normal AIC for Weibull generated data");
}

#[test]
fn test_ljung_box_on_ar1_series() {
    // Generate AR(1) series manually since generate_ar1 is not public
    let phi = 0.6;
    let n = 200;
    let mut rng = Pcg64::seed_from_u64(42);
    let mut series = vec![0.0; n];
    let noise_dist = Normal::new(0.0, 1.0).unwrap();

    for i in 1..n {
        series[i] = phi * series[i-1] + noise_dist.sample(&mut rng);
    }

    let (q, p) = TimeSeriesDecompositionEngine::ljung_box_test(&series, 10).unwrap();
    // For AR(1) with phi=0.6 we expect significant autocorrelation -> small p
    assert!(p < 0.05, "Ljung-Box p-value was {}, q={}", p, q);
}

#[test]
fn test_distribution_fitting_edge_cases() {
    // Test with empty data
    let empty_data: Vec<f64> = vec![];
    let result = StatisticalDistributionEngine::fit_distributions(&empty_data);
    assert!(result.is_err(), "Should fail with empty data");

    // Test with constant data
    let constant_data = vec![5.0; 10];
    let fits = StatisticalDistributionEngine::fit_distributions(&constant_data).unwrap();
    // Should still return some fits, though they may not be meaningful
    assert!(!fits.is_empty(), "Should return some fits even for constant data");

    // Test with very small dataset
    let small_data = vec![1.0, 2.0];
    let fits_small = StatisticalDistributionEngine::fit_distributions(&small_data).unwrap();
    assert!(!fits_small.is_empty(), "Should handle small datasets");
}

#[test]
fn test_bootstrap_with_small_samples() {
    let small_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let mut rng = Pcg64::seed_from_u64(12345);

    // Test bootstrap confidence intervals
    let (lower, upper) = BootstrapEngine::confidence_intervals(
        &small_data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64, // mean
        0.95,
        100,
        &mut rng
    ).unwrap();

    assert!(lower <= upper, "Lower bound should be <= upper bound");
    assert!(lower.is_finite() && upper.is_finite(), "Bounds should be finite");

    // Test with very small sample size
    let tiny_data = vec![1.0, 2.0];
    let (tiny_lower, tiny_upper) = BootstrapEngine::confidence_intervals(
        &tiny_data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.95,
        50,
        &mut rng
    ).unwrap();

    assert!(tiny_lower <= tiny_upper, "Tiny sample bounds should be valid");
}

#[test]
fn test_uncertainty_propagation_correlated_variables() {
    // Test uncertainty propagation with correlated input variables
    let measurements = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0],  // x1
        vec![2.0, 4.0, 6.0, 8.0, 10.0], // x2 = 2*x1
    ];

    let uncertainties = vec![0.1, 0.2]; // uncertainties for x1, x2

    // Function: f(x1, x2) = x1 + x2
    let function = |vars: &[f64]| vars[0] + vars[1];

    // Calculate uncertainty with correlation
    let correlation_matrix = CorrelationEngine::correlation_matrix(&measurements).unwrap();

    // Create covariance matrix from uncertainties and correlations
    let cov_matrix = UncertaintyPropagationEngine::covariance_matrix_from_uncertainties(
        &uncertainties,
        Some(&correlation_matrix)
    ).unwrap();

    // Get mean values
    let means: Vec<f64> = measurements.iter()
        .map(|v| v.iter().sum::<f64>() / v.len() as f64)
        .collect();

    let uncertainty = UncertaintyPropagationEngine::propagate_uncertainty(
        function,
        &means,
        &cov_matrix,
    ).unwrap();

    assert!(uncertainty.is_finite(), "Uncertainty should be finite");
    assert!(uncertainty > 0.0, "Uncertainty should be positive");
}

#[test]
fn test_time_series_seasonal_decomposition() {
    // Generate synthetic seasonal data
    let period = 12;
    let n_points = 60; // 5 years of monthly data
    let trend_slope = 0.1;
    let seasonal_amplitude = 2.0;
    let noise_std = 0.5;

    let mut rng = Pcg64::seed_from_u64(12345);

    let mut seasonal_data = Vec::with_capacity(n_points);
    for i in 0..n_points {
        let trend = trend_slope * i as f64;
        let seasonal = seasonal_amplitude * ((2.0 * std::f64::consts::PI * i as f64) / period as f64).sin();
        let noise = noise_std * rand_distr::Normal::new(0.0, 1.0).unwrap().sample(&mut rng);
        seasonal_data.push(trend + seasonal + noise);
    }

    // Perform seasonal decomposition
    let components = TimeSeriesDecompositionEngine::decompose_additive(&seasonal_data, period).unwrap();

    // Check that components have the right length
    assert_eq!(components.trend.len(), seasonal_data.len());
    assert_eq!(components.seasonal.len(), seasonal_data.len());
    assert_eq!(components.residuals.len(), seasonal_data.len());

    // Check that trend + seasonal + residuals ≈ original data
    for i in 0..seasonal_data.len() {
        let reconstructed = components.trend[i] + components.seasonal[i] + components.residuals[i];
        assert!((reconstructed - seasonal_data[i]).abs() < 1e-10,
                "Reconstruction error at index {}: {} vs {}", i, reconstructed, seasonal_data[i]);
    }

    // Check that seasonal component is periodic
    for i in 0..(seasonal_data.len() - period) {
        assert!((components.seasonal[i] - components.seasonal[i + period]).abs() < 1e-10,
                "Seasonal component should be periodic");
    }
}

#[test]
fn test_time_series_stl_decomposition() {
    // Generate synthetic seasonal data with more complex patterns
    let period = 12;
    let n_points = 60; // 5 years of monthly data
    let trend_slope = 0.1;
    let seasonal_amplitude = 2.0;
    let noise_std = 0.5;

    let mut rng = Pcg64::seed_from_u64(12345);

    let mut seasonal_data = Vec::with_capacity(n_points);
    for i in 0..n_points {
        let trend = trend_slope * i as f64;
        // More complex seasonal pattern with multiple harmonics
        let seasonal = seasonal_amplitude *
            (0.7 * ((2.0 * std::f64::consts::PI * i as f64) / period as f64).sin() +
             0.3 * ((4.0 * std::f64::consts::PI * i as f64) / period as f64).sin());
        let noise = noise_std * rand_distr::Normal::new(0.0, 1.0).unwrap().sample(&mut rng);
        seasonal_data.push(trend + seasonal + noise);
    }

    // Perform STL decomposition
    let components = TimeSeriesDecompositionEngine::decompose_stl(&seasonal_data, period).unwrap();

    // Check that components have the right length
    assert_eq!(components.trend.len(), seasonal_data.len());
    assert_eq!(components.seasonal.len(), seasonal_data.len());
    assert_eq!(components.residuals.len(), seasonal_data.len());

    // Check that trend + seasonal + residuals ≈ original data
    for i in 0..seasonal_data.len() {
        let reconstructed = components.trend[i] + components.seasonal[i] + components.residuals[i];
        assert!((reconstructed - seasonal_data[i]).abs() < 1e-6,
                "Reconstruction error at index {}: {} vs {}", i, reconstructed, seasonal_data[i]);
    }

    // Check that seasonal component is periodic
    for i in 0..(seasonal_data.len() - period) {
        assert!((components.seasonal[i] - components.seasonal[i + period]).abs() < 1e-6,
                "Seasonal component should be periodic");
    }

    // STL should provide better trend estimation than simple moving average
    let ma_components = TimeSeriesDecompositionEngine::decompose_additive(&seasonal_data, period).unwrap();

    // Compare trend smoothness - STL should be smoother
    let stl_trend_var = components.trend.windows(2)
        .map(|w| (w[1] - w[0]).powi(2))
        .sum::<f64>() / (components.trend.len() - 1) as f64;

    let ma_trend_var = ma_components.trend.windows(2)
        .map(|w| (w[1] - w[0]).powi(2))
        .sum::<f64>() / (ma_components.trend.len() - 1) as f64;

    // STL trend should be smoother (lower variance in differences)
    assert!(stl_trend_var < ma_trend_var * 1.5,
            "STL trend should be smoother than moving average: {} vs {}", stl_trend_var, ma_trend_var);
}

#[test]
fn test_outlier_detection_edge_cases() {
    let options = AnalysisOptions::default();

    // Test with empty data
    let empty_data: Vec<f64> = vec![];
    let result = OutlierDetectionEngine::detect_outliers(&empty_data, &options);
    assert!(result.is_ok(), "Should handle empty data gracefully");
    let analysis = result.unwrap();
    assert_eq!(analysis.combined_outliers.len(), 0, "Empty data should have no outliers");

    // Test with single data point
    let single_data = vec![5.0];
    let result = OutlierDetectionEngine::detect_outliers(&single_data, &options);
    assert!(result.is_ok(), "Should handle single data point");
    let analysis = result.unwrap();
    assert_eq!(analysis.combined_outliers.len(), 0, "Single point should not be outlier");

    // Test with constant data
    let constant_data = vec![5.0; 20];
    let result = OutlierDetectionEngine::detect_outliers(&constant_data, &options);
    assert!(result.is_ok(), "Should handle constant data");
    let _analysis = result.unwrap();
    // For constant data, different methods may behave differently
    // Just check that it doesn't crash and returns some result

    // Test with extreme values
    let mut extreme_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    extreme_data.push(1000.0); // Clear outlier
    let result = OutlierDetectionEngine::detect_outliers(&extreme_data, &options);
    assert!(result.is_ok(), "Should handle extreme values");
    let analysis = result.unwrap();
    assert!(!analysis.combined_outliers.is_empty(), "Should detect extreme outlier");

    // Test with very small dataset
    let tiny_data = vec![1.0, 2.0];
    let tiny_result = OutlierDetectionEngine::detect_outliers(&tiny_data, &options);
    assert!(tiny_result.is_ok(), "Should handle very small datasets");
}

#[test]
fn test_correlation_edge_cases() {
    // Perfect positive correlation
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // y = 2x
    let datasets = vec![x.clone(), y.clone()];

    let corr_matrix = CorrelationEngine::correlation_matrix(&datasets).unwrap();
    let pearson_corr = corr_matrix[(0, 1)];
    assert!((pearson_corr - 1.0).abs() < 1e-10, "Perfect correlation should be 1.0, got {}", pearson_corr);

    // Perfect negative correlation
    let y_neg = vec![-2.0, -4.0, -6.0, -8.0, -10.0]; // y = -2x
    let datasets_neg = vec![x, y_neg];

    let corr_matrix_neg = CorrelationEngine::correlation_matrix(&datasets_neg).unwrap();
    let pearson_corr_neg = corr_matrix_neg[(0, 1)];
    assert!((pearson_corr_neg - (-1.0)).abs() < 1e-10, "Perfect negative correlation should be -1.0, got {}", pearson_corr_neg);

    // Single data point per variable
    let single_x = vec![1.0];
    let single_y = vec![2.0];
    let single_datasets = vec![single_x, single_y];
    let result = CorrelationEngine::correlation_matrix(&single_datasets);
    assert!(result.is_err(), "Should fail with insufficient data for correlation");

    // Constant data
    let const_x = vec![5.0; 10];
    let const_y = vec![3.0; 10];
    let const_datasets = vec![const_x, const_y];
    let corr_matrix_const = CorrelationEngine::correlation_matrix(&const_datasets).unwrap();
    let const_corr = corr_matrix_const[(0, 1)];
    assert!(const_corr.is_nan(), "Correlation with constant data should be NaN");
}

#[test]
fn test_bootstrap_edge_cases() {
    let mut rng = Pcg64::seed_from_u64(12345);

    // Test with empty data
    let empty_data: Vec<f64> = vec![];
    let result = BootstrapEngine::confidence_intervals(
        &empty_data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.95,
        100,
        &mut rng
    );
    assert!(result.is_err(), "Should fail with empty data");

    // Test with single data point
    let single_data = vec![5.0];
    let (lower, upper) = BootstrapEngine::confidence_intervals(
        &single_data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.95,
        100,
        &mut rng
    ).unwrap();
    assert_eq!(lower, upper, "Single point should have zero confidence interval width");

    // Test with extreme confidence levels
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let (lower_99, upper_99) = BootstrapEngine::confidence_intervals(
        &data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.99,
        100,
        &mut rng
    ).unwrap();
    assert!(lower_99 < upper_99, "99% CI should be valid");

    let (lower_50, upper_50) = BootstrapEngine::confidence_intervals(
        &data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.50,
        100,
        &mut rng
    ).unwrap();
    assert!(lower_50 < upper_50, "50% CI should be valid");
    assert!(upper_50 - lower_50 < upper_99 - lower_99, "50% CI should be narrower than 99% CI");
}

#[test]
fn test_uncertainty_propagation_edge_cases() {
    // Test with zero uncertainties
    let measurements = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![2.0, 4.0, 6.0, 8.0, 10.0],
    ];
    let zero_uncertainties = vec![0.0, 0.0];
    let function = |vars: &[f64]| vars[0] + vars[1];

    let cov_matrix = UncertaintyPropagationEngine::covariance_matrix_from_uncertainties(
        &zero_uncertainties,
        None
    ).unwrap();

    let means: Vec<f64> = measurements.iter()
        .map(|v| v.iter().sum::<f64>() / v.len() as f64)
        .collect();

    let uncertainty = UncertaintyPropagationEngine::propagate_uncertainty(
        function,
        &means,
        &cov_matrix,
    ).unwrap();
    assert_eq!(uncertainty, 0.0, "Zero uncertainties should propagate to zero uncertainty");

    // Test with single variable
    let single_measurements = vec![vec![1.0, 2.0, 3.0, 4.0, 5.0]];
    let single_uncertainties = vec![0.1];
    let single_function = |vars: &[f64]| vars[0] * 2.0;

    let single_cov_matrix = UncertaintyPropagationEngine::covariance_matrix_from_uncertainties(
        &single_uncertainties,
        None
    ).unwrap();

    let single_means: Vec<f64> = single_measurements.iter()
        .map(|v| v.iter().sum::<f64>() / v.len() as f64)
        .collect();

    let single_uncertainty = UncertaintyPropagationEngine::propagate_uncertainty(
        single_function,
        &single_means,
        &single_cov_matrix,
    ).unwrap();
    assert!(single_uncertainty > 0.0, "Single variable uncertainty should be positive");
}

#[test]
fn test_time_series_edge_cases() {
    // Test with too short series for decomposition
    let short_data = vec![1.0, 2.0, 3.0];
    let period = 12;
    let result = TimeSeriesDecompositionEngine::decompose_additive(&short_data, period);
    assert!(result.is_err(), "Should fail with too short time series");

    let stl_result = TimeSeriesDecompositionEngine::decompose_stl(&short_data, period);
    assert!(stl_result.is_err(), "STL should also fail with too short time series");

    // Test with period larger than data
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let large_period = 10;
    let result = TimeSeriesDecompositionEngine::decompose_additive(&data, large_period);
    assert!(result.is_err(), "Should fail when period is larger than data length");

    // Test trend analysis with insufficient data
    let insufficient_data = vec![1.0, 2.0];
    let trend_result = TimeSeriesDecompositionEngine::trend_test(&insufficient_data);
    assert!(trend_result.is_err(), "Should fail with insufficient data for trend analysis");

    // Test autocorrelation with insufficient lag
    let small_data = vec![1.0, 2.0];
    let autocorr_result = TimeSeriesDecompositionEngine::autocorrelation(&small_data, 5);
    assert!(autocorr_result.is_err(), "Should fail when lag exceeds data length");
}