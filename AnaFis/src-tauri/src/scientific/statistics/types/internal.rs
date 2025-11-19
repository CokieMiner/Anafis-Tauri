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
    pub hypothesis_testing: bool,
    pub power_analysis: bool,
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
    pub hypothesis_testing: Option<super::analysis::HypothesisTestingResult>,
    pub power_analysis: Option<super::analysis::PowerAnalysisInternalResult>,
}