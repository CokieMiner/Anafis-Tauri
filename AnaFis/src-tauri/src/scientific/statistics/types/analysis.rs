use serde::{Deserialize, Serialize};
use crate::scientific::statistics::types::errors::AnalysisError;
use crate::scientific::statistics::types::time_series::{TimeSeriesComponents, TrendAnalysis, StationarityResult, ForecastResult};
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::outliers::OutlierInfo;

/// Input options for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub statistical_confidence_level: Option<f64>,
    pub uncertainties: Option<Vec<f64>>,
    pub uncertainty_confidences: Option<Vec<f64>>,
    pub bootstrap_samples: Option<usize>,
    pub correlation_method: Option<String>,
    pub nan_handling: NanHandling,
    /// If true (default) treat multiple datasets as paired observations and remove rows containing NaNs across all variables when removing.
    /// If false, NaN handling is applied independently to each dataset.
    pub treat_as_paired: Option<bool>,
    pub random_seed: Option<u64>,
    pub enabled_analyses: Option<Vec<String>>,
    pub lsl: Option<f64>, // Lower specification limit
    pub usl: Option<f64>, // Upper specification limit
    // Configurable heuristics and thresholds
    pub min_samples_for_time_series: Option<usize>,
    pub autocorr_threshold: Option<f64>,
    /// Number of lags to use in Ljung-Box test
    pub autocorr_lags: Option<usize>,
    /// p-value threshold to determine significance in Ljung-Box test
    pub ljung_box_pvalue: Option<f64>,
    pub cv_threshold: Option<f64>,
    pub correlation_strength_threshold: Option<f64>,
    pub reliability_alpha_threshold: Option<f64>,
    pub reliability_omega_threshold: Option<f64>,
    pub decimal_precision: Option<usize>,
    pub n_permutations: Option<usize>,
    // Outlier detection thresholds
    pub z_score_threshold: Option<f64>,
    pub iqr_multiplier: Option<f64>,
    pub modified_z_threshold: Option<f64>,
    pub lof_k: Option<usize>,
    pub lof_threshold: Option<f64>,
    pub isolation_forest_contamination: Option<f64>,
    pub biweight_tuning_constant: Option<f64>,
}

impl AnalysisOptions {
    /// Validate analysis options and return a structured error if any constraint fails
    pub fn validate(&self) -> Result<(), AnalysisError> {
        if let Some(conf) = self.statistical_confidence_level {
            if !(0.0..=1.0).contains(&conf) {
                return Err(AnalysisError::ConfigError(format!("Confidence level must be between 0 and 1: {}", conf)));
            }
        }

        if let Some(samples) = self.bootstrap_samples {
            if samples < 100 {
                return Err(AnalysisError::ConfigError("Bootstrap samples should be at least 100".to_string()));
            }
        }

        if let Some(nperm) = self.n_permutations {
            if nperm < 100 {
                return Err(AnalysisError::ConfigError("Permutation samples should be at least 100".to_string()));
            }
        }

        if let Some(p) = self.ljung_box_pvalue {
            if !(0.0..=1.0).contains(&p) {
                return Err(AnalysisError::ConfigError(format!("Ljung-Box p-value must be between 0 and 1: {}", p)));
            }
        }

        Ok(())
    }
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            statistical_confidence_level: Some(0.95),
            uncertainties: None,
            uncertainty_confidences: None,
            bootstrap_samples: Some(1000),
            correlation_method: Some("pearson".to_string()),
            nan_handling: NanHandling::Error,
            treat_as_paired: Some(true),
            random_seed: Some(42),
            enabled_analyses: None,
            lsl: None,
            usl: None,
            min_samples_for_time_series: Some(20),
            autocorr_threshold: Some(0.3),
            autocorr_lags: Some(10),
            ljung_box_pvalue: Some(0.05),
            cv_threshold: Some(0.1),
            correlation_strength_threshold: Some(0.3),
            reliability_alpha_threshold: Some(0.7),
            reliability_omega_threshold: Some(0.6),
            decimal_precision: Some(10),
            n_permutations: Some(5000),
            z_score_threshold: Some(3.0),
            iqr_multiplier: Some(1.5),
            modified_z_threshold: Some(3.5),
            lof_k: Some(5),
            lof_threshold: Some(1.5),
            isolation_forest_contamination: Some(0.1),
            biweight_tuning_constant: Some(9.0),
        }
    }
}

/// How NaN and infinite values are handled during sanitization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum NanHandling {
    #[default]
    Error,
    Remove,
    Mean,
    Multiple,
    Median,
    Zero,
    Ignore,
}

impl From<Option<String>> for NanHandling {
    fn from(s: Option<String>) -> Self {
        match s.as_deref() {
            Some("remove") => NanHandling::Remove,
            Some("mean") => NanHandling::Mean,
            Some("multiple") => NanHandling::Multiple,
            Some("median") => NanHandling::Median,
            Some("zero") => NanHandling::Zero,
            Some("ignore") => NanHandling::Ignore,
            _ => NanHandling::Error,
        }
    }
}

// Layer 2 Coordinator Types
#[derive(Debug, Clone)]
pub struct OutlierAnalysisResult {
    pub outlier_analysis: OutlierAnalysis,
    pub robust_statistics: RobustStatistics,
    pub winsorized_statistics: WinsorizedStatistics,
}

#[derive(Debug, Clone)]
pub struct RobustStatistics {
    pub median: f64,
    pub mad: f64,
    pub trimmed_mean: f64,
    pub robust_std_dev: f64,
}

#[derive(Debug, Clone)]
pub struct WinsorizedStatistics {
    pub winsorized_mean: f64,
    pub winsorized_variance: f64,
    pub winsorized_std_dev: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub trimmed_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct ReliabilityAnalysisResult {
    pub cronbach_alpha: f64,
    pub item_total_correlations: Vec<f64>,
    pub scale_reliability: ScaleReliability,
}

#[derive(Debug, Clone)]
pub struct ScaleReliability {
    pub omega: f64,
    pub average_interitem_corr: f64,
}

#[derive(Debug, Clone)]
pub struct QualityControlAnalysis {
    pub control_limits: ControlLimits,
    pub capability_indices: CapabilityIndices,
    pub stability_assessment: StabilityAssessment,
}

#[derive(Debug, Clone)]
pub struct ControlLimits {
    pub x_bar_upper: f64,
    pub x_bar_lower: f64,
    pub range_upper: f64,
    pub range_lower: f64,
}

#[derive(Debug, Clone)]
pub struct CapabilityIndices {
    pub cp: f64,
    pub cpk: f64,
    pub pp: f64,
    pub ppk: f64,
}

#[derive(Debug, Clone)]
pub struct StabilityAssessment {
    pub is_stable: bool,
    pub violations: Vec<String>,
}

impl std::fmt::Display for StabilityAssessment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_stable {
            write!(f, "Stable")
        } else {
            write!(f, "Unstable ({} violations)", self.violations.len())
        }
    }
}

impl PartialEq<&str> for StabilityAssessment {
    fn eq(&self, other: &&str) -> bool {
        match *other {
            "Stable" => self.is_stable,
            "Unstable" => !self.is_stable,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UncertaintyAnalysis {
    pub propagated_uncertainties: Vec<f64>,
    pub covariance_matrix: Vec<Vec<f64>>,
    pub sensitivity_coefficients: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct DistributionAnalysis {
    pub distribution_fits: Vec<crate::scientific::statistics::types::distribution::DistributionFit>,
    pub best_fit_distribution: Option<crate::scientific::statistics::types::distribution::DistributionFit>,
    pub recommended_transformations: Vec<String>,
    pub transformation_results: Vec<TransformationResult>,
}

#[derive(Debug, Clone)]
pub struct TransformationResult {
    pub transformation: String,
    pub improvement_score: f64,
    pub transformed_data: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct TimeSeriesAnalysisResult {
    pub components: Option<TimeSeriesComponents>,
    pub trend_analysis: Option<TrendAnalysis>,
    pub stationarity: Option<StationarityResult>,
    pub forecast: Option<ForecastResult>,
    pub seasonality_present: bool,
}

#[derive(Debug, Clone)]
pub struct VisualizationSuggestions {
    pub primary_plots: Vec<String>,
    pub secondary_plots: Vec<String>,
    pub diagnostic_plots: Vec<String>,
}

// Layer 3 Algorithm Types
#[derive(Debug, Clone)]
pub struct OutlierAnalysis {
    pub method: String,
    pub outliers: Vec<OutlierInfo>,
    pub threshold: f64,
    pub contamination_rate: f64,
}



/// Hypothesis testing results
#[derive(Debug, Clone)]
pub struct HypothesisTestingResult {
    pub t_test_results: Vec<crate::scientific::statistics::types::results::TTestResult>,
    pub anova_results: Vec<crate::scientific::statistics::types::results::AnovaResult>,
    pub two_way_anova_results: Vec<crate::scientific::statistics::types::results::TwoWayAnovaResult>,
    pub repeated_measures_anova_results: Vec<crate::scientific::statistics::types::results::RepeatedMeasuresAnovaResult>,
    pub chi_square_results: Vec<crate::scientific::statistics::types::results::ChiSquareResult>,
}

/// Power analysis results
#[derive(Debug, Clone)]
pub struct PowerAnalysisInternalResult {
    pub power_calculations: Vec<crate::scientific::statistics::types::results::PowerAnalysisResult>,
    pub power_curves: Vec<crate::scientific::statistics::types::results::PowerCurveResult>,
    pub recommendations: Vec<String>,
}