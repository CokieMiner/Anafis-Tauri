use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::outliers::OutlierDetectionEngine;
use crate::scientific::statistics::types::{AnalysisOptions, OutlierAnalysisResult, RobustStatistics, WinsorizedStatistics, analysis::OutlierAnalysis};
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::outliers::OutlierInfo;
use crate::scientific::statistics::comprehensive_analysis::descriptive_stats::{DescriptiveStatsCoordinator, quantiles::QuantileMethod};

/// Outlier Analysis Coordinator
/// Coordinates outlier detection and robust statistics
pub struct OutlierAnalysisCoordinator;

impl OutlierAnalysisCoordinator {
    /// Analyze outliers in a dataset
    pub fn analyze(data: &[f64], options: &AnalysisOptions) -> Result<OutlierAnalysisResult, String> {
        if data.is_empty() {
            return Err("Cannot analyze outliers in empty dataset".to_string());
        }

        // Detect outliers using multiple methods
        let outlier_result = OutlierDetectionEngine::detect_outliers(data, options)?;

        // Convert to consolidated OutlierAnalysis
        let outlier_analysis = OutlierAnalysis {
            method: "Combined methods".to_string(),
            outliers: outlier_result.combined_outliers.iter().map(|&idx| OutlierInfo {
                index: idx,
                value: data[idx],
                z_score: None, // Would need to compute
                iqr_distance: None, // Would need to compute
                lof_score: None,
                isolation_score: None,
            }).collect(),
            threshold: 0.1, // Default threshold
            contamination_rate: outlier_result.outlier_percentage / 100.0,
        };

        // Compute robust statistics
        let robust_stats = Self::compute_robust_statistics(data)?;

        // Winsorized statistics
        let winsorized_stats = Self::compute_winsorized_statistics(data, 0.1)?;

        Ok(OutlierAnalysisResult {
            outlier_analysis,
            robust_statistics: robust_stats,
            winsorized_statistics: winsorized_stats,
        })
    }

    /// Analyze outliers in a dataset with uncertainty bounds
    pub fn analyze_with_uncertainties(
        data: &[f64], 
        uncertainties: Option<&[f64]>, 
        confidence_levels: Option<&[f64]>,
        options: &AnalysisOptions
    ) -> Result<OutlierAnalysisResult, String> {
        if data.is_empty() {
            return Err("Cannot analyze outliers in empty dataset".to_string());
        }

        // Detect outliers using multiple methods with uncertainty consideration
        let outlier_result = OutlierDetectionEngine::detect_outliers_with_uncertainties(
            data, 
            uncertainties, 
            confidence_levels, 
            options
        )?;

        // Convert to consolidated OutlierAnalysis
        let outlier_analysis = OutlierAnalysis {
            method: "Combined methods with uncertainties".to_string(),
            outliers: outlier_result.combined_outliers.iter().map(|&idx| OutlierInfo {
                index: idx,
                value: data[idx],
                z_score: None, // Would need to compute
                iqr_distance: None, // Would need to compute
                lof_score: None,
                isolation_score: None,
            }).collect(),
            threshold: 0.1, // Default threshold
            contamination_rate: outlier_result.outlier_percentage / 100.0,
        };

        // Compute robust statistics
        let robust_stats = Self::compute_robust_statistics(data)?;

        // Winsorized statistics
        let winsorized_stats = Self::compute_winsorized_statistics(data, 0.1)?;

        Ok(OutlierAnalysisResult {
            outlier_analysis,
            robust_statistics: robust_stats,
            winsorized_statistics: winsorized_stats,
        })
    }

    /// Compute robust statistics
    fn compute_robust_statistics(data: &[f64]) -> Result<RobustStatistics, String> {
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.total_cmp(b));

        let median = DescriptiveStatsCoordinator::quantile(&sorted_data, 0.50, QuantileMethod::Type8);

        // MAD (Median Absolute Deviation)
        let deviations: Vec<f64> = data.iter().map(|x| (x - median).abs()).collect();
        let mut sorted_deviations = deviations.to_vec();
        sorted_deviations.sort_by(|a, b| a.total_cmp(b));
        let mad = DescriptiveStatsCoordinator::quantile(&sorted_deviations, 0.50, QuantileMethod::Type8);

        // Trimmed mean (10% trimmed) - fallback to mean if trimmed_data is empty
        let trim_percent = 0.1;
        let trim_count = (data.len() as f64 * trim_percent) as usize;
        let trimmed_data = if data.len() > 2 * trim_count {
            &sorted_data[trim_count..(data.len() - trim_count)]
        } else {
            &sorted_data[..]
        };
        let trimmed_mean = if trimmed_data.is_empty() {
            sorted_data.iter().sum::<f64>() / sorted_data.len() as f64
        } else {
            trimmed_data.iter().sum::<f64>() / trimmed_data.len() as f64
        };

        // Robust standard deviation (using MAD)
        let robust_std_dev = mad / 0.6745; // MAD to std conversion for normal distribution

        Ok(RobustStatistics {
            median,
            mad,
            trimmed_mean,
            robust_std_dev,
        })
    }

    /// Compute Winsorized statistics
    fn compute_winsorized_statistics(data: &[f64], trim_percent: f64) -> Result<WinsorizedStatistics, String> {
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.total_cmp(b));

    let trim_count = (data.len() as f64 * trim_percent) as usize;
    let lower_bound = if data.len() > trim_count { sorted_data[trim_count] } else { sorted_data[0] };
    let upper_bound = if data.len() > trim_count { sorted_data[data.len() - 1 - trim_count] } else { sorted_data[sorted_data.len() - 1] };

        // Winsorize the data
        let winsorized_data: Vec<f64> = data.iter()
            .map(|&x| {
                if x < lower_bound {
                    lower_bound
                } else if x > upper_bound {
                    upper_bound
                } else {
                    x
                }
            })
            .collect();

        let winsorized_mean = winsorized_data.iter().sum::<f64>() / winsorized_data.len() as f64;
        let winsorized_variance = winsorized_data.iter()
            .map(|x| (x - winsorized_mean).powi(2))
            .sum::<f64>() / (winsorized_data.len() - 1) as f64;

        Ok(WinsorizedStatistics {
            winsorized_mean,
            winsorized_variance,
            winsorized_std_dev: winsorized_variance.sqrt(),
            lower_bound,
            upper_bound,
            trimmed_percentage: trim_percent * 100.0,
        })
    }
}
