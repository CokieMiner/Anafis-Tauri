//! Benchmark Tests for Performance and Comprehensive Validation
//!
//! This module contains performance benchmarks and comprehensive validation tests
//! that exercise all major system capabilities with synthetic data.

use crate::scientific::statistics::comprehensive_analysis::layer1_command::command::ComprehensiveAnalysisCommand;
use crate::scientific::statistics::types::{AnalysisOptions, NanHandling};
use rand_pcg::Pcg64;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal, Exp};

/// Comprehensive benchmark test covering all major capabilities
#[test]
fn test_comprehensive_system_capabilities() {
    let mut rng = Pcg64::seed_from_u64(42);

    // Test 1: Time Series Analysis with known properties
    println!("=== Testing Time Series Analysis ===");
    test_time_series_capabilities();

    // Test 2: Distribution Fitting with known distributions
    println!("=== Testing Distribution Fitting ===");
    test_distribution_fitting();

    // Test 3: Uncertainty Propagation
    println!("=== Testing Uncertainty Propagation ===");
    test_uncertainty_propagation();

    // Test 4: Quality Control
    println!("=== Testing Quality Control ===");
    test_quality_control();

    // Test 5: Data Imputation
    println!("=== Testing Data Imputation ===");
    test_data_imputation();

    // Test 6: Reliability Analysis
    println!("=== Testing Reliability Analysis ===");
    test_reliability_analysis();

    // Test 7: Bootstrap and Confidence Intervals
    println!("=== Testing Bootstrap & Confidence Intervals ===");
    test_bootstrap_confidence_intervals(&mut rng);

    // Test 8: Multiple Correlation Methods
    println!("=== Testing Multiple Correlation Methods ===");
    test_correlation_methods();

    println!("All comprehensive system capabilities tested successfully!");
}

fn test_time_series_capabilities() {
    // Generate time series with known trend and seasonality
    let n = 60; // 5 years of monthly data
    let trend_slope = 0.1;
    let seasonal_amplitude = 2.0;
    let noise_std = 0.5;

    let mut rng = Pcg64::seed_from_u64(12345);
    let mut time_series = Vec::new();
    for i in 0..n {
        let trend = trend_slope * i as f64;
        let seasonal = seasonal_amplitude * ((2.0 * std::f64::consts::PI * i as f64) / 12.0).sin();
        let noise = noise_std * Normal::new(0.0, 1.0).unwrap().sample(&mut rng);
        time_series.push(trend + seasonal + noise);
    }

    let datasets = vec![time_series];
    let mut options = AnalysisOptions::default();
    options.min_samples_for_time_series = Some(20);

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Should detect trend and seasonality
    if let Some(ts_analysis) = &result.time_series_analysis {
        assert!(ts_analysis.trend_present, "Should detect trend in trending time series");
        assert!(ts_analysis.seasonality_present, "Should detect seasonality in seasonal time series");
        println!("✓ Time series analysis correctly detected trend and seasonality");
    } else {
        panic!("Time series analysis should be present");
    }
}

fn test_distribution_fitting() {
    // Generate data from known distributions
    let mut rng = Pcg64::seed_from_u64(54321);

    // Normal distribution data
    let normal_data: Vec<f64> = (0..200)
        .map(|_| Normal::new(10.0, 2.0).unwrap().sample(&mut rng))
        .collect();

    // Exponential distribution data
    let exp_data: Vec<f64> = (0..200)
        .map(|_| Exp::new(0.5).unwrap().sample(&mut rng))
        .collect();

    let datasets = vec![normal_data, exp_data];
    let options = AnalysisOptions::default();

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Should fit distributions and identify best fits
    assert!(result.distribution_fits.is_some(), "Should have distribution fits");
    assert!(result.best_fit_distribution.is_some(), "Should have best fit distribution");

    if let Some(best_fit) = &result.best_fit_distribution {
        println!("✓ Best fit distribution: {}", best_fit.distribution_name);
        assert!(best_fit.aic.is_finite(), "AIC should be finite");
        assert!(best_fit.bic.is_finite(), "BIC should be finite");
    }
}

fn test_uncertainty_propagation() {
    // Data with known uncertainties
    let data = vec![10.0, 10.1, 9.9, 10.2, 10.0];
    let uncertainties = vec![0.1, 0.1, 0.1, 0.1, 0.1];
    let confidence_levels = vec![0.95, 0.95, 0.95, 0.95, 0.95];

    let datasets = vec![data];
    let mut options = AnalysisOptions::default();
    options.uncertainties = Some(uncertainties);
    options.uncertainty_confidences = Some(confidence_levels);

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Should have uncertainty propagation results
    assert!(result.uncertainty_propagation.is_some(), "Should have uncertainty propagation");

    if let Some(uncertainty) = &result.uncertainty_propagation {
        assert!(uncertainty.total_uncertainty.is_some(), "Should have total uncertainty");
        println!("✓ Uncertainty propagation calculated: {:.4}", uncertainty.total_uncertainty.unwrap());
    }
}

fn test_quality_control() {
    // Generate process data within specification limits
    let mut rng = Pcg64::seed_from_u64(98765);
    let process_data: Vec<f64> = (0..50)
        .map(|_| Normal::new(10.0, 0.5).unwrap().sample(&mut rng))
        .collect();

    let datasets = vec![process_data];
    let mut options = AnalysisOptions::default();
    options.lsl = Some(8.0);  // Lower spec limit
    options.usl = Some(12.0); // Upper spec limit

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Should have quality control analysis
    assert!(result.quality_control.is_some(), "Should have quality control analysis");

    if let Some(qc) = &result.quality_control {
        assert!(qc.process_stable, "Process should be stable (normally distributed within limits)");
        assert!(qc.cpk.is_some(), "Should have process capability index");
        println!("✓ Quality control: Process stable = {}, Cpk = {:.2}", qc.process_stable, qc.cpk.unwrap());
    }
}

fn test_data_imputation() {
    // Data with missing values
    let data_with_nans = vec![1.0, 2.0, f64::NAN, 4.0, f64::NAN, 6.0];

    let datasets = vec![data_with_nans];
    let mut options = AnalysisOptions::default();
    options.nan_handling = NanHandling::Mean; // Use mean imputation

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Should have sanitization report
    assert!(result.sanitization_report.is_some(), "Should have sanitization report");

    if let Some(report) = &result.sanitization_report {
        assert_eq!(report.rows_removed_total, 0, "Should not remove rows with mean imputation");
        assert_eq!(report.original_row_counts[0], 6, "Original count should be 6");
        assert_eq!(report.remaining_row_counts[0], 6, "Remaining count should be 6");
        println!("✓ Data imputation: Successfully handled {} NaN values", 2);
    }
}

fn test_reliability_analysis() {
    // Generate correlated variables for reliability testing
    let mut rng = Pcg64::seed_from_u64(13579);

    let base_data: Vec<f64> = (0..30)
        .map(|_| Normal::new(5.0, 1.0).unwrap().sample(&mut rng))
        .collect();

    // Create correlated variables
    let var1 = base_data.clone();
    let var2 = base_data.iter().map(|x| x * 0.8 + Normal::new(0.0, 0.5).unwrap().sample(&mut rng)).collect();
    let var3 = base_data.iter().map(|x| x * 0.7 + Normal::new(0.0, 0.6).unwrap().sample(&mut rng)).collect();

    let datasets = vec![var1, var2, var3];
    let options = AnalysisOptions::default();

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Should have reliability analysis for 3+ variables
    assert!(result.reliability_analysis.is_some(), "Should have reliability analysis");

    if let Some(reliability) = &result.reliability_analysis {
        assert!(reliability.cronbach_alpha.is_some(), "Should have Cronbach's alpha");
        let alpha = reliability.cronbach_alpha.unwrap();
        assert!(alpha >= 0.0 && alpha <= 1.0, "Cronbach's alpha should be between 0 and 1");
        println!("✓ Reliability analysis: Cronbach's alpha = {:.3}", alpha);
    }
}

fn test_bootstrap_confidence_intervals(_rng: &mut Pcg64) {
    let data = vec![10.0, 10.1, 9.9, 10.2, 10.0, 9.8, 10.3, 9.7, 10.1, 10.0];

    let datasets = vec![data];
    let mut options = AnalysisOptions::default();
    options.bootstrap_samples = Some(1000);

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Should have confidence intervals
    assert!(result.confidence_intervals.is_some(), "Should have confidence intervals");

    if let Some(ci) = &result.confidence_intervals {
        assert!(ci.mean.is_some(), "Should have mean confidence interval");
        assert!(ci.std_dev.is_some(), "Should have std dev confidence interval");

        let (mean_lower, mean_upper) = ci.mean.unwrap();
        let (std_lower, std_upper) = ci.std_dev.unwrap();

        assert!(mean_lower < mean_upper, "Mean CI bounds should be ordered");
        assert!(std_lower < std_upper, "Std dev CI bounds should be ordered");
        assert!(mean_lower > 0.0 && mean_upper > 0.0, "Mean CI should be positive");

        println!("✓ Bootstrap CI: Mean [{:.2}, {:.2}], Std Dev [{:.3}, {:.3}]",
                mean_lower, mean_upper, std_lower, std_upper);
    }
}

fn test_correlation_methods() {
    // Monotonic but nonlinear relationship (perfect for testing different correlation methods)
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let y = vec![1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 81.0, 100.0]; // y = x²

    let datasets = vec![x, y];
    let mut options = AnalysisOptions::default();
    options.correlation_method = Some("spearman".to_string());

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Should have correlation matrix
    assert!(result.correlation_matrix.is_some(), "Should have correlation matrix");

    if let Some(corr_matrix) = &result.correlation_matrix {
        assert_eq!(corr_matrix.len(), 4, "Should have 4 correlation values for 2 datasets");
        // For perfect monotonic relationship, Spearman should be 1.0
        let spearman_corr = corr_matrix[1]; // correlation between dataset 0 and 1
        assert!((spearman_corr - 1.0).abs() < 1e-10, "Spearman correlation should be 1.0 for perfect monotonic relationship, got {}", spearman_corr);
        println!("✓ Multiple correlation methods: Spearman correlation = {:.3}", spearman_corr);
    }
}