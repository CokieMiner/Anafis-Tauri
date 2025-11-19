//! Analysis pipeline orchestration and coordination

use crate::scientific::statistics::types::{AnalysisOptions, AnalysisResults, RequiredAnalyses};
use super::super::descriptive_stats::DescriptiveStatsCoordinator;
use super::super::layer3_algorithms::correlation::hypothesis_testing::CorrelationHypothesisTestingEngine;
use super::super::layer2_coordinators::{
    distribution_analysis::DistributionAnalysisCoordinator,
    outlier_analysis::OutlierAnalysisCoordinator,
    correlation_analysis::CorrelationAnalysisCoordinator,
    uncertainty_propagation::UncertaintyPropagationCoordinator,
    time_series_analysis::TimeSeriesAnalysisCoordinator,
    quality_control::QualityControlCoordinator,
    reliability_analysis::ReliabilityAnalysisCoordinator,
    visualization_suggestion::VisualizationSuggestionCoordinator,
    hypothesis_testing::{HypothesisTestingCoordinator, PowerAnalysisCoordinator},
};
use rand_pcg::Pcg64;

/// Analysis pipeline orchestration
pub struct AnalysisOrchestrator;

impl AnalysisOrchestrator {
    /// Orchestrate the analysis pipeline
    pub fn orchestrate_analysis_pipeline(
        datasets: &[Vec<f64>],
        required: &RequiredAnalyses,
        options: &AnalysisOptions,
        rng: &mut Pcg64,
    ) -> Result<AnalysisResults, String> {
        let mut results = AnalysisResults::default();

        // Always analyze first dataset for core statistics
        let primary_dataset = &datasets[0];

        // Prepare uncertainty data if available
        let (expanded_uncertainties, expanded_confidence_levels) = if let Some(uncertainties) = &options.uncertainties {
            if uncertainties.len() == datasets.len() {
                // One uncertainty per dataset - expand to per-data-point
                let expanded_uncertainties: Vec<f64> = uncertainties.iter()
                    .zip(datasets.iter())
                    .flat_map(|(uncertainty, dataset)| {
                        std::iter::repeat_n(*uncertainty, dataset.len())
                    })
                    .collect();
                
                // Handle confidence levels similarly
                let expanded_confidence_levels = if let Some(conf_levels) = &options.uncertainty_confidences {
                    if conf_levels.len() == datasets.len() {
                        // One confidence level per dataset - expand to per-data-point
                        conf_levels.iter()
                            .zip(datasets.iter())
                            .flat_map(|(conf, dataset)| {
                                std::iter::repeat_n(*conf, dataset.len())
                            })
                            .collect()
                    } else if conf_levels.len() == primary_dataset.len() {
                        // Already per-data-point
                        conf_levels.clone()
                    } else {
                        // Use default
                        vec![options.statistical_confidence_level.unwrap_or(0.95); primary_dataset.len()]
                    }
                } else {
                    vec![options.statistical_confidence_level.unwrap_or(0.95); primary_dataset.len()]
                };
                
                (Some(expanded_uncertainties), Some(expanded_confidence_levels))
            } else if uncertainties.len() == primary_dataset.len() {
                // Already per-data-point for primary dataset
                let confidence_levels = if let Some(conf_levels) = &options.uncertainty_confidences {
                    if conf_levels.len() == primary_dataset.len() {
                        conf_levels.clone()
                    } else {
                        vec![options.statistical_confidence_level.unwrap_or(0.95); primary_dataset.len()]
                    }
                } else {
                    vec![options.statistical_confidence_level.unwrap_or(0.95); primary_dataset.len()]
                };
                
                (Some(uncertainties.clone()), Some(confidence_levels))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Descriptive statistics
        if required.descriptive_stats {
            results.descriptive_stats = Some(DescriptiveStatsCoordinator::analyze_with_uncertainties(
                primary_dataset,
                expanded_uncertainties.as_deref(),
                expanded_confidence_levels.as_deref(),
                options.bootstrap_samples,
                rng,
            )?);
        }

        // Normality test
        if required.normality_test {
            results.normality_test = Some(CorrelationHypothesisTestingEngine::normality_tests(primary_dataset)?);
        }

        // Distribution analysis
        if required.distribution_analysis {
            let dist_result = DistributionAnalysisCoordinator::analyze(primary_dataset)?;
            results.distribution_analysis = Some(crate::scientific::statistics::types::analysis::DistributionAnalysis {
                distribution_fits: dist_result.distribution_fits,
                best_fit_distribution: dist_result.best_fit_distribution,
                recommended_transformations: dist_result.transformation_suggestions.iter().map(|t| t.transformation.clone()).collect(),
                transformation_results: dist_result.transformation_suggestions.into_iter().map(|t| crate::scientific::statistics::types::analysis::TransformationResult {
                    transformation: t.transformation,
                    improvement_score: t.improvement_score,
                    transformed_data: vec![], // Not available from coordinator
                }).collect(),
            });
        }

        // Outlier analysis
        if required.outlier_analysis {
            results.outlier_analysis = Some(OutlierAnalysisCoordinator::analyze_with_uncertainties(
                primary_dataset,
                expanded_uncertainties.as_deref(),
                expanded_confidence_levels.as_deref(),
                options,
            )?);
        }

        // Correlation analysis (if multiple datasets)
        if required.correlation_analysis && datasets.len() > 1 {
            let corr_result = CorrelationAnalysisCoordinator::analyze(
                datasets,
                options,
                rng,
            )?;
            results.correlation_analysis = Some(crate::scientific::statistics::types::correlation::CorrelationAnalysis {
                matrix: corr_result.correlation_matrix.outer_iter().map(|row| row.to_vec()).collect(),
                methods: vec![corr_result.method],
                significance_tests: corr_result.correlation_tests,
            });
        }

        // Uncertainty propagation
        if required.uncertainty_propagation {
            if let (Some(uncertainties), Some(confidence_levels)) = (&expanded_uncertainties, &expanded_confidence_levels) {
                let unc_result = UncertaintyPropagationCoordinator::analyze(
                    primary_dataset,
                    Some(uncertainties),
                    Some(confidence_levels),
                    options.statistical_confidence_level,
                )?;
                results.uncertainty_analysis = Some(crate::scientific::statistics::types::analysis::UncertaintyAnalysis {
                    propagated_uncertainties: vec![unc_result.uncertainty_contributions.total_uncertainty],
                    covariance_matrix: vec![], // Not available
                    sensitivity_coefficients: vec![], // Not available
                });
            }
        }

        // Time series analysis
        if required.time_series_analysis {
            let ts_result = TimeSeriesAnalysisCoordinator::analyze(primary_dataset)?;
            results.time_series_analysis = Some(crate::scientific::statistics::types::analysis::TimeSeriesAnalysisResult {
                components: None, // Not available from coordinator
                trend_analysis: ts_result.trend_analysis,
                stationarity: None, // Not available from coordinator
                forecast: None, // Not available from coordinator
                seasonality_present: ts_result.seasonality_analysis.map(|s| s.seasonality_present).unwrap_or(false),
            });
        }

        // Quality control
        if required.quality_control {
            let qc_result = QualityControlCoordinator::analyze(
                primary_dataset,
                options.lsl,
                options.usl,
            )?;
            results.quality_control = Some(crate::scientific::statistics::types::analysis::QualityControlAnalysis {
                control_limits: crate::scientific::statistics::types::analysis::ControlLimits {
                    x_bar_upper: qc_result.control_limits.upper_control_limit,
                    x_bar_lower: qc_result.control_limits.lower_control_limit,
                    range_upper: qc_result.control_limits.upper_control_limit - qc_result.control_limits.center_line,
                    range_lower: qc_result.control_limits.center_line - qc_result.control_limits.lower_control_limit,
                },
                capability_indices: qc_result.process_capability.as_ref().map(|pc| crate::scientific::statistics::types::analysis::CapabilityIndices {
                    cp: pc.cp,
                    cpk: pc.cpk,
                    pp: pc.cp, // Same as cp for now
                    ppk: pc.cpk, // Same as cpk for now
                }).unwrap_or(crate::scientific::statistics::types::analysis::CapabilityIndices {
                    cp: 1.0,
                    cpk: 1.0,
                    pp: 1.0,
                    ppk: 1.0,
                }),
                stability_assessment: crate::scientific::statistics::types::analysis::StabilityAssessment {
                    is_stable: qc_result.stability_assessment == "Stable",
                    violations: if qc_result.stability_assessment == "Stable" { vec![] } else { vec![qc_result.stability_assessment] },
                },
            });
        }

        // Reliability analysis
        if required.reliability_analysis && datasets.len() >= 3 {
            results.reliability_analysis = Some(ReliabilityAnalysisCoordinator::analyze(datasets)?);
        }

        // Visualization suggestions
        if required.visualization_suggestions {
            let multi_datasets = if datasets.len() > 1 { Some(datasets) } else { None };
            let viz_result = VisualizationSuggestionCoordinator::analyze(
                primary_dataset,
                multi_datasets,
            )?;
            results.visualization_suggestions = Some(crate::scientific::statistics::types::analysis::VisualizationSuggestions {
                primary_plots: viz_result.recommended_plots.iter().map(|p| p.plot_type.clone()).collect(),
                secondary_plots: vec![], // Not categorized in coordinator
                diagnostic_plots: vec![], // Not categorized in coordinator
            });
        }

        // Hypothesis testing
        if required.hypothesis_testing {
            let ht_result = HypothesisTestingCoordinator::analyze(
                datasets,
                options,
                &crate::scientific::statistics::comprehensive_analysis::traits::NoOpProgressCallback,
            )?;
            results.hypothesis_testing = Some(ht_result);
        }

        // Power analysis
        if required.power_analysis {
            let pa_result = PowerAnalysisCoordinator::analyze(
                datasets,
                options,
                &crate::scientific::statistics::comprehensive_analysis::traits::NoOpProgressCallback,
            )?;
            results.power_analysis = Some(pa_result);
        }

        Ok(results)
    }
}