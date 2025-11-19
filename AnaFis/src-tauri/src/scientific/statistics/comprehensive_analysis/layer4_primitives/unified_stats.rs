//! Unified Basic Statistics Module
//!
//! This module consolidates all basic statistical functions that were duplicated
//! across multiple layers. This eliminates the code duplication identified in
//! the comprehensive analysis.
//!
//! ## Consolidated Functions
//! - Mean, variance, standard deviation
//! - Distribution functions and quantiles
//! - P-value calculations
//! - Confidence interval calculations

use statrs::distribution::{Continuous, ContinuousCDF, Normal, StudentsT, FisherSnedecor, ChiSquared};
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::non_central_distributions::{NonCentralT, NonCentralF};

/// Unified Basic Statistics Engine
/// Consolidates all duplicated basic statistical functions
pub struct UnifiedStats;

impl UnifiedStats {
    /// Calculate arithmetic mean
    pub fn mean(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        data.iter().sum::<f64>() / data.len() as f64
    }

    /// Calculate sample variance
    pub fn variance(data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let mean = Self::mean(data);
        data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / ((data.len() - 1) as f64)
    }

    /// Calculate sample standard deviation
    pub fn std_dev(data: &[f64]) -> f64 {
        Self::variance(data).sqrt()
    }

    /// Normal distribution PDF
    pub fn normal_pdf(x: f64, mean: f64, std: f64) -> f64 {
        if !std.is_finite() || std <= 0.0 {
            return 0.0; // Return 0 for invalid parameters
        }
        let normal = Normal::new(mean, std)
            .expect("Failed to create normal distribution with valid parameters");
        normal.pdf(x)
    }

    /// Normal distribution CDF
    pub fn normal_cdf(x: f64, mean: f64, std: f64) -> f64 {
        if !std.is_finite() || std <= 0.0 {
            return if x < mean { 0.0 } else { 1.0 }; // Return step function for invalid std
        }
        let normal = Normal::new(mean, std)
            .expect("Failed to create normal distribution with valid parameters");
        normal.cdf(x)
    }

    /// Student's t-distribution PDF
    pub fn t_pdf(x: f64, df: f64) -> f64 {
        if !df.is_finite() || df <= 0.0 {
            return 0.0; // Return 0 for invalid degrees of freedom
        }
        let t = StudentsT::new(0.0, 1.0, df)
            .expect("Failed to create t-distribution with valid degrees of freedom");
        t.pdf(x)
    }

    /// Student's t-distribution CDF
    pub fn t_cdf(x: f64, df: f64) -> f64 {
        if !df.is_finite() || df <= 0.0 {
            return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid df
        }
        let t = StudentsT::new(0.0, 1.0, df)
            .expect("Failed to create t-distribution with valid degrees of freedom");
        t.cdf(x)
    }

    /// Chi-squared distribution PDF
    pub fn chi_squared_pdf(x: f64, df: f64) -> f64 {
        if !df.is_finite() || df <= 0.0 || x < 0.0 {
            return 0.0; // Return 0 for invalid parameters
        }
        let chi2 = ChiSquared::new(df)
            .expect("Failed to create chi-squared distribution with valid degrees of freedom");
        chi2.pdf(x)
    }

    /// Chi-squared distribution CDF
    pub fn chi_squared_cdf(x: f64, df: f64) -> f64 {
        if !df.is_finite() || df <= 0.0 {
            return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid df
        }
        let chi2 = ChiSquared::new(df)
            .expect("Failed to create chi-squared distribution with valid degrees of freedom");
        chi2.cdf(x)
    }

    /// F-distribution CDF
    pub fn f_cdf(x: f64, df1: f64, df2: f64) -> f64 {
        if !df1.is_finite() || df1 <= 0.0 || !df2.is_finite() || df2 <= 0.0 {
            return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid df
        }
        let f_dist = FisherSnedecor::new(df1, df2)
            .expect("Failed to create F-distribution with valid degrees of freedom");
        f_dist.cdf(x)
    }

    /// Calculate pooled variance for two samples
    pub fn pooled_variance(data1: &[f64], data2: &[f64]) -> f64 {
        let n1 = data1.len() as f64;
        let n2 = data2.len() as f64;
        let var1 = Self::variance(data1);
        let var2 = Self::variance(data2);

        ((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0)
    }

    /// Calculate standard error of the mean
    pub fn standard_error(data: &[f64]) -> f64 {
        Self::std_dev(data) / (data.len() as f64).sqrt()
    }

    /// Calculate standard error for mean difference (two samples)
    pub fn standard_error_difference(data1: &[f64], data2: &[f64], equal_var: bool) -> f64 {
        if equal_var {
            let pooled_var = Self::pooled_variance(data1, data2);
            (pooled_var / data1.len() as f64 + pooled_var / data2.len() as f64).sqrt()
        } else {
            (Self::variance(data1) / data1.len() as f64 + Self::variance(data2) / data2.len() as f64).sqrt()
        }
    }

    /// Calculate degrees of freedom for two-sample t-test
    pub fn degrees_of_freedom_two_sample(data1: &[f64], data2: &[f64], equal_var: bool) -> f64 {
        if equal_var {
            data1.len() as f64 + data2.len() as f64 - 2.0
        } else {
            // Welch's degrees of freedom
            let var1 = Self::variance(data1);
            let var2 = Self::variance(data2);
            let n1 = data1.len() as f64;
            let n2 = data2.len() as f64;

            let numerator = (var1 / n1 + var2 / n2).powi(2);
            let denominator = (var1 / n1).powi(2) / (n1 - 1.0) + (var2 / n2).powi(2) / (n2 - 1.0);
            numerator / denominator
        }
    }

    /// Calculate t-statistic for one-sample t-test
    pub fn t_statistic_one_sample(data: &[f64], mu: f64) -> f64 {
        let mean = Self::mean(data);
        let se = Self::standard_error(data);
        (mean - mu) / se
    }

    /// Calculate t-statistic for two-sample t-test
    pub fn t_statistic_two_sample(data1: &[f64], data2: &[f64], equal_var: bool) -> f64 {
        let mean1 = Self::mean(data1);
        let mean2 = Self::mean(data2);
        let se_diff = Self::standard_error_difference(data1, data2, equal_var);
        (mean1 - mean2) / se_diff
    }

    /// Calculate t-statistic for paired t-test
    pub fn t_statistic_paired(data1: &[f64], data2: &[f64]) -> f64 {
        let differences: Vec<f64> = data1.iter().zip(data2.iter())
            .map(|(a, b)| a - b)
            .collect();

        let mean_diff = Self::mean(&differences);
        let se_diff = Self::standard_error(&differences);
        mean_diff / se_diff
    }

    /// Calculate p-value for t-distribution (two-sided)
    pub fn t_p_value(t_statistic: f64, df: f64) -> Result<f64, String> {
        let t_dist = StudentsT::new(0.0, 1.0, df)
            .map_err(|e| format!("Failed to create t-distribution: {}", e))?;
        Ok(2.0 * (1.0 - t_dist.cdf(t_statistic.abs())))
    }

    /// Calculate p-value for F-distribution
    pub fn f_p_value(f_statistic: f64, df1: f64, df2: f64) -> Result<f64, String> {
        let f_dist = FisherSnedecor::new(df1, df2)
            .map_err(|e| format!("Failed to create F-distribution: {}", e))?;
        Ok(1.0 - f_dist.cdf(f_statistic))
    }

    /// Calculate p-value for chi-square distribution
    pub fn chi_square_p_value(chi_square: f64, df: f64) -> Result<f64, String> {
        let chi_dist = ChiSquared::new(df)
            .map_err(|e| format!("Failed to create chi-squared distribution: {}", e))?;
        Ok(1.0 - chi_dist.cdf(chi_square))
    }

    /// Calculate quantile from t-distribution
    pub fn t_quantile(p: f64, df: f64) -> Result<f64, String> {
        let t_dist = StudentsT::new(0.0, 1.0, df)
            .map_err(|e| format!("Failed to create t-distribution: {}", e))?;
        Ok(t_dist.inverse_cdf(p))
    }

    /// Calculate quantile from F-distribution
    pub fn f_quantile(p: f64, df1: f64, df2: f64) -> Result<f64, String> {
        let f_dist = FisherSnedecor::new(df1, df2)
            .map_err(|e| format!("Failed to create F-distribution: {}", e))?;
        Ok(f_dist.inverse_cdf(p))
    }

    /// Calculate quantile from chi-square distribution
    pub fn chi_square_quantile(p: f64, df: f64) -> Result<f64, String> {
        let chi_dist = ChiSquared::new(df)
            .map_err(|e| format!("Failed to create chi-squared distribution: {}", e))?;
        Ok(chi_dist.inverse_cdf(p))
    }

    /// Calculate quantile from normal distribution
    pub fn normal_quantile(p: f64) -> f64 {
        if !p.is_finite() || p < 0.0 || p > 1.0 {
            return 0.0; // Return 0 for invalid probability
        }
        let normal = Normal::new(0.0, 1.0)
            .expect("Failed to create standard normal distribution");
        normal.inverse_cdf(p)
    }

    /// Required sample size for detecting a difference in mean with given effect size
    pub fn required_sample_size_for_mean(std_dev: f64, effect_size: f64, alpha: f64, power: f64) -> Result<usize, String> {
        if !std_dev.is_finite() || std_dev <= 0.0 {
            return Err("Invalid standard deviation".to_string());
        }
        if !effect_size.is_finite() || effect_size <= 0.0 {
            return Err("Invalid effect size".to_string());
        }
        if !(0.0..=1.0).contains(&alpha) || !(0.0..=1.0).contains(&power) {
            return Err("alpha and power must be between 0 and 1".to_string());
        }

        let z_alpha = Self::normal_quantile(1.0 - alpha / 2.0);
        let z_beta = Self::normal_quantile(power);
        let n = ((z_alpha + z_beta) * std_dev / effect_size).powi(2).ceil();
        Ok(n as usize)
    }

    /// Calculate confidence interval using t-distribution
    pub fn confidence_interval_t(mean: f64, se: f64, df: f64, confidence: f64) -> Result<(f64, f64), String> {
        let t_critical = Self::t_quantile(1.0 - (1.0 - confidence) / 2.0, df)?;
        let margin = t_critical * se;
        Ok((mean - margin, mean + margin))
    }

    /// Calculate effect size (Cohen's d) for two-sample comparison
    pub fn cohen_d(data1: &[f64], data2: &[f64], equal_var: bool) -> f64 {
        let mean1 = Self::mean(data1);
        let mean2 = Self::mean(data2);
        let mean_diff = mean1 - mean2;

        let pooled_sd = if equal_var {
            Self::pooled_variance(data1, data2).sqrt()
        } else {
            // Use average of standard deviations for unequal variance
            (Self::std_dev(data1) + Self::std_dev(data2)) / 2.0
        };

        if pooled_sd > 0.0 {
            mean_diff / pooled_sd
        } else {
            0.0
        }
    }

    /// Calculate effect size (Cohen's d) for paired comparison
    pub fn cohen_d_paired(data1: &[f64], data2: &[f64]) -> f64 {
        let differences: Vec<f64> = data1.iter().zip(data2.iter())
            .map(|(a, b)| a - b)
            .collect();

        let mean_diff = Self::mean(&differences);
        let std_diff = Self::std_dev(&differences);

        if std_diff > 0.0 {
            mean_diff / std_diff
        } else {
            0.0
        }
    }

    /// Power calculation for t-test (convenience wrapper)
    pub fn power_t_test(delta: f64, sigma: f64, n: usize, alpha: f64, alternative: &str) -> Result<f64, String> {
        match alternative {
            "two-sided" => Self::power_t_test_two_sample(delta, sigma, n, n, alpha, true),
            _ => Err("Only two-sided alternative supported for general t-test".to_string()),
        }
    }

    /// Calculate power for one-sample t-test using non-central t distribution
    pub fn power_t_test_one_sample(delta: f64, sigma: f64, n: usize, alpha: f64) -> Result<f64, String> {
        if sigma <= 0.0 || n == 0 {
            return Err("Invalid parameters".to_string());
        }

        let df = n as f64 - 1.0;
        let t_critical = Self::t_quantile(1.0 - alpha / 2.0, df)?;

        // Non-centrality parameter
        let ncp = delta / (sigma / (n as f64).sqrt());

        // Use exact non-central t distribution
        let nct = NonCentralT::new(ncp, df)?;
        Ok(nct.power_two_sided(t_critical))
    }

    /// Calculate power for two-sample t-test using non-central t distribution
    pub fn power_t_test_two_sample(delta: f64, sigma: f64, n1: usize, n2: usize, alpha: f64, equal_var: bool) -> Result<f64, String> {
        if sigma <= 0.0 || n1 == 0 || n2 == 0 {
            return Err("Invalid parameters".to_string());
        }

        let se = if equal_var {
            let pooled_var = sigma.powi(2);
            (pooled_var / n1 as f64 + pooled_var / n2 as f64).sqrt()
        } else {
            (sigma.powi(2) / n1 as f64 + sigma.powi(2) / n2 as f64).sqrt()
        };

        let df = if equal_var {
            n1 as f64 + n2 as f64 - 2.0
        } else {
            // Welch's approximation
            let df_num = (sigma.powi(2) / n1 as f64 + sigma.powi(2) / n2 as f64).powi(2);
            let df_denom = sigma.powi(4) / (n1 as f64 * n1 as f64 - 1.0) + sigma.powi(4) / (n2 as f64 * n2 as f64 - 1.0);
            df_num / df_denom
        };

        let t_critical = Self::t_quantile(1.0 - alpha / 2.0, df)?;
        let ncp = delta / se;

        // Use exact non-central t distribution
        let nct = NonCentralT::new(ncp, df)?;
        Ok(nct.power_two_sided(t_critical))
    }

    /// Sample size calculation for two-sample t-test
    pub fn sample_size_t_test(delta: f64, sigma: f64, alpha: f64, power: f64, equal_var: bool) -> Result<usize, String> {
        if delta <= 0.0 || sigma <= 0.0 || !(0.0..=1.0).contains(&alpha) || !(0.0..=1.0).contains(&power) {
            return Err("Invalid parameters".to_string());
        }

        // Use iterative approach to find sample size
        let mut n = 10;
        let max_iterations = 1000;

        for _ in 0..max_iterations {
            let current_power = Self::power_t_test_two_sample(delta, sigma, n, n, alpha, equal_var)?;
            if current_power >= power {
                return Ok(n);
            }
            n += 1;
        }

        Err("Could not find suitable sample size".to_string())
    }

    /// Power calculation for chi-square test using non-central chi-square distribution
    pub fn power_chi_square(w: f64, df: f64, n: usize, alpha: f64) -> Result<f64, String> {
        if w <= 0.0 || df <= 0.0 || n == 0 {
            return Err("Invalid parameters".to_string());
        }

        let chi_critical = Self::chi_square_quantile(1.0 - alpha, df)?;

        // Non-centrality parameter: λ = w² * N where w is effect size
        let lambda = Self::chi_square_ncp(w, n);

        Self::non_central_chi_square_power(lambda, df, chi_critical)
    }

    /// Calculate non-centrality parameter for chi-square test
    fn chi_square_ncp(w: f64, n: usize) -> f64 {
        w * w * n as f64
    }

    /// Power calculation using non-central chi-square distribution series expansion
    fn non_central_chi_square_power(lambda: f64, df: f64, chi_critical: f64) -> Result<f64, String> {
        if lambda < 0.0 {
            return Ok(0.0);
        }

        // Use series expansion for non-central chi-square
        let mut power = 0.0;
        let mut term = (-lambda / 2.0).exp();
        let max_terms = 50;

        for k in 0..max_terms {
            if term.abs() < 1e-12 {
                break;
            }

            // Central chi-square with adjusted df
            let adjusted_df = df + 2.0 * k as f64;
            if adjusted_df > 0.0 {
                let chi_dist = ChiSquared::new(adjusted_df)
                    .map_err(|e| format!("Failed to create chi-squared distribution: {}", e))?;
                let term_power = 1.0 - chi_dist.cdf(chi_critical);
                power += term * term_power;
            }

            // Next term: multiply by (λ/2) / (k+1)
            term *= lambda / (2.0 * (k + 1) as f64);
        }

        Ok(power.clamp(0.0, 1.0))
    }

    /// Post-hoc power analysis for t-test
    pub fn post_hoc_power(t_statistic: f64, df: f64, alpha: f64) -> Result<f64, String> {
        let t_critical = Self::t_quantile(1.0 - alpha / 2.0, df)?;

        // Non-centrality parameter is the observed t-statistic
        let nct = NonCentralT::new(t_statistic, df)?;
        Ok(nct.sf(t_critical))
    }

    /// Power calculation for ANOVA
    pub fn power_anova(k: usize, n: usize, f: f64, alpha: f64) -> Result<f64, String> {
        if k < 2 || n == 0 || f <= 0.0 {
            return Err("Invalid parameters".to_string());
        }

        let df_between = k - 1;
        let df_within = k * (n - 1);
        let f_critical = Self::f_quantile(1.0 - alpha, df_between as f64, df_within as f64)?;

        // Non-centrality parameter
        let lambda = f * f * n as f64 * k as f64;

        Self::non_central_f_power(lambda, df_between as f64, df_within as f64, f_critical)
    }

    /// Power calculation using non-central F distribution
    fn non_central_f_power(lambda: f64, df1: f64, df2: f64, f_critical: f64) -> Result<f64, String> {
        if lambda < 0.0 {
            return Ok(0.0);
        }

        // Use approximation for non-central F
        let ncf = NonCentralF::new(lambda, df1, df2)?;
        Ok(1.0 - ncf.cdf(f_critical))
    }
}