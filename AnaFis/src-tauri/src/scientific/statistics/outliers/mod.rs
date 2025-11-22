//! Outlier Detection Module
//!
//! This module provides comprehensive outlier detection methods:
//! - Z-score based detection
//! - IQR (Interquartile Range) based detection
//! - Modified Z-score for robustness
//! - Local Outlier Factor (LOF)
//! - Isolation Forest
//! - Support for uncertainty-aware detection

pub mod types;

use crate::scientific::statistics::descriptive::{Quantiles, StatisticalMoments};
use crate::scientific::statistics::distributions::distribution_functions;
use extended_isolation_forest::{Forest, ForestOptions};

// Re-export types for public API
pub use types::{OutlierDetectionConfig, OutlierAnalysisResult, OutlierInfo, AnalysisOptions};
use rayon::prelude::*;

/// Outlier detection engine
pub struct OutlierDetectionEngine;

impl OutlierDetectionEngine {
    /// Detect outliers using multiple methods
    pub fn detect_outliers(
        data: &[f64],
        options: &AnalysisOptions,
    ) -> Result<OutlierAnalysisResult, String> {
        Self::detect_outliers_with_uncertainties(data, None, None, options)
    }

    /// Detect outliers using multiple methods with uncertainty consideration
    pub fn detect_outliers_with_uncertainties(
        data: &[f64],
        uncertainties: Option<&[f64]>,
        confidence_levels: Option<&[f64]>,
        options: &AnalysisOptions,
    ) -> Result<OutlierAnalysisResult, String> {
        let method_configs = vec![
            ("Z-score", options.z_score_threshold.unwrap_or(3.0)),
            ("IQR", options.iqr_multiplier.unwrap_or(1.5)),
            ("Modified Z-score", options.modified_z_threshold.unwrap_or(3.5)),
        ];

        let methods: Vec<(String, Vec<OutlierInfo>)> = method_configs
            .into_par_iter()
            .filter_map(|(name, threshold)| {
                let result = match name {
                    "Z-score" => Self::z_score_outliers_with_uncertainties(
                        data, uncertainties, confidence_levels, threshold),
                    "IQR" => Self::iqr_outliers_with_uncertainties(
                        data, uncertainties, confidence_levels, threshold),
                    "Modified Z-score" => Self::modified_z_score_outliers_with_uncertainties(
                        data, uncertainties, confidence_levels, threshold),
                    _ => Ok(Vec::new()),
                };
                result.ok().map(|outliers| (name.to_string(), outliers))
            })
            .collect();

        let mut all_methods = methods;

        // Add LOF if enough data
        if data.len() >= 10 {
            let lof_k = options.lof_k.unwrap_or(5);
            let lof_threshold = options.lof_threshold.unwrap_or(1.5);
            if let Ok(lof_outliers) = Self::lof_outliers(data, lof_k, lof_threshold) {
                all_methods.push(("Local Outlier Factor".to_string(), lof_outliers));
            }
        }

        // Add Isolation Forest
        let contamination = options.isolation_forest_contamination.unwrap_or(0.1);
        if let Ok(if_outliers) = Self::isolation_forest_outliers(data, contamination) {
            all_methods.push(("Isolation Forest".to_string(), if_outliers));
        }

        // Combine results
        let mut all_outlier_indices = std::collections::HashSet::new();
        for (_, outliers) in &all_methods {
            for outlier in outliers {
                all_outlier_indices.insert(outlier.index);
            }
        }

        let outlier_indices: Vec<usize> = all_outlier_indices.into_iter().collect();
        let outlier_percentage = outlier_indices.len() as f64 / data.len() as f64 * 100.0;

        Ok(OutlierAnalysisResult {
            methods: all_methods,
            combined_outliers: outlier_indices,
            outlier_percentage,
        })
    }

    /// Z-score outlier detection with uncertainty consideration
    fn z_score_outliers_with_uncertainties(
        data: &[f64],
        uncertainties: Option<&[f64]>,
        confidence_levels: Option<&[f64]>,
        threshold: f64,
    ) -> Result<Vec<OutlierInfo>, String> {
        if data.len() < 2 {
            return Ok(Vec::new());
        }

        // Use centralized standardization
        let (standardized_data, mean, std_dev) = Self::standardize(data);

        let mut outliers = Vec::new();
        for (i, &z_score) in standardized_data.iter().enumerate() {
            let is_outlier = if let (Some(uncs), Some(confs)) = (uncertainties, confidence_levels) {
                if i < uncs.len() && i < confs.len() {
                    let uncertainty = uncs[i];
                    let confidence = confs[i];
                    let z_conf = Self::confidence_to_z_score(confidence);
                    let lower_bound = data[i] - uncertainty * z_conf;
                    let upper_bound = data[i] + uncertainty * z_conf;
                    let acceptable_lower = mean - threshold * std_dev;
                    let acceptable_upper = mean + threshold * std_dev;
                    upper_bound < acceptable_lower || lower_bound > acceptable_upper
                } else {
                    z_score.abs() > threshold
                }
            } else {
                z_score.abs() > threshold
            };

            if is_outlier {
                outliers.push(OutlierInfo {
                    index: i,
                    value: data[i],
                    z_score: Some(z_score),
                    iqr_distance: None,
                    lof_score: None,
                    isolation_score: None,
                });
            }
        }
        Ok(outliers)
    }

    /// IQR-based outlier detection with uncertainty consideration
    fn iqr_outliers_with_uncertainties(
        data: &[f64],
        uncertainties: Option<&[f64]>,
        confidence_levels: Option<&[f64]>,
        multiplier: f64,
    ) -> Result<Vec<OutlierInfo>, String> {
        if data.len() < 4 {
            return Ok(Vec::new());
        }

        let mut sorted_data = data.to_vec();
        let n = sorted_data.len();
        let q1_idx = (n as f64 * 0.25) as usize;
        let q3_idx = (n as f64 * 0.75) as usize;

        // Use select_nth_unstable to find quartiles without full sort
        // We find Q1 first
        sorted_data.select_nth_unstable_by(q1_idx, |a, b| a.total_cmp(b));
        let q1 = sorted_data[q1_idx];

        // Then find Q3 (since q3_idx > q1_idx, it works on the partially sorted array)
        sorted_data.select_nth_unstable_by(q3_idx, |a, b| a.total_cmp(b));
        let q3 = sorted_data[q3_idx];
        
        let iqr = q3 - q1;

        let lower_bound = q1 - multiplier * iqr;
        let upper_bound = q3 + multiplier * iqr;

        let mut outliers = Vec::new();
        for (i, &x) in data.iter().enumerate() {
            let is_outlier = if let (Some(uncs), Some(confs)) = (uncertainties, confidence_levels) {
                if i < uncs.len() && i < confs.len() {
                    let uncertainty = uncs[i];
                    let confidence = confs[i];
                    let z_conf = Self::confidence_to_z_score(confidence);
                    let lower_bound_with_uncertainty = x - uncertainty * z_conf;
                    let upper_bound_with_uncertainty = x + uncertainty * z_conf;
                    upper_bound_with_uncertainty < lower_bound || lower_bound_with_uncertainty > upper_bound
                } else {
                    x < lower_bound || x > upper_bound
                }
            } else {
                x < lower_bound || x > upper_bound
            };

            if is_outlier {
                let iqr_distance = if iqr == 0.0 {
                    if x != q1 { f64::INFINITY } else { 0.0 }
                } else if x < lower_bound {
                    (q1 - x) / iqr
                } else {
                    (x - q3) / iqr
                };
                outliers.push(OutlierInfo {
                    index: i,
                    value: x,
                    z_score: None,
                    iqr_distance: Some(iqr_distance),
                    lof_score: None,
                    isolation_score: None,
                });
            }
        }
        Ok(outliers)
    }

    /// Modified Z-score outlier detection with uncertainty consideration
    fn modified_z_score_outliers_with_uncertainties(
        data: &[f64],
        uncertainties: Option<&[f64]>,
        confidence_levels: Option<&[f64]>,
        threshold: f64,
    ) -> Result<Vec<OutlierInfo>, String> {
        if data.len() < 2 {
            return Ok(Vec::new());
        }

        // Use centralized modified Z-score calculation
        let (modified_z_scores, median, mad) = Self::modified_z_scores(data);

        if mad == 0.0 {
            return Ok(Vec::new());
        }

        let mut outliers = Vec::new();
        for (i, &modified_z) in modified_z_scores.iter().enumerate() {
            let is_outlier = if let (Some(uncs), Some(confs)) = (uncertainties, confidence_levels) {
                if i < uncs.len() && i < confs.len() {
                    let uncertainty = uncs[i];
                    let confidence = confs[i];
                    let z_conf = Self::confidence_to_z_score(confidence);
                    let lower_bound_with_uncertainty = data[i] - uncertainty * z_conf;
                    let upper_bound_with_uncertainty = data[i] + uncertainty * z_conf;
                    let acceptable_lower = median - (threshold / 0.6745) * mad;
                    let acceptable_upper = median + (threshold / 0.6745) * mad;
                    upper_bound_with_uncertainty < acceptable_lower || lower_bound_with_uncertainty > acceptable_upper
                } else {
                    modified_z.abs() > threshold
                }
            } else {
                modified_z.abs() > threshold
            };

            if is_outlier {
                outliers.push(OutlierInfo {
                    index: i,
                    value: data[i],
                    z_score: Some(modified_z),
                    iqr_distance: None,
                    lof_score: None,
                    isolation_score: None,
                });
            }
        }
        Ok(outliers)
    }

    /// Local Outlier Factor (LOF) outlier detection
    fn lof_outliers(data: &[f64], k: usize, threshold: f64) -> Result<Vec<OutlierInfo>, String> {
        let n = data.len();
        if n <= k {
            return Err("Not enough data points for LOF analysis".to_string());
        }

        let mut lof_scores = vec![0.0; n];

        // Optimized 1D LOF using sorting
        // 1. Sort data while keeping indices
        let mut sorted_indices: Vec<usize> = (0..n).collect();
        sorted_indices.sort_by(|&a, &b| data[a].total_cmp(&data[b]));

        let mut all_k_distances = vec![0.0; n];
        let mut all_neighbors = vec![Vec::with_capacity(k); n];

        // 2. Find k-nearest neighbors for each point using the sorted array
        for i in 0..n {
            let original_idx = sorted_indices[i];
            let val = data[original_idx];
            
            // Search neighbors in sorted array
            // We look left and right from i
            let mut left = i.checked_sub(1);
            let mut right = if i + 1 < n { Some(i + 1) } else { None };
            let mut neighbors = Vec::with_capacity(k);
            
            while neighbors.len() < k {
                let left_dist = if let Some(l) = left { (val - data[sorted_indices[l]]).abs() } else { f64::INFINITY };
                let right_dist = if let Some(r) = right { (data[sorted_indices[r]] - val).abs() } else { f64::INFINITY };

                if left_dist <= right_dist {
                    if let Some(l) = left {
                        neighbors.push((sorted_indices[l], left_dist));
                        left = l.checked_sub(1);
                    }
                } else if let Some(r) = right {
                    neighbors.push((sorted_indices[r], right_dist));
                    right = if r + 1 < n { Some(r + 1) } else { None };
                }
            }
            
            // k-distance is the distance to the k-th neighbor
            let k_distance = neighbors.last().map(|&(_, d)| d).unwrap_or(0.0);
            all_k_distances[original_idx] = k_distance;
            all_neighbors[original_idx] = neighbors.into_iter().map(|(idx, _)| idx).collect();
        }

        // Calculate Local Reachability Density (LRD)
        let mut lrds = vec![0.0; n];
        for i in 0..n {
            let mut reach_dist_sum = 0.0;
            for &neighbor_idx in &all_neighbors[i] {
                let reach_dist = all_k_distances[neighbor_idx].max((data[i] - data[neighbor_idx]).abs());
                reach_dist_sum += reach_dist;
            }
            if reach_dist_sum > 0.0 {
                lrds[i] = k as f64 / reach_dist_sum;
            }
        }

        // Calculate LOF scores
        for i in 0..n {
            if lrds[i] > 0.0 {
                let mut lrd_ratio_sum = 0.0;
                for &neighbor_idx in &all_neighbors[i] {
                    lrd_ratio_sum += lrds[neighbor_idx];
                }
                lof_scores[i] = (lrd_ratio_sum / (k as f64)) / lrds[i];
            }
        }

        let mut outliers = Vec::new();
        for i in 0..n {
            if lof_scores[i] > threshold {
                outliers.push(OutlierInfo {
                    index: i,
                    value: data[i],
                    z_score: None,
                    iqr_distance: None,
                    lof_score: Some(lof_scores[i]),
                    isolation_score: None,
                });
            }
        }

        Ok(outliers)
    }

    /// Isolation Forest outlier detection
    fn isolation_forest_outliers(data: &[f64], contamination: f64) -> Result<Vec<OutlierInfo>, String> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let dataset: Vec<[f64; 1]> = data.iter().map(|&x| [x]).collect();
        let options = ForestOptions {
            n_trees: 100,
            sample_size: 256.min(data.len()),
            max_tree_depth: None,
            extension_level: 0,
        };

        let forest = Forest::<f64, 1>::from_slice(&dataset, &options)
            .map_err(|e| format!("Failed to create forest: {:?}", e))?;

        let scores: Vec<f64> = dataset.iter().map(|point| forest.score(point)).collect();
        let mut sorted_scores = scores.clone();
        sorted_scores.sort_by(|a, b| a.total_cmp(b));
        let threshold_index = ((1.0 - contamination) * sorted_scores.len() as f64) as usize;
        let score_threshold = sorted_scores[threshold_index.min(sorted_scores.len() - 1)];

        let mut outliers = Vec::new();
        for (i, &score) in scores.iter().enumerate() {
            if score > score_threshold {
                outliers.push(OutlierInfo {
                    index: i,
                    value: data[i],
                    z_score: None,
                    iqr_distance: None,
                    lof_score: None,
                    isolation_score: Some(score),
                });
            }
        }

        Ok(outliers)
    }

    /// Convert confidence level to z-score
    fn confidence_to_z_score(confidence: f64) -> f64 {
        distribution_functions::normal_quantile(1.0 - (1.0 - confidence) / 2.0)
    }

    /// Standardize data (Z-score normalization)
    fn standardize(data: &[f64]) -> (Vec<f64>, f64, f64) {
        let mean = data.mean();
        let std_dev = data.std_dev();
        let z_scores = if std_dev > 0.0 {
            data.iter().map(|&x| (x - mean) / std_dev).collect()
        } else {
            vec![0.0; data.len()]
        };
        (z_scores, mean, std_dev)
    }

    /// Calculate modified Z-scores
    fn modified_z_scores(data: &[f64]) -> (Vec<f64>, f64, f64) {
        let median = Quantiles::nan_safe_median(data);
        let deviations: Vec<f64> = data.iter().map(|&x| (x - median).abs()).collect();
        let mad = Quantiles::nan_safe_median(&deviations);
        let modified_z = if mad > 0.0 {
            data.iter().map(|&x| 0.6745 * (x - median) / mad).collect()
        } else {
            vec![0.0; data.len()]
        };
        (modified_z, median, mad)
    }
}