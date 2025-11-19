//! Pattern detection and analysis type determination

use crate::scientific::statistics::types::AnalysisOptions;
use crate::scientific::statistics::types::RequiredAnalyses;
use super::super::layer2_coordinators::quality_control::QualityControlCoordinator;
use super::super::layer2_coordinators::reliability_analysis::ReliabilityAnalysisCoordinator;
use super::super::layer3_algorithms::time_series::decomposition::TimeSeriesDecompositionEngine;
use super::super::layer3_algorithms::time_series::stationarity::StationarityEngine;

/// Pattern detection and analysis type determination
pub struct PatternDetector;

impl PatternDetector {
    /// Detect which analyses are needed based on input characteristics
    pub fn detect_required_analyses(
        datasets: &[Vec<f64>],
        options: &AnalysisOptions,
    ) -> Result<RequiredAnalyses, String> {
        let mut required = RequiredAnalyses {
            descriptive_stats: true, // Always needed
            normality_test: datasets[0].len() >= 3,    // Need at least 3 observations for normality tests
            correlation_analysis: datasets.len() > 1,
            outlier_analysis: true,  // Usually beneficial
            distribution_analysis: true, // Always useful
            uncertainty_propagation: options.uncertainties.is_some(),
            time_series_analysis: Self::detect_temporal_patterns(datasets, options)?,
            quality_control: Self::detect_process_data(datasets, options)?,
            reliability_analysis: datasets.len() >= 3 && Self::detect_scale_data(datasets, options)?,
            visualization_suggestions: true, // Always provide
            hypothesis_testing: Self::detect_hypothesis_testing_needs(datasets, options)?,
            power_analysis: Self::detect_power_analysis_needs(datasets, options)?,
        };

        // Override based on options
        if let Some(analyses) = &options.enabled_analyses {
            required.descriptive_stats = analyses.contains(&"descriptive_stats".to_string());
            required.normality_test = analyses.contains(&"normality_test".to_string());
            required.correlation_analysis = analyses.contains(&"correlation_analysis".to_string());
            required.outlier_analysis = analyses.contains(&"outlier_analysis".to_string());
            required.distribution_analysis = analyses.contains(&"distribution_analysis".to_string());
            required.uncertainty_propagation = analyses.contains(&"uncertainty_propagation".to_string());
            required.time_series_analysis = analyses.contains(&"time_series_analysis".to_string());
            required.quality_control = analyses.contains(&"quality_control".to_string());
            required.reliability_analysis = analyses.contains(&"reliability_analysis".to_string());
            required.visualization_suggestions = analyses.contains(&"visualization_suggestions".to_string());
            required.hypothesis_testing = analyses.contains(&"hypothesis_testing".to_string());
            required.power_analysis = analyses.contains(&"power_analysis".to_string());
        }

        Ok(required)
    }

    /// Detect if data shows temporal patterns
    pub fn detect_temporal_patterns(datasets: &[Vec<f64>], options: &AnalysisOptions) -> Result<bool, String> {
        let min_len = options.min_samples_for_time_series.unwrap_or(20);
        if datasets[0].len() < min_len {
            return Ok(false); // Too short for temporal analysis
        }

        let data = &datasets[0];
        let lags = options.autocorr_lags.unwrap_or(10).min(data.len().saturating_sub(1));
        let ljung_p = options.ljung_box_pvalue.unwrap_or(0.05);

        // Run core detectors
        let (_q_stat, p_value) = TimeSeriesDecompositionEngine::ljung_box_test(data, lags)?;
        let trend_test = TimeSeriesDecompositionEngine::trend_test(data)?;

        // Run ADF and KPSS heuristics (fallback gracefully if unavailable)
        let adf_is_stationary = match StationarityEngine::adf_test(data) {
            Ok(st) => st.is_stationary,
            Err(_) => false,
        };
        let kpss_is_stationary = match StationarityEngine::kpss_test(data) {
            Ok(st) => st.is_stationary,
            Err(_) => true,
        };

        Ok(p_value < ljung_p || trend_test.trend_present || !adf_is_stationary || !kpss_is_stationary)
    }

    /// Detect if data appears to be from a process control context
    pub fn detect_process_data(datasets: &[Vec<f64>], options: &AnalysisOptions) -> Result<bool, String> {
        if datasets.len() > 1 {
            return Ok(false); // Multiple variables suggest different context
        }

        let data = &datasets[0];
        let min_len = options.min_samples_for_time_series.unwrap_or(20);
        if data.len() < min_len {
            return Ok(false);
        }

        // Check for process-like characteristics
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std_dev = crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats::variance(data).sqrt();

        // Process data often has low coefficient of variation
        let cv = if mean != 0.0 { std_dev / mean.abs() } else { f64::INFINITY };
        let threshold = options.cv_threshold.unwrap_or(0.1);

        // Check for autocorrelation (drift)
        let autocorr = TimeSeriesDecompositionEngine::autocorrelation(data, 1).unwrap_or(vec![0.0])[0].abs();

        // Run quality control analysis to check Western Electric and control-limit violations
        let qc_analysis = QualityControlCoordinator::analyze(data, options.lsl, options.usl);
        let qc_unstable = if let Ok(qc) = qc_analysis { qc.stability_assessment != "Stable" } else { false };

        Ok(qc_unstable || cv < threshold && autocorr.abs() < 0.3)
    }

    /// Detect if data appears to be from psychometric scales
    pub fn detect_scale_data(datasets: &[Vec<f64>], options: &AnalysisOptions) -> Result<bool, String> {
        if datasets.len() < 3 {
            return Ok(false);
        }
        // Use reliability analysis heuristics (Cronbach's alpha, McDonald's omega, item-total correlations)
        let rel = ReliabilityAnalysisCoordinator::analyze(datasets)?;
        let alpha_threshold = options.reliability_alpha_threshold.unwrap_or(0.7);
        let omega_threshold = options.reliability_omega_threshold.unwrap_or(0.6);

        // Item-total minimum absolute correlation
        let min_item_total = rel.item_total_correlations.iter().map(|r| r.abs()).fold(f64::INFINITY, |a, b| a.min(b));

        Ok(rel.cronbach_alpha >= alpha_threshold && rel.scale_reliability.omega >= omega_threshold && min_item_total > 0.3)
    }

    /// Detect if hypothesis testing is appropriate
    pub fn detect_hypothesis_testing_needs(datasets: &[Vec<f64>], _options: &AnalysisOptions) -> Result<bool, String> {
        // Hypothesis testing is useful when we have enough data for meaningful tests
        let min_samples = 5; // Minimum samples for basic t-tests
        Ok(datasets.iter().all(|d| d.len() >= min_samples) && !datasets.is_empty())
    }

    /// Detect if power analysis is appropriate
    pub fn detect_power_analysis_needs(datasets: &[Vec<f64>], _options: &AnalysisOptions) -> Result<bool, String> {
        // Power analysis is useful for experimental design and interpreting results
        let min_samples = 10; // Need reasonable sample size for power calculations
        Ok(datasets.iter().any(|d| d.len() >= min_samples))
    }
}