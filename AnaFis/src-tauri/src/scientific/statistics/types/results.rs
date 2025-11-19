use serde::{Deserialize, Serialize};

/// Output structures for the frontend API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveResult {
    // Essential (always shown)
    pub descriptive_stats: Option<DescriptiveStatsOutput>,
    pub normality_test: Option<NormalityTestResult>,
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

    // Hypothesis testing results
    pub hypothesis_testing: Option<HypothesisTestingOutput>,

    // Power analysis results
    pub power_analysis: Option<PowerAnalysisOutput>,
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

/// Hypothesis testing results output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisTestingOutput {
    pub t_tests: Option<Vec<TTestResult>>,
    pub anova_tests: Option<Vec<AnovaResult>>,
    pub two_way_anova_tests: Option<Vec<TwoWayAnovaResult>>,
    pub repeated_measures_anova_tests: Option<Vec<RepeatedMeasuresAnovaResult>>,
    pub chi_square_tests: Option<Vec<ChiSquareResult>>,
}

/// Power analysis results output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysisOutput {
    pub power_calculations: Option<Vec<PowerAnalysisResult>>,
    pub power_curves: Option<Vec<PowerCurveResult>>,
    pub sample_size_recommendations: Option<Vec<String>>,
}

/// Result of a normality test
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NormalityTestResult {
    pub test_name: String,
    pub statistic: f64,
    pub p_value: f64,
    pub is_normal: bool,
    pub method: String,
}

/// Result of a t-test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTestResult {
    pub test_type: String,
    pub t_statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom: f64,
    pub mean_difference: f64,
    pub confidence_interval: (f64, f64),
    pub effect_size: f64,
    pub significant: bool,
    pub alternative: String,
}

/// Result of ANOVA test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnovaResult {
    pub test_type: String,
    pub f_statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom_between: f64,
    pub degrees_of_freedom_within: f64,
    pub sum_of_squares_between: f64,
    pub sum_of_squares_within: f64,
    pub eta_squared: f64,
    pub significant: bool,
    pub post_hoc_results: Option<Vec<PostHocResult>>,
}

/// Result of two-way ANOVA test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoWayAnovaResult {
    pub f_statistic_factor1: f64,
    pub f_statistic_factor2: f64,
    pub f_statistic_interaction: f64,
    pub p_value_factor1: f64,
    pub p_value_factor2: f64,
    pub p_value_interaction: f64,
    pub degrees_of_freedom_factor1: f64,
    pub degrees_of_freedom_factor2: f64,
    pub degrees_of_freedom_interaction: f64,
    pub degrees_of_freedom_residual: f64,
    pub eta_squared_factor1: f64,
    pub eta_squared_factor2: f64,
    pub eta_squared_interaction: f64,
    pub significant_factor1: bool,
    pub significant_factor2: bool,
    pub significant_interaction: bool,
}

/// Result of repeated measures ANOVA test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatedMeasuresAnovaResult {
    pub f_statistic_time: f64,
    pub f_statistic_subject: f64,
    pub f_statistic_interaction: f64,
    pub p_value_time: f64,
    pub p_value_subject: f64,
    pub p_value_interaction: f64,
    pub degrees_of_freedom_time: f64,
    pub degrees_of_freedom_subject: f64,
    pub degrees_of_freedom_interaction: f64,
    pub degrees_of_freedom_residual: f64,
    pub eta_squared_time: f64,
    pub eta_squared_subject: f64,
    pub eta_squared_interaction: f64,
    pub significant_time: bool,
    pub significant_subject: bool,
    pub significant_interaction: bool,
    pub sphericity_test: Option<SphericityTestResult>,
    pub post_hoc_results: Option<Vec<PostHocResult>>,
}

/// Result of sphericity test (Mauchly's test)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphericityTestResult {
    pub mauchly_w: f64,
    pub chi_square: f64,
    pub df: f64,
    pub p_value: f64,
    pub sphericity_assumed: bool,
    pub epsilon_gg: f64, // Greenhouse-Geisser epsilon
    pub epsilon_hf: f64, // Huynh-Feldt epsilon
}

/// Result of chi-square test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChiSquareResult {
    pub test_type: String,
    pub chi_square_statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom: f64,
    pub expected_frequencies: Vec<Vec<f64>>,
    pub residuals: Vec<Vec<f64>>,
    pub significant: bool,
    pub effect_size: Option<f64>,
}

/// Result of power analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysisResult {
    pub test_type: String,
    pub power: f64,
    pub effect_size: f64,
    pub sample_size: usize,
    pub alpha: f64,
    pub alternative: String,
    pub method: String,
}

/// Result of power curve calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerCurveResult {
    pub test_type: String,
    pub effect_size: f64,
    pub alpha: f64,
    pub alternative: String,
    pub curve_data: Vec<(usize, f64)>, // (sample_size, power)
}

/// Post-hoc test result for ANOVA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostHocResult {
    pub comparison: String,
    pub mean_difference: f64,
    pub standard_error: f64,
    pub confidence_interval: (f64, f64),
    pub p_value: f64,
    pub significant: bool,
    pub effect_size: f64,
}