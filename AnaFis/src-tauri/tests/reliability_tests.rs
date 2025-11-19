//! Integration tests for reliability analysis
//!
//! These tests validate reliability analysis functionality including
//! Cronbach's alpha, item-total correlations, and scale reliability measures.

use anafis_lib::scientific::statistics::comprehensive_analysis::layer2_coordinators::reliability_analysis::ReliabilityAnalysisCoordinator;

#[test]
fn test_analyze_basic() {
    let datasets = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![2.0, 3.0, 4.0, 5.0, 6.0],
        vec![1.5, 2.5, 3.5, 4.5, 5.5],
    ];

    let result = ReliabilityAnalysisCoordinator::analyze(&datasets);
    assert!(result.is_ok());

    let analysis = result.unwrap();
    assert!(analysis.cronbach_alpha >= 0.0 && analysis.cronbach_alpha <= 1.0);
    assert_eq!(analysis.item_total_correlations.len(), 3);
    assert!(analysis.scale_reliability.omega >= 0.0 && analysis.scale_reliability.omega <= 1.0);
    assert!(analysis.scale_reliability.average_interitem_corr >= -1.0 && analysis.scale_reliability.average_interitem_corr <= 1.0);
}

#[test]
fn test_analyze_insufficient_variables() {
    let datasets = vec![
        vec![1.0, 2.0, 3.0],
    ];

    let result = ReliabilityAnalysisCoordinator::analyze(&datasets);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Need at least 2 variables"));
}

#[test]
fn test_analyze_mismatched_lengths() {
    let datasets = vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0], // Different length
    ];

    let result = ReliabilityAnalysisCoordinator::analyze(&datasets);
    assert!(result.is_err());
}