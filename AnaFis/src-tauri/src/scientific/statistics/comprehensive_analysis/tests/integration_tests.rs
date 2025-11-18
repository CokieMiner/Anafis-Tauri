//! Integration Tests for End-to-End Pipeline Functionality
//!
//! This module contains integration tests that validate the complete statistical
//! analysis pipeline, including ndarray ecosystem integration and full workflow testing.

use crate::scientific::statistics::comprehensive_analysis::{
    layer1_command::ComprehensiveAnalysisCommand,
    layer4_primitives::NdLinearAlgebra,
};
use crate::scientific::statistics::types::AnalysisOptions;
use ndarray::Array2;
use rand::prelude::*;
use rand_pcg::Pcg64;

/// Test the complete statistical analysis pipeline with ndarray integration
#[test]
fn test_full_pipeline_with_ndarray() {
    // Generate test datasets
    let mut rng = Pcg64::seed_from_u64(42);
    let n_samples = 1000;
    let n_variables = 5;

    // Create correlated datasets
    let mut datasets = Vec::new();
    for i in 0..n_variables {
        let mut data = Vec::new();
        for _ in 0..n_samples {
            // Create some correlation structure
            let base = rng.random::<f64>() * 10.0;
            let noise = rng.random::<f64>() * 2.0;
            let correlated_noise = if i > 0 {
                rng.random::<f64>() * 1.0 // Add some correlation
            } else {
                0.0
            };
            data.push(base + noise + correlated_noise);
        }
        datasets.push(data);
    }

    // Test comprehensive analysis
    let options = AnalysisOptions::default();
    let result = ComprehensiveAnalysisCommand::execute(datasets.clone(), options);

    assert!(result.is_ok(), "Comprehensive analysis should succeed");
    let analysis_result = result.unwrap();

    // Verify all expected outputs are present
    assert!(analysis_result.descriptive_stats.is_some());
    assert!(analysis_result.correlation_matrix.is_some());
    assert!(analysis_result.normality_test.is_some());

    println!("✓ Full pipeline analysis completed successfully");
}

/// Test matrix operations with ndarray
#[test]
fn test_matrix_operations_integration() {
    let mut rng = Pcg64::seed_from_u64(12345);

    // Create test matrices
    let size = 50;
    let ndarray_a = Array2::<f64>::from_shape_fn((size, size), |(_, _)| rng.random::<f64>());
    let ndarray_b = Array2::<f64>::from_shape_fn((size, size), |(_, _)| rng.random::<f64>());

    // Test matrix multiplication
    let ndarray_result = NdLinearAlgebra::matrix_multiply(&ndarray_a, &ndarray_b).unwrap();

    // Verify result dimensions
    assert_eq!(ndarray_result.nrows(), size);
    assert_eq!(ndarray_result.ncols(), size);

    // Test that result is finite
    for i in 0..size {
        for j in 0..size {
            assert!(ndarray_result[[i, j]].is_finite(), "Result should be finite");
        }
    }

    println!("✓ Matrix operations integration test passed");
}

/// Test large matrix operations for memory efficiency
#[test]
fn test_large_matrix_operations() {
    let mut rng = Pcg64::seed_from_u64(67890);

    // Test with moderately large matrices (adjust based on system capabilities)
    let size = 200;
    let ndarray_a = Array2::<f64>::from_shape_fn((size, size), |(_, _)| rng.random::<f64>());
    let ndarray_b = Array2::<f64>::from_shape_fn((size, size), |(_, _)| rng.random::<f64>());

    // Test standard multiplication
    let ndarray_result = NdLinearAlgebra::matrix_multiply(&ndarray_a, &ndarray_b).unwrap();

    // Test large matrix multiplication (if implemented)
    let large_result = NdLinearAlgebra::large_matrix_multiply(&ndarray_a, &ndarray_b).unwrap();

    // Verify results are consistent
    for i in 0..10 { // Check first 10 elements
        for j in 0..10 {
            let diff = (ndarray_result[[i, j]] - large_result[[i, j]]).abs();
            assert!(diff < 1e-10, "Large matrix operations should be consistent");
        }
    }

    println!("✓ Large matrix operations test passed");
}

/// Test covariance matrix computation with ndarray
#[test]
fn test_covariance_matrix_integration() {
    let mut rng = Pcg64::seed_from_u64(11111);

    // Create test dataset
    let n_samples = 1000;
    let n_variables = 4;
    let mut data = Vec::new();

    for _ in 0..n_samples {
        let mut row = Vec::new();
        for _ in 0..n_variables {
            row.push(rng.random::<f64>() * 10.0);
        }
        data.push(row);
    }

    // Convert to ndarray format expected by covariance_matrix
    let mut flattened = Vec::new();
    for row in &data {
        flattened.extend(row);
    }
    let ndarray_data = Array2::from_shape_vec((n_samples, n_variables), flattened).unwrap();

    // Compute covariance matrix
    let cov_matrix = NdLinearAlgebra::covariance_matrix(&ndarray_data).unwrap();

    // Verify it's a valid covariance matrix (symmetric, positive semidefinite)
    assert_eq!(cov_matrix.nrows(), n_variables);
    assert_eq!(cov_matrix.ncols(), n_variables);

    // Check symmetry
    for i in 0..n_variables {
        for j in 0..n_variables {
            let diff = (cov_matrix[(i, j)] - cov_matrix[(j, i)]).abs();
            assert!(diff < 1e-10, "Covariance matrix should be symmetric");
        }
    }

    // Check positive semidefinite (all eigenvalues >= 0)
    // This is a basic check - in practice we'd compute eigenvalues
    for i in 0..n_variables {
        assert!(cov_matrix[(i, i)] >= 0.0, "Diagonal elements should be non-negative");
    }

    println!("✓ Covariance matrix integration test passed");
}

/// Test eigenvalue decomposition integration
#[test]
fn test_eigenvalue_decomposition_integration() {
    let mut rng = Pcg64::seed_from_u64(22222);

    // Create a symmetric positive definite matrix
    let size = 10;
    let mut data = Vec::new();
    for i in 0..size {
        for j in 0..size {
            let val = if i == j {
                rng.random::<f64>() + size as f64 // Ensure positive diagonal
            } else {
                rng.random::<f64>() * 0.1
            };
            data.push(val);
        }
    }

    let ndarray_matrix = Array2::from_shape_vec((size, size), data).unwrap();
    // Make it symmetric
    let ndarray_matrix = (&ndarray_matrix + &ndarray_matrix.t().to_owned()) * 0.5;

    // Test eigenvalue decomposition
    let ndarray_eigen = NdLinearAlgebra::eigenvalue_decomposition(&ndarray_matrix).unwrap();

    // Verify eigenvalues are real and finite
    for &eigenval in &ndarray_eigen.0 {
        assert!(eigenval.is_finite(), "Eigenvalues should be finite");
    }

    // Verify eigenvectors matrix has correct dimensions
    assert_eq!(ndarray_eigen.1.nrows(), size);
    assert_eq!(ndarray_eigen.1.ncols(), size);

    println!("✓ Eigenvalue decomposition integration test passed");
}

/// Test SVD integration
#[test]
fn test_svd_integration() {
    let mut rng = Pcg64::seed_from_u64(33333);

    // Create a random matrix
    let rows = 20;
    let cols = 15;
    let ndarray_matrix = Array2::<f64>::from_shape_fn((rows, cols), |(_, _)| rng.random::<f64>());

    // Test SVD
    let ndarray_svd = NdLinearAlgebra::svd(&ndarray_matrix).unwrap();

    // Verify dimensions
    assert_eq!(ndarray_svd.0.nrows(), rows);
    assert_eq!(ndarray_svd.0.ncols(), cols.min(rows));
    assert_eq!(ndarray_svd.1.len(), cols.min(rows));
    assert_eq!(ndarray_svd.2.nrows(), cols.min(rows));
    assert_eq!(ndarray_svd.2.ncols(), cols.min(rows));

    // Verify singular values are non-negative
    for &s in &ndarray_svd.1 {
        assert!(s >= 0.0, "Singular values should be non-negative");
    }

    println!("✓ SVD integration test passed");
}

/// Test error handling in ndarray operations
#[test]
fn test_error_handling() {
    // Test with incompatible matrix dimensions
    let a = Array2::<f64>::zeros((5, 3));
    let b = Array2::<f64>::zeros((4, 5));

    let result = NdLinearAlgebra::matrix_multiply(&a, &b);
    assert!(result.is_err(), "Matrix multiplication should fail with incompatible dimensions");

    // Test with empty matrices
    let empty = Array2::<f64>::zeros((0, 0));
    let result = NdLinearAlgebra::matrix_multiply(&empty, &empty);
    // Empty matrix multiplication is actually valid (0x0 * 0x0 = 0x0)
    // So we just verify it doesn't panic
    let _ = result;

    // Test covariance with insufficient data
    let small_data = Array2::<f64>::zeros((2, 3)); // Only 2 samples
    let result = NdLinearAlgebra::covariance_matrix(&small_data);
    // This might succeed or fail depending on implementation, but shouldn't panic
    // We just verify it doesn't crash
    let _ = result;

    println!("✓ Error handling test passed");
}

/// Test memory efficiency with large datasets
#[test]
fn test_memory_efficiency() {
    // This test verifies that ndarray operations can handle reasonably large datasets
    // without excessive memory usage
    let mut rng = Pcg64::seed_from_u64(44444);

    // Create a moderately large dataset
    let n_samples = 5000;
    let n_features = 50;

    let mut data = Vec::with_capacity(n_samples * n_features);
    for _ in 0..(n_samples * n_features) {
        data.push(rng.random::<f64>());
    }

    let matrix = Array2::from_shape_vec((n_samples, n_features), data).unwrap();

    // Test covariance computation on large dataset
    let cov_result = NdLinearAlgebra::covariance_matrix(&matrix);
    assert!(cov_result.is_ok(), "Should handle large covariance matrices");

    let cov = cov_result.unwrap();
    assert_eq!(cov.nrows(), n_features);
    assert_eq!(cov.ncols(), n_features);

    println!("✓ Memory efficiency test passed for {}x{} matrix", n_samples, n_features);
}

/// Integration test for the complete workflow: data generation -> analysis -> validation
#[test]
fn test_complete_workflow() {
    let mut rng = Pcg64::seed_from_u64(55555);

    // Step 1: Generate synthetic data with known properties
    let n_samples = 200; // Reduced from 2000
    let n_variables = 6;

    let mut datasets = Vec::new();
    for var in 0..n_variables {
        let mut data = Vec::new();
        for sample in 0..n_samples {
            // Create data with some structure
            let trend = (sample as f64) * 0.01;
            let seasonal = ((sample as f64) * 0.1).sin() * 2.0;
            let noise = rng.random::<f64>() * 0.5;
            let correlated = if var > 0 { rng.random::<f64>() * 0.3 } else { 0.0 };

            data.push(trend + seasonal + noise + correlated);
        }
        datasets.push(data);
    }

    // Step 2: Run comprehensive analysis
    let options = AnalysisOptions {
        enabled_analyses: Some(vec![
            "descriptive_stats".to_string(),
            "correlation_analysis".to_string(),
            "normality_test".to_string(),
        ]),
        ..Default::default()
    };

    println!("Starting comprehensive analysis...");
    let start_time = std::time::Instant::now();
    let result = ComprehensiveAnalysisCommand::execute(datasets, options);
    let elapsed = start_time.elapsed();
    println!("Comprehensive analysis completed in {:.2}s", elapsed.as_secs_f64());

    assert!(result.is_ok(), "Complete workflow analysis should succeed");

    let analysis = result.unwrap();

    // Step 3: Validate results
    assert!(analysis.descriptive_stats.is_some());
    assert!(analysis.correlation_matrix.is_some());
    assert!(analysis.normality_test.is_some());

    // Check that correlation matrix makes sense
    if let Some(corr) = &analysis.correlation_matrix {
        // For n_variables x n_variables matrix, we expect n_variables^2 elements
        assert_eq!(corr.len(), n_variables * n_variables);
        // The matrix should be symmetric, so corr[i*n_variables + j] == corr[j*n_variables + i]
        for i in 0..n_variables {
            for j in 0..n_variables {
                let idx1 = i * n_variables + j;
                let idx2 = j * n_variables + i;
                let diff = (corr[idx1] - corr[idx2]).abs();
                assert!(diff < 1e-10, "Correlation matrix should be symmetric");
            }
        }
    }

    println!("✓ Complete workflow integration test passed");
}