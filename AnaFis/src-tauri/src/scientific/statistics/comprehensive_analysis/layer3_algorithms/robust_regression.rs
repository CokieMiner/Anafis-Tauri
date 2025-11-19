use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::LinearAlgebra;
use ndarray::{Array1, Array2, Axis};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct RobustRegressionResult {
    pub coefficients: Vec<f64>,
    pub residuals: Vec<f64>,
    pub r_squared: f64,
    pub converged: bool,
    pub iterations: usize,
}

/// Robust regression algorithms
pub struct RobustRegressionEngine;

impl RobustRegressionEngine {
    /// Huber regression - robust to outliers
    pub fn huber_regression(
        x_vec: &[Vec<f64>], // Design matrix (n_samples x n_features)
        y_vec: &[f64],      // Response variable
        k: f64,             // Huber parameter (typically 1.345)
        max_iter: usize,
        tol: f64,
    ) -> Result<RobustRegressionResult, String> {
        let n_samples = x_vec.len();
        if n_samples == 0 {
            return Err("Empty data".to_string());
        }
        let n_features = x_vec[0].len();

        if n_samples != y_vec.len() {
            return Err("X and y must have the same number of samples".to_string());
        }

        // Convert to ndarray
        let x = Array2::from_shape_vec(
            (n_samples, n_features),
            x_vec.iter().flatten().cloned().collect(),
        )
        .map_err(|e| format!("Failed to create ndarray from x_vec: {}", e))?;
        let y = Array1::from_vec(y_vec.to_vec());
        let n_samples = x.nrows();
        if n_samples == 0 {
            return Err("Empty data".to_string());
        }
        let n_features = x.ncols();

        if n_samples != y.len() {
            return Err("X and y must have the same number of samples".to_string());
        }

        // Initialize coefficients
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
            let max_weight_change = (&weights - &old_weights)
                .mapv(f64::abs)
                .into_iter()
                .fold(0.0, f64::max);

            if max_weight_change < tol {
                converged = true;
                break;
            }

            // Weighted least squares update
            coefficients = Self::weighted_least_squares_update(&x, &y, &weights)?;
        }

        // Compute final R-squared
        let ss_res: f64 = residuals.mapv(|r| r.powi(2)).sum();
        let y_mean = y.mean().ok_or("Failed to compute mean of y for R-squared calculation")?;
        let ss_tot: f64 = y.mapv(|yi| (yi - y_mean).powi(2)).sum();
        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

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
        _stop_probability: f64,   // Probability of finding optimal model
    ) -> Result<RobustRegressionResult, String> {
        let n_samples = x_vec.len();

        if n_samples != y_vec.len() {
            return Err("X and y must have the same number of samples".to_string());
        }

        if n_samples < min_samples {
            return Err("Not enough samples for RANSAC".to_string());
        }
        
        // Convert to ndarray
        let n_features = x_vec[0].len();
        let x = Array2::from_shape_vec(
            (n_samples, n_features),
            x_vec.iter().flatten().cloned().collect(),
        )
        .map_err(|e| format!("Failed to create ndarray from x_vec: {}", e))?;
        let y = Array1::from_vec(y_vec.to_vec());
        let n_samples = x.nrows();

        if n_samples != y.len() {
            return Err("X and y must have the same number of samples".to_string());
        }

        if n_samples < min_samples {
            return Err("Not enough samples for RANSAC".to_string());
        }

        // Generate seeds for parallel RNGs
        use rand::rng;
        use rand::Rng;
        let mut seed_rng = rng();
        let seeds: Vec<u64> = (0..max_trials).map(|_| seed_rng.random::<u64>()).collect();

        // Parallel RANSAC trials
        let best_model = seeds
            .into_par_iter()
            .filter_map(|seed| {
                use rand::seq::index;
                use rand_pcg::Pcg64;
                use rand::SeedableRng;
                let mut thread_rng = Pcg64::seed_from_u64(seed);

                // Randomly sample min_samples points
                let sample_indices = index::sample(&mut thread_rng, n_samples, min_samples).into_vec();

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

                    let score = inliers.len();
                    Some((inliers, sample_coeffs, score))
                } else {
                    None
                }
            })
            .max_by_key(|(_, _, score)| *score);

        // Refit model using all inliers from the best trial
        if let Some((best_inliers, _, _)) = best_model {
            if best_inliers.is_empty() {
                return Err("RANSAC failed to find any inliers.".to_string());
            }

            let inlier_x = x.select(Axis(0), &best_inliers);
            let inlier_y = y.select(Axis(0), &best_inliers);

            let final_coefficients = Self::least_squares_fit(&inlier_x, &inlier_y)?;
            let final_residuals = &y - x.dot(&final_coefficients);

            // Compute R-squared
            let ss_res: f64 = final_residuals.mapv(|r| r.powi(2)).sum();
            let y_mean = y.mean().ok_or("Failed to compute mean of y for R-squared calculation")?;
            let ss_tot: f64 = y.mapv(|yi| (yi - y_mean).powi(2)).sum();
            let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

            Ok(RobustRegressionResult {
                coefficients: final_coefficients.to_vec(),
                residuals: final_residuals.to_vec(),
                r_squared,
                converged: true,
                iterations: max_trials,
            })
        } else {
            Err("RANSAC failed to find a valid model in any trial.".to_string())
        }
    }

    /// Helper: Weighted least squares update
    fn weighted_least_squares_update(
        x: &Array2<f64>,
        y: &Array1<f64>,
        weights: &Array1<f64>,
    ) -> Result<Array1<f64>, String> {
        // --- Use ndarray operations ---
        let w_sqrt = weights.mapv(f64::sqrt);
        // Element-wise multiplication of each row of x with w_sqrt
        let broadcast_shape = (x.ncols(), x.nrows());
        let w_sqrt_b = w_sqrt.broadcast(broadcast_shape)
            .ok_or_else(|| "Failed to broadcast weights".to_string())?;
        let weighted_x = x * &w_sqrt_b.t();
        let weighted_y = y * &w_sqrt;

        Self::least_squares_fit(&weighted_x, &weighted_y)
    }

    /// --- Ordinary least squares fit using ndarray-linalg ---
    fn least_squares_fit(x: &Array2<f64>, y: &Array1<f64>) -> Result<Array1<f64>, String> {
        if x.nrows() < x.ncols() {
            return Err("Not enough samples for regression (underdetermined system)".to_string());
        }

        // Solve the normal equations: (X^T * X) * b = X^T * y
        let xt = x.t();
        let xtx = xt.dot(x);
        let xty = xt.dot(y);

        // Use the robust LinearAlgebra primitive from layer4
        LinearAlgebra::solve_linear_system(&xtx, &xty)
    }
}