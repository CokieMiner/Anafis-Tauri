//! Data imputation utilities - Reviewed
//!
//! This module provides missing data imputation methods including
//! KNN imputation, statistical imputation, and model-based approaches.
//!
//! All imputation methods are parallelized using rayon for improved performance
//! on multi-core systems, with appropriate parallelization strategies for each algorithm.

use serde::{Deserialize, Serialize};
use crate::scientific::statistics::primitives::Distance;
use crate::scientific::statistics::descriptive::{StatisticalMoments, Quantiles};
use crate::scientific::statistics::CorrelationMethods;
use rand::SeedableRng;
use ndarray::{Array1, Array2};
use crate::scientific::statistics::primitives::LinearRegression;
use rand_distr::{Normal, Distribution};

type ImputeFn = dyn Fn(&[Vec<f64>], f64) -> Result<ImputationResult, String>;
use rand::seq::SliceRandom;
use kiddo::{KdTree, SquaredEuclidean};
use rayon::prelude::*;



/// Result of imputation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImputationResult {
    /// Imputed dataset
    pub imputed_data: Vec<Vec<f64>>,
    /// Method used for imputation
    pub method: String,
    /// Number of missing values that were imputed
    pub imputed_count: usize,
    /// Quality metrics for the imputation
    pub quality_metrics: ImputationQuality,
}

/// Quality metrics for imputation evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImputationQuality {
    /// Mean absolute error (if validation data available)
    pub mae: Option<f64>,
    /// Root mean squared error (if validation data available)
    pub rmse: Option<f64>,
    /// Preservation of correlation structure
    pub correlation_preservation: f64,
    /// Preservation of variance
    pub variance_preservation: f64,
}

/// Data imputation engine
pub struct DataImputationEngine;

impl DataImputationEngine {
    // ===== HELPER FUNCTIONS =====

    /// Validate input data dimensions and consistency
    fn validate_data(data: &[Vec<f64>]) -> Result<(usize, usize), String> {
        if data.is_empty() || data[0].is_empty() {
            return Err("Cannot impute empty dataset".to_string());
        }

        let n_rows = data.len();
        let n_cols = data[0].len();

        // Validate data dimensions
        for row in data {
            if row.len() != n_cols {
                return Err("Inconsistent row lengths in data matrix".to_string());
            }
        }

        Ok((n_rows, n_cols))
    }

    /// Check if a value is considered missing
    #[inline]
    fn is_missing(value: f64, missing_value: f64) -> bool {
        value.is_nan() || value == missing_value
    }

    /// Check if a value is valid (not missing)
    #[inline]
    fn is_valid(value: f64, missing_value: f64) -> bool {
        !Self::is_missing(value, missing_value)
    }

    /// Find all positions of missing values in the dataset
    /// Parallelized across rows for improved performance on large datasets
    fn find_missing_positions(data: &[Vec<f64>], missing_value: f64) -> Vec<(usize, usize)> {
        data.par_iter()
            .enumerate()
            .flat_map(|(i, row)| {
                row.par_iter()
                    .enumerate()
                    .filter_map(|(j, &val)| {
                        if Self::is_missing(val, missing_value) {
                            Some((i, j))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Calculate column statistics (mean/std) for valid values
    /// Parallelized across columns for improved performance
    fn calculate_column_stats(data: &[Vec<f64>], missing_value: f64) -> (Vec<f64>, Vec<f64>) {
        let n_cols = data[0].len();
        let (means, stds): (Vec<f64>, Vec<f64>) = (0..n_cols)
            .into_par_iter()
            .map(|j| {
                let values: Vec<f64> = data.iter()
                    .filter_map(|row| {
                        let val = row[j];
                        if Self::is_valid(val, missing_value) {
                            Some(val)
                        } else {
                            None
                        }
                    })
                    .collect();

                if values.is_empty() {
                    (0.0, 1.0) // Default values
                } else {
                    let mean = values.nan_safe_mean();
                    let std = values.nan_safe_std_dev();
                    (mean, if std == 0.0 { 1.0 } else { std }) // Avoid division by zero
                }
            })
            .unzip();

        (means, stds)
    }

    /// Calculate Euclidean distance between two rows with standardization
    /// Returns (distance, valid_features_count) or None if no valid features
    fn calculate_standardized_distance(
        row1: &[f64],
        row2: &[f64],
        exclude_col: Option<usize>,
        _means: &[f64],
        stds: &[f64],
        missing_value: f64,
    ) -> Option<f64> {
        // Create filtered vectors excluding the missing column and invalid values
        let mut valid_indices = Vec::new();
        for j in 0..row1.len() {
            if Some(j) == exclude_col {
                continue;
            }
            let val1 = row1[j];
            let val2 = row2[j];
            if Self::is_valid(val1, missing_value) && Self::is_valid(val2, missing_value) {
                valid_indices.push(j);
            }
        }

        if valid_indices.is_empty() {
            return None;
        }

        // Extract valid values and corresponding weights (standard deviations)
        let mut vec1 = Vec::new();
        let mut vec2 = Vec::new();
        let mut weights = Vec::new();

        for &j in &valid_indices {
            vec1.push(row1[j]);
            vec2.push(row2[j]);
            weights.push(stds[j]);
        }

        // Use centralized standardized Euclidean distance
        Some(Distance::standardized_euclidean(&vec1, &vec2, Some(&weights)))
    }

    /// Apply imputation result to dataset
    /// Parallelized application of imputation values for improved performance
    fn apply_imputation_result(
        data: &[Vec<f64>],
        imputed_values: &[(usize, usize, f64)],
        missing_value: f64,
    ) -> (Vec<Vec<f64>>, usize) {
        let mut imputed_data = data.to_vec();

        // Count valid imputations in parallel
        let imputed_count = imputed_values.par_iter()
            .filter(|&&(row_idx, col_idx, _)| Self::is_missing(data[row_idx][col_idx], missing_value))
            .count();

        // Apply imputations sequentially (since we're modifying the same vector)
        for &(row_idx, col_idx, value) in imputed_values {
            if Self::is_missing(data[row_idx][col_idx], missing_value) {
                imputed_data[row_idx][col_idx] = value;
            }
        }

        (imputed_data, imputed_count)
    }
    /// Perform KNN imputation on multivariate data
    ///
    /// # Arguments
    /// * `data` - Matrix of data (rows = observations, columns = variables)
    /// * `k` - Number of nearest neighbors to use
    /// * `missing_value` - Value representing missing data (usually f64::NAN)
    ///
    /// Uses hierarchical parallelization: KD-tree construction, distance calculations,
    /// and neighbor searches are all parallelized where appropriate.
    pub fn knn_impute(data: &[Vec<f64>], k: usize, missing_value: f64) -> Result<ImputationResult, String> {
        let (n_rows, _n_cols) = Self::validate_data(data)?;

        if k == 0 || k >= n_rows {
            return Err("Invalid k value for KNN imputation".to_string());
        }

        // Calculate global statistics for standardization
        let (global_means, global_stds) = Self::calculate_column_stats(data, missing_value);

        // Find all missing positions
        let missing_positions = Self::find_missing_positions(data, missing_value);

        if missing_positions.is_empty() {
            let quality_metrics = Self::calculate_imputation_quality(data, data, missing_value)?;
            return Ok(ImputationResult {
                imputed_data: data.to_vec(),
                method: "knn_k0".to_string(),
                imputed_count: 0,
                quality_metrics,
            });
        }

        // Impute each missing value
        let mut imputed_values = Vec::new();
        for &(row_idx, col_idx) in &missing_positions {
            let neighbors = Self::find_knn_neighbors(data, row_idx, col_idx, k, missing_value, &global_means, &global_stds)?;

            if neighbors.is_empty() {
                return Err(format!("No valid neighbors found for imputation at ({}, {})", row_idx, col_idx));
            }

            // Impute using weighted average of neighbors
            let imputed_value = Self::weighted_average_impute(&neighbors, col_idx, data, missing_value);
            imputed_values.push((row_idx, col_idx, imputed_value));
        }

        // Apply imputation results
        let (imputed_data, imputed_count) = Self::apply_imputation_result(data, &imputed_values, missing_value);

        // Calculate quality metrics
        let quality_metrics = Self::calculate_imputation_quality(data, &imputed_data, missing_value)?;

        Ok(ImputationResult {
            imputed_data,
            method: format!("knn_k{}", k),
            imputed_count,
            quality_metrics,
        })
    }

    /// Find k nearest neighbors for a given missing value position
    /// Uses approximate nearest neighbor search for large datasets to avoid O(N²) complexity
    fn find_knn_neighbors(
        data: &[Vec<f64>],
        target_row: usize,
        target_col: usize,
        k: usize,
        missing_value: f64,
        means: &[f64],
        stds: &[f64],
    ) -> Result<Vec<(usize, f64)>, String> {
        let n_rows = data.len();

        // For large datasets, use approximate search to avoid O(N²) complexity
        if n_rows > 1000 {
            // Try KD-tree first (best accuracy), then sort-based heuristic, then sampling
            if n_rows > 5000 && data[0].len() - 1 <= 3 {
                // For very large datasets with low dimensions, try KD-tree approach
                match Self::kd_tree_knn_neighbors(data, target_row, target_col, k, missing_value, means, stds) {
                    Ok(result) if result.len() >= k => return Ok(result),
                    _ => {}
                }
            }

            // Try sort-based heuristic (zero dependencies)
            match Self::sort_based_knn_neighbors(data, target_row, target_col, k, missing_value, means, stds) {
                Ok(result) if result.len() >= k => return Ok(result),
                _ => return Self::approximate_knn_neighbors(data, target_row, target_col, k, missing_value, means, stds),
            }
        }

        // Exact search for smaller datasets
        let mut distances: Vec<(usize, f64)> = (0..n_rows)
            .into_par_iter()
            .filter_map(|i| {
                if i == target_row {
                    return None;
                }

                Self::calculate_standardized_distance(
                    &data[target_row],
                    &data[i],
                    Some(target_col),
                    means,
                    stds,
                    missing_value,
                ).map(|distance| (i, distance))
            })
            .collect();

        // Sort by distance and take k nearest
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        distances.truncate(k);

        Ok(distances)
    }

    /// Approximate KNN search for large datasets using sampling
    fn approximate_knn_neighbors(
        data: &[Vec<f64>],
        target_row: usize,
        target_col: usize,
        k: usize,
        missing_value: f64,
        means: &[f64],
        stds: &[f64],
    ) -> Result<Vec<(usize, f64)>, String> {
        let n_rows = data.len();

        // Sample a subset of rows to check (sqrt(N) should give good approximation)
        let sample_size = (n_rows as f64).sqrt().max(100.0).min(n_rows as f64) as usize;
        let mut rng = rand_pcg::Pcg64::seed_from_u64(42);

        use rand::seq::SliceRandom;
        let mut candidate_indices: Vec<usize> = (0..n_rows).filter(|&i| i != target_row).collect();
        candidate_indices.shuffle(&mut rng);
        candidate_indices.truncate(sample_size);

        let mut distances = Vec::new();

        // Calculate distances only for sampled candidates
        for &i in &candidate_indices {
            if let Some(distance) = Self::calculate_standardized_distance(
                &data[target_row],
                &data[i],
                Some(target_col),
                means,
                stds,
                missing_value,
            ) {
                distances.push((i, distance));
            }
        }

        // If we don't have enough candidates, fall back to exact search with a smaller subset
        if distances.len() < k * 2 {
            // Try exact search but limit to first 2000 rows for performance
            let limit = 2000.min(n_rows);
            distances.clear();

            for i in 0..limit {
                if i == target_row {
                    continue;
                }

                if let Some(distance) = Self::calculate_standardized_distance(
                    &data[target_row],
                    &data[i],
                    Some(target_col),
                    means,
                    stds,
                    missing_value,
                ) {
                    distances.push((i, distance));
                }
            }
        }

        // Sort by distance and take k nearest
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        distances.truncate(k);

        Ok(distances)
    }

    /// State-of-the-art KNN using KD-tree for exact nearest neighbor search
    /// Complexity: O(N log N) construction, O(log N) query
    /// Uses kiddo crate with parallel construction via rayon for large datasets
    fn kd_tree_knn_neighbors(
        data: &[Vec<f64>],
        target_row: usize,
        target_col: usize,
        k: usize,
        missing_value: f64,
        means: &[f64],
        stds: &[f64],
    ) -> Result<Vec<(usize, f64)>, String> {
        let n_rows = data.len();
        let n_cols = data[0].len();

        // Extract target point (excluding missing column)
        let mut target_point = Vec::new();
        let mut valid_dims = Vec::new();

        for j in 0..n_cols {
            if j == target_col {
                continue;
            }
            let val = data[target_row][j];
            if !val.is_nan() && val != missing_value {
                target_point.push(val);
                valid_dims.push(j);
            }
        }

        if target_point.is_empty() {
            return Err("No valid features for target point".to_string());
        }

        // Extract means and stds for valid dimensions
        let mut local_means = vec![];
        let mut local_stds = vec![];
        for &dim in &valid_dims {
            local_means.push(means[dim]);
            local_stds.push(stds[dim]);
        }

        // Standardize target_point
        for i in 0..target_point.len() {
            target_point[i] = (target_point[i] - local_means[i]) / local_stds[i];
        }

        // Pad target_point to 3D if necessary
        while target_point.len() < 3 {
            target_point.push(0.0);
        }

        // Build KD-tree from complete data points
        let mut kdtree = KdTree::new();

        // Collect valid points in parallel
        let valid_points: Vec<(usize, [f64; 3])> = (0..n_rows)
            .into_par_iter()
            .filter_map(|i| {
                if i == target_row {
                    return None;
                }

                let mut point = Vec::new();
                let mut is_complete = true;

                // Only include points with complete data for the valid dimensions
                for &dim in &valid_dims {
                    let val = data[i][dim];
                    if val.is_nan() || val == missing_value {
                        is_complete = false;
                        break;
                    }
                    point.push(val);
                }

                if is_complete && point.len() <= 3 {
                    // Standardize point
                    for j in 0..point.len() {
                        point[j] = (point[j] - local_means[j]) / local_stds[j];
                    }
                    // Pad to 3D if necessary
                    while point.len() < 3 {
                        point.push(0.0);
                    }

                    // Convert to fixed-size array
                    let arr: [f64; 3] = [point[0], point[1], point[2]];
                    Some((i, arr))
                } else {
                    None
                }
            })
            .collect();

        // Add points to KD-tree (this operation itself may not be easily parallelized)
        for (i, arr) in valid_points {
            kdtree.add(&arr, i as u64);
        }

        if kdtree.size() < k as u64 {
            return Err(format!("Not enough complete data points for KNN (need {}, have {})", k, kdtree.size()));
        }

        // Pad target point to 3D
        while target_point.len() < 3 {
            target_point.push(0.0);
        }
        let target_arr: [f64; 3] = [target_point[0], target_point[1], target_point[2]];

        // Query k nearest neighbors
        let neighbors = kdtree.nearest_n::<SquaredEuclidean>(&target_arr, k);

        // Convert results to expected format (row_index, distance)
        let mut result = Vec::new();
        for neighbor in neighbors {
            let distance = neighbor.distance.sqrt(); // Convert squared distance back
            result.push((neighbor.item as usize, distance));
        }

        Ok(result)
    }

    /// Zero-dependency KNN heuristic: sort by high-variance dimension and search local windows
    /// This preserves locality better than random sampling while maintaining O(N log N) complexity
    /// Uses parallel distance calculations with rayon for improved performance
    fn sort_based_knn_neighbors(
        data: &[Vec<f64>],
        target_row: usize,
        target_col: usize,
        k: usize,
        missing_value: f64,
        means: &[f64],
        stds: &[f64],
    ) -> Result<Vec<(usize, f64)>, String> {
        let n_rows = data.len();
        let n_cols = data[0].len();

        // Step 1: Calculate variance for each column (excluding target column)
        let mut column_variances = Vec::new();
        for j in 0..n_cols {
            if j == target_col {
                column_variances.push(0.0); // Skip target column
                continue;
            }

            let mut values = Vec::new();
            for row in data.iter().take(n_rows) {
                let val = row[j];
                if !val.is_nan() && val != missing_value {
                    values.push(val);
                }
            }

            if values.len() > 1 {
                let variance = values.variance();
                column_variances.push(variance);
            } else {
                column_variances.push(0.0);
            }
        }

        // Step 2: Find column with highest variance
        let mut max_var = 0.0;
        let mut sort_column = 0;
        for (j, &var) in column_variances.iter().enumerate() {
            if j != target_col && var > max_var {
                max_var = var;
                sort_column = j;
            }
        }

        if max_var == 0.0 {
            // No variance in any column, fall back to sampling
            return Self::approximate_knn_neighbors(data, target_row, target_col, k, missing_value, means, stds);
        }

        // Step 3: Sort dataset by the high-variance column
        let mut sorted_indices: Vec<usize> = (0..n_rows).collect();
        sorted_indices.sort_by(|&a, &b| {
            let val_a = data[a][sort_column];
            let val_b = data[b][sort_column];
            val_a.partial_cmp(&val_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Step 4: Find position of target row in sorted order
        let target_pos = sorted_indices.iter().position(|&x| x == target_row)
            .unwrap_or(n_rows / 2); // Fallback if not found

        // Step 5: Search in local window [target_pos - window_size, target_pos + window_size]
        let window_size = ((n_rows as f64).sqrt() as usize).max(k * 2).min(n_rows / 4);
        let start = target_pos.saturating_sub(window_size);
        let end = (target_pos + window_size).min(n_rows - 1);

        let mut distances = Vec::new();

        // Calculate distances for rows in the window (parallelized)
        let window_distances: Vec<(usize, f64)> = sorted_indices[start..=end]
            .par_iter()
            .filter_map(|&sorted_idx| {
                if sorted_idx == target_row {
                    return None;
                }

                Self::calculate_standardized_distance(
                    &data[target_row],
                    &data[sorted_idx],
                    Some(target_col),
                    means,
                    stds,
                    missing_value,
                ).map(|distance| (sorted_idx, distance))
            })
            .collect();

        distances.extend(window_distances);

        // Sort by distance and take k nearest
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        distances.truncate(k);

        Ok(distances)
    }

    /// Compute weighted average imputation from neighbors
    fn weighted_average_impute(neighbors: &[(usize, f64)], target_col: usize, data: &[Vec<f64>], missing_value: f64) -> f64 {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for &(row_idx, distance) in neighbors {
            // Use inverse distance weighting
            let weight = if distance > 0.0 { 1.0 / distance } else { 1.0 };
            let value = data[row_idx][target_col];
            if !value.is_nan() && value != missing_value && value.is_finite() {
                weighted_sum += weight * value;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Perform mean imputation for missing values
    /// Parallelized across columns for statistical calculations and imputation operations.
    pub fn mean_impute(data: &[Vec<f64>], missing_value: f64) -> Result<ImputationResult, String> {
        let (_n_rows, n_cols) = Self::validate_data(data)?;

        // Calculate column means (ignoring missing values) - parallelized across columns
        let column_means: Vec<f64> = (0..n_cols)
            .into_par_iter()
            .map(|j| {
                let values: Vec<f64> = data.iter()
                    .filter_map(|row| {
                        let val = row[j];
                        if Self::is_valid(val, missing_value) {
                            Some(val)
                        } else {
                            None
                        }
                    })
                    .collect();

                if values.is_empty() {
                    0.0 // Fallback for columns with all missing values
                } else {
                    values.nan_safe_mean()
                }
            })
            .collect();

        // Find missing positions and impute
        let missing_positions = Self::find_missing_positions(data, missing_value);
        let imputed_values: Vec<(usize, usize, f64)> = missing_positions.into_iter()
            .map(|(i, j)| (i, j, column_means[j]))
            .collect();

        // Apply imputation results
        let (imputed_data, imputed_count) = Self::apply_imputation_result(data, &imputed_values, missing_value);

        let quality_metrics = Self::calculate_imputation_quality(data, &imputed_data, missing_value)?;

        Ok(ImputationResult {
            imputed_data,
            method: "mean".to_string(),
            imputed_count,
            quality_metrics,
        })
    }

    /// Perform median imputation for missing values
    /// Parallelized across columns for statistical calculations and imputation operations.
    pub fn median_impute(data: &[Vec<f64>], missing_value: f64) -> Result<ImputationResult, String> {
        let (_n_rows, n_cols) = Self::validate_data(data)?;

        // Calculate column medians (ignoring missing values) - parallelized across columns
        let column_medians: Vec<f64> = (0..n_cols)
            .into_par_iter()
            .map(|j| {
                let column_values: Vec<f64> = data.iter()
                    .filter_map(|row| {
                        let val = row[j];
                        if Self::is_valid(val, missing_value) {
                            Some(val)
                        } else {
                            None
                        }
                    })
                    .collect();

                if column_values.is_empty() {
                    0.0 // Fallback
                } else {
                    Quantiles::nan_safe_median(&column_values)
                }
            })
            .collect();

        // Find missing positions and impute
        let missing_positions = Self::find_missing_positions(data, missing_value);
        let imputed_values: Vec<(usize, usize, f64)> = missing_positions.into_iter()
            .map(|(i, j)| (i, j, column_medians[j]))
            .collect();

        // Apply imputation results
        let (imputed_data, imputed_count) = Self::apply_imputation_result(data, &imputed_values, missing_value);

        let quality_metrics = Self::calculate_imputation_quality(data, &imputed_data, missing_value)?;

        Ok(ImputationResult {
            imputed_data,
            method: "median".to_string(),
            imputed_count,
            quality_metrics,
        })
    }

    /// Perform iterative regression imputation (MICE-like)
    /// Uses multiple imputation by chained equations approach
    #[allow(clippy::needless_range_loop)]
    pub fn regression_impute(data: &[Vec<f64>], missing_value: f64) -> Result<ImputationResult, String> {
        let (_n_rows, n_cols) = Self::validate_data(data)?;

        let mut imputed_data = data.to_vec();
        let missing_positions = Self::find_missing_positions(data, missing_value);
        let imputed_count = missing_positions.len();

        if imputed_count == 0 {
            let quality_metrics = Self::calculate_imputation_quality(data, &imputed_data, missing_value)?;
            return Ok(ImputationResult {
                imputed_data,
                method: "regression".to_string(),
                imputed_count: 0,
                quality_metrics,
            });
        }

        // Create missing mask
        let mut missing_mask = vec![vec![false; n_cols]; data.len()];
        for &(i, j) in &missing_positions {
            missing_mask[i][j] = true;
        }

        // Initialize with mean imputation
        let column_means: Vec<f64> = (0..n_cols)
            .into_par_iter()
            .map(|j| {
                let values: Vec<f64> = data.iter()
                    .filter_map(|row| {
                        let val = row[j];
                        if Self::is_valid(val, missing_value) {
                            Some(val)
                        } else {
                            None
                        }
                    })
                    .collect();
                if values.is_empty() {
                    0.0
                } else {
                    values.mean()
                }
            })
            .collect();

        for &(i, j) in &missing_positions {
            imputed_data[i][j] = column_means[j];
        }

        // Iterative imputation (MICE-lite)
        let cycles = 10;
        let lambda = 1e-5; // Ridge regularization

        for _cycle in 0..cycles {
            for target_col in 0..column_means.len() {
                // Prepare matrices using all rows (no dropping)
                let (x_matrix, y_vector, missing_indices) = Self::prepare_matrices(&imputed_data, target_col, &missing_mask);

                if x_matrix.nrows() == 0 || missing_indices.is_empty() {
                    continue;
                }

                // Ridge regression
                match LinearRegression::solve_ridge_regression(&x_matrix, &y_vector, lambda) {
                    Ok((coeffs, residual_std)) => {
                        // Update only the originally missing values
                        Self::update_column(&mut imputed_data, target_col, &coeffs, &missing_indices, residual_std);
                    }
                    Err(_) => {
                        // Fallback to mean for this column
                        for &idx in &missing_indices {
                            imputed_data[idx][target_col] = column_means[target_col];
                        }
                    }
                }
            }
        }

        let quality_metrics = Self::calculate_imputation_quality(data, &imputed_data, missing_value)?;

        Ok(ImputationResult {
            imputed_data,
            method: "iterative_regression".to_string(),
            imputed_count,
            quality_metrics,
        })
    }

    /// Prepare design matrix X and response vector y for regression
    /// Parallelized matrix preparation for improved performance
    fn prepare_matrices(filled_data: &[Vec<f64>], target_col: usize, missing_mask: &[Vec<bool>]) -> (Array2<f64>, Array1<f64>, Vec<usize>) {
        let n_rows = filled_data.len();
        let n_cols = filled_data[0].len();
        let n_features = n_cols - 1;

        // Parallel collection of training data and missing indices
        let (x_data, y_data, missing_indices): (Vec<f64>, Vec<f64>, Vec<usize>) = (0..n_rows)
            .into_par_iter()
            .fold(
                || (Vec::new(), Vec::new(), Vec::new()),
                |(mut x_acc, mut y_acc, mut missing_acc), i| {
                    if missing_mask[i][target_col] {
                        missing_acc.push(i);
                    } else {
                        // Use this row for training
                        let mut row = vec![1.0]; // intercept
                        for j in 0..n_cols {
                            if j != target_col {
                                row.push(filled_data[i][j]);
                            }
                        }
                        x_acc.extend(row);
                        y_acc.push(filled_data[i][target_col]);
                    }
                    (x_acc, y_acc, missing_acc)
                },
            )
            .reduce(
                || (Vec::new(), Vec::new(), Vec::new()),
                |(mut x_acc, mut y_acc, mut missing_acc), (x_part, y_part, missing_part)| {
                    x_acc.extend(x_part);
                    y_acc.extend(y_part);
                    missing_acc.extend(missing_part);
                    (x_acc, y_acc, missing_acc)
                },
            );

        let n_samples = y_data.len();
        if n_samples == 0 {
            return (Array2::zeros((0, n_features + 1)), Array1::zeros(0), missing_indices);
        }

        let x_matrix = Array2::from_shape_vec((n_samples, n_features + 1), x_data).unwrap();
        let y_vector = Array1::from_vec(y_data);

        (x_matrix, y_vector, missing_indices)
    }

    /// Update the target column with predicted values for missing indices
    /// Parallelized column updates for improved performance
    fn update_column(data: &mut [Vec<f64>], target_col: usize, coeffs: &Array1<f64>, missing_indices: &[usize], residual_std: f64) {
        let normal = if residual_std > 0.0 {
            Some(Normal::new(0.0, residual_std).unwrap())
        } else {
            None
        };

        // Parallel computation of predictions
        let updates: Vec<(usize, f64)> = missing_indices.par_iter().map(|&idx| {
            let mut predictor_row = vec![1.0]; // intercept
            for j in 0..data[idx].len() {
                if j != target_col {
                    predictor_row.push(data[idx][j]);
                }
            }
            let pred: f64 = predictor_row.iter().zip(coeffs.iter()).map(|(x, c)| x * c).sum();

            let noise = if let Some(ref dist) = normal {
                let mut rng = rand::rng();
                dist.sample(&mut rng)
            } else {
                0.0
            };

            (idx, pred + noise)
        }).collect();

        // Sequential application of updates
        for (idx, value) in updates {
            data[idx][target_col] = value;
        }
    }

    /// Calculate quality metrics for imputation
    /// Parallelized across column pairs for correlation calculations and across columns for variance calculations.
    pub fn calculate_imputation_quality(
        original: &[Vec<f64>],
        imputed: &[Vec<f64>],
        missing_value: f64,
    ) -> Result<ImputationQuality, String> {
        let n_rows = original.len();
        let n_cols = original[0].len();

        // Calculate correlation preservation - parallelized across column pairs
        let correlation_pairs: Vec<(f64, f64)> = (0..n_cols)
            .into_par_iter()
            .flat_map(|i| {
                let mut local_pairs = Vec::new();
                for j in (i + 1)..n_cols {
                    // Original correlation (using only complete pairs)
                    let mut orig_pairs = Vec::new();
                    for row in original.iter().take(n_rows) {
                        let val1 = row[i];
                        let val2 = row[j];
                        if !val1.is_nan() && val1 != missing_value &&
                           !val2.is_nan() && val2 != missing_value {
                            orig_pairs.push((val1, val2));
                        }
                    }

                    let orig_corr = if orig_pairs.len() > 1 {
                        let x_vals: Vec<f64> = orig_pairs.iter().map(|(x, _)| *x).collect();
                        let y_vals: Vec<f64> = orig_pairs.iter().map(|(_, y)| *y).collect();
                        CorrelationMethods::pearson_correlation(&x_vals, &y_vals, None, None).map(|(r, _)| r).unwrap_or(0.0)
                    } else {
                        0.0
                    };

                    // Imputed correlation
                    let mut imp_pairs = Vec::new();
                    for row in imputed.iter().take(n_rows) {
                        imp_pairs.push((row[i], row[j]));
                    }
                    let imp_corr = {
                        let x_vals: Vec<f64> = imp_pairs.iter().map(|(x, _)| *x).collect();
                        let y_vals: Vec<f64> = imp_pairs.iter().map(|(_, y)| *y).collect();
                        CorrelationMethods::pearson_correlation(&x_vals, &y_vals, None, None).map(|(r, _)| r).unwrap_or(0.0)
                    };

                    local_pairs.push((orig_corr, imp_corr));
                }
                local_pairs
            })
            .collect();

        let (original_correlations, imputed_correlations): (Vec<f64>, Vec<f64>) =
            correlation_pairs.into_iter().unzip();

        let correlation_preservation = if !original_correlations.is_empty() {
            let avg_orig = original_correlations.iter().sum::<f64>() / original_correlations.len() as f64;
            let avg_imp = imputed_correlations.iter().sum::<f64>() / imputed_correlations.len() as f64;
            1.0 - (avg_orig - avg_imp).abs()
        } else {
            1.0
        };

        // Calculate variance preservation - parallelized across columns
        let variance_results: Vec<f64> = (0..n_cols)
            .into_par_iter()
            .map(|j| {
                let orig_values: Vec<f64> = original.iter()
                    .filter_map(|row| {
                        let val = row[j];
                        if !val.is_nan() && val != missing_value {
                            Some(val)
                        } else {
                            None
                        }
                    })
                    .collect();

                if orig_values.is_empty() {
                    return 0.0;
                }

                let orig_var = orig_values.variance();
                let imp_var = imputed.iter().map(|row| row[j]).collect::<Vec<f64>>().variance();

                if orig_var > 0.0 {
                    1.0 - (orig_var - imp_var).abs() / orig_var
                } else {
                    0.0
                }
            })
            .collect();

        let variance_preservation = if !variance_results.is_empty() {
            variance_results.iter().sum::<f64>() / variance_results.len() as f64
        } else {
            0.0
        };

        Ok(ImputationQuality {
            mae: None,  // Would need validation data
            rmse: None, // Would need validation data
            correlation_preservation,
            variance_preservation,
        })
    }



    /// Automatically choose and apply the best imputation method
    /// Runs all imputation methods in parallel and selects the best based on quality metrics.
    pub fn auto_impute(data: &[Vec<f64>], missing_value: f64) -> Result<ImputationResult, String> {
        let (_n_rows, _n_cols) = Self::validate_data(data)?;

        // Count missing values
        let total_missing = Self::find_missing_positions(data, missing_value).len();

        if total_missing == 0 {
            return Ok(ImputationResult {
                imputed_data: data.to_vec(),
                method: "none".to_string(),
                imputed_count: 0,
                quality_metrics: ImputationQuality {
                    mae: None,
                    rmse: None,
                    correlation_preservation: 1.0,
                    variance_preservation: 1.0,
                },
            });
        }

        let complete_rows: Vec<usize> = (0..data.len()).filter(|&i| data[i].iter().all(|&x| Self::is_valid(x, missing_value))).collect();

        let k = if data.len() < 50 { 3 } else { 5 };

        if complete_rows.len() < 20 {
            // Fall back to old method
            let results: Vec<Result<ImputationResult, String>> = vec![
                Self::mean_impute(data, missing_value),
                Self::median_impute(data, missing_value),
                Self::knn_impute(data, k, missing_value),
                Self::regression_impute(data, missing_value),
            ].into_par_iter().collect();

            let mut best_result = None;
            let mut best_score = f64::NEG_INFINITY;

            for imputation_result in results.into_iter().flatten() {
                let score = imputation_result.quality_metrics.correlation_preservation +
                           imputation_result.quality_metrics.variance_preservation;
                if score > best_score {
                    best_score = score;
                    best_result = Some(imputation_result);
                }
            }

            return best_result.ok_or_else(|| "No imputation method succeeded".to_string());
        }

        // Use cross-validation
        let methods: Vec<(&str, Box<ImputeFn>)> = vec![
            ("mean", Box::new(Self::mean_impute)),
            ("median", Box::new(Self::median_impute)),
            ("knn", Box::new(move |d, mv| Self::knn_impute(d, k, mv))),
            ("regression", Box::new(Self::regression_impute)),
        ];

        let mut method_scores: Vec<(&str, f64)> = methods.into_iter().map(|(name, method_fn)| {
            let rmse = Self::compute_cv_rmse(data, missing_value, &complete_rows, &method_fn);
            (name, rmse)
        }).collect();

        method_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let best_name = method_scores[0].0;

        match best_name {
            "mean" => Self::mean_impute(data, missing_value),
            "median" => Self::median_impute(data, missing_value),
            "knn" => Self::knn_impute(data, k, missing_value),
            "regression" => Self::regression_impute(data, missing_value),
            _ => Self::mean_impute(data, missing_value),
        }
    }

    fn compute_cv_rmse(data: &[Vec<f64>], missing_value: f64, complete_rows: &[usize], method_fn: &ImputeFn) -> f64 {
        let k_folds = 5;
        let fold_size = complete_rows.len() / k_folds;
        let mut total_error = 0.0;
        let mut total_count = 0;

        for _fold in 0..k_folds {
            let mut test_rows = complete_rows.to_vec();
            let mut rng = rand::rng();
            test_rows.shuffle(&mut rng);
            let test = &test_rows[0..fold_size.min(test_rows.len())];

            let mut masked_data = data.to_vec();
            let mut masked_positions = vec![];

            for &i in test {
                let cols: Vec<usize> = (0..data[i].len()).collect();
                let mut shuffled_cols = cols.clone();
                shuffled_cols.shuffle(&mut rng);
                let mask_col = shuffled_cols[0];
                masked_data[i][mask_col] = missing_value;
                masked_positions.push((i, mask_col, data[i][mask_col]));
            }

            if let Ok(imputed) = method_fn(&masked_data, missing_value) {
                for (i, j, original) in masked_positions {
                    let imp_val = imputed.imputed_data[i][j];
                    total_error += (imp_val - original).powi(2);
                    total_count += 1;
                }
            }
        }

        if total_count > 0 {
            (total_error / total_count as f64).sqrt()
        } else {
            f64::INFINITY
        }
    }
}