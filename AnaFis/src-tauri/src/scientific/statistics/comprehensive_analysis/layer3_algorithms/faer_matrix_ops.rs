//! High-performance matrix operations using faer crate for extreme matrix sizes
//!
//! This module provides an alternative implementation of matrix operations using the faer crate,
//! which offers superior performance for large matrices compared to ndarray.

use faer::Mat;
use ndarray::Array2;
use crate::scientific::statistics::comprehensive_analysis::utils;

/// High-performance matrix operations engine using faer
pub struct FaerMatrixEngine;

/// Matrix operations that can switch between nalgebra and faer backends
#[derive(Default)]
pub enum MatrixBackend {
    /// Use nalgebra (default, more compatible)
    #[default]
    Nalgebra,
    /// Use faer (higher performance for large matrices)
    Faer,
}

impl FaerMatrixEngine {
    /// Compute covariance matrix using faer for better performance on large datasets
    pub fn covariance_matrix_faer(data: &[Vec<f64>]) -> Result<Mat<f64>, String> {
        let n_obs = utils::validate_variable_lengths(data)?;
        let n_vars = data.len();

        // Convert data to faer matrix (n_obs x n_vars)
        let mut data_matrix = Mat::<f64>::zeros(n_obs, n_vars);
        for i in 0..n_obs {
            for j in 0..n_vars {
                data_matrix[(i, j)] = data[j][i];
            }
        }

        // Compute means
        let mut means = vec![0.0; n_vars];
        for j in 0..n_vars {
            for i in 0..n_obs {
                means[j] += data_matrix[(i, j)];
            }
            means[j] /= n_obs as f64;
        }

        // Center the data
        let mut centered = data_matrix.clone();
        for j in 0..n_vars {
            let mean = means[j];
            for i in 0..n_obs {
                centered[(i, j)] -= mean;
            }
        }

        // Compute covariance matrix: (X^T * X) / (n-1)
        let centered_clone = centered.clone();
        let xt = centered_clone.transpose();
        let cov = xt * centered;
        let cov_scaled = cov / (n_obs - 1) as f64;

        Ok(cov_scaled)
    }

    /// Compute covariance matrix with backend selection
    pub fn covariance_matrix(data: &[Vec<f64>], backend: &MatrixBackend) -> Result<Array2<f64>, String> {
        match backend {
            MatrixBackend::Nalgebra => {
                // Use ndarray implementation
                Self::covariance_matrix_ndarray(data)
            }
            MatrixBackend::Faer => {
                // Use faer implementation and convert back to ndarray
                let faer_result = Self::covariance_matrix_faer(data)?;
                Self::faer_to_ndarray_matrix(&faer_result)
            }
        }
    }

    /// Convert faer matrix to ndarray matrix
    fn faer_to_ndarray_matrix(faer_mat: &Mat<f64>) -> Result<Array2<f64>, String> {
        let nrows = faer_mat.nrows();
        let ncols = faer_mat.ncols();
        let mut ndarray_mat = Array2::<f64>::zeros((nrows, ncols));

        for i in 0..nrows {
            for j in 0..ncols {
                ndarray_mat[[i, j]] = faer_mat[(i, j)];
            }
        }

        Ok(ndarray_mat)
    }

    /// Convert ndarray Array2 to faer Mat
    fn ndarray_to_faer_matrix(ndarray_mat: &Array2<f64>) -> Result<Mat<f64>, String> {
        let (rows, cols) = ndarray_mat.dim();
        let mut faer_mat = Mat::<f64>::zeros(rows, cols);

        for i in 0..rows {
            for j in 0..cols {
                faer_mat[(i, j)] = ndarray_mat[[i, j]];
            }
        }

        Ok(faer_mat)
    }

    /// Fallback ndarray implementation
    fn covariance_matrix_ndarray(data: &[Vec<f64>]) -> Result<Array2<f64>, String> {
        let n_obs = utils::validate_variable_lengths(data)?;

        // Convert data to ndarray format (n_observations x n_variables)
        let mut data_array = Array2::<f64>::zeros((n_obs, data.len()));
        for (j, var) in data.iter().enumerate() {
            for (i, &val) in var.iter().enumerate() {
                data_array[[i, j]] = val;
            }
        }

        // Use NdLinearAlgebra for efficient covariance computation
        super::super::layer4_primitives::NdLinearAlgebra::covariance_matrix(&data_array)
    }

    /// Matrix multiplication using faer
    pub fn matrix_multiply_faer(a: &Mat<f64>, b: &Mat<f64>) -> Result<Mat<f64>, String> {
        if a.ncols() != b.nrows() {
            return Err(format!("Matrix dimension mismatch: {}x{} vs {}x{}",
                a.nrows(), a.ncols(), b.nrows(), b.ncols()));
        }

        Ok(a * b)
    }

    /// Cholesky decomposition using faer
    pub fn cholesky_faer(matrix: &Mat<f64>) -> Result<Mat<f64>, String> {
        if matrix.nrows() != matrix.ncols() {
            return Err("Matrix must be square for Cholesky decomposition".to_string());
        }

        // Convert to ndarray and use proven implementation
        let ndarray_matrix = Self::faer_to_ndarray_matrix(matrix)?;
        let ndarray_result = super::super::layer4_primitives::LinearAlgebra::cholesky_decomposition(&ndarray_matrix)?;
        Self::ndarray_to_faer_matrix(&ndarray_result)
    }

    /// Eigenvalue decomposition using faer
    pub fn eigen_faer(matrix: &Mat<f64>) -> Result<(Vec<f64>, Mat<f64>), String> {
        if matrix.nrows() != matrix.ncols() {
            return Err("Matrix must be square for eigenvalue decomposition".to_string());
        }

        // Convert to ndarray and use parallel implementation for large matrices
        let ndarray_matrix = Self::faer_to_ndarray_matrix(matrix)?;
        let (eigenvals, eigenvecs) = super::super::layer4_primitives::LinearAlgebra::parallel_eigenvalue_decomposition(&ndarray_matrix)?;

        let eigenvalues = eigenvals.to_vec();
        let eigenvectors = Self::ndarray_to_faer_matrix(&eigenvecs)?;

        Ok((eigenvalues, eigenvectors))
    }

    /// Performance comparison between nalgebra and faer
    pub fn benchmark_matrix_operations(size: usize, iterations: usize) -> Result<String, String> {
        use std::time::Instant;

        // Generate test data
        let data: Vec<Vec<f64>> = (0..size)
            .map(|i| (0..size).map(|j| (i as f64 * j as f64).sin()).collect())
            .collect();

        let mut nalgebra_times = Vec::new();
        let mut faer_times = Vec::new();

        // Benchmark nalgebra (now ndarray)
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = Self::covariance_matrix_ndarray(&data)?;
            nalgebra_times.push(start.elapsed());
        }

        // Benchmark faer
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = Self::covariance_matrix_faer(&data)?;
            faer_times.push(start.elapsed());
        }

        let nalgebra_avg = nalgebra_times.iter().sum::<std::time::Duration>() / nalgebra_times.len() as u32;
        let faer_avg = faer_times.iter().sum::<std::time::Duration>() / faer_times.len() as u32;

        let speedup = nalgebra_avg.as_nanos() as f64 / faer_avg.as_nanos() as f64;

        Ok(format!(
            "Matrix size: {}x{}, Iterations: {}\n\
             Nalgebra average time: {:.2}ms\n\
             Faer average time: {:.2}ms\n\
             Speedup: {:.2}x",
            size, size, iterations,
            nalgebra_avg.as_millis(),
            faer_avg.as_millis(),
            speedup
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faer_covariance_basic() {
        let data = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![2.0, 3.0, 4.0, 5.0],
            vec![3.0, 4.0, 5.0, 6.0],
        ];

        let result = FaerMatrixEngine::covariance_matrix_faer(&data);
        assert!(result.is_ok());

        let cov = result.unwrap();
        assert_eq!(cov.nrows(), 3);
        assert_eq!(cov.ncols(), 3);

        // Check that diagonal elements are positive (variances)
        for i in 0..3 {
            assert!(cov[(i, i)] > 0.0);
        }
    }

    #[test]
    fn test_matrix_multiply_faer() {
        let a = Mat::<f64>::from_fn(2, 3, |i, j| (i + j) as f64);
        let b = Mat::<f64>::from_fn(3, 2, |i, j| (i * j + 1) as f64);

        let result = FaerMatrixEngine::matrix_multiply_faer(&a, &b);
        assert!(result.is_ok());

        let c = result.unwrap();
        assert_eq!(c.nrows(), 2);
        assert_eq!(c.ncols(), 2);
    }

    #[test]
    fn test_benchmark_small() {
        let result = FaerMatrixEngine::benchmark_matrix_operations(5, 3);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.contains("Speedup"));
        println!("Benchmark results:\n{}", report);
    }
}