use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats;
use crate::scientific::statistics::types::AnalysisOptions;
use extended_isolation_forest::{Forest, ForestOptions};

#[derive(Debug, Clone)]
pub struct OutlierInfo {
    pub index: usize,
    pub value: f64,
    pub z_score: Option<f64>,
    pub iqr_distance: Option<f64>,
    pub lof_score: Option<f64>,
    pub isolation_score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct OutlierAnalysis {
    pub methods: Vec<(String, Vec<OutlierInfo>)>,
    pub combined_outliers: Vec<usize>,
    pub outlier_percentage: f64,
}

/// Outlier detection algorithms
pub struct OutlierDetectionEngine;

impl OutlierDetectionEngine {
    /// Detect outliers using multiple methods
    pub fn detect_outliers(
        data: &[f64],
        options: &AnalysisOptions,
    ) -> Result<OutlierAnalysis, String> {
        Self::detect_outliers_with_uncertainties(data, None, None, options)
    }

    /// Detect outliers using multiple methods with uncertainty consideration
    pub fn detect_outliers_with_uncertainties(
        data: &[f64],
        uncertainties: Option<&[f64]>,
        confidence_levels: Option<&[f64]>,
        options: &AnalysisOptions,
    ) -> Result<OutlierAnalysis, String> {
        use rayon::prelude::*;

        // Define the outlier detection methods
        let method_configs = vec![
            ("Z-score", options.z_score_threshold.unwrap_or(3.0)),
            ("IQR", options.iqr_multiplier.unwrap_or(1.5)),
            ("Modified Z-score", options.modified_z_threshold.unwrap_or(3.5)),
        ];

        // Run methods in parallel
        let methods: Vec<(String, Vec<OutlierInfo>)> = method_configs
            .into_par_iter()
            .filter_map(|(name, threshold)| {
                let result = match name {
                    "Z-score" => Self::z_score_outliers_with_uncertainties(
                        data,
                        uncertainties,
                        confidence_levels,
                        threshold,
                    ),
                    "IQR" => Self::iqr_outliers_with_uncertainties(
                        data,
                        uncertainties,
                        confidence_levels,
                        threshold,
                    ),
                    "Modified Z-score" => Self::modified_z_score_outliers_with_uncertainties(
                        data,
                        uncertainties,
                        confidence_levels,
                        threshold,
                    ),
                    _ => Ok(Vec::new()),
                };
                result.ok().map(|outliers| (name.to_string(), outliers))
            })
            .collect();

        // Add LOF and Isolation Forest separately since they have different parameters
        let mut all_methods = methods;

        // Local Outlier Factor
        if data.len() >= 10 {
            let lof_k = options.lof_k.unwrap_or(5);
            let lof_threshold = options.lof_threshold.unwrap_or(1.5);
            if let Ok(lof_outliers) = Self::lof_outliers(data, lof_k, lof_threshold) {
                all_methods.push(("Local Outlier Factor".to_string(), lof_outliers));
            }
        }

        // Isolation Forest
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

        Ok(OutlierAnalysis {
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
        let mean = UnifiedStats::mean(data);
        let std_dev = UnifiedStats::std_dev(data);

        if std_dev == 0.0 {
            return Ok(Vec::new());
        }

        let mut outliers = Vec::new();
        for (i, &x) in data.iter().enumerate() {
            let z_score = (x - mean).abs() / std_dev;
            let is_outlier = if let (Some(uncs), Some(confs)) = (uncertainties, confidence_levels) {
                if i < uncs.len() && i < confs.len() {
                    let uncertainty = uncs[i];
                    let confidence = confs[i];
                    let z_conf = Self::confidence_to_z_score(confidence);
                    let lower_bound = x - uncertainty * z_conf;
                    let upper_bound = x + uncertainty * z_conf;
                    let acceptable_lower = mean - threshold * std_dev;
                    let acceptable_upper = mean + threshold * std_dev;
                    upper_bound < acceptable_lower || lower_bound > acceptable_upper
                } else {
                    z_score > threshold
                }
            } else {
                z_score > threshold
            };

            if is_outlier {
                outliers.push(OutlierInfo {
                    index: i,
                    value: x,
                    z_score: Some(z_score),
                    iqr_distance: None,
                    lof_score: None,
                    isolation_score: None,
                });
            }
        }
        Ok(outliers)
    }

    /// Convert confidence level to z-score (two-tailed)
    fn confidence_to_z_score(confidence: f64) -> f64 {
        UnifiedStats::normal_quantile(1.0 - (1.0 - confidence) / 2.0)
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
        sorted_data.sort_by(|a, b| a.total_cmp(b));

        let q1 = UnifiedStats::t_quantile(0.25, data.len() as f64 - 1.0)?; // Using a more robust quantile
        let q3 = UnifiedStats::t_quantile(0.75, data.len() as f64 - 1.0)?;
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
                    index: i, value: x, z_score: None, iqr_distance: Some(iqr_distance), lof_score: None, isolation_score: None,
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
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.total_cmp(b));

        let median = if sorted_data.len().is_multiple_of(2) {
            (sorted_data[sorted_data.len() / 2 - 1] + sorted_data[sorted_data.len() / 2]) / 2.0
        } else {
            sorted_data[sorted_data.len() / 2]
        };

        let mut deviations: Vec<f64> = data.iter().map(|x| (x - median).abs()).collect();
        deviations.sort_by(|a, b| a.total_cmp(b));
        let mad = if deviations.len().is_multiple_of(2) {
            (deviations[deviations.len() / 2 - 1] + deviations[deviations.len() / 2]) / 2.0
        } else {
            deviations[deviations.len() / 2]
        };

        if mad == 0.0 {
            return Ok(Vec::new());
        }

        let mut outliers = Vec::new();
        for (i, &x) in data.iter().enumerate() {
            let modified_z = 0.6745 * (x - median) / mad;
            let is_outlier = if let (Some(uncs), Some(confs)) = (uncertainties, confidence_levels) {
                if i < uncs.len() && i < confs.len() {
                    let uncertainty = uncs[i];
                    let confidence = confs[i];
                    let z_conf = Self::confidence_to_z_score(confidence);
                    let lower_bound_with_uncertainty = x - uncertainty * z_conf;
                    let upper_bound_with_uncertainty = x + uncertainty * z_conf;
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
                    index: i, value: x, z_score: Some(modified_z), iqr_distance: None, lof_score: None, isolation_score: None,
                });
            }
        }
        Ok(outliers)
    }

    /// Local Outlier Factor (LOF) outlier detection for 1D data
    fn lof_outliers(data: &[f64], k: usize, threshold: f64) -> Result<Vec<OutlierInfo>, String> {
        let n = data.len();
        if n <= k {
            return Err("Not enough data points for LOF analysis (n must be > k)".to_string());
        }

        let indexed_data: Vec<(usize, f64)> = data.iter().cloned().enumerate().collect();
        let mut lof_scores = vec![0.0; n];

        // Pre-calculate all k-distances and neighbor sets
        let mut all_k_distances = vec![0.0; n];
        let mut all_neighbors = vec![Vec::new(); n];

        for i in 0..n {
            let mut dists: Vec<(usize, f64)> = indexed_data.iter().map(|(j, val)| (*j, (data[i] - val).abs())).collect();
            dists.sort_by(|a, b| a.1.total_cmp(&b.1));
            all_k_distances[i] = dists[k].1; // k-th neighbor is at index k since dists[0] is the point itself
            all_neighbors[i] = dists[1..=k].iter().map(|(idx, _)| *idx).collect();
        }

        // Calculate LRD for all points
        let mut lrds = vec![0.0; n];
        for i in 0..n {
            let mut reach_dist_sum = 0.0;
            for neighbor_idx in &all_neighbors[i] {
                let reach_dist = all_k_distances[*neighbor_idx].max((data[i] - data[*neighbor_idx]).abs());
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
                for neighbor_idx in &all_neighbors[i] {
                    lrd_ratio_sum += lrds[*neighbor_idx];
                }
                lof_scores[i] = (lrd_ratio_sum / (k as f64)) / lrds[i];
            }
        }

        let mut outliers = Vec::new();
        for i in 0..n {
            if lof_scores[i] > threshold {
                outliers.push(OutlierInfo {
                    index: i, value: data[i], z_score: None, iqr_distance: None, lof_score: Some(lof_scores[i]), isolation_score: None,
                });
            }
        }

        Ok(outliers)
    }

    /// Isolation Forest outlier detection using extended_isolation_forest crate
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
                    index: i, value: data[i], z_score: None, iqr_distance: None, lof_score: None, isolation_score: Some(score),
                });
            }
        }

        Ok(outliers)
    }
}
