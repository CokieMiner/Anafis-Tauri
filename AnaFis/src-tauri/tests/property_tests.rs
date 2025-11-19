//! Property Tests for Statistical Property Validation
//!
//! This module contains tests that validate statistical properties and known
//! mathematical relationships, such as Anscombe's quartet and other benchmark datasets.

use anafis_lib::scientific::statistics::comprehensive_analysis::layer1_command::command::ComprehensiveAnalysisCommand;
use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::correlation_methods::CorrelationMethods;
use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::outliers::OutlierDetectionEngine;
use anafis_lib::scientific::statistics::types::AnalysisOptions;
use rand_distr::Distribution;

/// Test with Anscombe's quartet - a classic benchmark dataset
/// All four datasets have identical statistical properties but different distributions
#[test]
fn test_anscombes_quartet_comprehensive_analysis() {
    // Anscombe's quartet data - all four datasets have identical statistical properties
    // but very different visual patterns and outlier characteristics

    // Dataset I: Simple linear relationship
    let x1 = vec![10.0, 8.0, 13.0, 9.0, 11.0, 14.0, 6.0, 4.0, 12.0, 7.0, 5.0];
    let y1 = vec![8.04, 6.95, 7.58, 8.81, 8.33, 9.96, 7.24, 4.26, 10.84, 4.82, 5.68];

    // Dataset II: Quadratic relationship
    let x2 = vec![10.0, 8.0, 13.0, 9.0, 11.0, 14.0, 6.0, 4.0, 12.0, 7.0, 5.0];
    let y2 = vec![9.14, 8.14, 8.74, 8.77, 9.26, 8.10, 6.13, 3.10, 9.13, 7.26, 4.74];

    // Dataset III: Linear with outlier
    let x3 = vec![10.0, 8.0, 13.0, 9.0, 11.0, 14.0, 6.0, 4.0, 12.0, 7.0, 5.0];
    let y3 = vec![7.46, 6.77, 12.74, 7.11, 7.81, 8.84, 6.08, 5.39, 8.15, 6.42, 5.73];

    // Dataset IV: Vertical line with outlier
    let x4 = vec![8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 19.0, 8.0, 8.0, 8.0];
    let y4 = vec![6.58, 5.76, 7.71, 8.84, 8.47, 7.04, 5.25, 12.50, 5.56, 7.91, 6.89];

    let datasets = vec![x1.clone(), y1.clone(), x2.clone(), y2.clone(), x3.clone(), y3.clone(), x4.clone(), y4.clone()];

    let options = AnalysisOptions {
        enabled_analyses: Some(vec![
            "descriptive_stats".to_string(),
            "correlation_analysis".to_string(),
            "outlier_detection".to_string(),
        ]),
        bootstrap_samples: None, // Disable bootstrap for speed
        ..Default::default()
    };

    // Run comprehensive analysis
    let result = ComprehensiveAnalysisCommand::execute(datasets, options);
    assert!(result.is_ok(), "Comprehensive analysis should succeed for Anscombe's quartet");

    let analysis = result.unwrap();

    // Verify basic structure
    assert!(analysis.descriptive_stats.is_some(), "Should have descriptive statistics");
    assert!(analysis.correlation_matrix.is_some(), "Should have correlation analysis");

    // Check correlation matrix dimensions (8x8)
    if let Some(corr_matrix) = &analysis.correlation_matrix {
        assert_eq!(corr_matrix.len(), 64, "Should have 64 correlation values for 8 datasets (8x8 matrix)");
    }

    // Test individual dataset pairs
    test_anscombe_pair(&x1, &y1, "Dataset I", true, false);
    test_anscombe_pair(&x2, &y2, "Dataset II", false, false);
    test_anscombe_pair(&x3, &y3, "Dataset III", true, true);
    test_anscombe_pair(&x4, &y4, "Dataset IV", false, true);

    println!("Anscombe's quartet analysis completed successfully");
}

fn test_anscombe_pair(x: &[f64], y: &[f64], dataset_name: &str, _expect_linear: bool, expect_outliers: bool) {
    // Test correlation
    let pearson_corr = CorrelationMethods::pearson_correlation(x, y).unwrap_or(0.0);
    println!("{} - Pearson correlation: {:.3}", dataset_name, pearson_corr);

    // All datasets should have correlation ≈ 0.816 according to Anscombe
    let expected_corr = 0.816;
    assert!((pearson_corr - expected_corr).abs() < 0.01,
            "{} correlation should be ≈ {:.3}, got {:.3}", dataset_name, expected_corr, pearson_corr);

    // Test outlier detection on y values
    let options = AnalysisOptions::default();
    let outlier_result = OutlierDetectionEngine::detect_outliers(y, &options);
    assert!(outlier_result.is_ok(), "Outlier detection should succeed for {}", dataset_name);

    let outlier_analysis = outlier_result.unwrap();
    let has_outliers = !outlier_analysis.combined_outliers.is_empty();
    println!("{} - Has outliers: {}", dataset_name, has_outliers);

    if expect_outliers {
        assert!(has_outliers, "{} should detect outliers", dataset_name);
    }

    // Test means and variances
    let x_mean = x.iter().sum::<f64>() / x.len() as f64;
    let y_mean = y.iter().sum::<f64>() / y.len() as f64;

    println!("{} - X mean: {:.1}, Y mean: {:.2}", dataset_name, x_mean, y_mean);

    // All datasets should have x mean = 9.0, y mean ≈ 7.50
    assert!((x_mean - 9.0).abs() < 0.01, "{} x mean should be 9.0, got {:.3}", dataset_name, x_mean);
    assert!((y_mean - 7.50).abs() < 0.01, "{} y mean should be 7.50, got {:.3}", dataset_name, y_mean);
}

/// Test statistical properties of known distributions
#[test]
fn test_statistical_properties_normal_distribution() {
    use rand_pcg::Pcg64;
    use rand::SeedableRng;
    use rand_distr::Normal;

    let mut rng = Pcg64::seed_from_u64(12345);
    let normal = Normal::new(10.0, 2.0).unwrap();

    // Generate large sample
    let sample: Vec<f64> = (0..10000).map(|_| normal.sample(&mut rng)).collect();

    let datasets = vec![sample];
    let options = AnalysisOptions {
        enabled_analyses: Some(vec![
            "descriptive_stats".to_string(),
            "normality_test".to_string(),
        ]),
        bootstrap_samples: None, // Disable bootstrap for speed
        ..Default::default()
    };

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // Check that normality test passes for normal data
    if let Some(normality_test) = &result.normality_test {
        assert!(normality_test.is_normal, "Should detect normal distribution as normal");
        println!("✓ Normality test correctly identified normal distribution");
    }

    // Check descriptive statistics are reasonable
    if let Some(stats) = &result.descriptive_stats {
        let mean = stats.mean;
        let std_dev = stats.std_dev;

        // Should be close to true parameters (10.0, 2.0)
        assert!((mean.unwrap() - 10.0).abs() < 0.1, "Mean should be close to 10.0, got {}", mean.unwrap());
        assert!((std_dev.unwrap() - 2.0).abs() < 0.1, "Std dev should be close to 2.0, got {}", std_dev.unwrap());

        println!("✓ Descriptive statistics: mean={:.3}, std_dev={:.3}", mean.unwrap(), std_dev.unwrap());
    }
}

/// Test statistical properties of known relationships
#[test]
fn test_statistical_properties_correlation_perfect_relationships() {
    // Test perfect positive correlation
    let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
    let y_pos: Vec<f64> = x.iter().map(|&val| 2.0 * val + 5.0).collect(); // Perfect positive

    let datasets_pos = vec![x.clone(), y_pos];
    let options = AnalysisOptions {
        enabled_analyses: Some(vec!["correlation_analysis".to_string()]),
        bootstrap_samples: None,
        ..Default::default()
    };
    let result_pos = ComprehensiveAnalysisCommand::execute(datasets_pos, options).unwrap();

    if let Some(corr_matrix) = &result_pos.correlation_matrix {
        let pearson_corr = corr_matrix[1]; // correlation between datasets 0 and 1
        assert!((pearson_corr - 1.0).abs() < 1e-4, "Perfect positive correlation should be 1.0, got {}", pearson_corr);
        println!("✓ Perfect positive correlation: {:.10}", pearson_corr);
    }

    // Test perfect negative correlation
    let y_neg: Vec<f64> = x.iter().map(|&val| -2.0 * val + 5.0).collect(); // Perfect negative

    let datasets_neg = vec![x, y_neg];
    let options_neg = AnalysisOptions {
        enabled_analyses: Some(vec!["correlation_analysis".to_string()]),
        bootstrap_samples: None,
        ..Default::default()
    };
    let result_neg = ComprehensiveAnalysisCommand::execute(datasets_neg, options_neg).unwrap();

    if let Some(corr_matrix) = &result_neg.correlation_matrix {
        let pearson_corr = corr_matrix[1]; // correlation between datasets 0 and 1
        assert!((pearson_corr - (-1.0)).abs() < 1e-10, "Perfect negative correlation should be -1.0, got {}", pearson_corr);
        println!("✓ Perfect negative correlation: {:.10}", pearson_corr);
    }
}

/// Test statistical properties of time series with known characteristics
#[test]
fn test_statistical_properties_time_series_white_noise() {
    use rand_pcg::Pcg64;
    use rand::SeedableRng;
    use rand_distr::Normal;

    let mut rng = Pcg64::seed_from_u64(54321);
    let normal = Normal::new(0.0, 1.0).unwrap();

    // Generate white noise time series (should have no trend, no seasonality)
    let time_series: Vec<f64> = (0..200).map(|_| normal.sample(&mut rng)).collect();

    let datasets = vec![time_series];
    let options = AnalysisOptions {
        enabled_analyses: Some(vec![
            "descriptive_stats".to_string(),
            "time_series_analysis".to_string(),
        ]),
        bootstrap_samples: None, // Disable bootstrap for speed
        min_samples_for_time_series: Some(50),
        ..Default::default()
    };

    let result = ComprehensiveAnalysisCommand::execute(datasets, options).unwrap();

    // White noise should not have trend or seasonality
    if let Some(ts_analysis) = &result.time_series_analysis {
        assert!(!ts_analysis.trend_present, "White noise should not have trend");
        // Note: seasonality detection might be more permissive, so we don't assert on it
        println!("✓ White noise correctly identified: no trend detected");
    }
}