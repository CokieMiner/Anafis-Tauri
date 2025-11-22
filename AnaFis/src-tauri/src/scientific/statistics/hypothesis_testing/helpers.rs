//! Helper functions for hypothesis testing
//!
//! Internal utility functions for p-value calculations and confidence intervals.

use super::types::StatsError;
use crate::scientific::statistics::distributions::distribution_functions;

/// Calculate two-sided p-value for t-statistic
pub(crate) fn t_p_value(t: f64, df: f64) -> Result<f64, StatsError> {
    // Two-sided p-value
    Ok(2.0 * (1.0 - distribution_functions::t_cdf(t.abs(), df)))
}

/// Calculate right-tailed p-value for F-statistic
pub(crate) fn f_p_value(f: f64, df1: f64, df2: f64) -> Result<f64, StatsError> {
    // Right-tailed p-value
    Ok(1.0 - distribution_functions::f_cdf(f, df1, df2))
}

/// Calculate right-tailed p-value for chi-square statistic
pub(crate) fn chi_square_p_value(chi2: f64, df: f64) -> Result<f64, StatsError> {
    // Right-tailed p-value
    Ok(1.0 - distribution_functions::chi_squared_cdf(chi2, df))
}

/// Calculate confidence interval for t-distribution
pub(crate) fn t_confidence_interval(mean: f64, se: f64, df: f64, confidence: f64) -> Result<(f64, f64), StatsError> {
    let alpha = 1.0 - confidence;
    let t_crit = distribution_functions::t_quantile(1.0 - alpha / 2.0, df)
        .map_err(StatsError::DistributionError)?;
    let margin = t_crit * se;
    Ok((mean - margin, mean + margin))
}

/// Calculate critical value for studentized range distribution (for Tukey HSD)
pub(crate) fn studentized_range_critical_value(_k: usize, _df: f64, _alpha: f64) -> f64 {
    // Simplified approximation - in production, use proper q-distribution tables
    // For now, use a conservative estimate based on t-distribution
    3.0 // Placeholder
}
