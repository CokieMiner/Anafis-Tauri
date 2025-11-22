//! Correlation Analysis Types
//!
//! Type definitions for correlation analysis methods and results.

use serde::{Deserialize, Serialize};

/// Result of correlation analysis between two variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub coefficient: f64,
    pub p_value: f64,
    pub confidence_interval: Option<(f64, f64)>,
    pub method: CorrelationMethod,
}

/// Available correlation methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CorrelationMethod {
    Pearson,
    Spearman,
    Kendall,
    BiweightMidcorrelation,
    PercentageBend,
}

/// Correlation matrix result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub matrix: Vec<Vec<f64>>,
    pub p_values: Vec<Vec<f64>>,
    pub methods: Vec<CorrelationMethod>,
    pub variable_names: Vec<String>,
}

/// Hypothesis testing result for correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationHypothesisTest {
    pub correlation_coefficient: f64,
    pub test_statistic: f64,
    pub p_value: f64,
    pub degrees_of_freedom: usize,
    pub confidence_interval: (f64, f64),
    pub method: CorrelationMethod,
    pub alternative: String,
    pub significant: bool,
}

/// Result of normality test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalityTestResult {
    pub is_normal: bool,
    pub method: String,
    pub test_name: String,
    pub statistic: f64,
    pub p_value: f64,
}

/// Result of correlation test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationTestResult {
    pub correlation: f64,
    pub p_value: f64,
    pub method: String,
    pub variable_1: usize,
    pub variable_2: usize,
    pub statistic: f64,
    pub significant: bool,
}