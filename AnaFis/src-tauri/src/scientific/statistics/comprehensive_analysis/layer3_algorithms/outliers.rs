use crate::scientific::statistics::types::AnalysisOptions;
use crate::scientific::statistics::comprehensive_analysis::descriptive_stats::quantiles::Quantiles;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::distribution::StatisticalDistributionEngine;
use geo::OutlierDetection;
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
    pub fn detect_outliers(data: &[f64], options: &AnalysisOptions) -> Result<OutlierAnalysis, String> {
        Self::detect_outliers_with_uncertainties(data, None, None, options)
    }

    /// Detect outliers using multiple methods with uncertainty consideration
    pub fn detect_outliers_with_uncertainties(
        data: &[f64], 
        uncertainties: Option<&[f64]>, 
        confidence_levels: Option<&[f64]>,
        options: &AnalysisOptions
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
                    "Z-score" => Self::z_score_outliers_with_uncertainties(data, uncertainties, confidence_levels, threshold),
                    "IQR" => Self::iqr_outliers_with_uncertainties(data, uncertainties, confidence_levels, threshold),
                    "Modified Z-score" => Self::modified_z_score_outliers_with_uncertainties(data, uncertainties, confidence_levels, threshold),
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
        threshold: f64
    ) -> Result<Vec<OutlierInfo>, String> {
        if data.len() < 2 {
            return Ok(Vec::new()); // Need at least 2 points for variance
        }
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std_dev = StatisticalDistributionEngine::variance(data).sqrt();

        if std_dev == 0.0 {
            return Ok(Vec::new()); // No variation, no outliers
        }

        let mut outliers = Vec::new();
        for (i, &x) in data.iter().enumerate() {
            let z_score = (x - mean).abs() / std_dev;
            
            // Check if this point should be considered an outlier considering uncertainties
            let is_outlier = if let (Some(uncs), Some(confs)) = (uncertainties, confidence_levels) {
                if i < uncs.len() && i < confs.len() {
                    // Calculate the uncertainty-adjusted threshold
                    // A point is an outlier if its uncertainty bounds don't overlap with the normal range
                    let uncertainty = uncs[i];
                    let confidence = confs[i];
                    
                    // For normal distribution, the z-score threshold corresponds to a certain confidence interval
                    // We need to see if the point's uncertainty interval overlaps with the acceptable range
                    let lower_bound = x - uncertainty * Self::confidence_to_z_score(confidence);
                    let upper_bound = x + uncertainty * Self::confidence_to_z_score(confidence);
                    
                    let acceptable_lower = mean - threshold * std_dev;
                    let acceptable_upper = mean + threshold * std_dev;
                    
                    // If the uncertainty interval doesn't overlap with the acceptable range, it's an outlier
                    upper_bound < acceptable_lower || lower_bound > acceptable_upper
                } else {
                    // No uncertainty data for this point, use standard method
                    z_score > threshold
                }
            } else {
                // No uncertainty data, use standard method
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
        // For normal distribution, confidence level to z-score
        // Using approximation: for 95% confidence, z ≈ 1.96
        // More generally: z = sqrt(2) * erfinv(2*confidence - 1)
        // But using a simple approximation for common values
        if confidence >= 0.999 {
            3.291
        } else if confidence >= 0.99 {
            2.576
        } else if confidence >= 0.95 {
            1.96
        } else if confidence >= 0.90 {
            1.645
        } else if confidence >= 0.80 {
            1.282
        } else {
            // Fallback approximation
            (-2.0 * (1.0 - confidence).ln()).sqrt()
        }
    }

    /// IQR-based outlier detection with uncertainty consideration
    fn iqr_outliers_with_uncertainties(
        data: &[f64], 
        uncertainties: Option<&[f64]>, 
        confidence_levels: Option<&[f64]>,
        multiplier: f64
    ) -> Result<Vec<OutlierInfo>, String> {
        if data.len() < 4 {
            return Ok(Vec::new()); // Need at least 4 points for meaningful quartiles
        }
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = Self::quantile(&sorted_data, 0.25);
        let q3 = Self::quantile(&sorted_data, 0.75);
        let iqr = q3 - q1;

        let lower_bound = q1 - multiplier * iqr;
        let upper_bound = q3 + multiplier * iqr;

        let mut outliers = Vec::new();
        for (i, &x) in data.iter().enumerate() {
            // Check if this point should be considered an outlier considering uncertainties
            let is_outlier = if let (Some(uncs), Some(confs)) = (uncertainties, confidence_levels) {
                if i < uncs.len() && i < confs.len() {
                    let uncertainty = uncs[i];
                    let confidence = confs[i];
                    
                    // Calculate uncertainty bounds
                    let uncertainty_range = uncertainty * Self::confidence_to_z_score(confidence);
                    let lower_bound_with_uncertainty = x - uncertainty_range;
                    let upper_bound_with_uncertainty = x + uncertainty_range;
                    
                    // A point is an outlier if its uncertainty interval doesn't overlap with the acceptable range
                    upper_bound_with_uncertainty < lower_bound || lower_bound_with_uncertainty > upper_bound
                } else {
                    // No uncertainty data for this point, use standard method
                    x < lower_bound || x > upper_bound
                }
            } else {
                // No uncertainty data, use standard method
                x < lower_bound || x > upper_bound
            };
            
            if is_outlier {
                let iqr_distance = if iqr == 0.0 {
                    // For constant data, any deviation is infinite distance
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
        threshold: f64
    ) -> Result<Vec<OutlierInfo>, String> {
        if data.len() < 2 {
            return Ok(Vec::new()); // Need at least 2 points for meaningful MAD
        }
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if sorted_data.len().is_multiple_of(2) {
            (sorted_data[sorted_data.len() / 2 - 1] + sorted_data[sorted_data.len() / 2]) / 2.0
        } else {
            sorted_data[sorted_data.len() / 2]
        };

        // MAD (Median Absolute Deviation)
        let mut deviations: Vec<f64> = data.iter()
            .map(|x| (x - median).abs())
            .collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());

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
            
            // Check if this point should be considered an outlier considering uncertainties
            let is_outlier = if let (Some(uncs), Some(confs)) = (uncertainties, confidence_levels) {
                if i < uncs.len() && i < confs.len() {
                    let uncertainty = uncs[i];
                    let confidence = confs[i];
                    
                    // Calculate uncertainty bounds
                    let uncertainty_range = uncertainty * Self::confidence_to_z_score(confidence);
                    let lower_bound_with_uncertainty = x - uncertainty_range;
                    let upper_bound_with_uncertainty = x + uncertainty_range;
                    
                    // For modified z-score, the acceptable range is around the median
                    let acceptable_lower = median - (threshold / 0.6745) * mad;
                    let acceptable_upper = median + (threshold / 0.6745) * mad;
                    
                    // A point is an outlier if its uncertainty interval doesn't overlap with the acceptable range
                    upper_bound_with_uncertainty < acceptable_lower || lower_bound_with_uncertainty > acceptable_upper
                } else {
                    // No uncertainty data for this point, use standard method
                    modified_z.abs() > threshold
                }
            } else {
                // No uncertainty data, use standard method
                modified_z.abs() > threshold
            };
            
            if is_outlier {
                outliers.push(OutlierInfo {
                    index: i,
                    value: x,
                    z_score: Some(modified_z),
                    iqr_distance: None,
                    lof_score: None,
                    isolation_score: None,
                });
            }
        }

        Ok(outliers)
    }

    /// Compute quantile from sorted data
    fn quantile(sorted_data: &[f64], p: f64) -> f64 {
        Quantiles::quantile_legacy(sorted_data, p)
    }

    /// Local Outlier Factor (LOF) outlier detection using geo crate
    fn lof_outliers(data: &[f64], k: usize, threshold: f64) -> Result<Vec<OutlierInfo>, String> {
        if data.len() < k + 1 {
            return Err("Not enough data points for LOF analysis".to_string());
        }

        // Convert to Vec<Point<f64>> for geo
        let points: Vec<geo::Point<f64>> = data.iter().enumerate()
            .map(|(i, &x)| geo::Point::new(i as f64, x))
            .collect();

        // Compute LOF scores
        let scores = points.outliers(k);

        let mut outliers = Vec::new();
        for (i, &score) in scores.iter().enumerate() {
            if score > threshold {
                outliers.push(OutlierInfo {
                    index: i,
                    value: data[i],
                    z_score: None,
                    iqr_distance: None,
                    lof_score: Some(score),
                    isolation_score: None,
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

        // Prepare data as [[f64; 1]]
        let dataset: Vec<[f64; 1]> = data.iter().map(|&x| [x]).collect();

        // Create options
        let options = ForestOptions {
            n_trees: 100,
            sample_size: 256,
            max_tree_depth: None,
            extension_level: 1,
        };

        // Create forest
        let forest = Forest::<f64, 1>::from_slice(&dataset, &options)
            .map_err(|e| format!("Failed to create forest: {:?}", e))?;

        let mut outliers = Vec::new();
        for (i, &value) in data.iter().enumerate() {
            let score = forest.score(&[value]);

            // Higher scores indicate outliers
            if score > contamination {
                outliers.push(OutlierInfo {
                    index: i,
                    value,
                    z_score: None,
                    iqr_distance: None,
                    lof_score: None,
                    isolation_score: Some(score),
                });
            }
        }

        Ok(outliers)
    }
}
