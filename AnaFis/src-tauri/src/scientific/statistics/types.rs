// Re-export commonly used types
pub use self::results::*;
pub use self::internal::*;
pub use self::analysis::*;
pub use self::descriptive::*;
pub use self::correlation::*;
pub use self::time_series::*;
pub use self::distribution::*;
pub use self::errors::*;

/// Module for result/output types
pub mod results {
    use serde::{Deserialize, Serialize};

    /// Output structures for the frontend API
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ComprehensiveResult {
        // Essential (always shown)
        pub descriptive_stats: Option<DescriptiveStatsOutput>,
        pub normality_test: Option<NormalityTestOutput>,
        pub correlation_matrix: Option<Vec<f64>>,
        pub data_quality: DataQualityOutput,
        pub recommendations: Vec<String>,
        pub visualization_suggestions: Option<VisualizationSuggestionsOutput>,
        pub sanitization_report: Option<SanitizationReport>,
        // New: distribution fit results included in the output for richer comparisons
        pub distribution_fits: Option<Vec<DistributionFitOutput>>,
        pub best_fit_distribution: Option<DistributionFitOutput>,

        // Advanced (shown in advanced tabs)
        pub confidence_intervals: Option<ConfidenceIntervalsOutput>,
        pub outlier_analysis: Option<OutlierAnalysisOutput>,
        pub distribution_analysis: Option<DistributionAnalysisOutput>,
        pub time_series_analysis: Option<TimeSeriesAnalysisOutput>,
        pub reliability_analysis: Option<ReliabilityAnalysisOutput>,
        pub uncertainty_propagation: Option<UncertaintyPropagationOutput>,
        pub quality_control: Option<QualityControlOutput>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DescriptiveStatsOutput {
        pub count: usize,
        pub mean: Option<f64>,
        pub median: Option<f64>,
        pub std_dev: Option<f64>,
        pub min: Option<f64>,
        pub max: Option<f64>,
        pub range: Option<f64>,
        pub q1: Option<f64>,
        pub q3: Option<f64>,
        pub iqr: Option<f64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NormalityTestOutput {
        pub test_name: String,
        pub statistic: Option<f64>,
        pub p_value: Option<f64>,
        pub is_normal: bool,
        pub method: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DataQualityOutput {
        pub sample_size_adequate: bool,
        pub is_normal: bool,
        pub outlier_summary: String,
        pub missing_data: String,
        pub quality_score: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SanitizationReport {
        /// Original number of rows per dataset
        pub original_row_counts: Vec<usize>,
        /// Rows remaining per dataset after sanitization
        pub remaining_row_counts: Vec<usize>,
        /// Total rows removed (sum across datasets when paired, or per dataset when independent)
        pub rows_removed_total: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VisualizationSuggestionsOutput {
        pub recommended_plots: Vec<RecommendedPlot>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RecommendedPlot {
        pub plot_type: String,
        pub variables: Vec<usize>,
        pub rationale: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConfidenceIntervalsOutput {
        pub mean: Option<(f64, f64)>,
        pub median: Option<(f64, f64)>,
        pub std_dev: Option<(f64, f64)>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OutlierAnalysisOutput {
        pub outlier_percentage: Option<f64>,
        pub robust_statistics: RobustStatisticsOutput,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RobustStatisticsOutput {
        pub trimmed_mean: Option<f64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DistributionAnalysisOutput {
        pub transformation_suggestions: Vec<TransformationSuggestionOutput>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DistributionFitOutput {
        pub distribution_name: String,
        pub parameters: Vec<(String, f64)>,
        pub log_likelihood: f64,
        pub aic: f64,
        pub bic: f64,
        pub goodness_of_fit: Option<f64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TransformationSuggestionOutput {
        pub transformation: String,
        pub improvement_score: Option<f64>,
        pub rationale: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TimeSeriesAnalysisOutput {
        pub trend_present: bool,
        pub seasonality_present: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ReliabilityAnalysisOutput {
        pub cronbach_alpha: Option<f64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UncertaintyPropagationOutput {
        pub total_uncertainty: Option<f64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct QualityControlOutput {
        pub process_stable: bool,
        pub cpk: Option<f64>,
    }

    /// Result of a normality test
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NormalityTestResult {
        pub test_name: String,
        pub statistic: Option<f64>,
        pub p_value: Option<f64>,
        pub is_normal: bool,
        pub method: String,
    }
}

/// Module for internal coordination types
pub mod internal {
    /// Which analyses are required
    #[derive(Debug, Clone)]
    pub struct RequiredAnalyses {
        pub descriptive_stats: bool,
        pub normality_test: bool,
        pub correlation_analysis: bool,
        pub outlier_analysis: bool,
        pub distribution_analysis: bool,
        pub uncertainty_propagation: bool,
        pub time_series_analysis: bool,
        pub quality_control: bool,
        pub reliability_analysis: bool,
        pub visualization_suggestions: bool,
    }

    /// Internal analysis results
    #[derive(Debug, Default)]
    pub struct AnalysisResults {
        pub descriptive_stats: Option<super::descriptive::DescriptiveStats>,
        pub normality_test: Option<Vec<super::results::NormalityTestResult>>,
        pub correlation_analysis: Option<super::correlation::CorrelationAnalysis>,
        pub outlier_analysis: Option<super::analysis::OutlierAnalysisResult>,
        pub distribution_analysis: Option<super::analysis::DistributionAnalysis>,
        pub uncertainty_analysis: Option<super::analysis::UncertaintyAnalysis>,
        pub time_series_analysis: Option<super::analysis::TimeSeriesAnalysisResult>,
        pub quality_control: Option<super::analysis::QualityControlAnalysis>,
        pub reliability_analysis: Option<super::analysis::ReliabilityAnalysisResult>,
        pub visualization_suggestions: Option<super::analysis::VisualizationSuggestions>,
    }
}

/// Module for analysis configuration and options
pub mod analysis {
    use serde::{Deserialize, Serialize};
    use crate::scientific::statistics::types::AnalysisError;
    use super::time_series::{TimeSeriesComponents, TrendAnalysis, StationarityResult, ForecastResult};

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

    #[derive(Debug, Clone)]
    pub struct OutlierInfo {
        pub index: usize,
        pub value: f64,
        pub z_score: Option<f64>,
        pub iqr_distance: Option<f64>,
    }
}

/// Module for descriptive statistics types
pub mod descriptive {
    /// Descriptive statistics results
    #[derive(Debug, Clone)]
    pub struct DescriptiveStats {
        pub count: usize,
        pub mean: f64,
        pub median: f64,
        pub mode: Vec<f64>,
        pub std_dev: f64,
        pub variance: f64,
        pub min: f64,
        pub max: f64,
        pub range: f64,
        pub q1: f64,
        pub q3: f64,
        pub iqr: f64,
        pub skewness: f64,
        pub kurtosis: f64,
        pub cv: f64,
        pub mad: f64,
        pub confidence_intervals: Option<ConfidenceIntervals>,
        pub robust_cv: Option<f64>,
    }

    #[derive(Debug, Clone)]
    pub struct ConfidenceIntervals {
        pub mean_ci: (f64, f64),
        pub median_ci: (f64, f64),
        pub std_dev_ci: (f64, f64),
    }
}

/// Module for correlation analysis types
pub mod correlation {
    #[derive(Debug, Clone)]
    pub struct CorrelationTestResult {
        pub method: String,
        pub variable_1: usize,
        pub variable_2: usize,
        pub correlation: f64,
        pub statistic: f64,
        pub p_value: f64,
        pub significant: bool,
    }

    #[derive(Debug, Clone)]
    pub struct CorrelationAnalysis {
        pub matrix: Vec<Vec<f64>>,
        pub methods: Vec<String>,
        pub significance_tests: Vec<CorrelationTestResult>,
    }
}

/// Module for time series analysis types
pub mod time_series {
    #[derive(Debug, Clone)]
    pub struct TimeSeriesComponents {
        pub trend: Vec<f64>,
        pub seasonal: Vec<f64>,
        pub residuals: Vec<f64>,
        pub period: usize,
    }

    #[derive(Debug, Clone)]
    pub struct TrendAnalysis {
        pub trend_present: bool,
        pub slope: f64,
        pub intercept: f64,
        pub r_squared: f64,
        pub significance: f64,
    }

    #[derive(Debug, Clone)]
    pub struct StationarityResult {
        pub is_stationary: bool,
        pub p_value: f64,
    }

    #[derive(Debug, Clone)]
    pub struct ForecastResult {
        pub forecasts: Vec<f64>,
        pub model_type: String,
        pub metrics: Option<ForecastMetrics>,
    }

    #[derive(Debug, Clone)]
    pub struct ForecastMetrics {
        pub mse: f64,   // Mean Squared Error
        pub rmse: f64,  // Root Mean Squared Error
        pub mae: f64,   // Mean Absolute Error
        pub mape: f64,  // Mean Absolute Percentage Error
    }

    /// Configuration for Prophet forecasting
    #[derive(Debug, Clone)]
    pub struct ProphetConfig {
        pub seasonality_periods: Option<Vec<usize>>,
        pub changepoint_prior_scale: Option<f64>,
        pub seasonality_prior_scale: Option<f64>,
        pub holidays: Option<Vec<Holiday>>,
        pub growth_model: Option<GrowthModel>,
        pub auto_tune: Option<bool>,
    }

    impl Default for ProphetConfig {
        fn default() -> Self {
            Self {
                seasonality_periods: Some(vec![7, 365]),
                changepoint_prior_scale: Some(0.05),
                seasonality_prior_scale: Some(10.0),
                holidays: None,
                growth_model: Some(GrowthModel::Linear),
                auto_tune: Some(false),
            }
        }
    }

    /// Prophet-style forecasting for time series with seasonality and trend
    #[derive(Debug)]
    pub struct ProphetForecast {
        pub forecasts: Vec<f64>,
        pub trend_component: Vec<f64>,
        pub seasonal_component: Vec<f64>,
        pub holiday_component: Vec<f64>,
        pub model_info: String,
    }

    /// Holiday specification for Prophet
    #[derive(Debug, Clone)]
    pub struct Holiday {
        pub name: String,
        pub dates: Vec<usize>, // Indices in the time series where holidays occur
        pub prior_scale: f64,
    }

    /// Growth model types for Prophet
    #[derive(Debug, Clone, Copy)]
    pub enum GrowthModel {
        Linear,
        Logistic { capacity: f64 },
    }
}

/// Module for distribution analysis types
pub mod distribution {
    use argmin::core::{CostFunction, Error as ArgminError, Operator, Gradient};

    /// Result of fitting a statistical distribution to data
    #[derive(Debug, Clone)]
    pub struct DistributionFit {
        /// Name of the fitted distribution
        pub distribution_name: String,
        /// Fitted parameters as (name, value) pairs
        pub parameters: Vec<(String, f64)>,
        /// Log-likelihood of the fit
        pub log_likelihood: f64,
        /// Akaike Information Criterion
        pub aic: f64,
        /// Bayesian Information Criterion
        pub bic: f64,
        /// Goodness of fit statistic (Kolmogorov-Smirnov statistic)
        pub goodness_of_fit: f64,
    }

    /// Cost function for Weibull distribution maximum likelihood estimation
    #[derive(Clone)]
    pub struct WeibullCost<'a> {
        /// Reference to the data being fitted
        pub data: &'a [f64],
    }

    impl<'a> CostFunction for WeibullCost<'a> {
        type Param = Vec<f64>;
        type Output = f64;

        fn cost(&self, param: &Self::Param) -> Result<Self::Output, ArgminError> {
            if param.len() < 2 {
                return Ok(f64::INFINITY);
            }
            let k = param[0];
            let s = param[1];
            if k <= 0.0 || s <= 0.0 || !k.is_finite() || !s.is_finite() {
                return Ok(f64::INFINITY);
            }
            let mut nll = 0.0;
            for &x in self.data.iter() {
                let z = (x / s).powf(k);
                let pdf = (k / s) * (x / s).powf(k - 1.0) * (-z).exp();
                if !pdf.is_finite() || pdf <= 0.0 {
                    return Ok(f64::INFINITY);
                }
                nll -= pdf.ln();
            }
            Ok(nll)
        }
    }

    impl<'a> Operator for WeibullCost<'a> {
        type Param = Vec<f64>;
        type Output = f64;

        fn apply(&self, param: &Vec<f64>) -> Result<Self::Output, ArgminError> {
            // Negative log-likelihood
            if param.len() < 2 {
                return Ok(f64::INFINITY);
            }
            let k = param[0];
            let s = param[1];
            if k <= 0.0 || s <= 0.0 || !k.is_finite() || !s.is_finite() {
                return Ok(f64::INFINITY);
            }
            let mut nll = 0.0;
            for &x in self.data.iter() {
                let z = (x / s).powf(k);
                let pdf = (k / s) * (x / s).powf(k - 1.0) * (-z).exp();
                if !pdf.is_finite() || pdf <= 0.0 {
                    return Ok(f64::INFINITY);
                }
                nll -= pdf.ln();
            }
            Ok(nll)
        }
    }

    impl<'a> Gradient for WeibullCost<'a> {
        type Param = Vec<f64>;
        type Gradient = Vec<f64>;

        fn gradient(&self, param: &Vec<f64>) -> Result<Self::Gradient, ArgminError> {
            if param.len() < 2 {
                return Ok(vec![f64::INFINITY, f64::INFINITY]);
            }
            let k = param[0];
            let s = param[1];
            if k <= 0.0 || s <= 0.0 || !k.is_finite() || !s.is_finite() {
                return Ok(vec![f64::INFINITY, f64::INFINITY]);
            }

            let mut dk = 0.0;
            let mut ds = 0.0;
            for &x in self.data.iter() {
                let ln_x = x.ln();
                let ln_s = s.ln();
                let ln_x_over_s = ln_x - ln_s;
                let z = (x / s).powf(k);

                // d/dk log f = 1/k + ln x - ln s - z * ln(x/s)
                let dlogf_dk = 1.0 / k + ln_x_over_s - z * ln_x_over_s;
                // d/ds log f = (k/s) * (z - 1)
                let dlogf_ds = (k / s) * (z - 1.0);

                dk -= dlogf_dk;
                ds -= dlogf_ds;
            }
            Ok(vec![dk, ds])
        }
    }
}

/// Module for error types
pub mod errors {
    use thiserror::Error;

    /// A structured error type for analysis operations
    #[derive(Debug, Error)]
    pub enum AnalysisError {
        #[error("Insufficient data: {0}")]
        InsufficientData(String),
        #[error("Invalid input: {0}")]
        InvalidInput(String),
        #[error("Numerical instability: {0}")]
        NumericalInstability(String),
        #[error("Configuration error: {0}")]
        ConfigError(String),
    }
}