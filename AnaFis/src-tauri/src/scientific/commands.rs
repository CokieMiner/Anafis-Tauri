//! Scientific Computation Commands
//!
//! This module provides Tauri command handlers for statistical analysis and scientific computations.
//! It serves as the main interface between the frontend and the comprehensive statistical analysis
//! capabilities, handling data serialization and error propagation.

use serde::{Deserialize, Serialize};

use crate::scientific::statistics::{
    StatisticalAnalysisPipeline,
    pipeline::ComprehensiveAnalysis as NewComprehensiveAnalysis,
};

/// Perform comprehensive statistical analysis on datasets
///
/// This command runs a complete suite of statistical analyses including:
/// - Descriptive statistics
/// - Normality testing
/// - Correlation analysis (for multiple datasets)
/// - Outlier detection
/// - Distribution analysis
/// - Uncertainty propagation
/// - Time series analysis
/// - Quality control
/// - Reliability analysis
/// - Visualization suggestions
///
/// # Parameters
/// - `datasets`: Vector of numeric datasets (each dataset is a vector of f64 values)
/// - `options`: Analysis configuration options
///
/// # Returns
/// NewComprehensiveAnalysis containing all analysis results
#[tauri::command]
pub fn perform_comprehensive_statistical_analysis(
    datasets: Vec<Vec<f64>>,
    options: AnalysisOptionsRequest,
) -> Result<NewComprehensiveAnalysis, String> {
    // Input validation
    if datasets.is_empty() {
        return Err("At least one dataset is required for analysis".to_string());
    }

    // Validate dataset sizes
    let first_len = datasets[0].len();
    if first_len < 2 {
        return Err("Each dataset must have at least 2 observations".to_string());
    }

    for (i, dataset) in datasets.iter().enumerate() {
        if dataset.len() != first_len {
            return Err(format!(
                "All datasets must have the same length. Dataset 0 has {} observations, but dataset {} has {}",
                first_len, i, dataset.len()
            ));
        }
    }

    // Validate incoming options
    // Note: The new pipeline uses a simplified API, so we don't need the full AnalysisOptions conversion

    // Execute the comprehensive analysis using the new pipeline
    // Convert datasets to the expected format: &[Vec<f64>]
    let data_ref: Vec<&Vec<f64>> = datasets.iter().collect();
    let data_slice: &[&Vec<f64>] = &data_ref;
    let data_transposed: Vec<Vec<f64>> = (0..datasets[0].len())
        .map(|col| data_slice.iter().map(|row| row[col]).collect())
        .collect();

    // Generate variable names
    let variable_names = Some((0..datasets.len()).map(|i| format!("Dataset{}", i + 1)).collect());

    // Determine if this should be treated as time series data
    let is_time_series = options.min_samples_for_time_series
        .map(|min_samples| datasets[0].len() >= min_samples)
        .unwrap_or(false);

    StatisticalAnalysisPipeline::comprehensive_analysis(
        &data_transposed,
        variable_names,
        is_time_series,
    )
}

/// Request structure for analysis options
/// This mirrors AnalysisOptions but uses serde-compatible types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptionsRequest {
    /// Statistical confidence level (default: 0.95)
    pub statistical_confidence_level: Option<f64>,
    /// Measurement uncertainties for each data point
    pub uncertainties: Option<Vec<f64>>,
    /// Confidence levels for uncertainty measurements
    pub uncertainty_confidences: Option<Vec<f64>>,
    /// Number of bootstrap samples for uncertainty estimation
    pub bootstrap_samples: Option<usize>,
    /// Correlation method: "pearson", "spearman", "kendall", "biweight"
    pub correlation_method: Option<String>,
    /// How to handle NaN values: "error" (default), "remove"
    pub nan_handling: Option<String>,
    /// If true, treat multiple datasets as paired (remove rows across datasets when removing NaNs)
    pub treat_as_paired: Option<bool>,
    /// Random seed for reproducible results
    pub random_seed: Option<u64>,
    /// Which analyses to enable (if None, enables all)
    pub enabled_analyses: Option<Vec<String>>,
    /// Lower specification limit for quality control
    pub lsl: Option<f64>,
    /// Upper specification limit for quality control
    pub usl: Option<f64>,
    /// Number of permutations for permutation-based p-value estimation (default: 5000)
    pub n_permutations: Option<usize>,
    /// Number of lags to use in Ljung-Box test
    pub autocorr_lags: Option<usize>,
    /// Ljung-Box p-value threshold for temporal detection
    pub ljung_box_pvalue: Option<f64>,
    /// Minimum number of samples required to treat data as time series
    pub min_samples_for_time_series: Option<usize>,
}

impl Default for AnalysisOptionsRequest {
    fn default() -> Self {
        Self {
            statistical_confidence_level: Some(0.95),
            uncertainties: None,
            uncertainty_confidences: None,
            bootstrap_samples: Some(1000),
            correlation_method: Some("pearson".to_string()),
            nan_handling: Some("error".to_string()),
            random_seed: Some(42),
            enabled_analyses: None,
            lsl: None,
            usl: None,
            n_permutations: Some(5000),
            autocorr_lags: Some(10),
            ljung_box_pvalue: Some(0.05),
            treat_as_paired: Some(true),
            min_samples_for_time_series: Some(30),
        }
    }
}
