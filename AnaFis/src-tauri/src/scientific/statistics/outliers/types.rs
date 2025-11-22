//! Outlier Detection Types
//!
//! Type definitions for outlier detection methods and results.

use serde::{Deserialize, Serialize};

/// Information about a detected outlier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierInfo {
    pub index: usize,
    pub value: f64,
    pub z_score: Option<f64>,
    pub iqr_distance: Option<f64>,
    pub lof_score: Option<f64>,
    pub isolation_score: Option<f64>,
}

/// Comprehensive outlier analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierAnalysisResult {
    pub methods: Vec<(String, Vec<OutlierInfo>)>,
    pub combined_outliers: Vec<usize>,
    pub outlier_percentage: f64,
}

/// Outlier detection method configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierDetectionConfig {
    pub z_score_threshold: Option<f64>,
    pub iqr_multiplier: Option<f64>,
    pub modified_z_threshold: Option<f64>,
    pub lof_k: Option<usize>,
    pub lof_threshold: Option<f64>,
    pub isolation_forest_contamination: Option<f64>,
}

/// Alias for backward compatibility
pub type AnalysisOptions = OutlierDetectionConfig;

impl Default for OutlierDetectionConfig {
    fn default() -> Self {
        Self {
            z_score_threshold: Some(3.0),
            iqr_multiplier: Some(1.5),
            modified_z_threshold: Some(3.5),
            lof_k: Some(5),
            lof_threshold: Some(1.5),
            isolation_forest_contamination: Some(0.1),
        }
    }
}