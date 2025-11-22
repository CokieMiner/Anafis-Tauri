//! Domain-Driven Statistics Library
//!
//! This module provides a refactored, domain-driven approach to statistical analysis,
//! eliminating the rigid 4-layer architecture and consolidating duplicated functionality.
//!
//! ## Organization
//! - `correlation/`: Correlation methods and matrix operations
//! - `hypothesis_testing/`: Statistical hypothesis testing (t-tests, ANOVA, chi-square)
//! - `outliers/`: Multi-method outlier detection with uncertainty support
//! - `robust_regression/`: Robust regression methods (Huber, RANSAC, IRLS)
//! - `matrix_ops/`: Matrix operations (covariance, PCA, SVD, eigenvalue decomposition)
//! - `stationarity/`: Time series stationarity testing (ADF, KPSS, Phillips-Perron)
//! - `prophet/`: Time series forecasting with trend changepoints and seasonality
//! - `descriptive/`: Basic statistical measures (mean, variance, quantiles, etc.)
//! - `distributions/`: Distribution fitting, testing, and analysis
//! - `time_series/`: Time series analysis and forecasting
//! - `uncertainty/`: Uncertainty propagation and bootstrap methods
//! - `primitives/`: Low-level mathematical utilities
//! - `quality_control/`: Process quality control with control charts and capability analysis
//! - `reliability/`: Scale reliability analysis for psychometric measurements
//! - `visualization/`: Automated visualization suggestions based on data characteristics
//! - `power/`: Statistical power analysis for various test types
//! - `formatting/`: Structured output formatting for statistical results
//! - `preprocessing/`: Data preprocessing including imputation and transformations
//! - `pipeline/`: Automated statistical analysis pipeline

pub mod correlation;
pub mod hypothesis_testing;
pub mod outliers;
pub mod robust_regression;
pub mod matrix_ops;
pub mod stationarity;
pub mod prophet;
pub mod descriptive;
pub mod distributions;
pub mod time_series;
pub mod uncertainty;
pub mod primitives;
pub mod quality_control;
pub mod reliability;
pub mod visualization;
pub mod power;
pub mod formatting;
pub mod preprocessing;
pub mod pipeline;

// Test modules
#[cfg(test)]
pub mod tests;
pub use correlation::CorrelationMethods;
pub use hypothesis_testing::HypothesisTestingEngine;
pub use outliers::{OutlierDetectionEngine, OutlierDetectionConfig};
pub use robust_regression::RobustRegressionEngine;
pub use matrix_ops::MatrixOpsEngine;
pub use stationarity::StationarityEngine;
pub use prophet::ProphetEngine;
pub use descriptive::StatisticalMoments;
pub use time_series::{TimeSeriesDecompositionEngine, TimeSeriesForecastingEngine};
pub use uncertainty::{UncertaintyPropagationEngine, BootstrapEngine};
pub use correlation::CorrelationHypothesisTestingEngine;
pub use quality_control::QualityControlEngine;
pub use reliability::ReliabilityEngine;
pub use visualization::VisualizationEngine;
pub use power::PowerAnalysisEngine;
pub use formatting::OutputFormatter;
pub use preprocessing::DataImputationEngine;
pub use pipeline::StatisticalAnalysisPipeline;