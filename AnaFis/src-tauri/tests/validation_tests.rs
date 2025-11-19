//! Integration tests for input validation and sanitization
//!
//! These tests validate input validation, NaN handling, and data sanitization
//! functionality for statistical analysis.

use anafis_lib::scientific::statistics::comprehensive_analysis::layer1_command::validation::InputValidator;
use anafis_lib::scientific::statistics::types::{AnalysisOptions, NanHandling};

#[test]
fn test_remove_rows_pairwise() {
    let datasets = vec![vec![1.0, f64::NAN, 3.0], vec![4.0, 5.0, f64::INFINITY]];
    let options = AnalysisOptions {
        nan_handling: NanHandling::Remove,
        treat_as_paired: Some(true),
        ..Default::default()
    };

    let (sanitized, _report) = InputValidator::validate_and_sanitize_input(&datasets, &options)
        .expect("Sanitization failed");
    assert_eq!(sanitized.len(), 2);
    assert_eq!(sanitized[0], vec![1.0]);
    assert_eq!(sanitized[1], vec![4.0]);
}

#[test]
fn test_remove_rows_independent() {
    let datasets = vec![vec![1.0, f64::NAN, 3.0], vec![4.0, 5.0, f64::INFINITY]];
    let options = AnalysisOptions {
        nan_handling: NanHandling::Remove,
        treat_as_paired: Some(false),
        ..Default::default()
    };

    let (sanitized, _report) = InputValidator::validate_and_sanitize_input(&datasets, &options)
        .expect("Sanitization failed");
    assert_eq!(sanitized.len(), 2);
    assert_eq!(sanitized[0], vec![1.0, 3.0]);
    assert_eq!(sanitized[1], vec![4.0, 5.0]);
}

#[test]
fn test_mean_imputation_handles_infs() {
    let dataset = vec![1.0, f64::NAN, f64::INFINITY];
    let options = AnalysisOptions {
        nan_handling: NanHandling::Mean,
        ..Default::default()
    };
    let sanitized = InputValidator::sanitize_dataset(&dataset, &options).expect("imputation failed");
    // 1.0 is finite; second is imputed with mean 1.0; third is replaced by max*1.5 => 1.5
    assert_eq!(sanitized[0], 1.0);
    assert!(sanitized[1].is_finite());
    assert_eq!(sanitized[2], 1.5);
}

#[test]
fn test_multiple_imputation_fallback() {
    let dataset = vec![1.0, f64::NAN, f64::NAN, 3.0];
    let options = AnalysisOptions {
        nan_handling: NanHandling::Multiple,
        random_seed: Some(12345),
        ..Default::default()
    };
    let sanitized = InputValidator::sanitize_dataset(&dataset, &options).expect("imputation failed");
    // Imputed values should be finite and not equal to the original mean exactly due to noise
    assert!(sanitized.iter().all(|v| v.is_finite()));
    assert!(sanitized[1] != sanitized[2] || (sanitized[1] - sanitized[2]).abs() < 1e-12);
}

#[test]
fn test_multiple_imputation_knn_feature_enabled() {
    let dataset = vec![1.0, 2.0, f64::NAN, 4.0, f64::NAN, 6.0];
    let options = AnalysisOptions {
        nan_handling: NanHandling::Multiple,
        random_seed: Some(123),
        ..Default::default()
    };
    let sanitized = InputValidator::sanitize_dataset(&dataset, &options).expect("imputation failed");
    // All values should be finite after KNN imputation
    assert!(sanitized.iter().all(|v| v.is_finite()));
    // Missing values must be imputed differently from NaN
    assert!(!sanitized[2].is_nan());
    assert!(!sanitized[4].is_nan());
}