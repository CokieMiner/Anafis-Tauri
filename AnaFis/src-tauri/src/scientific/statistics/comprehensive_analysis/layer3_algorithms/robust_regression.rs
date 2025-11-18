

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
        x: &[Vec<f64>],  // Design matrix (n_samples x n_features)
        y: &[f64],       // Response variable
        k: f64,          // Huber parameter (typically 1.345)
        max_iter: usize,
        tol: f64,
    ) -> Result<RobustRegressionResult, String> {
        let n_samples = x.len();
        let n_features = x[0].len();

        if n_samples != y.len() {
            return Err("X and y must have the same number of samples".to_string());
        }

        if n_samples == 0 || n_features == 0 {
            return Err("Empty data".to_string());
        }

        // Initialize coefficients
        let mut coefficients = vec![0.0; n_features];
        let mut residuals = vec![0.0; n_samples];
        let mut weights = vec![1.0; n_samples];

        let mut converged = false;
        let mut iterations = 0;

        for iter in 0..max_iter {
            iterations = iter + 1;

            // Compute residuals (parallelize for large datasets)
            if n_samples > 1000 {
                residuals.par_iter_mut().enumerate().for_each(|(i, residual)| {
                    let mut prediction = 0.0;
                    for (j, coeff) in coefficients.iter().enumerate().take(n_features) {
                        prediction += coeff * x[i][j];
                    }
                    *residual = y[i] - prediction;
                });
            } else {
                for i in 0..n_samples {
                    let mut prediction = 0.0;
                    for (j, coeff) in coefficients.iter().enumerate().take(n_features) {
                        prediction += coeff * x[i][j];
                    }
                    residuals[i] = y[i] - prediction;
                }
            }

            // Update weights using Huber function
            let mut max_weight_change = 0.0f64;
            if n_samples > 1000 {
                let weight_changes: Vec<f64> = weights.par_iter_mut().enumerate().map(|(i, weight)| {
                    let r = residuals[i];
                    let old_weight = *weight;
                    if r.abs() <= k {
                        *weight = 1.0;
                    } else {
                        *weight = k / r.abs();
                    }
                    (*weight - old_weight).abs()
                }).collect();
                max_weight_change = weight_changes.into_iter().fold(0.0f64, f64::max);
            } else {
                for i in 0..n_samples {
                    let r = residuals[i];
                    let old_weight = weights[i];
                    if r.abs() <= k {
                        weights[i] = 1.0;
                    } else {
                        weights[i] = k / r.abs();
                    }
                    max_weight_change = max_weight_change.max((weights[i] - old_weight).abs());
                }
            }

            // Check convergence
            if max_weight_change < tol {
                converged = true;
                break;
            }

            // Weighted least squares update
            Self::weighted_least_squares_update(x, y, &weights, &mut coefficients)?;
        }

        // Compute final R-squared
        let ss_res = residuals.iter().map(|r| r * r).sum::<f64>();
        let y_mean = y.iter().sum::<f64>() / n_samples as f64;
        let ss_tot = y.iter().map(|yi| (yi - y_mean).powi(2)).sum::<f64>();
        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

        Ok(RobustRegressionResult {
            coefficients,
            residuals,
            r_squared,
            converged,
            iterations,
        })
    }

    /// RANSAC (Random Sample Consensus) regression
    pub fn ransac_regression(
        x: &[Vec<f64>],  // Design matrix
        y: &[f64],       // Response variable
        min_samples: usize,  // Minimum samples for model
        max_trials: usize,   // Maximum RANSAC trials
        residual_threshold: f64,  // Threshold for inlier classification
        _stop_probability: f64,    // Probability of finding optimal model
    ) -> Result<RobustRegressionResult, String> {
        let n_samples = x.len();

        if n_samples != y.len() {
            return Err("X and y must have the same number of samples".to_string());
        }

        if n_samples < min_samples {
            return Err("Not enough samples for RANSAC".to_string());
        }

        // Generate seeds for parallel RNGs
        use rand::rng;
        let mut seed_rng = rng();
        use rand::Rng;
        let seeds: Vec<u64> = (0..max_trials).map(|_| seed_rng.random::<u64>()).collect();

        // Parallel RANSAC trials
        use rayon::prelude::*;
        let trial_results: Vec<(Vec<usize>, Vec<f64>, f64)> = seeds.into_par_iter()
            .filter_map(|seed| {
                use rand_pcg::Pcg64;
                use rand::SeedableRng;
                let mut thread_rng = Pcg64::seed_from_u64(seed);

                // Randomly sample min_samples points
                let sample_indices = Self::random_sample_indices_with_rng(&mut thread_rng, n_samples, min_samples);

                // Fit model to sample
                let sample_x: Vec<Vec<f64>> = sample_indices.iter().map(|&i| x[i].clone()).collect();
                let sample_y: Vec<f64> = sample_indices.iter().map(|&i| y[i]).collect();

                if let Ok(sample_result) = Self::least_squares_fit(&sample_x, &sample_y) {
                    // Count inliers
                    let mut inliers = Vec::new();
                    for i in 0..n_samples {
                        let prediction = Self::predict(&sample_result, &x[i]);
                        let residual = (y[i] - prediction).abs();
                        if residual < residual_threshold {
                            inliers.push(i);
                        }
                    }

                    let score = inliers.len() as f64 / n_samples as f64;
                    Some((inliers, sample_result, score))
                } else {
                    None
                }
            })
            .collect();

        // Find the best model from parallel results
        let mut best_inliers = Vec::new();
        let mut best_score = 0.0;

        for (inliers, _coefficients, score) in trial_results {
            if score > best_score {
                best_inliers = inliers;
                best_score = score;
            }
        }

        // Refit model using all inliers
        if best_inliers.is_empty() {
            return Err("RANSAC failed to find a good model".to_string());
        }

        let inlier_x: Vec<Vec<f64>> = best_inliers.iter().map(|&i| x[i].clone()).collect();
        let inlier_y: Vec<f64> = best_inliers.iter().map(|&i| y[i]).collect();

        let final_coefficients = Self::least_squares_fit(&inlier_x, &inlier_y)?;

        // Compute residuals for all points
        let mut residuals = vec![0.0; n_samples];
        for i in 0..n_samples {
            let prediction = Self::predict(&final_coefficients, &x[i]);
            residuals[i] = y[i] - prediction;
        }

        // Compute R-squared
        let ss_res = residuals.iter().map(|r| r * r).sum::<f64>();
        let y_mean = y.iter().sum::<f64>() / n_samples as f64;
        let ss_tot = y.iter().map(|yi| (yi - y_mean).powi(2)).sum::<f64>();
        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

        Ok(RobustRegressionResult {
            coefficients: final_coefficients,
            residuals,
            r_squared,
            converged: true,
            iterations: max_trials,
        })
    }

    /// Helper: Weighted least squares update
    fn weighted_least_squares_update(
        x: &[Vec<f64>],
        y: &[f64],
        weights: &[f64],
        coefficients: &mut [f64],
    ) -> Result<(), String> {
        let n_samples = x.len();
        let n_features = x[0].len();

        // Build weighted design matrix and response
        let mut weighted_x = vec![vec![0.0; n_features]; n_samples];
        let mut weighted_y = vec![0.0; n_samples];
        let mut sqrt_weights = vec![0.0; n_samples];

        for i in 0..n_samples {
            let w_sqrt = weights[i].sqrt();
            sqrt_weights[i] = w_sqrt;
            weighted_y[i] = y[i] * w_sqrt;
            for j in 0..n_features {
                weighted_x[i][j] = x[i][j] * w_sqrt;
            }
        }

        // Solve weighted least squares
        Self::least_squares_fit(&weighted_x, &weighted_y)
            .map(|new_coeffs| coefficients.copy_from_slice(&new_coeffs))
    }

    /// Helper: Ordinary least squares fit
    fn least_squares_fit(x: &[Vec<f64>], y: &[f64]) -> Result<Vec<f64>, String> {
        let n_samples = x.len();
        let n_features = x[0].len();

        if n_samples < n_features {
            return Err("Not enough samples for regression".to_string());
        }

        // Simple implementation using normal equations
        // For production, consider using a proper linear algebra library
        let mut xtx = vec![vec![0.0; n_features]; n_features];
        let mut xty = vec![0.0; n_features];

        // Compute X^T * X and X^T * y
        for i in 0..n_samples {
            for j in 0..n_features {
                xty[j] += x[i][j] * y[i];
                for k in 0..n_features {
                    xtx[j][k] += x[i][j] * x[i][k];
                }
            }
        }

        // Solve the system (simplified - assumes well-conditioned)
        Self::solve_linear_system(&xtx, &xty)
    }

    /// Helper: Solve linear system Ax = b using Gaussian elimination (simplified)
    fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>, String> {
        let n = a.len();
        let mut aug = vec![vec![0.0; n + 1]; n];

        // Create augmented matrix
        for i in 0..n {
            for j in 0..n {
                aug[i][j] = a[i][j];
            }
            aug[i][n] = b[i];
        }

        // Gaussian elimination
        for i in 0..n {
            // Find pivot
            let mut max_row = i;
            for k in i + 1..n {
                if aug[k][i].abs() > aug[max_row][i].abs() {
                    max_row = k;
                }
            }

            // Swap rows
            aug.swap(i, max_row);

            // Check for singular matrix
            if aug[i][i].abs() < 1e-10 {
                return Err("Singular matrix in linear system".to_string());
            }

            // Eliminate
            for k in i + 1..n {
                let factor = aug[k][i] / aug[i][i];
                for j in i..=n {
                    aug[k][j] -= factor * aug[i][j];
                }
            }
        }

        // Back substitution
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            x[i] = aug[i][n];
            for j in i + 1..n {
                x[i] -= aug[i][j] * x[j];
            }
            x[i] /= aug[i][i];
        }

        Ok(x)
    }

    /// Helper: Make prediction with coefficients
    fn predict(coefficients: &[f64], features: &[f64]) -> f64 {
        coefficients.iter().zip(features.iter()).map(|(c, f)| c * f).sum()
    }

    /// Helper: Random sample indices without replacement using provided RNG
    fn random_sample_indices_with_rng<R: rand::Rng>(rng: &mut R, n_total: usize, n_sample: usize) -> Vec<usize> {
        use rand::seq::SliceRandom;

        let mut indices: Vec<usize> = (0..n_total).collect();
        indices.shuffle(rng);
        indices.into_iter().take(n_sample).collect()
    }
}