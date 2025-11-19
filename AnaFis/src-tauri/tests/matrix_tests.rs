//! Integration tests for matrix operations
//!
//! These tests validate matrix operations including covariance matrices,
//! PCA, and other linear algebra functionality.

use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::matrix_ops::MatrixOperationsEngine;

#[test]
fn test_covariance_matrix_basic() {
    let data = vec![
        vec![1.0, 2.0, 3.0, 4.0],
        vec![2.0, 3.0, 4.0, 5.0],
        vec![3.0, 4.0, 5.0, 6.0],
    ];

    let result = MatrixOperationsEngine::covariance_matrix(&data);
    assert!(result.is_ok());

    let cov = result.unwrap();
    assert_eq!(cov.nrows(), 3);
    assert_eq!(cov.ncols(), 3);

    // Check that diagonal elements are positive (variances)
    for i in 0..3 {
        assert!(cov[[i, i]] > 0.0);
    }
}

#[test]
fn test_pca_basic() {
    let data = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![2.0, 3.0, 4.0, 5.0, 6.0],
        vec![3.0, 4.0, 5.0, 6.0, 7.0],
    ];

    let result = MatrixOperationsEngine::pca(&data, 2);
    assert!(result.is_ok());

    let pca = result.unwrap();
    assert_eq!(pca.components.len(), 2);
    assert_eq!(pca.explained_variance.len(), 2);
    assert_eq!(pca.explained_variance_ratio.len(), 2);
    assert_eq!(pca.singular_values.len(), 2);

    // Check that explained variance ratios sum to less than or equal to 1
    let total_ratio: f64 = pca.explained_variance_ratio.iter().sum();
    assert!(total_ratio <= 1.0);
}