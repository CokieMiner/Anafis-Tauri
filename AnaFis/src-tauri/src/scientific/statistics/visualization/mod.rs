//! Visualization Suggestions Module
//!
//! This module provides automated recommendations for data visualization based on:
//! - Data characteristics and distributions
//! - Statistical properties
//! - Relationships between variables
//! - Data transformation suggestions

use crate::scientific::statistics::descriptive::moments::StatisticalMoments;
use crate::scientific::statistics::correlation::CorrelationMethods;
use crate::scientific::statistics::time_series::trend::TrendAnalysisEngine;

/// Visualization suggestions and recommendations
#[derive(Debug, Clone)]
pub struct VisualizationSuggestions {
    pub recommended_plots: Vec<VisualizationRecommendation>,
    pub data_transformations: Vec<DataTransformation>,
    pub diagnostic_plots: Vec<DiagnosticPlot>,
}

/// Individual visualization recommendation
#[derive(Debug, Clone)]
pub struct VisualizationRecommendation {
    pub plot_type: String,
    pub title: String,
    pub description: String,
    pub variables: Vec<String>,
    pub rationale: String,
    pub priority: PlotPriority,
}

/// Data transformation suggestion
#[derive(Debug, Clone)]
pub struct DataTransformation {
    pub transformation_type: String,
    pub target_variable: String,
    pub purpose: String,
    pub expected_benefit: String,
    pub confidence: f64, // 0.0 to 1.0
}

/// Diagnostic plot for statistical analysis
#[derive(Debug, Clone)]
pub struct DiagnosticPlot {
    pub plot_type: String,
    pub title: String,
    pub description: String,
    pub purpose: String,
}

/// Priority level for plot recommendations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlotPriority {
    Essential,
    Recommended,
    Optional,
}

/// Visualization Engine
/// Main engine for generating visualization suggestions
pub struct VisualizationEngine;

impl VisualizationEngine {
    /// Generate comprehensive visualization suggestions
    pub fn suggest_visualizations(
        primary_data: &[f64],
        datasets: Option<&[Vec<f64>]>,
        variable_names: Option<&[String]>,
    ) -> Result<VisualizationSuggestions, String> {
        let mut recommendations = Vec::new();
        let mut transformations = Vec::new();
        let mut diagnostics = Vec::new();

        // Basic univariate plots
        Self::add_univariate_recommendations(primary_data, &mut recommendations)?;

        // Multivariate plots if multiple datasets
        if let Some(multi_datasets) = datasets {
            if multi_datasets.len() > 1 {
                Self::add_multivariate_recommendations(multi_datasets, variable_names, &mut recommendations)?;
            }
        }

        // Data transformation suggestions
        Self::suggest_transformations(primary_data, &mut transformations)?;

        // Diagnostic plots
        Self::add_diagnostic_plots(primary_data, datasets, &mut diagnostics)?;

        Ok(VisualizationSuggestions {
            recommended_plots: recommendations,
            data_transformations: transformations,
            diagnostic_plots: diagnostics,
        })
    }

    /// Add univariate visualization recommendations
    fn add_univariate_recommendations(
        data: &[f64],
        recommendations: &mut Vec<VisualizationRecommendation>,
    ) -> Result<(), String> {
        // Histogram - always recommended
        recommendations.push(VisualizationRecommendation {
            plot_type: "histogram".to_string(),
            title: "Distribution Histogram".to_string(),
            description: "Shows the frequency distribution of values".to_string(),
            variables: vec!["primary".to_string()],
            rationale: "Essential for understanding data distribution and central tendency".to_string(),
            priority: PlotPriority::Essential,
        });

        // Box plot
        recommendations.push(VisualizationRecommendation {
            plot_type: "boxplot".to_string(),
            title: "Box Plot".to_string(),
            description: "Shows median, quartiles, and potential outliers".to_string(),
            variables: vec!["primary".to_string()],
            rationale: "Excellent for identifying outliers and spread".to_string(),
            priority: PlotPriority::Essential,
        });

        // Q-Q plot for normality assessment
        let normality_assessment = Self::assess_normality(data)?;
        if !normality_assessment.is_normal {
            recommendations.push(VisualizationRecommendation {
                plot_type: "qqplot".to_string(),
                title: "Q-Q Plot".to_string(),
                description: "Assesses normality by comparing to theoretical normal distribution".to_string(),
                variables: vec!["primary".to_string()],
                rationale: format!("Data shows non-normal characteristics: {}", normality_assessment.reason),
                priority: PlotPriority::Recommended,
            });
        }

        // Density plot for smooth distribution
        if data.len() >= 50 {
            recommendations.push(VisualizationRecommendation {
                plot_type: "density".to_string(),
                title: "Kernel Density Plot".to_string(),
                description: "Smooth estimate of the probability density function".to_string(),
                variables: vec!["primary".to_string()],
                rationale: "Provides smooth view of distribution shape".to_string(),
                priority: PlotPriority::Recommended,
            });
        }

        // Time series plot if data might be temporal
        if Self::might_be_time_series(data)? {
            recommendations.push(VisualizationRecommendation {
                plot_type: "timeseries".to_string(),
                title: "Time Series Plot".to_string(),
                description: "Shows values over time or sequence".to_string(),
                variables: vec!["primary".to_string()],
                rationale: "Data appears to have temporal or sequential structure".to_string(),
                priority: PlotPriority::Recommended,
            });
        }

        Ok(())
    }

    /// Add multivariate visualization recommendations
    fn add_multivariate_recommendations(
        datasets: &[Vec<f64>],
        variable_names: Option<&[String]>,
        recommendations: &mut Vec<VisualizationRecommendation>,
    ) -> Result<(), String> {
        let n_vars = datasets.len();

        // Scatter plot for two variables
        if n_vars >= 2 {
            let var1 = variable_names.and_then(|names| names.first()).cloned()
                .unwrap_or_else(|| "Variable 1".to_string());
            let var2 = variable_names.and_then(|names| names.get(1)).cloned()
                .unwrap_or_else(|| "Variable 2".to_string());

            recommendations.push(VisualizationRecommendation {
                plot_type: "scatter".to_string(),
                title: format!("{} vs {}", var1, var2),
                description: "Shows relationship between two continuous variables".to_string(),
                variables: vec![var1, var2],
                rationale: "Essential for examining bivariate relationships".to_string(),
                priority: PlotPriority::Essential,
            });
        }

        // Correlation heatmap for multiple variables
        if n_vars >= 3 {
            recommendations.push(VisualizationRecommendation {
                plot_type: "correlation_heatmap".to_string(),
                title: "Correlation Matrix Heatmap".to_string(),
                description: "Visualizes correlations between all variable pairs".to_string(),
                variables: (0..n_vars).map(|i| {
                    variable_names.and_then(|names| names.get(i)).cloned()
                        .unwrap_or_else(|| format!("Variable {}", i + 1))
                }).collect(),
                rationale: "Shows multivariate relationships and dependencies".to_string(),
                priority: PlotPriority::Recommended,
            });
        }

        // Pairwise scatter plots
        if (3..=5).contains(&n_vars) {
            recommendations.push(VisualizationRecommendation {
                plot_type: "pairs".to_string(),
                title: "Scatter Plot Matrix".to_string(),
                description: "Matrix of scatter plots for all variable pairs".to_string(),
                variables: (0..n_vars).map(|i| {
                    variable_names.and_then(|names| names.get(i)).cloned()
                        .unwrap_or_else(|| format!("Variable {}", i + 1))
                }).collect(),
                rationale: "Comprehensive view of all pairwise relationships".to_string(),
                priority: PlotPriority::Recommended,
            });
        }

        Ok(())
    }

    /// Suggest data transformations for better visualization
    fn suggest_transformations(
        data: &[f64],
        transformations: &mut Vec<DataTransformation>,
    ) -> Result<(), String> {
        // Check for skewness
        let skewness = data.skewness();

        // Positive skewness - suggest log or sqrt transformation
        if skewness > 1.0 {
            if data.iter().all(|&x| x > 0.0) {
                transformations.push(DataTransformation {
                    transformation_type: "log".to_string(),
                    target_variable: "primary".to_string(),
                    purpose: "Reduce positive skewness".to_string(),
                    expected_benefit: "Makes distribution more symmetric for better visualization".to_string(),
                    confidence: 0.8,
                });
            }

            if data.iter().all(|&x| x >= 0.0) {
                transformations.push(DataTransformation {
                    transformation_type: "sqrt".to_string(),
                    target_variable: "primary".to_string(),
                    purpose: "Reduce positive skewness".to_string(),
                    expected_benefit: "Stabilizes variance and reduces skewness".to_string(),
                    confidence: 0.7,
                });
            }
        }

        // Negative skewness - suggest square or cube transformation
        if skewness < -1.0 {
            transformations.push(DataTransformation {
                transformation_type: "square".to_string(),
                target_variable: "primary".to_string(),
                purpose: "Reduce negative skewness".to_string(),
                expected_benefit: "Makes distribution more symmetric".to_string(),
                confidence: 0.6,
            });
        }

        // Check for outliers that might affect scale
        let outlier_analysis = crate::scientific::statistics::outliers::OutlierDetectionEngine::detect_outliers(
            data,
            &crate::scientific::statistics::outliers::types::OutlierDetectionConfig::default(),
        )?;

        if outlier_analysis.outlier_percentage > 10.0 {
            transformations.push(DataTransformation {
                transformation_type: "robust_scale".to_string(),
                target_variable: "primary".to_string(),
                purpose: "Handle outliers for better scale visualization".to_string(),
                expected_benefit: "Reduces impact of extreme values on plot scales".to_string(),
                confidence: 0.9,
            });
        }

        // Check for wide range that might benefit from scaling
        let range = crate::scientific::statistics::descriptive::Dispersion::range(data);
        if range > 1000.0 {
            transformations.push(DataTransformation {
                transformation_type: "standardize".to_string(),
                target_variable: "primary".to_string(),
                purpose: "Standardize scale for better visualization".to_string(),
                expected_benefit: "Centers data around mean with unit variance".to_string(),
                confidence: 0.7,
            });
        }

        Ok(())
    }

    /// Add diagnostic plots for statistical analysis
    fn add_diagnostic_plots(
        primary_data: &[f64],
        datasets: Option<&[Vec<f64>]>,
        diagnostics: &mut Vec<DiagnosticPlot>,
    ) -> Result<(), String> {
        // Residual plots if we have multiple variables (potential regression context)
        if let Some(multi_datasets) = datasets {
            if multi_datasets.len() >= 2 {
                diagnostics.push(DiagnosticPlot {
                    plot_type: "residuals_vs_fitted".to_string(),
                    title: "Residuals vs Fitted Values".to_string(),
                    description: "Checks for non-linearity and heteroscedasticity".to_string(),
                    purpose: "Regression diagnostics".to_string(),
                });

                diagnostics.push(DiagnosticPlot {
                    plot_type: "qqplot_residuals".to_string(),
                    title: "Q-Q Plot of Residuals".to_string(),
                    description: "Checks normality of residuals".to_string(),
                    purpose: "Regression diagnostics".to_string(),
                });
            }
        }

        // Autocorrelation plot for time series
        if Self::might_be_time_series(primary_data)? {
            diagnostics.push(DiagnosticPlot {
                plot_type: "acf".to_string(),
                title: "Autocorrelation Function".to_string(),
                description: "Shows correlation with lagged values".to_string(),
                purpose: "Time series diagnostics".to_string(),
            });
        }

        Ok(())
    }

    /// Assess normality of the data
    fn assess_normality(data: &[f64]) -> Result<NormalityAssessment, String> {
        if data.len() < 3 {
            return Ok(NormalityAssessment {
                is_normal: true,
                reason: "Insufficient data for normality test".to_string(),
            });
        }

        // Use centralized normality tests
        let normality_tests = crate::scientific::statistics::distributions::normality_tests::NormalityTests::comprehensive_normality_tests(data)?;

        // Consider data normal if majority of tests pass
        let normal_tests = normality_tests.iter().filter(|test| test.is_normal).count();
        let total_tests = normality_tests.len();
        let is_normal = normal_tests >= total_tests.div_ceil(2); // Majority rule

        let reason = if normality_tests.is_empty() {
            "No normality tests could be performed".to_string()
        } else {
            let test_results: Vec<String> = normality_tests.iter()
                .map(|test| format!("{}: {}", test.test_name, if test.is_normal { "normal" } else { "non-normal" }))
                .collect();
            test_results.join(", ")
        };

        Ok(NormalityAssessment {
            is_normal,
            reason,
        })
    }

    /// Check if data might represent a time series
    fn might_be_time_series(data: &[f64]) -> Result<bool, String> {
        if data.len() < 10 {
            return Ok(false);
        }

        // Check for trend using centralized TrendAnalysisEngine
        let trend_result = TrendAnalysisEngine::linear_trend(data)?;
        let slope = trend_result.parameters.first().copied().unwrap_or(0.0);

        // Check for autocorrelation
        let autocorrelations = CorrelationMethods::autocorrelation(data, 1)?;
        let autocorr = autocorrelations.first().copied().unwrap_or(0.0);

        // Consider it a time series if there's trend or autocorrelation
        Ok(slope.abs() > 0.01 || autocorr.abs() > 0.3)
    }

}

/// Assessment of data normality
#[derive(Debug, Clone)]
struct NormalityAssessment {
    is_normal: bool,
    reason: String,
}