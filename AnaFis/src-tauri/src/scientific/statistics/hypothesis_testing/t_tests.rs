//! T-test implementations
//!
//! Statistical t-tests for comparing means.

use super::types::{StatsError, TTestResult};
use super::helpers::{t_p_value, t_confidence_interval};
use crate::scientific::statistics::descriptive::StatisticalMoments;

/// T-test engine
pub struct TTesting;

impl TTesting {
    /// One-sample t-test
    pub fn one_sample_t_test(data: &[f64], mu: f64) -> Result<TTestResult, StatsError> {
        if data.is_empty() {
            return Err(StatsError::EmptyData);
        }

        let n = data.len() as f64;
        let sample_mean = data.mean();
        let sample_std = data.std_dev();

        if sample_std == 0.0 {
            return Err(StatsError::DegenerateFeatures("Standard deviation is zero - all values are identical".to_string()));
        }

        let t_statistic = (sample_mean - mu) / (sample_std / n.sqrt());
        let df = n - 1.0;
        let p_value = t_p_value(t_statistic, df)?;
        let confidence_interval = t_confidence_interval(sample_mean, sample_std / n.sqrt(), df, 0.95)?;
        let effect_size = (sample_mean - mu) / sample_std; // Cohen's d

        Ok(TTestResult {
            test_type: "One-sample t-test".to_string(),
            t_statistic,
            p_value,
            degrees_of_freedom: df,
            mean_difference: sample_mean - mu,
            confidence_interval,
            effect_size,
            alternative: "two-sided".to_string(),
            significant: p_value < 0.05,
        })
    }

    /// Paired t-test
    pub fn paired_t_test(data1: &[f64], data2: &[f64]) -> Result<TTestResult, StatsError> {
        if data1.len() != data2.len() {
            return Err(StatsError::DimensionMismatch);
        }

        if data1.is_empty() {
            return Err(StatsError::EmptyData);
        }

        let differences: Vec<f64> = data1.iter().zip(data2.iter())
            .map(|(a, b)| a - b)
            .collect();

        let n = differences.len() as f64;
        let mean_diff = differences.mean();
        let std_diff = differences.std_dev();

        if std_diff == 0.0 {
            return Err(StatsError::DegenerateFeatures("Standard deviation of differences is zero".to_string()));
        }

        let t_statistic = mean_diff / (std_diff / n.sqrt());
        let df = n - 1.0;
        let p_value = t_p_value(t_statistic, df)?;
        let confidence_interval = t_confidence_interval(mean_diff, std_diff / n.sqrt(), df, 0.95)?;
        let effect_size = mean_diff / std_diff; // Cohen's d for paired

        Ok(TTestResult {
            test_type: "Paired t-test".to_string(),
            t_statistic,
            p_value,
            degrees_of_freedom: df,
            mean_difference: mean_diff,
            confidence_interval,
            effect_size,
            alternative: "two-sided".to_string(),
            significant: p_value < 0.05,
        })
    }

    /// Two-sample t-test
    pub fn two_sample_t_test(data1: &[f64], data2: &[f64], equal_var: bool) -> Result<TTestResult, StatsError> {
        if data1.is_empty() || data2.is_empty() {
            return Err(StatsError::EmptyData);
        }

        let mean1 = data1.mean();
        let mean2 = data2.mean();
        let n1 = data1.len() as f64;
        let n2 = data2.len() as f64;

        let (t_statistic, df) = if equal_var {
            // Pooled variance
            let var1 = data1.variance();
            let var2 = data2.variance();
            let pooled_var = ((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0);
            let se = (pooled_var * (1.0/n1 + 1.0/n2)).sqrt();
            ((mean1 - mean2) / se, n1 + n2 - 2.0)
        } else {
            // Welch's t-test
            let var1 = data1.variance();
            let var2 = data2.variance();
            let se = (var1/n1 + var2/n2).sqrt();
            let t = (mean1 - mean2) / se;
            let df_num = (var1/n1 + var2/n2).powi(2);
            let df_denom = (var1/n1).powi(2)/(n1-1.0) + (var2/n2).powi(2)/(n2-1.0);
            (t, df_num / df_denom)
        };

        let p_value = t_p_value(t_statistic, df)?;
        let se_diff = if equal_var {
            let var1 = data1.variance();
            let var2 = data2.variance();
            let pooled_var = ((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0);
            (pooled_var * (1.0/n1 + 1.0/n2)).sqrt()
        } else {
            (data1.variance()/n1 + data2.variance()/n2).sqrt()
        };
        let confidence_interval = t_confidence_interval(mean1 - mean2, se_diff, df, 0.95)?;
        let effect_size = (mean1 - mean2) / ((data1.variance() + data2.variance())/2.0).sqrt(); // Cohen's d

        Ok(TTestResult {
            test_type: if equal_var { "Two-sample t-test (equal variance)" } else { "Welch's t-test (unequal variance)" }.to_string(),
            t_statistic,
            p_value,
            degrees_of_freedom: df,
            mean_difference: mean1 - mean2,
            confidence_interval,
            effect_size,
            alternative: "two-sided".to_string(),
            significant: p_value < 0.05,
        })
    }
}
