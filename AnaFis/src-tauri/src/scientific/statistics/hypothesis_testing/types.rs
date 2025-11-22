//! Hypothesis Testing Types
//!
//! Type definitions for statistical hypothesis testing methods and results.

use serde::{Deserialize, Serialize};

/// Error type for statistical operations
#[derive(Debug, Clone, PartialEq)]
pub enum StatsError {
    /// Empty data provided
    EmptyData,
    /// Insufficient data for the requested operation
    InsufficientData { required: usize, found: usize },
    /// Degenerate features (e.g., zero variance)
    DegenerateFeatures(String),
    /// Linear algebra operation failed
    LinearAlgebraError(String),
    /// Dimension mismatch in data
    DimensionMismatch,
    /// Invalid parameter values
    InvalidParameter(String),
    /// Distribution function error
    DistributionError(String),
    /// ANOVA-specific errors
    AnovaError(String),
    /// Chi-square specific errors
    ChiSquareError(String),
}

impl std::fmt::Display for StatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatsError::EmptyData => write!(f, "Empty data provided"),
            StatsError::InsufficientData { required, found } => 
                write!(f, "Insufficient data: required {}, found {}", required, found),
            StatsError::DegenerateFeatures(msg) => write!(f, "Degenerate features: {}", msg),
            StatsError::LinearAlgebraError(msg) => write!(f, "Linear algebra error: {}", msg),
            StatsError::DimensionMismatch => write!(f, "Dimension mismatch in data"),
            StatsError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            StatsError::DistributionError(msg) => write!(f, "Distribution error: {}", msg),
            StatsError::AnovaError(msg) => write!(f, "ANOVA error: {}", msg),
            StatsError::ChiSquareError(msg) => write!(f, "Chi-square error: {}", msg),
        }
    }
}

impl std::error::Error for StatsError {}

impl From<String> for StatsError {
    fn from(error: String) -> Self {
        // Try to match common error patterns, otherwise use a generic error
        if error.contains("empty") || error.contains("Empty") {
            StatsError::EmptyData
        } else if error.contains("dimension") || error.contains("length") {
            StatsError::DimensionMismatch
        } else if error.contains("singular") || error.contains("ill-conditioned") {
            StatsError::LinearAlgebraError(error)
        } else if error.contains("variance") || error.contains("standard deviation") {
            StatsError::DegenerateFeatures(error)
        } else if error.contains("ANOVA") || error.contains("factor") {
            StatsError::AnovaError(error)
        } else if error.contains("chi") || error.contains("Chi") {
            StatsError::ChiSquareError(error)
        } else {
            StatsError::InvalidParameter(error)
        }
    }
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

/// Result of N-way factorial ANOVA test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NWayAnovaResult {
    pub test_type: String,
    pub factor_results: Vec<FactorResult>,
    pub interaction_results: Vec<InteractionResult>,
    pub degrees_of_freedom_residual: f64,
    pub total_sum_of_squares: f64,
    pub residual_sum_of_squares: f64,
}

/// Individual factor result in N-way ANOVA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorResult {
    pub factor_name: String,
    pub f_statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom: f64,
    pub sum_of_squares: f64,
    pub eta_squared: f64,
    pub significant: bool,
}

/// Interaction result in N-way ANOVA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResult {
    pub interaction_name: String,
    pub factors_involved: Vec<String>,
    pub f_statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom: f64,
    pub sum_of_squares: f64,
    pub eta_squared: f64,
    pub significant: bool,
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