use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::CorrelationHypothesisTestingEngine;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::distribution::moments;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::outliers::OutlierDetectionEngine;
use crate::scientific::statistics::types::AnalysisOptions;

#[derive(Debug, Clone)]
pub struct VisualizationSuggestions {
    pub recommended_plots: Vec<VisualizationSuggestion>,
    pub data_transformation_suggestions: Vec<DataTransformationSuggestion>,
}

#[derive(Debug, Clone)]
pub struct VisualizationSuggestion {
    pub plot_type: String,
    pub variables: Vec<usize>,
    pub rationale: String,
    pub transformation_needed: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DataTransformationSuggestion {
    pub transformation: String,
    pub purpose: String,
    pub expected_improvement: String,
}

/// Visualization Suggestion Coordinator
/// Coordinates chart and visualization recommendations
pub struct VisualizationSuggestionCoordinator;

impl VisualizationSuggestionCoordinator {
    /// Suggest appropriate visualizations for the data
    pub fn analyze(data: &[f64], datasets: Option<&[Vec<f64>]>) -> Result<VisualizationSuggestions, String> {
        let mut suggestions = Vec::new();

        // Always suggest histogram for distribution
        suggestions.push(VisualizationSuggestion {
            plot_type: "histogram".to_string(),
            variables: vec![0], // First variable
            rationale: "Shows the distribution shape and central tendency".to_string(),
            transformation_needed: None,
        });

        // Normality check - suggest Q-Q plot
        let normality_tests = if data.len() >= 3 {
            CorrelationHypothesisTestingEngine::normality_tests(data)?
        } else {
            Vec::new()
        };
        let is_normal = normality_tests.first().map(|t| t.is_normal).unwrap_or(false);

        if !is_normal {
            suggestions.push(VisualizationSuggestion {
                plot_type: "qqplot".to_string(),
                variables: vec![0],
                rationale: "Assesses normality and identifies deviations".to_string(),
                transformation_needed: None,
            });
        }

        // Box plot for outliers
        suggestions.push(VisualizationSuggestion {
            plot_type: "boxplot".to_string(),
            variables: vec![0],
            rationale: "Shows median, quartiles, and potential outliers".to_string(),
            transformation_needed: None,
        });

        // For multiple datasets, suggest correlation plots
        if let Some(multi_datasets) = datasets {
            if multi_datasets.len() >= 2 {
                suggestions.push(VisualizationSuggestion {
                    plot_type: "scatter".to_string(),
                    variables: vec![0, 1], // First two variables
                    rationale: "Shows relationship between two continuous variables".to_string(),
                    transformation_needed: None,
                });

                if multi_datasets.len() > 2 {
                    suggestions.push(VisualizationSuggestion {
                        plot_type: "correlation_heatmap".to_string(),
                        variables: (0..multi_datasets.len()).collect(),
                        rationale: "Shows correlations between all variable pairs".to_string(),
                        transformation_needed: None,
                    });
                }
            }
        }

        // Check for transformations that might help visualization
        let transformation_suggestions = Self::suggest_visualization_transformations(data)?;

        Ok(VisualizationSuggestions {
            recommended_plots: suggestions,
            data_transformation_suggestions: transformation_suggestions,
        })
    }

    /// Suggest data transformations for better visualization
    fn suggest_visualization_transformations(data: &[f64]) -> Result<Vec<DataTransformationSuggestion>, String> {
        let mut suggestions = Vec::new();

        // Check for skewness
        let (_, _, skewness, _) = moments::moments(data)?;

        if skewness.abs() > 1.0 {
            if data.iter().all(|&x| x > 0.0) {
                suggestions.push(DataTransformationSuggestion {
                    transformation: "log".to_string(),
                    purpose: "Reduce skewness for better visualization".to_string(),
                    expected_improvement: "Makes distribution more symmetric".to_string(),
                });
            }

            if data.iter().all(|&x| x >= 0.0) {
                suggestions.push(DataTransformationSuggestion {
                    transformation: "sqrt".to_string(),
                    purpose: "Reduce positive skewness".to_string(),
                    expected_improvement: "Stabilizes variance and reduces skewness".to_string(),
                });
            }
        }

        // Check for outliers that might affect scale
        let default_options = AnalysisOptions::default();
        let outlier_analysis = OutlierDetectionEngine::detect_outliers(data, &default_options)?;
        if outlier_analysis.outlier_percentage > 10.0 {
            suggestions.push(DataTransformationSuggestion {
                transformation: "robust_scale".to_string(),
                purpose: "Handle outliers for better scale visualization".to_string(),
                expected_improvement: "Reduces impact of extreme values on plot scales".to_string(),
            });
        }

        Ok(suggestions)
    }
}
