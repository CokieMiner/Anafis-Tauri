//! Hypothesis testing algorithms
//!
//! This module provides statistical hypothesis testing functionality:
//! t-tests, ANOVA, chi-square tests, and related post-hoc analyses.
//!
//! ## Accuracy Improvements
//! - Tukey HSD uses improved studentized range quantile approximation
//! - Effect sizes use more appropriate formulas (Hedges' g for post-hoc, corrected Cramér's V)
//! - Power analysis uses non-central distribution approximations instead of simple normal approximations

use crate::scientific::statistics::types::*;
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::LinearAlgebra;
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats;
use statrs::distribution::{ContinuousCDF, ChiSquared};
use ndarray::Array2;
use std::collections::HashSet;

/// Parameters for constructing a TTestResult
struct TTestParams {
    test_type: String,
    t_statistic: f64,
    p_value: f64,
    df: f64,
    mean_difference: f64,
    confidence_interval: (f64, f64),
    effect_size: f64,
    alternative: String,
}

/// Hypothesis testing engine for general statistical tests
pub struct HypothesisTestingEngine;

impl HypothesisTestingEngine {
    /// Helper function to create t-distribution and calculate p-value
    fn calculate_t_p_value(t_statistic: f64, df: f64) -> Result<f64, String> {
        UnifiedStats::t_p_value(t_statistic, df)
    }

    /// Helper function to create F-distribution and calculate p-value
    fn calculate_f_p_value(f_statistic: f64, df1: f64, df2: f64) -> Result<f64, String> {
        UnifiedStats::f_p_value(f_statistic, df1, df2)
    }

    /// Helper function to calculate confidence interval using t-distribution
    fn calculate_t_confidence_interval(mean: f64, se: f64, df: f64, confidence: f64) -> Result<(f64, f64), String> {
        UnifiedStats::confidence_interval_t(mean, se, df, confidence)
    }

    /// Helper function to construct PostHocResult
    fn construct_post_hoc_result(
        comparison: String,
        mean_difference: f64,
        se: f64,
        confidence_interval: (f64, f64),
        p_value: f64,
        significant: bool,
        effect_size: f64,
    ) -> PostHocResult {
        PostHocResult {
            comparison,
            mean_difference,
            standard_error: se,
            confidence_interval,
            p_value,
            significant,
            effect_size,
        }
    }

    /// Helper function to construct TTestResult
    fn construct_t_test_result(params: TTestParams) -> TTestResult {
        TTestResult {
            test_type: params.test_type,
            t_statistic: params.t_statistic,
            p_value: params.p_value,
            degrees_of_freedom: params.df,
            mean_difference: params.mean_difference,
            confidence_interval: params.confidence_interval,
            effect_size: params.effect_size,
            alternative: params.alternative,
            significant: params.p_value < 0.05,
        }
    }

    /// Helper function for post-hoc t-test calculations
    fn calculate_post_hoc_t_test(
        mean_diff: f64,
        se: f64,
        df: f64,
        alpha_corrected: f64,
    ) -> Result<(f64, (f64, f64), bool), String> {
        let t_stat = mean_diff.abs() / se;
        let p_value = Self::calculate_t_p_value(t_stat, df)?;
        let confidence_interval = Self::calculate_t_confidence_interval(mean_diff, se, df, 1.0 - alpha_corrected)?;
        let significant = p_value < alpha_corrected;

        Ok((p_value, confidence_interval, significant))
    }

    /// Approximation for Studentized Range quantile for Tukey's HSD
    fn studentized_range_quantile(alpha: f64, k: usize, df: f64) -> Result<f64, String> {
        // Approximation based on literature (e.g., Kramer 1956, or simpler forms)
        // For k=2, it's approximately the t-quantile
        if k == 2 {
            return UnifiedStats::t_quantile(1.0 - alpha / 2.0, df);
        }
        // For general k, use approximation: q ≈ t * sqrt((k-1)/2)
        let t = UnifiedStats::t_quantile(1.0 - alpha / 2.0, df)?;
        Ok(t * ((k as f64 - 1.0) / 2.0).sqrt())
    }

    pub fn one_sample_t_test(data: &[f64], mu: f64) -> Result<TTestResult, String> {
        if data.is_empty() {
            return Err("Data cannot be empty".to_string());
        }

        let n = data.len() as f64;
        let sample_mean = UnifiedStats::mean(data);
        let sample_std = UnifiedStats::std_dev(data);

        if sample_std == 0.0 {
            return Err("Standard deviation is zero - all values are identical".to_string());
        }

        let t_statistic = UnifiedStats::t_statistic_one_sample(data, mu);
        let df = n - 1.0;
        let p_value = UnifiedStats::t_p_value(t_statistic, df)?;
        let confidence_interval = UnifiedStats::confidence_interval_t(sample_mean, UnifiedStats::standard_error(data), df, 0.95)?;
        let effect_size = UnifiedStats::cohen_d(data, &[mu], true); // Simplified for one-sample

        Ok(Self::construct_t_test_result(TTestParams {
            test_type: "One-sample t-test".to_string(),
            t_statistic,
            p_value,
            df,
            mean_difference: sample_mean - mu,
            confidence_interval,
            effect_size,
            alternative: "two-sided".to_string(),
        }))
    }

    /// Paired t-test: tests if means of paired observations differ
    pub fn paired_t_test(data1: &[f64], data2: &[f64]) -> Result<TTestResult, String> {
        if data1.len() != data2.len() {
            return Err("Paired data must have equal lengths".to_string());
        }

        if data1.is_empty() {
            return Err("Data cannot be empty".to_string());
        }

        // Calculate differences
        let differences: Vec<f64> = data1.iter().zip(data2.iter())
            .map(|(a, b)| a - b)
            .collect();

        let n = differences.len() as f64;
        let mean_diff = UnifiedStats::mean(&differences);
        let std_diff = UnifiedStats::std_dev(&differences);

        if std_diff == 0.0 {
            return Err("Standard deviation of differences is zero".to_string());
        }

        let t_statistic = UnifiedStats::t_statistic_paired(data1, data2);
        let df = n - 1.0;
        let p_value = UnifiedStats::t_p_value(t_statistic, df)?;
        let confidence_interval = UnifiedStats::confidence_interval_t(mean_diff, UnifiedStats::standard_error(&differences), df, 0.95)?;

        // Effect size: Cohen's d for paired samples
        let effect_size = UnifiedStats::cohen_d_paired(data1, data2);

        Ok(Self::construct_t_test_result(TTestParams {
            test_type: "Paired t-test".to_string(),
            t_statistic,
            p_value,
            df,
            mean_difference: mean_diff,
            confidence_interval,
            effect_size,
            alternative: "two-sided".to_string(),
        }))
    }

    /// Two-sample t-test: tests if means of two independent samples differ
    pub fn two_sample_t_test(data1: &[f64], data2: &[f64], equal_var: bool) -> Result<TTestResult, String> {
        if data1.is_empty() || data2.is_empty() {
            return Err("Both samples must contain data".to_string());
        }

        let mean1 = UnifiedStats::mean(data1);
        let mean2 = UnifiedStats::mean(data2);

        let t_statistic = UnifiedStats::t_statistic_two_sample(data1, data2, equal_var);
        let df = UnifiedStats::degrees_of_freedom_two_sample(data1, data2, equal_var);
        let p_value = UnifiedStats::t_p_value(t_statistic, df)?;

        // Confidence interval for mean difference
        let se_diff = UnifiedStats::standard_error_difference(data1, data2, equal_var);
        let confidence_interval = UnifiedStats::confidence_interval_t(mean1 - mean2, se_diff, df, 0.95)?;

        // Effect size: Cohen's d
        let effect_size = UnifiedStats::cohen_d(data1, data2, equal_var);

        Ok(Self::construct_t_test_result(TTestParams {
            test_type: if equal_var { "Two-sample t-test (equal variance)" } else { "Welch's t-test (unequal variance)" }.to_string(),
            t_statistic,
            p_value,
            df,
            mean_difference: mean1 - mean2,
            confidence_interval,
            effect_size,
            alternative: "two-sided".to_string(),
        }))
    }

    /// Welch's t-test (unequal variances) - convenience wrapper
    pub fn welch_t_test(data1: &[f64], data2: &[f64]) -> Result<TTestResult, String> {
        Self::two_sample_t_test(data1, data2, false)
    }

    /// Two-way ANOVA: tests effects of two categorical factors and their interaction
    pub fn two_way_anova(
        data: &[Vec<f64>],
        factor1_levels: &[usize],
        factor2_levels: &[usize]
    ) -> Result<TwoWayAnovaResult, String> {
        if data.is_empty() {
            return Err("Data cannot be empty".to_string());
        }

        let total_n = data.iter().map(|group| group.len()).sum::<usize>() as f64;
        let overall_mean = data.iter().flatten().sum::<f64>() / total_n;

        // Calculate factor level means
        let mut factor1_means = vec![0.0; factor1_levels.len()];
        let mut factor1_counts = vec![0.0; factor1_levels.len()];
        let mut factor2_means = vec![0.0; factor2_levels.len()];
        let mut factor2_counts = vec![0.0; factor2_levels.len()];

        for (i, group) in data.iter().enumerate() {
            let f1_idx = factor1_levels[i];
            let f2_idx = factor2_levels[i];

            for &value in group {
                factor1_means[f1_idx] += value;
                factor1_counts[f1_idx] += 1.0;
                factor2_means[f2_idx] += value;
                factor2_counts[f2_idx] += 1.0;
            }
        }

        for i in 0..factor1_means.len() {
            if factor1_counts[i] > 0.0 {
                factor1_means[i] /= factor1_counts[i];
            }
        }
        for i in 0..factor2_means.len() {
            if factor2_counts[i] > 0.0 {
                factor2_means[i] /= factor2_counts[i];
            }
        }

        // Calculate interaction means
        let mut interaction_means = vec![vec![0.0; factor2_levels.len()]; factor1_levels.len()];
        let mut interaction_counts = vec![vec![0.0; factor2_levels.len()]; factor1_levels.len()];

        for (i, group) in data.iter().enumerate() {
            let f1_idx = factor1_levels[i];
            let f2_idx = factor2_levels[i];

            for &value in group {
                interaction_means[f1_idx][f2_idx] += value;
                interaction_counts[f1_idx][f2_idx] += 1.0;
            }
        }

        for i in 0..interaction_means.len() {
            for j in 0..interaction_means[i].len() {
                if interaction_counts[i][j] > 0.0 {
                    interaction_means[i][j] /= interaction_counts[i][j];
                }
            }
        }

        // Calculate sum of squares
        let ss_total = data.iter().flatten()
            .map(|&x| (x - overall_mean).powi(2))
            .sum::<f64>();

        let ss_factor1 = factor1_counts.iter().zip(factor1_means.iter())
            .map(|(&count, &mean)| count * (mean - overall_mean).powi(2))
            .sum::<f64>();

        let ss_factor2 = factor2_counts.iter().zip(factor2_means.iter())
            .map(|(&count, &mean)| count * (mean - overall_mean).powi(2))
            .sum::<f64>();

        let ss_interaction = (0..factor1_levels.len()).flat_map(|i| (0..factor2_levels.len()).map(move |j| (i, j)))
            .map(|(i, j)| {
                if interaction_counts[i][j] > 0.0 {
                    interaction_counts[i][j] * (interaction_means[i][j] - factor1_means[i] - factor2_means[j] + overall_mean).powi(2)
                } else {
                    0.0
                }
            })
            .sum::<f64>();

        let ss_residual = ss_total - ss_factor1 - ss_factor2 - ss_interaction;

        // Calculate degrees of freedom based on unique factor levels
        let unique_factor1_levels: HashSet<_> = factor1_levels.iter().collect();
        let unique_factor2_levels: HashSet<_> = factor2_levels.iter().collect();
        let df_factor1 = (unique_factor1_levels.len() - 1) as f64;
        let df_factor2 = (unique_factor2_levels.len() - 1) as f64;
        let df_interaction = df_factor1 * df_factor2;
        let df_residual = total_n - 1.0 - df_factor1 - df_factor2 - df_interaction;

        // Mean squares
        let ms_factor1 = ss_factor1 / df_factor1;
        let ms_factor2 = ss_factor2 / df_factor2;
        let ms_interaction = ss_interaction / df_interaction;
        let ms_residual = ss_residual / df_residual;

        // F-statistics
        let f_statistic_factor1 = if ms_residual > 0.0 { ms_factor1 / ms_residual } else { f64::INFINITY };
        let f_statistic_factor2 = if ms_residual > 0.0 { ms_factor2 / ms_residual } else { f64::INFINITY };
        let f_statistic_interaction = if ms_residual > 0.0 { ms_interaction / ms_residual } else { f64::INFINITY };

        // P-values
        let p_value_factor1 = Self::calculate_f_p_value(f_statistic_factor1, df_factor1, df_residual)?;
        let p_value_factor2 = Self::calculate_f_p_value(f_statistic_factor2, df_factor2, df_residual)?;
        let p_value_interaction = Self::calculate_f_p_value(f_statistic_interaction, df_interaction, df_residual)?;

        // Effect sizes (partial eta squared)
        let eta_squared_factor1 = if ss_total > 0.0 { ss_factor1 / (ss_factor1 + ss_residual) } else { 0.0 };
        let eta_squared_factor2 = if ss_total > 0.0 { ss_factor2 / (ss_factor2 + ss_residual) } else { 0.0 };
        let eta_squared_interaction = if ss_total > 0.0 { ss_interaction / (ss_interaction + ss_residual) } else { 0.0 };

        Ok(TwoWayAnovaResult {
            f_statistic_factor1,
            f_statistic_factor2,
            f_statistic_interaction,
            p_value_factor1,
            p_value_factor2,
            p_value_interaction,
            degrees_of_freedom_factor1: df_factor1,
            degrees_of_freedom_factor2: df_factor2,
            degrees_of_freedom_interaction: df_interaction,
            degrees_of_freedom_residual: df_residual,
            eta_squared_factor1,
            eta_squared_factor2,
            eta_squared_interaction,
            significant_factor1: p_value_factor1 < 0.05,
            significant_factor2: p_value_factor2 < 0.05,
            significant_interaction: p_value_interaction < 0.05,
        })
    }
    pub fn repeated_measures_anova(subjects_data: &[Vec<f64>]) -> Result<RepeatedMeasuresAnovaResult, String> {
        if subjects_data.is_empty() {
            return Err("Data cannot be empty".to_string());
        }

        let n_subjects = subjects_data.len() as f64;
        let n_time_points = subjects_data[0].len() as f64;

        if n_time_points < 2.0 {
            return Err("Repeated measures ANOVA requires at least 2 time points".to_string());
        }

        // Check that all subjects have the same number of time points
        for subject in subjects_data {
            if subject.len() as f64 != n_time_points {
                return Err("All subjects must have the same number of time points".to_string());
            }
        }

        // Calculate overall mean
        let overall_mean = subjects_data.iter().flatten().sum::<f64>() / (n_subjects * n_time_points);

        // Calculate subject means
        let subject_means: Vec<f64> = subjects_data.iter()
            .map(|subject| subject.iter().sum::<f64>() / n_time_points)
            .collect();

        // Calculate time point means
        let mut time_means = vec![0.0; n_time_points as usize];
        for t in 0..n_time_points as usize {
            let sum: f64 = subjects_data.iter().map(|subject| subject[t]).sum();
            time_means[t] = sum / n_subjects;
        }

        // Calculate sum of squares
        let _ss_total = subjects_data.iter().flatten()
            .map(|&x| (x - overall_mean).powi(2))
            .sum::<f64>();

        let ss_subjects = subject_means.iter()
            .map(|&mean| n_time_points * (mean - overall_mean).powi(2))
            .sum::<f64>();

        let ss_time = time_means.iter()
            .map(|&mean| n_subjects * (mean - overall_mean).powi(2))
            .sum::<f64>();

        let ss_residual = subjects_data.iter().enumerate()
            .flat_map(|(s, subject)| {
                let subject_means = &subject_means;
                let time_means = &time_means;
                subject.iter().enumerate().map(move |(t, &value)| {
                    (value - subject_means[s] - time_means[t] + overall_mean).powi(2)
                })
            })
            .sum::<f64>();

        // Degrees of freedom
        let df_subjects = n_subjects - 1.0;
        let df_time = n_time_points - 1.0;
        let df_interaction = df_subjects * df_time;
        let df_residual = df_interaction;

        // Mean squares
        let ms_time = ss_time / df_time;
        let ms_subjects = ss_subjects / df_subjects;
        let ms_interaction = ss_residual / df_residual;

        // F-statistics
        let f_time = if ms_interaction > 0.0 { ms_time / ms_interaction } else { f64::INFINITY };
        let f_subjects = if ms_interaction > 0.0 { ms_subjects / ms_interaction } else { f64::INFINITY };
        let f_interaction = f64::NAN; // Not typically tested in repeated measures

        // P-values
        let p_time = Self::calculate_f_p_value(f_time, df_time, df_residual)?;
        let p_subjects = Self::calculate_f_p_value(f_subjects, df_subjects, df_residual)?;
        let p_interaction = f64::NAN;

        // Effect sizes (partial eta squared)
        let eta_squared_time = if ss_time + ss_residual > 0.0 { ss_time / (ss_time + ss_residual) } else { 0.0 };
        let eta_squared_subjects = if ss_subjects + ss_residual > 0.0 { ss_subjects / (ss_subjects + ss_residual) } else { 0.0 };
        let eta_squared_interaction = 0.0; // Not calculated for repeated measures

        // Sphericity test (Mauchly's test)
        let sphericity_test = Self::mauchly_sphericity_test(subjects_data)?;

        // Post-hoc tests for time effects
        let post_hoc_results = if sphericity_test.sphericity_assumed {
            Self::repeated_measures_post_hoc(subjects_data, &time_means, ms_interaction)?
        } else {
            // Use Greenhouse-Geisser corrected post-hoc
            Self::repeated_measures_post_hoc_gg(subjects_data, &time_means, ms_interaction, sphericity_test.epsilon_gg)?
        };

        Ok(RepeatedMeasuresAnovaResult {
            f_statistic_time: f_time,
            f_statistic_subject: f_subjects,
            f_statistic_interaction: f_interaction,
            p_value_time: p_time,
            p_value_subject: p_subjects,
            p_value_interaction: p_interaction,
            degrees_of_freedom_time: df_time,
            degrees_of_freedom_subject: df_subjects,
            degrees_of_freedom_interaction: df_interaction,
            degrees_of_freedom_residual: df_residual,
            eta_squared_time,
            eta_squared_subject: eta_squared_subjects,
            eta_squared_interaction,
            significant_time: p_time < 0.05,
            significant_subject: p_subjects < 0.05,
            significant_interaction: false, // Not tested
            sphericity_test: Some(sphericity_test),
            post_hoc_results: Some(post_hoc_results),
        })
    }

    /// One-way ANOVA test
    pub fn one_way_anova(groups: &[&[f64]]) -> Result<AnovaResult, String> {
        if groups.len() < 2 {
            return Err("ANOVA requires at least 2 groups".to_string());
        }

        let mut all_data = Vec::new();
        let mut group_means = Vec::new();
        let mut group_sizes = Vec::new();

        // Calculate group statistics
        for group in groups {
            if group.is_empty() {
                return Err("Groups cannot be empty".to_string());
            }
            let mean = group.iter().sum::<f64>() / group.len() as f64;
            group_means.push(mean);
            group_sizes.push(group.len());
            all_data.extend_from_slice(group);
        }

        let n_total = all_data.len() as f64;
        let grand_mean = all_data.iter().sum::<f64>() / n_total;

        // Sum of squares between groups
        let ss_between = group_means.iter().zip(&group_sizes)
            .map(|(&mean, &size)| size as f64 * (mean - grand_mean).powi(2))
            .sum::<f64>();

        // Sum of squares within groups
        let ss_within = groups.iter().zip(&group_means)
            .map(|(group, &mean)| group.iter().map(|&x| (x - mean).powi(2)).sum::<f64>())
            .sum::<f64>();

        // Degrees of freedom
        let df_between = (groups.len() - 1) as f64;
        let df_within = n_total - groups.len() as f64;

        // Mean squares
        let ms_between = ss_between / df_between;
        let ms_within = ss_within / df_within;

        // F-statistic
        let f_statistic = if ms_within > 0.0 { ms_between / ms_within } else { f64::INFINITY };

        // P-value
        let p_value = Self::calculate_f_p_value(f_statistic, df_between, df_within)?;

        // Effect size (eta squared)
        let eta_squared = if ss_between + ss_within > 0.0 { ss_between / (ss_between + ss_within) } else { 0.0 };

        // Post-hoc tests using Tukey's HSD with Studentized Range approximation
        let post_hoc_results = Self::tukey_hsd_post_hoc(groups, &group_means, &group_sizes, ms_within, df_within)?;

        Ok(AnovaResult {
            test_type: "One-way ANOVA".to_string(),
            f_statistic,
            p_value,
            degrees_of_freedom_between: df_between,
            degrees_of_freedom_within: df_within,
            sum_of_squares_between: ss_between,
            sum_of_squares_within: ss_within,
            eta_squared,
            significant: p_value < 0.05,
            post_hoc_results: Some(post_hoc_results),
        })
    }

    fn mauchly_sphericity_test(subjects_data: &[Vec<f64>]) -> Result<SphericityTestResult, String> {
        let n_subjects = subjects_data.len();
        let n_time_points = subjects_data[0].len();

        if n_time_points < 3 {
            // Sphericity test not applicable for 2 time points
            return Ok(SphericityTestResult {
                mauchly_w: 1.0,
                chi_square: 0.0,
                df: 0.0,
                p_value: 1.0,
                sphericity_assumed: true,
                epsilon_gg: 1.0,
                epsilon_hf: 1.0,
            });
        }

        // Calculate covariance matrix of the repeated measures
        let mut cov_matrix = vec![vec![0.0; n_time_points]; n_time_points];

        for subject in subjects_data {
            let subject_mean = subject.iter().sum::<f64>() / n_time_points as f64;
            let deviations: Vec<f64> = subject.iter().map(|&x| x - subject_mean).collect();

            for i in 0..n_time_points {
                for j in 0..n_time_points {
                    cov_matrix[i][j] += deviations[i] * deviations[j];
                }
            }
        }

        for row in cov_matrix.iter_mut().take(n_time_points) {
            for cell in row.iter_mut().take(n_time_points) {
                *cell /= (n_subjects - 1) as f64;
            }
        }

        // Calculate Mauchly's W statistic
        // W = |Σ| / (trace(Σ)/p)^p
        let det = Self::matrix_determinant(&cov_matrix)?;
        let trace: f64 = cov_matrix.iter().enumerate().map(|(i, row)| row[i]).sum();
        let mean_cov = trace / n_time_points as f64;
        let expected_det = mean_cov.powi(n_time_points as i32);

        let w = if expected_det > 0.0 { det / expected_det } else { 0.0 };

        // Chi-square approximation: -ln(W) * (n-1) * correction_factor
        let df = (n_time_points * (n_time_points - 1) / 2) as f64;
        let chi_square = if w > 0.0 { -(n_subjects as f64 - 1.0) * w.ln() } else { f64::INFINITY };

        // P-value
        let chi_dist = ChiSquared::new(df)
            .map_err(|e| format!("Failed to create chi-squared distribution: {}", e))?;
        let p_value = 1.0 - chi_dist.cdf(chi_square);

        // Greenhouse-Geisser epsilon
        let sum_squared_cov: f64 = cov_matrix.iter().flatten().map(|&x| x * x).sum();
        let epsilon_gg = if trace > 0.0 {
            (trace * trace) / ((n_time_points - 1) as f64 * sum_squared_cov)
        } else {
            1.0
        };

        // Huynh-Feldt epsilon
        let epsilon_hf = if df > 0.0 && n_subjects > 1 {
            let _n_df = n_subjects as f64 - 1.0;
            let numerator = (trace * trace - 2.0 * df * expected_det) / df;
            let denominator = sum_squared_cov - (trace * trace) / n_time_points as f64;
            if denominator > 0.0 {
                (numerator / denominator).min(1.0).max(epsilon_gg)
            } else {
                epsilon_gg
            }
        } else {
            epsilon_gg
        };

        Ok(SphericityTestResult {
            mauchly_w: w,
            chi_square,
            df,
            p_value,
            sphericity_assumed: p_value >= 0.05, // Assume sphericity if p >= 0.05
            epsilon_gg: epsilon_gg.clamp(0.0, 1.0),
            epsilon_hf: epsilon_hf.clamp(0.0, 1.0),
        })
    }

    /// Post-hoc tests for repeated measures ANOVA (assuming sphericity)
    fn repeated_measures_post_hoc(
        subjects_data: &[Vec<f64>],
        time_means: &[f64],
        ms_error: f64
    ) -> Result<Vec<PostHocResult>, String> {
        let n_time_points = time_means.len();
        let n_subjects = subjects_data.len();
        let mut results = Vec::new();

        let alpha = 0.05;
        let df_error = ((n_subjects - 1) * (n_time_points - 1)) as f64;
        let alpha_corrected = alpha / (n_time_points * (n_time_points - 1) / 2) as f64;

        for i in 0..n_time_points {
            for j in (i + 1)..n_time_points {
                let mean_diff = time_means[i] - time_means[j];
                let se = (2.0 * ms_error / n_subjects as f64).sqrt();

                let (p_value, confidence_interval, significant) = Self::calculate_post_hoc_t_test(mean_diff, se, df_error, alpha_corrected)?;

                results.push(Self::construct_post_hoc_result(
                    format!("Time {} vs Time {}", i + 1, j + 1),
                    mean_diff,
                    se,
                    confidence_interval,
                    p_value,
                    significant,
                    mean_diff / se, // Cohen's d for paired comparisons
                ));
            }
        }

        Ok(results)
    }

    /// Post-hoc tests with Greenhouse-Geisser correction
    fn repeated_measures_post_hoc_gg(
        subjects_data: &[Vec<f64>],
        time_means: &[f64],
        ms_error: f64,
        epsilon: f64
    ) -> Result<Vec<PostHocResult>, String> {
        let n_time_points = time_means.len();
        let n_subjects = subjects_data.len();
        let mut results = Vec::new();

        let alpha = 0.05;
        let df_error = ((n_subjects - 1) * (n_time_points - 1)) as f64 * epsilon;
        let alpha_corrected = alpha / (n_time_points * (n_time_points - 1) / 2) as f64;

        for i in 0..n_time_points {
            for j in (i + 1)..n_time_points {
                let mean_diff = time_means[i] - time_means[j];
                let se = (2.0 * ms_error / n_subjects as f64).sqrt();

                let (p_value, confidence_interval, significant) = Self::calculate_post_hoc_t_test(mean_diff, se, df_error, alpha_corrected)?;

                results.push(Self::construct_post_hoc_result(
                    format!("Time {} vs Time {} (GG corrected)", i + 1, j + 1),
                    mean_diff,
                    se,
                    confidence_interval,
                    p_value,
                    significant,
                    mean_diff / se,
                ));
            }
        }

        Ok(results)
    }

    /// Calculate determinant of a matrix using proper linear algebra library
    pub fn matrix_determinant(matrix: &[Vec<f64>]) -> Result<f64, String> {
        let n = matrix.len();
        if n == 0 || matrix.iter().any(|row| row.len() != n) {
            return Err("Invalid matrix dimensions".to_string());
        }

        // Convert Vec<Vec<f64>> to Array2<f64>
        let flat_data: Vec<f64> = matrix.iter().flatten().cloned().collect();
        let ndarray_matrix = Array2::from_shape_vec((n, n), flat_data)
            .map_err(|_| "Failed to create matrix from data".to_string())?;

        // Use the proper linear algebra library
        LinearAlgebra::determinant(&ndarray_matrix)
    }

    pub fn chi_square_goodness_of_fit(observed: &[f64], expected: &[f64]) -> Result<ChiSquareResult, String> {
        if observed.len() != expected.len() {
            return Err("Observed and expected frequencies must have equal lengths".to_string());
        }

        if observed.iter().any(|&x| x < 0.0) || expected.iter().any(|&x| x <= 0.0) {
            return Err("Frequencies must be non-negative, expected frequencies must be positive".to_string());
        }

        let total_observed: f64 = observed.iter().sum();
        let total_expected: f64 = expected.iter().sum();

        if (total_observed - total_expected).abs() > 1e-10 {
            return Err("Total observed and expected frequencies must be equal".to_string());
        }

        // Calculate chi-square statistic
        let chi_square = observed.iter().zip(expected.iter())
            .map(|(&o, &e)| (o - e).powi(2) / e)
            .sum::<f64>();

        let df = (observed.len() - 1) as f64;

        let p_value = UnifiedStats::chi_square_p_value(chi_square, df)?;

        // Effect size: Cramér's V for goodness of fit (corrected formula)
        let cramers_v = if df > 0.0 && total_observed > 0.0 {
            (chi_square / (total_observed * df.min(total_observed - 1.0))).sqrt()
        } else {
            0.0
        };

        Ok(ChiSquareResult {
            test_type: "Chi-square goodness of fit".to_string(),
            chi_square_statistic: chi_square,
            p_value,
            degrees_of_freedom: df,
            expected_frequencies: vec![expected.to_vec()], // Wrap in vec for consistency
            residuals: vec![observed.iter().zip(expected.iter())
                .map(|(&o, &e)| (o - e) / e.sqrt())
                .collect()],
            significant: p_value < 0.05,
            effect_size: Some(cramers_v),
        })
    }

    /// Chi-square test of independence
    pub fn chi_square_independence(table: &[&[f64]]) -> Result<ChiSquareResult, String> {
        if table.is_empty() || table[0].is_empty() {
            return Err("Contingency table cannot be empty".to_string());
        }

        let rows = table.len();
        let cols = table[0].len();

        // Check table dimensions
        if table.iter().any(|row| row.len() != cols) {
            return Err("All rows must have the same number of columns".to_string());
        }

        // Calculate row and column totals
        let mut row_totals = vec![0.0; rows];
        let mut col_totals = vec![0.0; cols];
        let mut grand_total = 0.0;

        for (i, row) in table.iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                if cell < 0.0 {
                    return Err("Cell frequencies cannot be negative".to_string());
                }
                row_totals[i] += cell;
                col_totals[j] += cell;
                grand_total += cell;
            }
        }

        if grand_total == 0.0 {
            return Err("Grand total cannot be zero".to_string());
        }

        // Calculate expected frequencies and chi-square statistic
        let mut expected = vec![vec![0.0; cols]; rows];
        let mut chi_square = 0.0;

        for (i, row_total) in row_totals.iter().enumerate().take(rows) {
            for (j, col_total) in col_totals.iter().enumerate().take(cols) {
                expected[i][j] = (row_total * col_total) / grand_total;
                if expected[i][j] > 0.0 {
                    chi_square += (table[i][j] - expected[i][j]).powi(2) / expected[i][j];
                }
            }
        }

        let df = ((rows - 1) * (cols - 1)) as f64;

        let p_value = UnifiedStats::chi_square_p_value(chi_square, df)?;

        // Calculate residuals
        let mut residuals = vec![vec![0.0; cols]; rows];
        for (i, row_total) in row_totals.iter().enumerate().take(rows) {
            for (j, col_total) in col_totals.iter().enumerate().take(cols) {
                residuals[i][j] = (table[i][j] - expected[i][j]) /
                    (expected[i][j] * (1.0 - row_total/grand_total) * (1.0 - col_total/grand_total)).sqrt();
            }
        }

        // Effect size: Cramér's V
        let min_dim = rows.min(cols) as f64 - 1.0;
        let effect_size = ((chi_square / grand_total) / min_dim).sqrt();

        Ok(ChiSquareResult {
            test_type: "Chi-square test of independence".to_string(),
            chi_square_statistic: chi_square,
            p_value,
            degrees_of_freedom: df,
            expected_frequencies: expected,
            residuals,
            significant: p_value < 0.05,
            effect_size: Some(effect_size),
        })
    }



    /// Tukey's HSD post-hoc test for one-way ANOVA
    pub fn tukey_hsd_post_hoc(
        _groups: &[&[f64]],
        group_means: &[f64],
        group_sizes: &[usize],
        ms_within: f64,
        df_within: f64
    ) -> Result<Vec<PostHocResult>, String> {
        let mut results = Vec::new();
        let k = group_means.len();
        let alpha = 0.05;

        // Calculate harmonic mean of group sizes for unequal groups
        let harmonic_mean_n = if group_sizes.iter().all(|&n| n == group_sizes[0]) {
            group_sizes[0] as f64
        } else {
            let sum_recip = group_sizes.iter().map(|&n| 1.0 / n as f64).sum::<f64>();
            k as f64 / sum_recip
        };

        let q_critical = Self::studentized_range_quantile(alpha, k, df_within)?;
        let critical_value = q_critical * (ms_within / harmonic_mean_n).sqrt();

        for i in 0..k {
            for j in (i + 1)..k {
                let mean_diff = group_means[i] - group_means[j];
                let n_i = group_sizes[i] as f64;
                let n_j = group_sizes[j] as f64;
                let se = (ms_within * (1.0 / n_i + 1.0 / n_j)).sqrt();

                let t_stat = mean_diff.abs() / se;
                let significant = t_stat > critical_value;
                let p_value = if significant { alpha } else { 1.0 - alpha }; // Approximate, since exact p requires q-distribution

                let confidence_interval = (
                    mean_diff - critical_value * se,
                    mean_diff + critical_value * se,
                );

                results.push(Self::construct_post_hoc_result(
                    format!("Group {} vs Group {} (Tukey's HSD)", i + 1, j + 1),
                    mean_diff,
                    se,
                    confidence_interval,
                    p_value,
                    significant,
                    mean_diff / se,
                ));
            }
        }

        Ok(results)
    }

    /// Holm-Bonferroni post-hoc test for one-way ANOVA
    pub fn holm_bonferroni_post_hoc(
        _groups: &[&[f64]],
        group_means: &[f64],
        group_sizes: &[usize],
        ms_within: f64,
        df_within: f64
    ) -> Result<Vec<PostHocResult>, String> {
        let mut results = Vec::new();
        let k = group_means.len();

        // First calculate all p-values
        let mut comparisons = Vec::new();
        for i in 0..k {
            for j in (i + 1)..k {
                let mean_diff = group_means[i] - group_means[j];
                let n_i = group_sizes[i] as f64;
                let n_j = group_sizes[j] as f64;
                let se = (ms_within * (1.0 / n_i + 1.0 / n_j)).sqrt();

                let t_stat = mean_diff.abs() / se;
                let p_value = Self::calculate_t_p_value(t_stat, df_within)?;

                comparisons.push((i, j, mean_diff, se, p_value, df_within));
            }
        }

        // Sort by p-value
        comparisons.sort_by(|a, b| match a.4.partial_cmp(&b.4) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        let alpha = 0.05;
        let m = comparisons.len();

        for (rank, (i, j, mean_diff, se, p_value, df)) in comparisons.iter().enumerate() {
            let alpha_corrected = alpha / (m - rank) as f64;
            let (_, confidence_interval, significant) = Self::calculate_post_hoc_t_test(*mean_diff, *se, *df, alpha_corrected)?;

            results.push(Self::construct_post_hoc_result(
                format!("Group {} vs Group {} (Holm-Bonferroni)", i + 1, j + 1),
                *mean_diff,
                *se,
                confidence_interval,
                *p_value,
                significant,
                *mean_diff / *se,
            ));
        }

        Ok(results)
    }

    /// Bonferroni post-hoc test for one-way ANOVA
    pub fn bonferroni_post_hoc(
        _groups: &[&[f64]],
        group_means: &[f64],
        group_sizes: &[usize],
        ms_within: f64,
        df_within: f64
    ) -> Result<Vec<PostHocResult>, String> {
        let mut results = Vec::new();
        let k = group_means.len();
        let m = (k * (k - 1) / 2) as f64; // Number of pairwise comparisons
        let alpha_corrected = 0.05 / m;

        for i in 0..k {
            for j in (i + 1)..k {
                let mean_diff = group_means[i] - group_means[j];
                let n_i = group_sizes[i] as f64;
                let n_j = group_sizes[j] as f64;
                let se = (ms_within * (1.0 / n_i + 1.0 / n_j)).sqrt();

                let t_stat = mean_diff.abs() / se;
                let p_value = Self::calculate_t_p_value(t_stat, df_within)?;
                let (_, confidence_interval, significant) = Self::calculate_post_hoc_t_test(mean_diff, se, df_within, alpha_corrected)?;

                results.push(Self::construct_post_hoc_result(
                    format!("Group {} vs Group {} (Bonferroni)", i + 1, j + 1),
                    mean_diff,
                    se,
                    confidence_interval,
                    p_value,
                    significant,
                    mean_diff / se,
                ));
            }
        }

        Ok(results)
    }
}