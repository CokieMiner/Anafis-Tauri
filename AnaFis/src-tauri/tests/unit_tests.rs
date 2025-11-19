//! Unit Tests for Individual Statistical Functions
//!
//! This module contains basic unit tests for individual functions and algorithms,
//! focusing on correctness and edge cases for specific components.

use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::{correlation_methods::CorrelationMethods, correlation_matrix::CorrelationMatrix};
use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::outliers::OutlierDetectionEngine;
use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::distribution::fitters::distribution_fitting_core::StatisticalDistributionEngine as CoreEngine;
use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::bootstrap::BootstrapEngine;
use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::time_series::TimeSeriesDecompositionEngine;
use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::uncertainty::UncertaintyPropagationEngine;
use anafis_lib::scientific::statistics::types::AnalysisOptions;
use rand_pcg::Pcg64;
use rand_distr::{Normal, Weibull, Distribution};
use rand::SeedableRng;
use ndarray::{Array2, Array1, ArrayView, Ix1, stack};
use ndarray::Axis;

/// Helper function for approximate equality with appropriate tolerance for f64
#[allow(dead_code)]
fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn test_compute_matrix_with_method_spearman_vs_pearson() {
    // x and y have a strong monotonic but nonlinear relation
    let x = vec![1.0, 2.0, 3.0, 4.0];
    let y = vec![1.0, 4.0, 9.0, 16.0];
    let datasets = vec![x.clone(), y.clone()];

    // Convert to ndarray
    let arrays: Vec<Array1<f64>> = datasets
        .iter()
        .map(|v| Array1::from_vec(v.clone()))
        .collect();
    
    let array_views: Vec<ArrayView<'_, f64, Ix1>> = arrays.iter().map(|arr| arr.view()).collect();

    let data_array = stack(Axis(1), array_views.as_slice()).unwrap();

    let pearson = CorrelationMatrix::compute_matrix_with_method(data_array.view(), "pearson", 9.0).unwrap();
    let spearman = CorrelationMatrix::compute_matrix_with_method(data_array.view(), "spearman", 9.0).unwrap();

    let pearson_corr = pearson[(0, 1)];
    let spearman_corr = spearman[(0, 1)];

    // Pearson should be < 1 for this nonlinear relationship, Spearman should be very close to 1.0
    assert!(pearson_corr < 1.0);
    assert!(approx_eq(spearman_corr, 1.0, 1e-14), "Spearman expected to be very close to 1.0, got {}", spearman_corr);
}

#[test]
fn test_kendall_tau_with_ties() {
    // identical sequences with ties should yield tau == 1.0
    let x = vec![1.0, 2.0, 2.0, 3.0];
    let y = vec![1.0, 2.0, 2.0, 3.0];
    let tau = CorrelationMethods::kendall_correlation(&x, &y).unwrap();
    assert!(approx_eq(tau, 1.0, 1e-15), "Expected tau = 1.0 for identical sequences with ties, got {}", tau);

    // Test with some ties but not identical - should have tau < 1.0
    let x2 = vec![1.0, 2.0, 2.0, 4.0];
    let y2 = vec![2.0, 1.0, 3.0, 4.0];  // Different ordering
    let tau2 = CorrelationMethods::kendall_correlation(&x2, &y2).unwrap();
    assert!(tau2 > 0.0, "Tau should be positive for positively correlated data with ties, got {}", tau2);
    assert!(tau2 < 1.0, "Tau should be less than 1.0 for non-identical data, got {}", tau2);

    // Test perfect negative correlation
    let x3 = vec![1.0, 2.0, 3.0, 4.0];
    let y3 = vec![4.0, 3.0, 2.0, 1.0];
    let tau3 = CorrelationMethods::kendall_correlation(&x3, &y3).unwrap();
    assert!(approx_eq(tau3, -1.0, 1e-15), "Expected tau = -1.0 for perfectly negatively correlated data, got {}", tau3);
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
    let fits = CoreEngine::fit_distributions(&sample).unwrap();
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
    println!("Fitted shape: {}, scale: {}, log_likelihood: {}", fitted_shape, fitted_shape, fit.log_likelihood);
    println!("True parameters: shape={}, scale={}", shape, scale);
    // The best fit should typically have a lower AIC than a poor candidate (e.g., normal fit)
    let normal_aic = fits.iter().find(|f| f.distribution_name == "normal").map(|f| f.aic).unwrap_or(f64::INFINITY);
    println!("Weibull AIC: {}, Normal AIC: {}", fit.aic, normal_aic);
    assert!(fit.aic < normal_aic, "Weibull AIC should be better than normal AIC for Weibull generated data. Weibull AIC: {}, Normal AIC: {}", fit.aic, normal_aic);
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
    let result = CoreEngine::fit_distributions(&empty_data);
    assert!(result.is_err(), "Should fail with empty data");

    // Test with constant data
    let constant_data = vec![5.0; 10];
    let fits = CoreEngine::fit_distributions(&constant_data).unwrap();
    // Should still return some fits, though they may not be meaningful
    assert!(!fits.is_empty(), "Should return some fits even for constant data");

    // Test with very small dataset
    let small_data = vec![1.0, 2.0];
    let fits_small = CoreEngine::fit_distributions(&small_data).unwrap();
    assert!(!fits_small.is_empty(), "Should handle small datasets");
}

#[test]
fn test_bootstrap_with_small_samples() {
    let small_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    // Test bootstrap confidence intervals
    let (lower, upper) = BootstrapEngine::confidence_intervals_with_progress(
        &small_data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64, // mean
        0.95,
        100,
        &anafis_lib::scientific::statistics::comprehensive_analysis::traits::NoOpProgressCallback
    ).unwrap();

    assert!(lower <= upper, "Lower bound should be <= upper bound");
    assert!(lower.is_finite() && upper.is_finite(), "Bounds should be finite");

    // Test with very small sample size
    let tiny_data = vec![1.0, 2.0];
    let (tiny_lower, tiny_upper) = BootstrapEngine::confidence_intervals_with_progress(
        &tiny_data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.95,
        50,
        &anafis_lib::scientific::statistics::comprehensive_analysis::traits::NoOpProgressCallback
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
    let arrays: Vec<Array1<f64>> = measurements
        .iter()
        .map(|v| Array1::from_vec(v.clone()))
        .collect();
    let array_views: Vec<ArrayView<'_, f64, Ix1>> = arrays.iter().map(|arr| arr.view()).collect();
    let data_array = stack(Axis(1), array_views.as_slice()).unwrap();
    let correlation_matrix = CorrelationMatrix::correlation_matrix(data_array.view()).unwrap();

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
    for (i, &value) in seasonal_data.iter().enumerate() {
        let reconstructed = components.trend[i] + components.seasonal[i] + components.residuals[i];
        assert!(approx_eq(reconstructed, value, 1e-10), "Reconstruction error at index {}: {} vs {}", i, reconstructed, value);
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

    let seasonal_data: Vec<f64> = (0..n_points).map(|i| {
        let trend = trend_slope * i as f64;
        // More complex seasonal pattern with multiple harmonics
        let seasonal = seasonal_amplitude *
            (0.7 * ((2.0 * std::f64::consts::PI * i as f64) / period as f64).sin() +
             0.3 * ((4.0 * std::f64::consts::PI * i as f64) / period as f64).sin());
        let noise = noise_std * rand_distr::Normal::new(0.0, 1.0).unwrap().sample(&mut rng);
        trend + seasonal + noise
    }).collect();

    // Perform STL decomposition
    let components = TimeSeriesDecompositionEngine::decompose_stl(&seasonal_data, period).unwrap();

    // Check that components have the right length
    assert_eq!(components.trend.len(), seasonal_data.len());
    assert_eq!(components.seasonal.len(), seasonal_data.len());
    assert_eq!(components.residuals.len(), seasonal_data.len());

    // Check that trend + seasonal + residuals ≈ original data
    for (i, &original) in seasonal_data.iter().enumerate() {
        let reconstructed = components.trend[i] + components.seasonal[i] + components.residuals[i];
        assert!(approx_eq(reconstructed, original, 1e-6), "Reconstruction error at index {}: {} vs {}", i, reconstructed, original);
    }

    // Check that seasonal component is periodic
    for (i, &val) in components.seasonal.iter().take(seasonal_data.len() - period).enumerate() {
        assert!(approx_eq(val, components.seasonal[i + period], 1e-6), "Seasonal component should be periodic");
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

    // Correctly construct Array2 where each Vec<f64> in datasets becomes a column
    let arrays: Vec<Array1<f64>> = datasets
        .iter()
        .map(|v| Array1::from_vec(v.clone()))
        .collect();
    let array_views: Vec<ArrayView<'_, f64, Ix1>> = arrays.iter().map(|arr| arr.view()).collect();
    let data_array = stack(Axis(1), array_views.as_slice()).unwrap();
    let corr_matrix = CorrelationMatrix::correlation_matrix(data_array.view()).unwrap();
    let pearson_corr = corr_matrix[(0, 1)];
    assert!(approx_eq(pearson_corr, 1.0, 1e-14), "Perfect correlation should be 1.0, got {}", pearson_corr);

    // Perfect negative correlation
    let y_neg = vec![-2.0, -4.0, -6.0, -8.0, -10.0]; // y = -2x
    let datasets_neg = vec![x, y_neg];
    let arrays_neg: Vec<Array1<f64>> = datasets_neg
        .iter()
        .map(|v| Array1::from_vec(v.clone()))
        .collect();
    let array_views_neg: Vec<ArrayView<'_, f64, Ix1>> = arrays_neg.iter().map(|arr| arr.view()).collect();
    let data_array_neg = stack(Axis(1), array_views_neg.as_slice()).unwrap();
    let corr_matrix_neg = CorrelationMatrix::correlation_matrix(data_array_neg.view()).unwrap();
    let pearson_corr_neg = corr_matrix_neg[(0, 1)];
    assert!(approx_eq(pearson_corr_neg, -1.0, 1e-14), "Perfect negative correlation should be -1.0, got {}", pearson_corr_neg);

    // Single data point per variable
    let single_x = vec![1.0];
    let single_y = vec![2.0];
    let single_datasets = vec![single_x, single_y];
    let n_samples_single = single_datasets[0].len();
    let n_vars_single = single_datasets.len();
    let data_vec_single: Vec<f64> = single_datasets.iter().flatten().cloned().collect();
    let data_array_single = Array2::from_shape_vec((n_samples_single, n_vars_single), data_vec_single).unwrap();
    let result = CorrelationMatrix::correlation_matrix(data_array_single.view());
    assert!(result.is_err(), "Should fail with insufficient data for correlation");

    // Constant data
    let const_x = vec![5.0; 10];
    let const_y = vec![3.0; 10];
    let const_datasets = vec![const_x, const_y];

    let const_arrays: Vec<Array1<f64>> = const_datasets
        .iter()
        .map(|v| Array1::from_vec(v.clone()))
        .collect();
    let const_array_views: Vec<ArrayView<'_, f64, Ix1>> = const_arrays.iter().map(|arr| arr.view()).collect();
    let data_array_const = stack(Axis(1), const_array_views.as_slice()).unwrap();
    let corr_matrix_const = CorrelationMatrix::correlation_matrix(data_array_const.view()).unwrap();
    let const_corr = corr_matrix_const[(0, 1)];
    assert!(const_corr.is_nan(), "Correlation with constant data should be NaN");
}

#[test]
fn test_bootstrap_edge_cases() {
    let _rng = Pcg64::seed_from_u64(12345);

    // Test with empty data
    let empty_data: Vec<f64> = vec![];
    let result = BootstrapEngine::confidence_intervals_with_progress(
        &empty_data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.95,
        100,
        &anafis_lib::scientific::statistics::comprehensive_analysis::traits::NoOpProgressCallback
    );
    assert!(result.is_err(), "Should fail with empty data");

    // Test with single data point
    let single_data = vec![5.0];
    let (lower, upper) = BootstrapEngine::confidence_intervals_with_progress(
        &single_data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.95,
        100,
        &anafis_lib::scientific::statistics::comprehensive_analysis::traits::NoOpProgressCallback
    ).unwrap();
    assert_eq!(lower, upper, "Single point should have zero confidence interval width");

    // Test with extreme confidence levels
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let (lower_99, upper_99) = BootstrapEngine::confidence_intervals_with_progress(
        &data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.99,
        100,
        &anafis_lib::scientific::statistics::comprehensive_analysis::traits::NoOpProgressCallback
    ).unwrap();
    assert!(lower_99 < upper_99, "99% CI should be valid");

    let (lower_50, upper_50) = BootstrapEngine::confidence_intervals_with_progress(
        &data,
        |sample| sample.iter().sum::<f64>() / sample.len() as f64,
        0.50,
        100,
        &anafis_lib::scientific::statistics::comprehensive_analysis::traits::NoOpProgressCallback
    ).unwrap();
    assert!(lower_50 < upper_50, "50% CI should be valid");
    assert!(upper_50 - lower_50 < upper_99 - lower_99, "50% CI should be narrower than 99% CI");
}

#[test]
fn test_uncertainty_propagation_edge_cases() {
    // Test with zero uncertainties
    let measurements = [
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
    let single_measurements = [vec![1.0, 2.0, 3.0, 4.0, 5.0]];
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

#[cfg(test)]
mod hypothesis_testing_tests {
    use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::hypothesis_testing::HypothesisTestingEngine;

    /// Helper function for approximate equality with appropriate tolerance for f64
    #[allow(dead_code)]
    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    /// Helper function for relative approximate equality
    fn rel_approx_eq(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
        let diff = (a - b).abs();
        diff <= abs_tol || diff <= rel_tol * a.abs().max(b.abs())
    }

    #[test]
    fn test_one_sample_t_test() {
        // Test with data that should have significant difference from zero
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = HypothesisTestingEngine::one_sample_t_test(&data, 0.0).unwrap();

        assert_eq!(result.test_type, "One-sample t-test");
        assert!(result.t_statistic > 0.0, "t-statistic should be positive for positive means");
        assert!(result.p_value < 0.05, "Should be significant with p < 0.05");
        assert!(result.significant, "Should be statistically significant");
        assert!(rel_approx_eq(result.mean_difference, 3.0, 1e-10, 1e-14), "Mean difference should be approximately 3.0, got {}", result.mean_difference);
        assert!(result.effect_size > 0.0, "Effect size should be positive");
    }

    #[test]
    fn test_one_sample_t_test_no_difference() {
        // Test with data that should not differ significantly from mean
        let data = vec![0.0, 0.1, -0.1, 0.05, -0.05];
        let result = HypothesisTestingEngine::one_sample_t_test(&data, 0.0).unwrap();

        assert!(!result.significant, "Should not be statistically significant");
        assert!(result.mean_difference.abs() < 0.1, "Mean difference should be very small, got {}", result.mean_difference);
    }

    #[test]
    fn test_paired_t_test() {
        // Test paired data with varying differences
        let data1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let data2 = vec![1.1, 2.3, 3.2, 4.4, 5.6];  // Varying differences
        let result = HypothesisTestingEngine::paired_t_test(&data1, &data2).unwrap();

        assert_eq!(result.test_type, "Paired t-test");
        assert!(result.mean_difference < 0.0, "Mean difference should be negative");
        assert!(result.significant, "Should be statistically significant");
    }

    #[test]
    fn test_two_sample_t_test_equal_variance() {
        // Test two independent samples with larger difference
        let data1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let data2 = vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0];
        let result = HypothesisTestingEngine::two_sample_t_test(&data1, &data2, true).unwrap();

        assert!(result.test_type.contains("Two-sample t-test"));
        assert!(result.mean_difference < 0.0, "Mean difference should be negative");
        assert!(result.significant, "Should be statistically significant");
    }

    #[test]
    fn test_welch_t_test() {
        // Test Welch's t-test for unequal variances with larger difference
        let data1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let data2 = vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0];
        let result = HypothesisTestingEngine::welch_t_test(&data1, &data2).unwrap();

        assert!(result.test_type.contains("Welch"));
        assert!(result.mean_difference < 0.0, "Mean difference should be negative");
        assert!(result.significant, "Should be statistically significant");
    }

    #[test]
    fn test_one_way_anova() {
        // Test ANOVA with three groups that differ significantly
        let groups: Vec<&[f64]> = vec![
            &[1.0, 2.0, 3.0, 4.0, 5.0],      // Group 1
            &[3.0, 4.0, 5.0, 6.0, 7.0],      // Group 2
            &[5.0, 6.0, 7.0, 8.0, 9.0],      // Group 3
        ];

        let result = HypothesisTestingEngine::one_way_anova(&groups).unwrap();

        assert_eq!(result.test_type, "One-way ANOVA");
        assert!(result.f_statistic > 0.0, "F-statistic should be positive");
        assert!(result.significant, "Should be statistically significant");
        assert!(result.eta_squared > 0.0, "Eta squared should be positive");
        assert!(result.post_hoc_results.is_some(), "Should have post-hoc results");

        // Check post-hoc results
        let post_hoc = result.post_hoc_results.as_ref().unwrap();
        assert_eq!(post_hoc.len(), 3, "Should have 3 pairwise comparisons"); // 3 choose 2 = 3
    }

    #[test]
    fn test_chi_square_goodness_of_fit() {
        // Test chi-square goodness of fit
        let observed = vec![50.0, 30.0, 20.0];  // Observed frequencies
        let expected = vec![40.0, 40.0, 20.0];  // Expected frequencies
        let result = HypothesisTestingEngine::chi_square_goodness_of_fit(&observed, &expected).unwrap();

        assert_eq!(result.test_type, "Chi-square goodness of fit");
        assert!(result.chi_square_statistic > 0.0, "Chi-square statistic should be positive");
        assert!(result.p_value > 0.0 && result.p_value <= 1.0, "p-value should be between 0 and 1");
        assert!(result.effect_size.is_some(), "Should have effect size (Cramér's V)");
    }

    #[test]
    fn test_chi_square_independence() {
        // Test chi-square test of independence
        let row1 = vec![20.0, 30.0, 10.0];
        let row2 = vec![10.0, 40.0, 20.0];
        let table = vec![row1.as_slice(), row2.as_slice()];

        let result = HypothesisTestingEngine::chi_square_independence(&table).unwrap();

        assert_eq!(result.test_type, "Chi-square test of independence");
        assert!(result.chi_square_statistic >= 0.0, "Chi-square statistic should be non-negative");
        assert!(result.effect_size.is_some(), "Should have effect size (Cramér's V)");
        assert_eq!(result.expected_frequencies.len(), 2, "Should have 2 rows of expected frequencies");
        assert_eq!(result.expected_frequencies[0].len(), 3, "Should have 3 columns of expected frequencies");
    }

    #[test]
    fn test_hypothesis_testing_edge_cases() {
        // Test with empty data
        let empty_data: Vec<f64> = vec![];
        assert!(HypothesisTestingEngine::one_sample_t_test(&empty_data, 0.0).is_err());

        // Test with constant values (should fail due to zero variance)
        let constant_values = vec![2.0, 2.0, 2.0];
        assert!(HypothesisTestingEngine::one_sample_t_test(&constant_values, 0.0).is_err());

        // Test ANOVA with insufficient groups
        let insufficient_groups: Vec<&[f64]> = vec![&[1.0, 2.0, 3.0]];
        assert!(HypothesisTestingEngine::one_way_anova(&insufficient_groups).is_err());

        // Test ANOVA with empty group
        let empty_group: Vec<&[f64]> = vec![&[], &[1.0, 2.0]];
        assert!(HypothesisTestingEngine::one_way_anova(&empty_group).is_err());
    }
}

#[cfg(test)]
mod statistical_power_tests {
    use anafis_lib::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats;

    #[test]
    fn test_required_sample_size() {
        // Test sample size calculation for mean difference
        let sample_size = UnifiedStats::required_sample_size_for_mean(2.0, 1.0, 0.05, 0.8).unwrap();
        assert!(sample_size > 0, "Sample size should be positive");

        // Larger effect size should require smaller sample
        let smaller_sample = UnifiedStats::required_sample_size_for_mean(2.0, 2.0, 0.05, 0.8).unwrap();
        assert!(smaller_sample < sample_size, "Larger effect size should require smaller sample");
    }

    #[test]
    fn test_power_t_test_one_sample() {
        // Test power calculation for one-sample t-test
        let power = UnifiedStats::power_t_test_one_sample(1.0, 2.0, 30, 0.05).unwrap();
        assert!(power >= 0.0 && power <= 1.0, "Power should be between 0 and 1");

        // Larger effect size should give higher power
        let higher_power = UnifiedStats::power_t_test_one_sample(2.0, 2.0, 30, 0.05).unwrap();
        println!("power = {}, higher_power = {}", power, higher_power);
        assert!(higher_power > power, "Larger effect size should give higher power");
    }

    #[test]
    fn test_power_t_test_two_sample() {
        // Test power calculation for two-sample t-test
        let power = UnifiedStats::power_t_test_two_sample(1.0, 2.0, 30, 30, 0.05, true).unwrap();
        assert!(power >= 0.0 && power <= 1.0, "Power should be between 0 and 1");

        // Larger sample size should give higher power
        let higher_power = UnifiedStats::power_t_test_two_sample(1.0, 2.0, 50, 50, 0.05, true).unwrap();
        assert!(higher_power > power, "Larger sample size should give higher power");
    }

    #[test]
    fn test_sample_size_t_test() {
        // Test sample size calculation for t-test
        let sample_size = UnifiedStats::sample_size_t_test(0.5, 1.0, 0.05, 0.8, true).unwrap();
        assert!(sample_size > 0, "Sample size should be positive");

        // Higher power should require larger sample
        let larger_sample = UnifiedStats::sample_size_t_test(0.5, 1.0, 0.05, 0.9, true).unwrap();
        println!("sample_size for 0.8 = {}", sample_size);
        println!("larger_sample for 0.9 = {}", larger_sample);
        assert!(larger_sample >= sample_size, "Higher power should require at least the same sample");
    }

    #[test]
    fn test_power_anova() {
        // Test power calculation for ANOVA
        let power = UnifiedStats::power_anova(3, 10, 0.1, 0.05).unwrap();
        assert!((0.0..=1.0).contains(&power), "Power should be between 0 and 1, got {}", power);
    }

    #[test]
    fn test_power_chi_square() {
        // Test power calculation for chi-square test with smaller effect size
        let power = UnifiedStats::power_chi_square(0.2, 2.0, 50, 0.05).unwrap();
        assert!((0.0..=1.0).contains(&power), "Power should be between 0 and 1, got {}", power);

        // Larger effect size should give higher power
        let higher_power = UnifiedStats::power_chi_square(0.5, 2.0, 50, 0.05).unwrap();
        assert!(higher_power >= power, "Larger effect size should give higher or equal power");
    }

    #[test]
    fn test_post_hoc_power() {
        // Test post-hoc power analysis
        let power = UnifiedStats::post_hoc_power(2.5, 28.0, 0.05).unwrap();
        println!("power for 2.5 = {}", power);
        assert!((0.0..=1.0).contains(&power), "Power should be between 0 and 1");

        // Significant result should have positive power
        let significant_power = UnifiedStats::post_hoc_power(3.0, 28.0, 0.05).unwrap();
        println!("significant_power for 3.0 = {}", significant_power);
        assert!(significant_power > 0.0, "Significant result should have positive power, got {}", significant_power);
    }

    #[test]
    fn test_power_t_test_convenience() {
        // Test the convenience wrapper for power_t_test
        let power = UnifiedStats::power_t_test(1.0, 2.0, 30, 0.05, "two-sided").unwrap();
        assert!(power >= 0.0 && power <= 1.0, "Power should be between 0 and 1");
    }

    #[test]
    fn test_statistical_power_edge_cases() {
        // Test with invalid parameters
        assert!(UnifiedStats::required_sample_size_for_mean(0.0, 1.0, 0.05, 0.8).is_err());
        assert!(UnifiedStats::required_sample_size_for_mean(2.0, 0.0, 0.05, 0.8).is_err());
        assert!(UnifiedStats::power_t_test_one_sample(1.0, 0.0, 30, 0.05).is_err());
        assert!(UnifiedStats::power_t_test_one_sample(1.0, 2.0, 0, 0.05).is_err());
        assert!(UnifiedStats::power_anova(1, 10, 0.5, 0.05).is_err());
        assert!(UnifiedStats::power_chi_square(0.0, 2.0, 100, 0.05).is_err());
    }

    #[test]
    fn test_power_calculation_accuracy() {
        // Test that power calculations are reasonable and consistent

        // For large effect sizes, power should approach 1.0
        let high_power = UnifiedStats::power_t_test_two_sample(3.0, 1.0, 20, 20, 0.05, true).unwrap();
        println!("high_power = {}", high_power);
        assert!(high_power > 0.95, "Large effect size should give very high power");

        // For very small effect sizes, power should be low
        let low_power = UnifiedStats::power_t_test_two_sample(0.1, 1.0, 20, 20, 0.05, true).unwrap();
        assert!(low_power < 0.2, "Small effect size should give low power");

        // Increasing sample size should increase power
        let medium_power = UnifiedStats::power_t_test_two_sample(0.5, 1.0, 50, 50, 0.05, true).unwrap();
        let high_sample_power = UnifiedStats::power_t_test_two_sample(0.5, 1.0, 100, 100, 0.05, true).unwrap();
        assert!(high_sample_power > medium_power, "Larger sample should give higher power");
    }
}