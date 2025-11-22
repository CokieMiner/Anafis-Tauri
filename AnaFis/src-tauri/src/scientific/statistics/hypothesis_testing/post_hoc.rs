//! Post-hoc tests for ANOVA
//!
//! Multiple comparison procedures for ANOVA.

use super::types::{StatsError, PostHocResult};
use crate::scientific::statistics::distributions::distribution_functions;

/// Post-hoc testing engine
pub struct PostHocTesting;

impl PostHocTesting {
    /// Bonferroni's post-hoc test for one-way ANOVA.
    ///
    /// This is a simpler alternative to Tukey HSD that doesn't require external libraries.
    /// It uses t-tests with Bonferroni correction for multiple comparisons.
    pub fn bonferroni_post_hoc(
        _groups: &[&[f64]],
        group_means: &[f64],
        group_sizes: &[usize],
        ms_within: f64,
        df_within: f64,
    ) -> Result<Vec<PostHocResult>, StatsError> {
        let mut results = Vec::new();
        let k = group_means.len();
        let alpha = 0.05; // Significance level
        let bonferroni_alpha = alpha / (k * (k - 1) / 2) as f64; // Corrected alpha

        if k < 2 {
            return Ok(results);
        }
        if ms_within <= 0.0 || df_within <= 0.0 {
            return Err(StatsError::InvalidParameter("Mean squared error and degrees of freedom must be positive".to_string()));
        }

        for i in 0..k {
            for j in (i + 1)..k {
                let mean_diff = group_means[i] - group_means[j];
                let n_i = group_sizes[i] as f64;
                let n_j = group_sizes[j] as f64;

                // Pooled standard error
                let se = (ms_within * (1.0 / n_i + 1.0 / n_j)).sqrt();
                if se <= 1e-10 { continue; }

                // t-statistic
                let t_statistic = mean_diff.abs() / se;

                // Bonferroni-corrected p-value (two-tailed t-test)
                let p_value = 2.0 * (1.0 - distribution_functions::t_cdf(t_statistic, df_within));
                let p_value_corrected = (p_value * (k * (k - 1) / 2) as f64).min(1.0);

                // Confidence interval with Bonferroni correction
                let t_crit = distribution_functions::t_quantile(1.0 - bonferroni_alpha / 2.0, df_within)?;
                let margin_of_error = t_crit * se;

                let confidence_interval = (
                    mean_diff - margin_of_error,
                    mean_diff + margin_of_error,
                );

                results.push(PostHocResult {
                    comparison: format!("Group {} vs Group {}", i + 1, j + 1),
                    mean_difference: mean_diff,
                    standard_error: se,
                    confidence_interval,
                    p_value: p_value_corrected,
                    significant: p_value_corrected < alpha,
                    effect_size: mean_diff / ms_within.sqrt(),
                });
            }
        }

        Ok(results)
    }

    // TODO: Add Tukey HSD and Holm-Bonferroni methods
}
