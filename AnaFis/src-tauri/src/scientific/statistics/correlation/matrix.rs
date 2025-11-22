//! Correlation matrix operations

use ndarray::{Array2, ArrayView2};
use ndarray_linalg::Inverse;
use rayon::prelude::*;
use crate::scientific::statistics::correlation::CorrelationMethods;
use crate::scientific::statistics::primitives::Distance;

/// Correlation matrix operations
pub struct CorrelationMatrixOps;

impl CorrelationMatrixOps {
    /// Compute correlation matrix for multiple variables using Pearson correlation
    pub fn correlation_matrix(data: ArrayView2<f64>) -> Result<Array2<f64>, String> {
        Self::compute_matrix_with_method(data, "pearson", 9.0)
    }

    /// Compute correlation matrix using specified correlation method
    pub fn compute_matrix_with_method(data: ArrayView2<f64>, method: &str, biweight_tuning: f64) -> Result<Array2<f64>, String> {
        let n_vars = data.ncols();
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
                    let col_i = data.column(i).to_vec();
                    let col_j = data.column(j).to_vec();
                    let corr = match method {
                        "pearson" => CorrelationMethods::pearson_correlation(&col_i, &col_j, None, None)
                            .map(|(r, _)| r)
                            .expect("Correlation computation should not fail for valid datasets"),
                        "spearman" => CorrelationMethods::spearman_correlation(&col_i, &col_j, None, None)
                            .map(|(r, _)| r)
                            .expect("Correlation computation should not fail for valid datasets"),
                        "kendall" => CorrelationMethods::kendall_correlation(&col_i, &col_j, None, None)
                            .map(|(r, _)| r)
                            .expect("Correlation computation should not fail for valid datasets"),
                        "biweight" => CorrelationMethods::biweight_midcorrelation_tuned(&col_i, &col_j, biweight_tuning)
                            .expect("Correlation computation should not fail for valid datasets"),
                        _ => CorrelationMethods::pearson_correlation(&col_i, &col_j, None, None)
                            .map(|(r, _)| r)
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
    pub fn partial_correlations(data: ArrayView2<f64>) -> Result<Array2<f64>, String> {
        let n_vars = data.ncols();
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
            Err(_) => {
                // Singular matrix implies perfect multicollinearity.
                // Returning 0.0 is a design choice to handle this gracefully,
                // effectively treating the partial correlation as undefined/zero
                // rather than panicking or returning NaN.
                return 0.0; 
            },
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

        // Compute distance matrices using centralized functions
        let dist_x = Distance::distance_matrix(x);
        let dist_y = Distance::distance_matrix(y);

        // Center the distance matrices
        let centered_dist_x = Distance::center_distance_matrix(&dist_x);
        let centered_dist_y = Distance::center_distance_matrix(&dist_y);

        // Compute the distance covariance
        let dcov = Distance::distance_covariance(&centered_dist_x, &centered_dist_y);

        // Compute distance variances
        let dvar_x = Distance::distance_covariance(&centered_dist_x, &centered_dist_x).sqrt();
        let dvar_y = Distance::distance_covariance(&centered_dist_y, &centered_dist_y).sqrt();

        if dvar_x == 0.0 || dvar_y == 0.0 {
            return Ok(0.0);
        }

        Ok(dcov / (dvar_x * dvar_y))
    }

    /// Compute cross-correlation between two time series
    pub fn cross_correlation(x: &[f64], y: &[f64], max_lag: usize) -> Result<Vec<f64>, String> {
        if x.len() != y.len() || x.len() < max_lag + 1 {
            return Err("Time series must have equal length and be longer than max_lag".to_string());
        }

        let mut ccf = Vec::with_capacity(2 * max_lag + 1);

        // Negative lags (x leading y)
        for lag in (1..=max_lag).rev() {
            let corr = CorrelationMethods::pearson_correlation(&x[lag..], &y[..y.len() - lag], None, None).map(|(r, _)| r)?;
            ccf.push(corr);
        }

        // Zero lag
        ccf.push(CorrelationMethods::pearson_correlation(x, y, None, None).map(|(r, _)| r)?);

        // Positive lags (y leading x)
        for lag in 1..=max_lag {
            let corr = CorrelationMethods::pearson_correlation(&x[..x.len() - lag], &y[lag..], None, None).map(|(r, _)| r)?;
            ccf.push(corr);
        }

        Ok(ccf)
    }
}