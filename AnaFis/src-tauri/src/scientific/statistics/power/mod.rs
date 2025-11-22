//! Power Analysis Module
//!
//! This module provides comprehensive statistical power analysis for:
//! - A priori power analysis (sample size determination)
//! - Post hoc power analysis
//! - Power curves and sensitivity analysis
//! - Multiple test types (t-tests, ANOVA, chi-square, correlations)

use crate::scientific::statistics::distributions::distribution_functions;

/// Power analysis results
#[derive(Debug, Clone)]
pub struct PowerAnalysis {
    pub test_type: String,
    pub power: f64,
    pub effect_size: f64,
    pub sample_size: usize,
    pub alpha: f64,
    pub alternative: String,
    pub method: String,
    pub confidence_interval: Option<(f64, f64)>,
}

/// Power curve data for visualization
#[derive(Debug, Clone)]
pub struct PowerCurve {
    pub test_type: String,
    pub effect_size: f64,
    pub alpha: f64,
    pub alternative: String,
    pub curve_data: Vec<(usize, f64)>, // (sample_size, power)
}

/// Sample size recommendation
#[derive(Debug, Clone)]
pub struct SampleSizeRecommendation {
    pub test_type: String,
    pub target_power: f64,
    pub effect_size: f64,
    pub alpha: f64,
    pub recommended_sample_size: usize,
    pub achieved_power: f64,
    pub justification: String,
}

/// Power Analysis Engine
/// Main engine for statistical power analysis
pub struct PowerAnalysisEngine;

impl PowerAnalysisEngine {
    /// Calculate power for t-test
    pub fn t_test_power(
        effect_size: f64,
        sample_size: usize,
        alpha: f64,
        alternative: &str,
        df: Option<f64>,
    ) -> Result<f64, String> {
        if effect_size <= 0.0 {
            return Err("Effect size must be positive".to_string());
        }
        if sample_size == 0 {
            return Err("Sample size must be positive".to_string());
        }
        if !(0.0..=1.0).contains(&alpha) {
            return Err("Alpha must be between 0 and 1".to_string());
        }

        let degrees_of_freedom = df.unwrap_or((sample_size - 1) as f64);

        match alternative {
            "two.sided" | "two-sided" => {
                Self::t_test_power_two_sided(effect_size, degrees_of_freedom, alpha)
            }
            "one.sided" | "greater" => {
                Self::t_test_power_one_sided(effect_size, degrees_of_freedom, alpha, true)
            }
            "less" => {
                Self::t_test_power_one_sided(effect_size, degrees_of_freedom, alpha, false)
            }
            _ => Err("Invalid alternative hypothesis".to_string()),
        }
    }

    /// Calculate power for two-sample t-test
    pub fn two_sample_t_test_power(
        effect_size: f64,
        n1: usize,
        n2: usize,
        alpha: f64,
        alternative: &str,
    ) -> Result<f64, String> {
        if effect_size <= 0.0 {
            return Err("Effect size must be positive".to_string());
        }
        if n1 == 0 || n2 == 0 {
            return Err("Sample sizes must be positive".to_string());
        }

        // For unequal sample sizes, use harmonic mean for df approximation
        let n_harmonic = 2.0 / (1.0 / n1 as f64 + 1.0 / n2 as f64);
        let df = n_harmonic - 2.0;

        Self::t_test_power(effect_size, n_harmonic as usize, alpha, alternative, Some(df))
    }

    /// Calculate power for ANOVA
    pub fn anova_power(
        effect_size: f64, // f statistic
        groups: usize,
        sample_size_per_group: usize,
        alpha: f64,
    ) -> Result<f64, String> {
        if effect_size <= 0.0 {
            return Err("Effect size must be positive".to_string());
        }
        if groups < 2 {
            return Err("Must have at least 2 groups".to_string());
        }
        if sample_size_per_group == 0 {
            return Err("Sample size per group must be positive".to_string());
        }

        let df_between = (groups - 1) as f64;
        let df_within = (groups * sample_size_per_group - groups) as f64;
        let total_n = groups * sample_size_per_group;

        // Non-centrality parameter
        let lambda = effect_size * effect_size * total_n as f64;

        Self::f_distribution_power(lambda, df_between, df_within, alpha)
    }

    /// Calculate power for correlation test
    pub fn correlation_power(
        correlation: f64,
        sample_size: usize,
        alpha: f64,
        alternative: &str,
    ) -> Result<f64, String> {
        if correlation.abs() >= 1.0 {
            return Err("Correlation must be between -1 and 1".to_string());
        }
        if sample_size < 4 {
            return Err("Sample size must be at least 4".to_string());
        }

        // Fisher transformation
        let z_r = 0.5 * ((1.0 + correlation) / (1.0 - correlation)).ln();
        let se = 1.0 / (sample_size as f64 - 3.0).sqrt();

        match alternative {
            "two.sided" | "two-sided" => {
                let z_critical = distribution_functions::normal_quantile(1.0 - alpha / 2.0);
                let power = 1.0 - distribution_functions::normal_cdf(z_critical - z_r.abs() / se, 0.0, 1.0);
                Ok(power)
            }
            "one.sided" | "greater" => {
                let z_critical = distribution_functions::normal_quantile(1.0 - alpha);
                let power = 1.0 - distribution_functions::normal_cdf(z_critical - z_r / se, 0.0, 1.0);
                Ok(power)
            }
            "less" => {
                let z_critical = distribution_functions::normal_quantile(1.0 - alpha);
                let power = distribution_functions::normal_cdf(-z_critical - z_r / se, 0.0, 1.0);
                Ok(power)
            }
            _ => Err("Invalid alternative hypothesis".to_string()),
        }
    }

    /// Calculate power for chi-square test
    pub fn chi_square_power(
        effect_size: f64, // w statistic (Cohen's w)
        df: f64,
        sample_size: usize,
        alpha: f64,
    ) -> Result<f64, String> {
        if effect_size <= 0.0 {
            return Err("Effect size must be positive".to_string());
        }
        if df <= 0.0 {
            return Err("Degrees of freedom must be positive".to_string());
        }

        // Non-centrality parameter
        let lambda = effect_size * effect_size * sample_size as f64;

        Self::chi_square_power_noncentral(lambda, df, alpha)
    }

    /// Calculate required sample size for target power (t-test)
    pub fn t_test_sample_size(
        effect_size: f64,
        target_power: f64,
        alpha: f64,
        alternative: &str,
    ) -> Result<usize, String> {
        if effect_size <= 0.0 {
            return Err("Effect size must be positive".to_string());
        }
        if !(0.0..=1.0).contains(&target_power) {
            return Err("Target power must be between 0 and 1".to_string());
        }

        // Binary search for sample size
        let mut low = 2;
        let mut high = 10000;

        while low < high {
            let mid = (low + high) / 2;
            let current_power = Self::t_test_power(effect_size, mid, alpha, alternative, None)?;

            if current_power >= target_power {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        Ok(low)
    }

    /// Generate power curve for a range of sample sizes
    pub fn generate_power_curve(
        effect_size: f64,
        alpha: f64,
        alternative: &str,
        min_sample_size: usize,
        max_sample_size: usize,
        steps: usize,
    ) -> Result<PowerCurve, String> {
        if min_sample_size >= max_sample_size {
            return Err("Minimum sample size must be less than maximum".to_string());
        }
        if steps == 0 {
            return Err("Must have at least 1 step".to_string());
        }

        let step_size = (max_sample_size - min_sample_size) / steps;
        let mut curve_data = Vec::with_capacity(steps + 1);

        for i in 0..=steps {
            let sample_size = min_sample_size + i * step_size;
            let power = Self::t_test_power(effect_size, sample_size, alpha, alternative, None)?;
            curve_data.push((sample_size, power));
        }

        Ok(PowerCurve {
            test_type: "t-test".to_string(),
            effect_size,
            alpha,
            alternative: alternative.to_string(),
            curve_data,
        })
    }

    /// Post hoc power analysis
    pub fn post_hoc_power(
        test_statistic: f64,
        df: f64,
        alpha: f64,
        test_type: &str,
    ) -> Result<f64, String> {
        match test_type {
            "t-test" => {
                // For t-test, use non-central t distribution
                let nct = crate::scientific::statistics::primitives::non_central_distributions::NonCentralT::new(test_statistic, df)?;
                let t_critical = distribution_functions::t_quantile(1.0 - alpha / 2.0, df)?;
                Ok(nct.sf(t_critical))
            }
            "f-test" => {
                // For F-test, approximate using non-central F
                let ncf = crate::scientific::statistics::primitives::non_central_distributions::NonCentralF::new(test_statistic, 1.0, df)?;
                let f_critical = distribution_functions::f_quantile(1.0 - alpha, 1.0, df)?;
                Ok(1.0 - ncf.cdf(f_critical))
            }
            _ => Err(format!("Unsupported test type for post hoc power: {}", test_type)),
        }
    }

    /// Calculate comprehensive power analysis
    pub fn comprehensive_power_analysis(
        test_type: &str,
        effect_size: f64,
        sample_size: usize,
        alpha: f64,
        alternative: &str,
    ) -> Result<PowerAnalysis, String> {
        let power = match test_type {
            "one-sample-t" => Self::t_test_power(effect_size, sample_size, alpha, alternative, None)?,
            "two-sample-t" => Self::two_sample_t_test_power(effect_size, sample_size, sample_size, alpha, alternative)?,
            "paired-t" => Self::t_test_power(effect_size, sample_size, alpha, alternative, Some((sample_size - 1) as f64))?,
            "correlation" => Self::correlation_power(effect_size, sample_size, alpha, alternative)?,
            _ => return Err(format!("Unsupported test type: {}", test_type)),
        };

        // Calculate confidence interval for power (simplified)
        let se_power = (power * (1.0 - power) / sample_size as f64).sqrt();
        let z = distribution_functions::normal_quantile(0.975);
        let ci_lower = (power - z * se_power).max(0.0);
        let ci_upper = (power + z * se_power).min(1.0);

        Ok(PowerAnalysis {
            test_type: test_type.to_string(),
            power,
            effect_size,
            sample_size,
            alpha,
            alternative: alternative.to_string(),
            method: "Analytical".to_string(),
            confidence_interval: Some((ci_lower, ci_upper)),
        })
    }

    /// Generate sample size recommendations
    pub fn recommend_sample_size(
        test_type: &str,
        effect_size: f64,
        target_power: f64,
        alpha: f64,
        alternative: &str,
    ) -> Result<SampleSizeRecommendation, String> {
        let recommended_sample_size = match test_type {
            "one-sample-t" | "two-sample-t" | "paired-t" => {
                Self::t_test_sample_size(effect_size, target_power, alpha, alternative)?
            }
            _ => return Err(format!("Sample size calculation not implemented for: {}", test_type)),
        };

        let achieved_power = Self::t_test_power(effect_size, recommended_sample_size, alpha, alternative, None)?;

        let justification = format!(
            "Sample size of {} achieves {:.1}% power for effect size {:.2} with α={}",
            recommended_sample_size,
            achieved_power * 100.0,
            effect_size,
            alpha
        );

        Ok(SampleSizeRecommendation {
            test_type: test_type.to_string(),
            target_power,
            effect_size,
            alpha,
            recommended_sample_size,
            achieved_power,
            justification,
        })
    }

    // Helper functions

    fn t_test_power_two_sided(effect_size: f64, df: f64, alpha: f64) -> Result<f64, String> {
        let t_critical = distribution_functions::t_quantile(1.0 - alpha / 2.0, df)?;
        let nct = crate::scientific::statistics::primitives::non_central_distributions::NonCentralT::new(0.0, df)?;
        let power = nct.sf(t_critical - effect_size) + nct.cdf(-t_critical - effect_size);
        Ok(power)
    }

    fn t_test_power_one_sided(effect_size: f64, df: f64, alpha: f64, greater: bool) -> Result<f64, String> {
        let t_critical = distribution_functions::t_quantile(1.0 - alpha, df)?;
        let nct = crate::scientific::statistics::primitives::non_central_distributions::NonCentralT::new(0.0, df)?;

        let power = if greater {
            nct.sf(t_critical - effect_size)
        } else {
            nct.cdf(-t_critical - effect_size)
        };

        Ok(power)
    }

    fn f_distribution_power(lambda: f64, df1: f64, df2: f64, alpha: f64) -> Result<f64, String> {
        let f_critical = distribution_functions::f_quantile(1.0 - alpha, df1, df2)?;
        let ncf = crate::scientific::statistics::primitives::non_central_distributions::NonCentralF::new(lambda, df1, df2)?;
        Ok(1.0 - ncf.cdf(f_critical))
    }

    fn chi_square_power_noncentral(lambda: f64, df: f64, alpha: f64) -> Result<f64, String> {
        // Use approximation for non-central chi-square
        // For large df, non-central chi-square approaches normal
        if df > 30.0 {
            let mean = df + lambda;
            let sd = (2.0 * (df + 2.0 * lambda)).sqrt();
            let chi_critical = distribution_functions::chi_square_quantile(1.0 - alpha, df)?;
            let z = (chi_critical - mean) / sd;
            Ok(distribution_functions::normal_cdf(z, 0.0, 1.0))
        } else {
            // Simplified approximation
            let chi_critical = distribution_functions::chi_square_quantile(1.0 - alpha, df)?;
            // Approximation: P(χ² > χ²_α | λ) ≈ P(Z > (χ²_α - (df+λ)) / sqrt(2(df+2λ)))
            let mean = df + lambda;
            let variance = 2.0 * (df + 2.0 * lambda);
            let sd = variance.sqrt();
            let z = (chi_critical - mean) / sd;
            Ok(distribution_functions::normal_cdf(z, 0.0, 1.0))
        }
    }
}
