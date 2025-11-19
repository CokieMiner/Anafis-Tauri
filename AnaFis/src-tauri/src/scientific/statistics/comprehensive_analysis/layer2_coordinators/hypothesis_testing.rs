//! Hypothesis Testing Coordinator
//!
//! This module coordinates hypothesis testing and power analysis operations.

use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::hypothesis_testing::HypothesisTestingEngine;
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats;
use crate::scientific::statistics::types::*;
use crate::scientific::statistics::comprehensive_analysis::traits::ProgressCallback;

/// Hypothesis Testing Coordinator
/// Coordinates hypothesis testing and power analysis operations
pub struct HypothesisTestingCoordinator;

impl HypothesisTestingCoordinator {
    /// Perform hypothesis testing on datasets
    pub fn analyze(
        datasets: &[Vec<f64>],
        _options: &AnalysisOptions,
        progress_callback: &dyn ProgressCallback,
    ) -> Result<HypothesisTestingResult, String> {
        progress_callback.report_progress(0, 100, "Starting hypothesis testing analysis...");

        let mut t_test_results = Vec::new();
        let mut anova_results = Vec::new();
        let mut two_way_anova_results = Vec::new();
        let mut repeated_measures_anova_results = Vec::new();
        let chi_square_results = Vec::new();

        // Perform t-tests if we have appropriate data
        if !datasets.is_empty() {
            progress_callback.report_progress(10, 100, "Performing t-tests...");

            // One-sample t-test (compare each dataset to zero or specified mean)
            for dataset in datasets.iter() {
                if let Ok(result) = HypothesisTestingEngine::one_sample_t_test(dataset, 0.0) {
                    t_test_results.push(result);
                }
            }

            // Two-sample t-tests for pairs of datasets
            if datasets.len() >= 2 {
                for i in 0..datasets.len() {
                    for j in (i + 1)..datasets.len() {
                        if let Ok(result) = HypothesisTestingEngine::two_sample_t_test(&datasets[i], &datasets[j], true) {
                            t_test_results.push(result);
                        }
                        if let Ok(result) = HypothesisTestingEngine::welch_t_test(&datasets[i], &datasets[j]) {
                            t_test_results.push(result);
                        }
                    }
                }
            }

            // Paired t-test if datasets have equal length (assuming paired observations)
            if datasets.len() == 2 && datasets[0].len() == datasets[1].len() {
                if let Ok(result) = HypothesisTestingEngine::paired_t_test(&datasets[0], &datasets[1]) {
                    t_test_results.push(result);
                }
            }
        }

        // Perform ANOVA if we have multiple groups
        if datasets.len() >= 3 {
            progress_callback.report_progress(40, 100, "Performing ANOVA tests...");

            // One-way ANOVA
            let groups: Vec<&[f64]> = datasets.iter().map(|d| d.as_slice()).collect();
            if let Ok(result) = HypothesisTestingEngine::one_way_anova(&groups) {
                anova_results.push(result);
            }

            // Check for two-way ANOVA: assume first half of datasets are one factor, second half another
            if datasets.len() >= 4 && datasets.len().is_multiple_of(2) {
                let half = datasets.len() / 2;
                let factor1_levels: Vec<usize> = (0..half).chain(0..half).collect();
                let factor2_levels: Vec<usize> = (0..half).map(|_| 0).chain((0..half).map(|_| 1)).collect();

                if let Ok(result) = HypothesisTestingEngine::two_way_anova(datasets, &factor1_levels, &factor2_levels) {
                    two_way_anova_results.push(result);
                }
            }

            // Check for repeated measures ANOVA: assume all datasets have same length and represent time points
            if datasets.len() >= 3 && datasets.iter().all(|d| d.len() == datasets[0].len()) && datasets[0].len() >= 3 {
                // Transpose data: subjects become rows, time points become columns
                let n_subjects = datasets[0].len();
                let n_time_points = datasets.len();
                let mut subjects_data = vec![vec![0.0; n_time_points]; n_subjects];

                for (t, dataset) in datasets.iter().enumerate() {
                    for (s, &value) in dataset.iter().enumerate() {
                        subjects_data[s][t] = value;
                    }
                }

                if let Ok(result) = HypothesisTestingEngine::repeated_measures_anova(&subjects_data) {
                    repeated_measures_anova_results.push(result);
                }
            }
        }

        // Perform chi-square tests if appropriate
        // For now, skip as we need categorical data or contingency tables

        progress_callback.report_progress(70, 100, "Hypothesis testing completed");

        Ok(HypothesisTestingResult {
            t_test_results,
            anova_results,
            two_way_anova_results,
            repeated_measures_anova_results,
            chi_square_results,
        })
    }
}

/// Power Analysis Coordinator
/// Coordinates power analysis operations
pub struct PowerAnalysisCoordinator;

impl PowerAnalysisCoordinator {
    /// Perform power analysis
    pub fn analyze(
        datasets: &[Vec<f64>],
        _options: &AnalysisOptions,
        progress_callback: &dyn ProgressCallback,
    ) -> Result<PowerAnalysisInternalResult, String> {
        progress_callback.report_progress(0, 100, "Starting power analysis...");

        let mut power_calculations = Vec::new();
        let mut power_curves = Vec::new();
        let mut recommendations = Vec::new();

        // Calculate power for observed effects
        if !datasets.is_empty() {
            progress_callback.report_progress(20, 100, "Calculating statistical power...");

            // Estimate effect sizes from data
            for i in 0..datasets.len() {
                for j in (i + 1)..datasets.len().min(i + 2) { // Limit to avoid too many calculations
                    let mean1 = datasets[i].iter().sum::<f64>() / datasets[i].len() as f64;
                    let mean2 = datasets[j].iter().sum::<f64>() / datasets[j].len() as f64;
                    let delta = (mean1 - mean2).abs();

                    // Estimate pooled standard deviation
                    let var1 = datasets[i].iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (datasets[i].len() - 1) as f64;
                    let var2 = datasets[j].iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (datasets[j].len() - 1) as f64;
                    let sigma = ((var1 + var2) / 2.0).sqrt();

                    let n = (datasets[i].len() + datasets[j].len()) / 2;

                    if let Ok(power) = UnifiedStats::power_t_test(delta, sigma, n, 0.05, "two-sided") {
                        power_calculations.push(PowerAnalysisResult {
                            test_type: "Two-sample t-test".to_string(),
                            power,
                            effect_size: delta / sigma,
                            sample_size: n,
                            alpha: 0.05,
                            alternative: "two-sided".to_string(),
                            method: "Approximation".to_string(),
                        });
                    }
                }
            }
        }

        // Generate power curves
        progress_callback.report_progress(60, 100, "Generating power curves...");

        let effect_sizes = [0.2, 0.5, 0.8];
        let sample_sizes: Vec<usize> = (10..=100).step_by(10).collect();

        for &effect_size in &effect_sizes {
            let mut curve_data = Vec::new();
            for &n in &sample_sizes {
                if let Ok(power) = UnifiedStats::power_t_test(effect_size, 1.0, n, 0.05, "two-sided") {
                    curve_data.push((n, power));
                }
            }
            if !curve_data.is_empty() {
                power_curves.push(PowerCurveResult {
                    test_type: "Two-sample t-test".to_string(),
                    effect_size,
                    alpha: 0.05,
                    alternative: "two-sided".to_string(),
                    curve_data,
                });
            }
        }

        // Generate recommendations
        recommendations.push("For small effect sizes (d < 0.3), ensure adequate sample size (> 100 per group)".to_string());
        recommendations.push("Power analysis should be conducted before data collection".to_string());
        recommendations.push("Consider using 80% power as a minimum threshold for adequate study design".to_string());

        progress_callback.report_progress(100, 100, "Power analysis completed");

        Ok(PowerAnalysisInternalResult {
            power_calculations,
            power_curves,
            recommendations,
        })
    }
}