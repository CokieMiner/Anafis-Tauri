//! ANOVA Tests
//!
//! This module contains comprehensive tests for ANOVA functionality including:
//! - One-way ANOVA
//! - Two-way ANOVA
//! - Repeated measures ANOVA
//! - Post-hoc tests (Tukey HSD, Bonferroni, Holm-Bonferroni)

use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::hypothesis_testing::HypothesisTestingEngine;

/// Test one-way ANOVA with known groups
#[test]
fn test_one_way_anova_basic() {
    // Create three groups with different means
    let group1 = [2.0, 3.0, 4.0, 5.0, 6.0]; // mean = 4.0
    let group2 = [5.0, 6.0, 7.0, 8.0, 9.0]; // mean = 7.0
    let group3 = [8.0, 9.0, 10.0, 11.0, 12.0]; // mean = 10.0

    let groups = vec![&group1[..], &group2[..], &group3[..]];

    let result = HypothesisTestingEngine::one_way_anova(&groups);
    assert!(result.is_ok(), "One-way ANOVA should succeed");

    let anova = result.unwrap();
    assert!(anova.significant, "ANOVA should detect significant differences");
    assert!(anova.f_statistic > 0.0, "F-statistic should be positive");
    assert!(anova.p_value < 0.05, "P-value should be significant");

    // Check effect size
    assert!(anova.eta_squared > 0.0 && anova.eta_squared <= 1.0, "Eta squared should be between 0 and 1");

    // Check post-hoc results
    assert!(anova.post_hoc_results.is_some(), "Post-hoc results should be present");
    let post_hoc = anova.post_hoc_results.unwrap();
    assert!(!post_hoc.is_empty(), "Should have post-hoc comparisons");

    println!("✓ One-way ANOVA basic test passed");
}

/// Test one-way ANOVA with equal means (should not be significant)
#[test]
fn test_one_way_anova_no_difference() {
    // Create three groups with the same mean
    let group1 = [5.0, 5.1, 4.9, 5.2, 4.8];
    let group2 = [5.0, 5.1, 4.9, 5.2, 4.8];
    let group3 = [5.0, 5.1, 4.9, 5.2, 4.8];

    let groups = vec![&group1[..], &group2[..], &group3[..]];

    let result = HypothesisTestingEngine::one_way_anova(&groups);
    assert!(result.is_ok(), "One-way ANOVA should succeed");

    let anova = result.unwrap();
    assert!(!anova.significant, "ANOVA should not detect significant differences when means are equal");
    assert!(anova.p_value > 0.05, "P-value should not be significant");

    println!("✓ One-way ANOVA no difference test passed");
}

/// Test two-way ANOVA
#[test]
fn test_two_way_anova() {
    // Create data for 2x3 factorial design (2 levels of factor A, 3 levels of factor B)
    // Factor A: treatment (0=control, 1=treated)
    // Factor B: time (0=baseline, 1=week1, 2=week2)

    let data = vec![
        vec![10.0, 10.5, 9.5, 10.2, 9.8], // Control, baseline
        vec![12.0, 12.5, 11.5, 12.2, 11.8], // Control, week1
        vec![15.0, 15.5, 14.5, 15.2, 14.8], // Control, week2
        vec![15.0, 15.5, 14.5, 15.2, 14.8], // Treated, baseline
        vec![17.0, 17.5, 16.5, 17.2, 16.8], // Treated, week1
        vec![20.0, 20.5, 19.5, 20.2, 19.8], // Treated, week2
    ];

    // Factor levels
    let factor1_levels = vec![0, 0, 0, 1, 1, 1]; // Treatment factor
    let factor2_levels = vec![0, 1, 2, 0, 1, 2]; // Time factor

    let result = HypothesisTestingEngine::two_way_anova(&data, &factor1_levels, &factor2_levels);
    if let Err(e) = &result {
        println!("Two-way ANOVA failed with error: {}", e);
    }
    assert!(result.is_ok(), "Two-way ANOVA should succeed");

    let anova = result.unwrap();

    // Both main effects should be significant
    assert!(anova.significant_factor1, "Treatment main effect should be significant");
    assert!(anova.significant_factor2, "Time main effect should be significant");

    // Effect sizes should be reasonable
    assert!(anova.eta_squared_factor1 > 0.0 && anova.eta_squared_factor1 <= 1.0);
    assert!(anova.eta_squared_factor2 > 0.0 && anova.eta_squared_factor2 <= 1.0);

    println!("✓ Two-way ANOVA test passed");
}

/// Test repeated measures ANOVA
#[test]
fn test_repeated_measures_anova() {
    // Create data for 4 subjects measured at 3 time points
    let subjects_data = vec![
        vec![10.0, 12.0, 15.0], // Subject 1: baseline, week1, week2
        vec![11.0, 13.0, 16.0], // Subject 2
        vec![9.0, 11.0, 14.0],  // Subject 3
        vec![12.0, 14.0, 17.0], // Subject 4
    ];

    let result = HypothesisTestingEngine::repeated_measures_anova(&subjects_data);
    assert!(result.is_ok(), "Repeated measures ANOVA should succeed");

    let anova = result.unwrap();

    // Time effect should be significant (increasing means)
    assert!(anova.significant_time, "Time effect should be significant");

    // Effect size should be reasonable
    assert!(anova.eta_squared_time > 0.0 && anova.eta_squared_time <= 1.0);

    // Sphericity test should be present
    assert!(anova.sphericity_test.is_some(), "Sphericity test should be present");

    // Post-hoc results should be present
    assert!(anova.post_hoc_results.is_some(), "Post-hoc results should be present");

    println!("✓ Repeated measures ANOVA test passed");
}

/// Test Bonferroni post-hoc correction
#[test]
fn test_bonferroni_post_hoc() {
    let group1 = [2.0, 3.0, 4.0, 5.0, 6.0];
    let group2 = [5.0, 6.0, 7.0, 8.0, 9.0];
    let group3 = [8.0, 9.0, 10.0, 11.0, 12.0];

    let groups = vec![&group1[..], &group2[..], &group3[..]];
    let group_means = vec![4.0, 7.0, 10.0];
    let group_sizes = vec![5, 5, 5];
    let ms_within = 1.0; // Simplified for testing

    let result = HypothesisTestingEngine::bonferroni_post_hoc(&groups, &group_means, &group_sizes, ms_within, 12.0);
    assert!(result.is_ok(), "Bonferroni post-hoc should succeed");

    let post_hoc = result.unwrap();
    assert!(!post_hoc.is_empty(), "Should have post-hoc comparisons");

    // Check that all comparisons are labeled as Bonferroni
    for comparison in &post_hoc {
        assert!(comparison.comparison.contains("Bonferroni"), "Should be labeled as Bonferroni correction");
    }

    println!("✓ Bonferroni post-hoc test passed");
}

/// Test Holm-Bonferroni post-hoc correction
#[test]
fn test_holm_bonferroni_post_hoc() {
    let group1 = [2.0, 3.0, 4.0, 5.0, 6.0];
    let group2 = [5.0, 6.0, 7.0, 8.0, 9.0];
    let group3 = [8.0, 9.0, 10.0, 11.0, 12.0];

    let groups = vec![&group1[..], &group2[..], &group3[..]];
    let group_means = vec![4.0, 7.0, 10.0];
    let group_sizes = vec![5, 5, 5];
    let ms_within = 1.0;

    let result = HypothesisTestingEngine::holm_bonferroni_post_hoc(&groups, &group_means, &group_sizes, ms_within, 12.0);
    assert!(result.is_ok(), "Holm-Bonferroni post-hoc should succeed");

    let post_hoc = result.unwrap();
    assert!(!post_hoc.is_empty(), "Should have post-hoc comparisons");

    // Check that all comparisons are labeled as Holm-Bonferroni
    for comparison in &post_hoc {
        assert!(comparison.comparison.contains("Holm-Bonferroni"), "Should be labeled as Holm-Bonferroni correction");
    }

    println!("✓ Holm-Bonferroni post-hoc test passed");
}

/// Test ANOVA with insufficient data
#[test]
fn test_anova_insufficient_data() {
    // Test with only one group
    let groups = vec![&[1.0, 2.0, 3.0][..]];

    let result = HypothesisTestingEngine::one_way_anova(&groups);
    assert!(result.is_err(), "ANOVA should fail with insufficient groups");

    // Test with empty groups
    let groups = vec![&[][..], &[1.0][..]];

    let result = HypothesisTestingEngine::one_way_anova(&groups);
    assert!(result.is_err(), "ANOVA should fail with empty groups");

    println!("✓ ANOVA insufficient data test passed");
}

/// Test repeated measures ANOVA with insufficient time points
#[test]
fn test_repeated_measures_insufficient_timepoints() {
    // Only 1 time point
    let subjects_data = vec![
        vec![10.0],
        vec![11.0],
    ];

    let result = HypothesisTestingEngine::repeated_measures_anova(&subjects_data);
    assert!(result.is_err(), "Repeated measures ANOVA should fail with insufficient time points");

    println!("✓ Repeated measures insufficient timepoints test passed");
}

/// Test matrix determinant calculation
#[test]
fn test_matrix_determinant() {
    // Test 2x2 matrix
    let matrix = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
    let det = HypothesisTestingEngine::matrix_determinant(&matrix).unwrap();
    assert!((det - (-2.0)).abs() < 1e-10, "Determinant of [[1,2],[3,4]] should be -2");

    // Test 3x3 matrix
    let matrix = vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0]
    ];
    let det = HypothesisTestingEngine::matrix_determinant(&matrix).unwrap();
    assert!(det.abs() < 1e-10, "Determinant of singular matrix should be 0");

    println!("✓ Matrix determinant test passed");
}