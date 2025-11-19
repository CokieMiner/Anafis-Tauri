//! Layer 1: Command Interface & Orchestration
//!
//! This is the main entry point for the comprehensive statistical analysis system.
//! It handles input validation, analysis orchestration, and output formatting.

// Import types from the parent module
use crate::scientific::statistics::types::AnalysisOptions;
use crate::scientific::statistics::types::{ComprehensiveResult, AnalysisResults};

// Import modular components
use super::validation::InputValidator;
use super::detection::PatternDetector;
use super::orchestration::AnalysisOrchestrator;
use super::formatting::OutputFormatter;

// Import RandomSampling for RNG creation
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::RandomSampling;

/// Main command interface for comprehensive statistical analysis
pub struct ComprehensiveAnalysisCommand;

impl ComprehensiveAnalysisCommand {
    /// Perform comprehensive statistical analysis on datasets
    pub fn execute(
        datasets: Vec<Vec<f64>>,
        options: AnalysisOptions,
    ) -> Result<ComprehensiveResult, String> {
        // Input validation and sanitization
        let (sanitized_datasets, sanitization_report) = InputValidator::validate_and_sanitize_input(&datasets, &options)
            .map_err(|e| e.to_string())?;

        // Initialize random number generator for reproducible results
        let mut rng = RandomSampling::create_rng(options.random_seed.unwrap_or(42));

        // Detect which analyses are needed
        let required_analyses = PatternDetector::detect_required_analyses(&sanitized_datasets, &options)?;

        // Orchestrate the analysis pipeline
        let analysis_results = AnalysisOrchestrator::orchestrate_analysis_pipeline(
            &sanitized_datasets,
            &required_analyses,
            &options,
            &mut rng,
        )?;

        // Generate recommendations based on results
        let recommendations = Self::generate_recommendations(&analysis_results, &options)?;

        // Format and sanitize output
        OutputFormatter::format_and_sanitize_output(analysis_results, recommendations, &options, Some(sanitization_report))
    }

    /// Generate recommendations based on analysis results
    fn generate_recommendations(
        results: &AnalysisResults,
        _options: &AnalysisOptions,
    ) -> Result<Vec<String>, String> {
        let mut recommendations = Vec::new();

        // Sample size recommendations
        if let Some(desc_stats) = &results.descriptive_stats {
            if desc_stats.count < 30 {
                recommendations.push(format!(
                    "Sample size (n={}) is small. Consider collecting more data for reliable results.",
                    desc_stats.count
                ));
            }
        }

        // Normality recommendations
        if let Some(normality_tests) = &results.normality_test {
            if let Some(test) = normality_tests.first() {
                if !test.is_normal {
                    recommendations.push(
                        "Data does not appear normally distributed. Consider non-parametric methods or data transformation.".to_string()
                    );
                }
            }
        }

        // Outlier recommendations
        if let Some(outlier_analysis) = &results.outlier_analysis {
            let outlier_percentage = outlier_analysis.outlier_analysis.contamination_rate * 100.0;
            if outlier_percentage > 5.0 {
                recommendations.push(format!(
                    "{:.1}% of data points identified as outliers. Consider investigating these values.",
                    outlier_percentage
                ));
            }
        }

        // Correlation recommendations
        if let Some(corr_analysis) = &results.correlation_analysis {
            let strong_correlations: Vec<_> = corr_analysis.significance_tests.iter()
                .filter(|c| c.p_value < 0.05 && c.correlation.abs() > 0.7)
                .collect();

            if !strong_correlations.is_empty() {
                recommendations.push(format!(
                    "Found {} significant strong correlations between variables. These relationships may be important for your analysis.",
                    strong_correlations.len()
                ));
            }
        }

        // Distribution recommendations
        if let Some(dist_analysis) = &results.distribution_analysis {
            if let Some(best_fit) = &dist_analysis.best_fit_distribution {
                if best_fit.distribution_name != "normal" {
                    recommendations.push(format!(
                        "Data best fits a {} distribution rather than normal. Consider using specialized methods for this distribution.",
                        best_fit.distribution_name
                    ));
                }
            }
        }

        // Quality control recommendations
        if let Some(qc_analysis) = &results.quality_control {
            if qc_analysis.stability_assessment != "Stable" {
                recommendations.push(format!(
                    "Process appears unstable: {}. Consider investigating process changes.",
                    qc_analysis.stability_assessment
                ));
            }
        }

        // Default recommendations if none generated
        if recommendations.is_empty() {
            recommendations.push("Data appears suitable for standard statistical analyses.".to_string());
            recommendations.push("Consider visualizing your data to better understand patterns and relationships.".to_string());
        }

        Ok(recommendations)
    }
}
