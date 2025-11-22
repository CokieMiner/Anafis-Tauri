//! Hypothesis Testing Module
//!
//! This module provides comprehensive statistical hypothesis testing capabilities:
//! - t-tests (one-sample, paired, two-sample)
//! - ANOVA (one-way, two-way, repeated measures)
//! - Chi-square tests (goodness of fit, independence)
//! - Post-hoc tests (Tukey HSD, Holm-Bonferroni, Bonferroni)
//! - Non-parametric tests
//!
//! TODO: Add uncertainty support via bootstrap/permutation methods:
//! - Implement bootstrap resampling for t-tests to quantify p-value uncertainty
//! - Add permutation tests as robust alternatives when measurement errors are present
//! - Provide bootstrap confidence intervals for effect sizes
//! - Consider measurement error models for hypothesis testing with uncertain data
//! This would enable robust inference in the presence of measurement uncertainty.

pub mod types;
pub mod t_tests;
pub mod anova;
pub mod chi_square;
pub mod post_hoc;
pub(crate) mod helpers;

// Re-export main API
pub use types::*;
pub use t_tests::TTesting;
pub use anova::AnovaTesting;
pub use chi_square::ChiSquareTesting;
pub use post_hoc::PostHocTesting;

/// Main hypothesis testing engine that delegates to specialized modules
pub struct HypothesisTestingEngine;

impl HypothesisTestingEngine {
    // T-tests
    pub fn one_sample_t_test(data: &[f64], mu: f64) -> Result<TTestResult, StatsError> {
        TTesting::one_sample_t_test(data, mu)
    }

    pub fn paired_t_test(data1: &[f64], data2: &[f64]) -> Result<TTestResult, StatsError> {
        TTesting::paired_t_test(data1, data2)
    }

    pub fn two_sample_t_test(data1: &[f64], data2: &[f64], equal_var: bool) -> Result<TTestResult, StatsError> {
        TTesting::two_sample_t_test(data1, data2, equal_var)
    }

    // ANOVA
    pub fn one_way_anova(groups: &[&[f64]]) -> Result<AnovaResult, StatsError> {
        AnovaTesting::one_way_anova(groups)
    }

    pub fn two_way_anova(
        data: &[Vec<f64>],
        factor1_levels: &[usize],
        factor2_levels: &[usize]
    ) -> Result<TwoWayAnovaResult, StatsError> {
        AnovaTesting::two_way_anova(data, factor1_levels, factor2_levels)
    }

    pub fn n_way_anova(
        data: &[Vec<f64>],
        factor_data: &[Vec<String>],
        factor_names: Option<&[String]>,
    ) -> Result<NWayAnovaResult, StatsError> {
        AnovaTesting::n_way_anova(data, factor_data, factor_names)
    }

    // Chi-square tests
    pub fn chi_square_goodness_of_fit(observed: &[f64], expected: &[f64]) -> Result<ChiSquareResult, StatsError> {
        ChiSquareTesting::chi_square_goodness_of_fit(observed, expected)
    }

    pub fn chi_square_independence(table: &[&[f64]]) -> Result<ChiSquareResult, StatsError> {
        ChiSquareTesting::chi_square_independence(table)
    }

    // Post-hoc tests
    pub fn bonferroni_post_hoc(
        groups: &[&[f64]],
        group_means: &[f64],
        group_sizes: &[usize],
        ms_within: f64,
        df_within: f64,
    ) -> Result<Vec<PostHocResult>, StatsError> {
        PostHocTesting::bonferroni_post_hoc(groups, group_means, group_sizes, ms_within, df_within)
    }
}