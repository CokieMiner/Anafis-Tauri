//! Correlation matrix operations

use ndarray::{Array2, ArrayView2};
use ndarray_linalg::Inverse;
use rayon::prelude::*;
use statrs::distribution::ContinuousCDF;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::correlation_methods::CorrelationMethods;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::correlation_utils::ensure_positive_definite;

/// Correlation matrix operations
pub struct CorrelationMatrix;

impl CorrelationMatrix {
    /// Compute correlation matrix for multiple variables
    pub fn correlation_matrix(data: &[Vec<f64>]) -> Result<Array2<f64>, String> {
        let n_vars = data.len();
        if n_vars == 0 {
            return Err("No data provided".to_string());
        }

        let n_obs = data[0].len();
        if n_obs < 2 {
            return Err("Need at least 2 observations for correlation computation".to_string());
        }
        for (i, var) in data.iter().enumerate() {
            if var.len() != n_obs {
                return Err(format!("Variable {} has {} observations, expected {}", i, var.len(), n_obs));
            }
        }

        let mut matrix = Array2::<f64>::zeros((n_vars, n_vars));

        // Compute correlations in parallel
        let correlations: Vec<(usize, usize, f64)> = (0..n_vars)
            .flat_map(|i| (i..n_vars).map(move |j| (i, j)))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(i, j)| {
                let corr = if i == j {
                    1.0
                } else {
                    CorrelationMethods::pearson_correlation(&data[i], &data[j]).unwrap_or(f64::NAN)
                };
                (i, j, corr)
            })
            .collect();

        // Fill the matrix (symmetric)
        for (i, j, corr) in correlations {
            matrix[[i, j]] = corr;
            matrix[[j, i]] = corr;
        }

        Ok(matrix)
    }

    /// Compute correlation matrix using specified correlation method
    pub fn compute_matrix_with_method(datasets: &[Vec<f64>], method: &str, biweight_tuning: f64) -> Result<Array2<f64>, String> {
        let mut correlation_matrix = Self::compute_matrix_with_method_unchecked(datasets, method, biweight_tuning)?;

        // Ensure the matrix is positive semi-definite
        ensure_positive_definite(&mut correlation_matrix);

        Ok(correlation_matrix)
    }

    /// Compute correlation matrix without positive definiteness check
    pub fn compute_matrix_with_method_unchecked(datasets: &[Vec<f64>], method: &str, biweight_tuning: f64) -> Result<Array2<f64>, String> {
        let n_vars = datasets.len();
        if n_vars < 2 {
            return Err("Need at least 2 datasets for correlation matrix".to_string());
        }

        let mut correlation_matrix = Array2::<f64>::zeros((n_vars, n_vars));

        // Set diagonal to 1.0
        for i in 0..n_vars {
            correlation_matrix[[i, i]] = 1.0;
        }

        // Compute all pairwise correlations in parallel
        let correlation_results: Vec<(usize, usize, f64)> = (0..n_vars)
            .into_par_iter()
            .flat_map(|i| {
                (i + 1..n_vars).into_par_iter().map(move |j| {
                    let corr = match method {
                        "pearson" => CorrelationMethods::pearson_correlation(&datasets[i], &datasets[j])
                            .expect("Correlation computation should not fail for valid datasets"),
                        "spearman" => CorrelationMethods::spearman_correlation(&datasets[i], &datasets[j])
                            .expect("Correlation computation should not fail for valid datasets"),
                        "kendall" => CorrelationMethods::kendall_correlation(&datasets[i], &datasets[j])
                            .expect("Correlation computation should not fail for valid datasets"),
                        "biweight" => CorrelationMethods::biweight_midcorrelation_tuned(&datasets[i], &datasets[j], biweight_tuning)
                            .expect("Correlation computation should not fail for valid datasets"),
                        _ => CorrelationMethods::pearson_correlation(&datasets[i], &datasets[j])
                            .expect("Correlation computation should not fail for valid datasets"),
                    };
                    (i, j, corr)
                })
            })
            .collect();

        // Fill the matrix with computed correlations
        for (i, j, corr) in correlation_results {
            correlation_matrix[[i, j]] = corr;
            correlation_matrix[[j, i]] = corr; // Symmetric
        }

        Ok(correlation_matrix)
    }

    /// Compute partial correlations
    pub fn partial_correlations(data: &[Vec<f64>]) -> Result<Array2<f64>, String> {
        let n_vars = data.len();
        if n_vars < 3 {
            return Err("Partial correlations require at least 3 variables".to_string());
        }

        let full_corr = Self::correlation_matrix(data)?;

        let mut partial_corr = Array2::<f64>::zeros((n_vars, n_vars));

        for i in 0..n_vars {
            for j in 0..n_vars {
                if i == j {
                    partial_corr[[i, j]] = 1.0;
                } else {
                    // Compute partial correlation between i and j given all other variables
                    partial_corr[[i, j]] = Self::partial_correlation_ij(&full_corr.view(), i, j);
                }
            }
        }

        Ok(partial_corr)
    }

    /// Compute partial correlation between variables i and j given all others
    fn partial_correlation_ij(corr_matrix: &ArrayView2<f64>, i: usize, j: usize) -> f64 {
        let n = corr_matrix.nrows();

        if n == 2 {
            // Simple case: just the regular correlation
            return corr_matrix[[i, j]];
        }

        // Create the precision matrix (inverse of correlation matrix)
        let precision = match corr_matrix.inv() {
            Ok(inv) => inv,
            Err(_) => return 0.0, // Singular matrix
        };

        // Partial correlation is -precision[i,j] / sqrt(precision[i,i] * precision[j,j])
        let numerator = -precision[[i, j]];
        let denominator = (precision[[i, i]] * precision[[j, j]]).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Compute distance correlation (energy distance correlation)
    pub fn distance_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Vectors must have equal length and at least 2 elements".to_string());
        }

        // Compute distance matrices
        let dist_x = Self::euclidean_distance_matrix(x);
        let dist_y = Self::euclidean_distance_matrix(y);

        // Center the distance matrices
        let centered_dist_x = Self::center_distance_matrix(&dist_x);
        let centered_dist_y = Self::center_distance_matrix(&dist_y);

        // Compute the distance covariance
        let dcov = Self::distance_covariance_from_centered(&centered_dist_x, &centered_dist_y);

        // Compute distance variances
        let dvar_x = Self::distance_covariance_from_centered(&centered_dist_x, &centered_dist_x).sqrt();
        let dvar_y = Self::distance_covariance_from_centered(&centered_dist_y, &centered_dist_y).sqrt();

        if dvar_x == 0.0 || dvar_y == 0.0 {
            return Ok(0.0);
        }

        Ok(dcov / (dvar_x * dvar_y))
    }

    /// Compute distance covariance from centered distance matrices
    fn distance_covariance_from_centered(dist_x: &Array2<f64>, dist_y: &Array2<f64>) -> f64 {
        let n = dist_x.nrows() as f64;
        let sum = dist_x.iter().zip(dist_y.iter()).map(|(a, b)| a * b).sum::<f64>();
        sum / (n * n)
    }

    /// Center a distance matrix using double centering
    fn center_distance_matrix(dist: &Array2<f64>) -> Array2<f64> {
        let n = dist.nrows();
        let n_f = n as f64;

        // Row means
        let row_means: Vec<f64> = (0..n).map(|i| dist.row(i).sum() / n_f).collect();

        // Column means
        let col_means: Vec<f64> = (0..n).map(|j| dist.column(j).sum() / n_f).collect();

        // Overall mean
        let overall_mean = row_means.iter().sum::<f64>() / n_f;

        // Double centering
        let mut centered = Array2::<f64>::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                centered[[i, j]] = dist[[i, j]] - row_means[i] - col_means[j] + overall_mean;
            }
        }

        centered
    }

    /// Compute Euclidean distance matrix for a vector
    fn euclidean_distance_matrix(data: &[f64]) -> Array2<f64> {
        let n = data.len();
        let mut dist = Array2::<f64>::zeros((n, n));

        for i in 0..n {
            for j in 0..n {
                dist[[i, j]] = (data[i] - data[j]).abs();
            }
        }

        dist
    }

    /// Compute autocorrelation function
    pub fn autocorrelation(data: &[f64], max_lag: usize) -> Result<Vec<f64>, String> {
        if data.len() < max_lag + 1 {
            return Err("Data length must be greater than max_lag".to_string());
        }

        let mut acf = Vec::with_capacity(max_lag + 1);
        acf.push(1.0); // lag 0

        for lag in 1..=max_lag {
            let corr = CorrelationMethods::pearson_correlation(&data[lag..], &data[..data.len() - lag])?;
            acf.push(corr);
        }

        Ok(acf)
    }

    /// Compute cross-correlation between two time series
    pub fn cross_correlation(x: &[f64], y: &[f64], max_lag: usize) -> Result<Vec<f64>, String> {
        if x.len() != y.len() || x.len() < max_lag + 1 {
            return Err("Time series must have equal length and be longer than max_lag".to_string());
        }

        let mut ccf = Vec::with_capacity(2 * max_lag + 1);

        // Negative lags (x leading y)
        for lag in (1..=max_lag).rev() {
            let corr = CorrelationMethods::pearson_correlation(&x[lag..], &y[..y.len() - lag])?;
            ccf.push(corr);
        }

        // Zero lag
        ccf.push(CorrelationMethods::pearson_correlation(x, y)?);

        // Positive lags (y leading x)
        for lag in 1..=max_lag {
            let corr = CorrelationMethods::pearson_correlation(&x[..x.len() - lag], &y[lag..])?;
            ccf.push(corr);
        }

        Ok(ccf)
    }

    /// Compute correlation matrix with significance testing
    pub fn correlation_matrix_with_significance(
        data: &[Vec<f64>],
        alpha: f64,
    ) -> Result<(Array2<f64>, Array2<bool>), String> {
        let corr_matrix = Self::correlation_matrix(data)?;
        let n_vars = data.len();
        let n_obs = data[0].len();

        let mut significance_matrix = Array2::<bool>::from_elem((n_vars, n_vars), false);

        // Critical value for significance (two-tailed test)
        let df = n_obs - 2;
        let t_critical = Self::students_t_quantile(1.0 - alpha / 2.0, df);

        for i in 0..n_vars {
            for j in (i + 1)..n_vars {
                let r = corr_matrix[[i, j]];
                let t_stat = r * ((df as f64) / (1.0 - r * r)).sqrt();
                significance_matrix[[i, j]] = t_stat.abs() > t_critical;
                significance_matrix[[j, i]] = significance_matrix[[i, j]];
            }
        }

        Ok((corr_matrix, significance_matrix))
    }

    /// Approximate quantile of Student's t distribution
    fn students_t_quantile(p: f64, df: usize) -> f64 {
        // Approximation using normal distribution for large df
        if df > 30 {
            let normal = statrs::distribution::Normal::new(0.0, 1.0).unwrap();
            return normal.inverse_cdf(p);
        }

        // For small df, use approximation
        let df_f = df as f64;
        let z = statrs::distribution::Normal::new(0.0, 1.0).unwrap().inverse_cdf(p);
        z + (z.powi(3) + z) / (4.0 * df_f) + (5.0 * z.powi(5) + 16.0 * z.powi(3) + 3.0 * z) / (96.0 * df_f.powi(2))
            + (3.0 * z.powi(7) + 19.0 * z.powi(5) + 17.0 * z.powi(3) - 15.0 * z) / (384.0 * df_f.powi(3))
    }
}