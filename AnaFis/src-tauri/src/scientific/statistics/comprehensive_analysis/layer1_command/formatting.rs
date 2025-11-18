//! Output formatting and sanitization

use crate::scientific::statistics::types::{
    AnalysisOptions, ComprehensiveResult, DescriptiveStatsOutput, NormalityTestOutput, 
    VisualizationSuggestionsOutput, RecommendedPlot, ConfidenceIntervalsOutput, 
    OutlierAnalysisOutput, RobustStatisticsOutput, DistributionAnalysisOutput, 
    DistributionFitOutput, TransformationSuggestionOutput, TimeSeriesAnalysisOutput, 
    ReliabilityAnalysisOutput, UncertaintyPropagationOutput, QualityControlOutput, 
    SanitizationReport, AnalysisResults
};

/// Output formatting and sanitization utilities
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format and sanitize output for frontend consumption
    pub fn format_and_sanitize_output(
        results: AnalysisResults,
        recommendations: Vec<String>,
        options: &AnalysisOptions,
        sanitization_report: Option<SanitizationReport>,
    ) -> Result<ComprehensiveResult, String> {
        // Build the comprehensive result
        let comprehensive_result = ComprehensiveResult {
            descriptive_stats: results.descriptive_stats.as_ref().map(|stats| {
                DescriptiveStatsOutput {
                    count: stats.count,
                    mean: Self::sanitize_numeric_output(stats.mean, options.decimal_precision.unwrap_or(10)),
                    median: Self::sanitize_numeric_output(stats.median, options.decimal_precision.unwrap_or(10)),
                    std_dev: Self::sanitize_numeric_output(stats.std_dev, options.decimal_precision.unwrap_or(10)),
                    min: Self::sanitize_numeric_output(stats.min, options.decimal_precision.unwrap_or(10)),
                    max: Self::sanitize_numeric_output(stats.max, options.decimal_precision.unwrap_or(10)),
                    range: Self::sanitize_numeric_output(stats.range, options.decimal_precision.unwrap_or(10)),
                    q1: Self::sanitize_numeric_output(stats.q1, options.decimal_precision.unwrap_or(10)),
                    q3: Self::sanitize_numeric_output(stats.q3, options.decimal_precision.unwrap_or(10)),
                    iqr: Self::sanitize_numeric_output(stats.iqr, options.decimal_precision.unwrap_or(10)),
                }
            }),
            normality_test: results.normality_test.as_ref().and_then(|tests| {
                tests.first().map(|test| {
                    NormalityTestOutput {
                        test_name: test.test_name.clone(),
                        statistic: test.statistic.and_then(|s| Self::sanitize_numeric_output(s, options.decimal_precision.unwrap_or(10))),
                        p_value: test.p_value.and_then(|p| Self::sanitize_numeric_output(p, options.decimal_precision.unwrap_or(10))),
                        is_normal: test.is_normal,
                        method: test.test_name.clone(),
                    }
                })
            }),
            correlation_matrix: results.correlation_analysis.as_ref().map(|corr| {
                corr.matrix.iter().flatten().cloned().collect()
            }),
            data_quality: super::quality::QualityAssessor::compute_data_quality(&results),
            recommendations,
            visualization_suggestions: results.visualization_suggestions.map(|viz| {
                VisualizationSuggestionsOutput {
                    recommended_plots: viz.primary_plots.into_iter().chain(viz.secondary_plots).chain(viz.diagnostic_plots).map(|plot| {
                        RecommendedPlot {
                            plot_type: plot,
                            variables: vec![],
                            rationale: "Recommended visualization".to_string(),
                        }
                    }).collect(),
                }
            }),
            sanitization_report,
            // Advanced/specialized results (conditionally included)
            confidence_intervals: results.descriptive_stats
                .and_then(|stats| stats.confidence_intervals)
                .map(|ci| {
                    let mean_lo = Self::sanitize_numeric_output(ci.mean_ci.0, options.decimal_precision.unwrap_or(10));
                    let mean_hi = Self::sanitize_numeric_output(ci.mean_ci.1, options.decimal_precision.unwrap_or(10));
                    let median_lo = Self::sanitize_numeric_output(ci.median_ci.0, options.decimal_precision.unwrap_or(10));
                    let median_hi = Self::sanitize_numeric_output(ci.median_ci.1, options.decimal_precision.unwrap_or(10));
                    let std_lo = Self::sanitize_numeric_output(ci.std_dev_ci.0, options.decimal_precision.unwrap_or(10));
                    let std_hi = Self::sanitize_numeric_output(ci.std_dev_ci.1, options.decimal_precision.unwrap_or(10));

                    let mean_pair = if let (Some(a), Some(b)) = (mean_lo, mean_hi) { Some((a, b)) } else { None };
                    let median_pair = if let (Some(a), Some(b)) = (median_lo, median_hi) { Some((a, b)) } else { None };
                    let std_pair = if let (Some(a), Some(b)) = (std_lo, std_hi) { Some((a, b)) } else { None };

                    ConfidenceIntervalsOutput {
                        mean: mean_pair,
                        median: median_pair,
                        std_dev: std_pair,
                    }
                }),
            outlier_analysis: results.outlier_analysis.map(|outlier| {
                OutlierAnalysisOutput {
                    outlier_percentage: Self::sanitize_numeric_output(outlier.outlier_analysis.contamination_rate * 100.0, options.decimal_precision.unwrap_or(10)),
                    robust_statistics: RobustStatisticsOutput {
                        trimmed_mean: Self::sanitize_numeric_output(outlier.robust_statistics.trimmed_mean, options.decimal_precision.unwrap_or(10)),
                    },
                }
            }),
            distribution_analysis: results.distribution_analysis.as_ref().map(|dist| {
                DistributionAnalysisOutput {
                    transformation_suggestions: dist.transformation_results.clone().into_iter().take(2).map(|sugg| {
                        TransformationSuggestionOutput {
                            transformation: sugg.transformation,
                            improvement_score: Self::sanitize_numeric_output(sugg.improvement_score, options.decimal_precision.unwrap_or(10)),
                            rationale: "Distribution transformation suggestion".to_string(),
                        }
                    }).collect(),
                }
            }),
            distribution_fits: results.distribution_analysis.as_ref().map(|dist| {
                dist.distribution_fits.iter().map(|fit| {
                    DistributionFitOutput {
                        distribution_name: fit.distribution_name.clone(),
                        parameters: fit.parameters.clone(),
                        log_likelihood: Self::sanitize_numeric_output(fit.log_likelihood, options.decimal_precision.unwrap_or(10)).unwrap_or(fit.log_likelihood),
                        aic: Self::sanitize_numeric_output(fit.aic, options.decimal_precision.unwrap_or(10)).unwrap_or(fit.aic),
                        bic: Self::sanitize_numeric_output(fit.bic, options.decimal_precision.unwrap_or(10)).unwrap_or(fit.bic),
                        goodness_of_fit: Self::sanitize_numeric_output(fit.goodness_of_fit, options.decimal_precision.unwrap_or(10)),
                    }
                }).collect()
            }),
            best_fit_distribution: results.distribution_analysis.as_ref().and_then(|dist| {
                dist.best_fit_distribution.as_ref().map(|fit| {
                    DistributionFitOutput {
                        distribution_name: fit.distribution_name.clone(),
                        parameters: fit.parameters.clone(),
                        log_likelihood: Self::sanitize_numeric_output(fit.log_likelihood, options.decimal_precision.unwrap_or(10)).unwrap_or(fit.log_likelihood),
                        aic: Self::sanitize_numeric_output(fit.aic, options.decimal_precision.unwrap_or(10)).unwrap_or(fit.aic),
                        bic: Self::sanitize_numeric_output(fit.bic, options.decimal_precision.unwrap_or(10)).unwrap_or(fit.bic),
                        goodness_of_fit: Self::sanitize_numeric_output(fit.goodness_of_fit, options.decimal_precision.unwrap_or(10)),
                    }
                })
            }),
            time_series_analysis: results.time_series_analysis.map(|ts| {
                TimeSeriesAnalysisOutput {
                    trend_present: ts.trend_analysis.map(|t| t.trend_present).unwrap_or(false),
                    seasonality_present: ts.seasonality_present,
                }
            }),
            reliability_analysis: results.reliability_analysis.map(|rel| {
                ReliabilityAnalysisOutput {
                    cronbach_alpha: Self::sanitize_numeric_output(rel.cronbach_alpha, options.decimal_precision.unwrap_or(10)),
                }
            }),
            uncertainty_propagation: results.uncertainty_analysis.map(|unc| {
                UncertaintyPropagationOutput {
                    total_uncertainty: Some(unc.propagated_uncertainties.iter().sum::<f64>()),
                }
            }),
            quality_control: results.quality_control.map(|qc| {
                QualityControlOutput {
                    process_stable: qc.stability_assessment.is_stable,
                    cpk: Some(qc.capability_indices.cpk),
                }
            }),
        };

        Ok(comprehensive_result)
    }

    /// Sanitize numeric output to prevent NaN/Infinity
    pub fn sanitize_numeric_output(value: f64, precision: usize) -> Option<f64> {
        // Clamp precision to a safe range for f64 rounding
        let precision = precision.min(15);
        if value.is_finite() {
            // Round to reasonable precision to avoid floating point artifacts
            let factor = 10f64.powi(precision as i32);
            Some((value * factor).round() / factor)
        } else {
            None
        }
    }
}