//! Statistical power and sample size utilities
//!
//! This module provides functions for statistical power analysis
//! and sample size calculations.

use statrs::distribution::{ContinuousCDF, Normal};

/// Statistical power and sample size utilities
pub struct StatisticalPower;

impl StatisticalPower {
    /// Required sample size for detecting a difference in mean with given effect size
    /// Using normal approximation: n = ((z_{1-alpha/2} + z_{1-beta}) * sigma / delta)^2
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

        let z_alpha = Self::z_quantile(1.0 - alpha / 2.0);
        let z_beta = Self::z_quantile(power);
        let n = ((z_alpha + z_beta) * std_dev / effect_size).powi(2).ceil();
        Ok(n as usize)
    }

    fn z_quantile(p: f64) -> f64 {
        // Standard normal quantile
        let normal = Normal::new(0.0, 1.0).unwrap();
        normal.inverse_cdf(p)
    }
}