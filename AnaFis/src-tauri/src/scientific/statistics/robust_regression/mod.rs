//! Robust Regression Module
//!
//! This module provides robust regression methods that are resistant to outliers:
//! - Huber regression (robust to moderate outliers)
//! - RANSAC (Random Sample Consensus) for handling many outliers
//! - Iteratively reweighted least squares (IRLS)

pub mod types;

use types::*;
use crate::scientific::statistics::primitives::{LinearRegression, RobustRegressionUtils};
use ndarray::{Array1, Array2, Axis};
use rand::prelude::*;
use rand_pcg::Pcg64;
use rayon::prelude::*;

/// Robust regression engine
pub struct RobustRegressionEngine;

impl RobustRegressionEngine {
    /// Huber regression - robust to outliers using Huber loss function
    pub fn huber_regression(
        x_vec: &[Vec<f64>], // Design matrix (n_samples x n_features)
        y_vec: &[f64],      // Response variable
        k: f64,             // Huber parameter (typically 1.345)
        max_iter: usize,
        tol: f64,
    ) -> Result<RobustRegressionResult, String> {
        let (n_samples, n_features) = RobustRegressionUtils::validate_input_data(x_vec, y_vec)?;
        let (x, y) = RobustRegressionUtils::convert_to_ndarray(x_vec, y_vec)?;

        // Initialize coefficients and weights
        let mut coefficients = Array1::zeros(n_features);
        let mut residuals = Array1::zeros(n_samples);
        let mut weights = Array1::ones(n_samples);

        let mut converged = false;
        let mut iterations = 0;

        for iter in 0..max_iter {
            iterations = iter + 1;

            // Compute residuals
            residuals = &y - x.dot(&coefficients);

            // Update weights using Huber function
            let old_weights = weights.clone();
            weights = residuals.mapv(|r| if r.abs() <= k { 1.0 } else { k / r.abs() });

            // Check convergence
            let (conv, _) = RobustRegressionUtils::check_weight_convergence(&old_weights, &weights, tol);
            if conv {
                converged = true;
                break;
            }

            // Weighted least squares update
            coefficients = RobustRegressionUtils::weighted_least_squares_update(&x, &y, &weights)?;
        }

        // Compute final R-squared
        let r_squared = RobustRegressionUtils::calculate_r_squared_from_residuals(&residuals, &y)?;

        Ok(RobustRegressionResult {
            coefficients: coefficients.to_vec(),
            residuals: residuals.to_vec(),
            r_squared,
            converged,
            iterations,
        })
    }

    /// RANSAC (Random Sample Consensus) regression
    pub fn ransac_regression(
        x_vec: &[Vec<f64>], // Design matrix
        y_vec: &[f64],      // Response variable
        min_samples: usize, // Minimum samples for model
        max_trials: usize,  // Maximum RANSAC trials
        residual_threshold: f64, // Threshold for inlier classification
    ) -> Result<RobustRegressionResult, String> {
        let (n_samples, _) = RobustRegressionUtils::validate_input_data(x_vec, y_vec)?;
        let (x, y) = RobustRegressionUtils::convert_to_ndarray(x_vec, y_vec)?;

        if n_samples < min_samples {
            return Err("Not enough samples for RANSAC".to_string());
        }

        // Generate seeds for parallel RNGs
        let mut seed_rng = rand::rng();
        let seeds: Vec<u64> = (0..max_trials).map(|_| seed_rng.random::<u64>()).collect();

        // RANSAC trials in parallel
        let trial_results: Vec<(Vec<f64>, usize)> = seeds.into_par_iter()
            .map(|seed| {
                let mut thread_rng = Pcg64::seed_from_u64(seed);

                // Randomly sample min_samples points
                let sample_indices: Vec<usize> = (0..n_samples).choose_multiple(&mut thread_rng, min_samples);

                let sample_x = x.select(Axis(0), &sample_indices);
                let sample_y = y.select(Axis(0), &sample_indices);

                // Fit model to sample
                if let Ok(sample_coeffs) = Self::least_squares_fit(&sample_x, &sample_y) {
                    // Count inliers
                    let residuals = (&y - x.dot(&sample_coeffs)).mapv(f64::abs);
                    let inliers: Vec<usize> = residuals
                        .into_iter()
                        .enumerate()
                        .filter(|(_, r)| *r < residual_threshold)
                        .map(|(i, _)| i)
                        .collect();

                    (sample_coeffs.to_vec(), inliers.len())
                } else {
                    (vec![], 0)
                }
            })
            .collect();

        // Find best model (most inliers)
        let (best_coeffs, _) = trial_results.into_iter()
            .max_by_key(|(_, inlier_count)| *inlier_count)
            .ok_or("No valid RANSAC trials")?;

        if best_coeffs.is_empty() {
            return Err("RANSAC failed to find any valid models".to_string());
        }

        let best_coeffs_array = Array1::from_vec(best_coeffs);
        let residuals = (&y - x.dot(&best_coeffs_array)).mapv(f64::abs);
        let inlier_indices: Vec<usize> = residuals
            .into_iter()
            .enumerate()
            .filter(|(_, r)| *r < residual_threshold)
            .map(|(i, _)| i)
            .collect();

        // Refit the model using all inliers for better accuracy
        let inlier_x = x.select(Axis(0), &inlier_indices);
        let inlier_y = y.select(Axis(0), &inlier_indices);

        let final_coeffs = if inlier_x.nrows() >= x.ncols() {
            Self::least_squares_fit(&inlier_x, &inlier_y)?
        } else {
            // Not enough inliers to refit, return the best model from trials
            best_coeffs_array
        };

        let final_residuals = &y - x.dot(&final_coeffs);

        // Compute R-squared
        let r_squared = RobustRegressionUtils::calculate_r_squared_from_residuals(&final_residuals, &y)?;

        Ok(RobustRegressionResult {
            coefficients: final_coeffs.to_vec(),
            residuals: final_residuals.to_vec(),
            r_squared,
            converged: true,
            iterations: max_trials,
        })
    }

    /// Iteratively Reweighted Least Squares (IRLS) with Tukey's biweight function
    pub fn irls_regression(
        x_vec: &[Vec<f64>],
        y_vec: &[f64],
        tuning_constant: f64, // Tuning constant for biweight function
        max_iter: usize,
        tol: f64,
    ) -> Result<RobustRegressionResult, String> {
        let (_, n_features) = RobustRegressionUtils::validate_input_data(x_vec, y_vec)?;
        let (x, y) = RobustRegressionUtils::convert_to_ndarray(x_vec, y_vec)?;

        // Initialize coefficients
        let mut coefficients = Array1::zeros(n_features);
        let mut weights = Array1::ones(y.len());

        let mut converged = false;
        let mut iterations = 0;

        for iter in 0..max_iter {
            iterations = iter + 1;

            // Compute residuals
            let residuals = &y - x.dot(&coefficients);

            // Update weights using Tukey's biweight function
            let old_weights = weights.clone();
            weights = residuals.mapv(|r| {
                let u = r / tuning_constant;
                if u.abs() <= 1.0 {
                    let rho = 1.0 - u * u;
                    rho * rho
                } else {
                    0.0 // Outliers get zero weight
                }
            });            // Check convergence
            let (conv, _) = RobustRegressionUtils::check_weight_convergence(&old_weights, &weights, tol);
            if conv {
                converged = true;
                break;
            }

            // Weighted least squares update
            coefficients = RobustRegressionUtils::weighted_least_squares_update(&x, &y, &weights)?;
        }

        // Compute final R-squared
        let residuals = &y - x.dot(&coefficients);
        let r_squared = RobustRegressionUtils::calculate_r_squared_from_residuals(&residuals, &y)?;

        Ok(RobustRegressionResult {
            coefficients: coefficients.to_vec(),
            residuals: residuals.to_vec(),
            r_squared,
            converged,
            iterations,
        })
    }

    /// Ordinary least squares fit using SVD for numerical stability
    fn least_squares_fit(x: &Array2<f64>, y: &Array1<f64>) -> Result<Array1<f64>, String> {
        LinearRegression::ols_fit(x, y)
    }
}