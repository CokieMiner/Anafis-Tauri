//! Consolidated Normality Testing Module
//!
//! This module provides a unified interface for normality tests, consolidating
//! the duplicated implementations from hypothesis_testing.rs and goodness_of_fit.rs.
//! Uses the `normality` crate for reliable, well-tested implementations.

use crate::scientific::statistics::correlation::types::NormalityTestResult;

/// Consolidated normality testing engine
pub struct NormalityTests;

impl NormalityTests {
    /// Run Shapiro-Wilk normality test
    pub fn shapiro_wilk(data: &[f64]) -> Result<NormalityTestResult, String> {
        if data.len() < 3 {
            return Err("Shapiro-Wilk test requires at least 3 observations".to_string());
        }

        if data.len() > 5000 {
            return Err("Shapiro-Wilk test is not suitable for large samples (>5000). Use Shapiro-Francia instead.".to_string());
        }

        let result = normality::shapiro_wilk(data.iter().cloned())
            .map_err(|e| format!("Shapiro-Wilk test failed: {}", e))?;

        Ok(NormalityTestResult {
            test_name: "Shapiro-Wilk".to_string(),
            statistic: result.statistic,
            p_value: result.p_value,
            is_normal: result.p_value > 0.05,
            method: "Shapiro-Wilk W test".to_string(),
        })
    }

    /// Run Anderson-Darling normality test
    pub fn anderson_darling(data: &[f64]) -> Result<NormalityTestResult, String> {
        if data.len() < 2 {
            return Err("Anderson-Darling test requires at least 2 observations".to_string());
        }

        let result = normality::anderson_darling(data.iter().cloned())
            .map_err(|e| format!("Anderson-Darling test failed: {}", e))?;

        Ok(NormalityTestResult {
            test_name: "Anderson-Darling".to_string(),
            statistic: result.statistic,
            p_value: result.p_value,
            is_normal: result.p_value > 0.05,
            method: "Anderson-Darling A² test".to_string(),
        })
    }

    /// Run Jarque-Bera normality test
    pub fn jarque_bera(data: &[f64]) -> Result<NormalityTestResult, String> {
        if data.len() < 2 {
            return Err("Jarque-Bera test requires at least 2 observations".to_string());
        }

        let result = normality::jarque_bera(data.iter().cloned())
            .map_err(|e| format!("Jarque-Bera test failed: {}", e))?;

        Ok(NormalityTestResult {
            test_name: "Jarque-Bera".to_string(),
            statistic: result.statistic,
            p_value: result.p_value,
            is_normal: result.p_value > 0.05,
            method: "Jarque-Bera test".to_string(),
        })
    }

    /// Run Lilliefors normality test
    pub fn lilliefors(data: &[f64]) -> Result<NormalityTestResult, String> {
        if data.len() < 4 {
            return Err("Lilliefors test requires at least 4 observations".to_string());
        }

        let result = normality::lilliefors(data.iter().cloned())
            .map_err(|e| format!("Lilliefors test failed: {}", e))?;

        Ok(NormalityTestResult {
            test_name: "Lilliefors".to_string(),
            statistic: result.statistic,
            p_value: result.p_value,
            is_normal: result.p_value > 0.05,
            method: "Lilliefors test".to_string(),
        })
    }

    /// Run D'Agostino's K-squared normality test
    pub fn dagostino_k_squared(data: &[f64]) -> Result<NormalityTestResult, String> {
        if data.len() < 8 {
            return Err("D'Agostino's K-squared test requires at least 8 observations".to_string());
        }

        let result = normality::dagostino_k_squared(data.iter().cloned())
            .map_err(|e| format!("D'Agostino's K-squared test failed: {}", e))?;

        Ok(NormalityTestResult {
            test_name: "D'Agostino's K-squared".to_string(),
            statistic: result.statistic,
            p_value: result.p_value,
            is_normal: result.p_value > 0.05,
            method: "D'Agostino's K-squared test".to_string(),
        })
    }

    /// Run Anscombe-Glynn kurtosis normality test
    pub fn anscombe_glynn(data: &[f64]) -> Result<NormalityTestResult, String> {
        if data.len() < 5 {
            return Err("Anscombe-Glynn test requires at least 5 observations".to_string());
        }

        let result = normality::anscombe_glynn(data.iter().cloned())
            .map_err(|e| format!("Anscombe-Glynn test failed: {}", e))?;

        Ok(NormalityTestResult {
            test_name: "Anscombe-Glynn".to_string(),
            statistic: result.statistic,
            p_value: result.p_value,
            is_normal: result.p_value > 0.05,
            method: "Anscombe-Glynn kurtosis test".to_string(),
        })
    }

    /// Run Pearson Chi-squared normality test
    pub fn pearson_chi_squared(data: &[f64]) -> Result<NormalityTestResult, String> {
        if data.len() < 5 {
            return Err("Pearson Chi-squared test requires at least 5 observations".to_string());
        }

        let result = normality::pearson_chi_squared(data.iter().cloned(), None, true)
            .map_err(|e| format!("Pearson Chi-squared test failed: {}", e))?;

        Ok(NormalityTestResult {
            test_name: "Pearson Chi-squared".to_string(),
            statistic: result.statistic,
            p_value: result.p_value,
            is_normal: result.p_value > 0.05,
            method: "Pearson Chi-squared test".to_string(),
        })
    }

    /// Run comprehensive normality test battery
    pub fn comprehensive_normality_tests(data: &[f64]) -> Result<Vec<NormalityTestResult>, String> {
        if data.len() < 3 {
            return Err("Normality tests require at least 3 observations".to_string());
        }

        // Deduplicate data for certain tests
        let mut unique_data = data.to_vec();
        unique_data.sort_by(|a, b| a.total_cmp(b));
        unique_data.dedup();
        if unique_data.len() < 3 {
            return Ok(vec![]);
        }

        let mut results = Vec::new();

        // Run tests based on sample size
        let n = data.len();

        // Shapiro-Wilk (generally applicable for n ≥ 3, though less reliable for very large samples)
        if (3..5000).contains(&n) {
            if let Ok(result) = Self::shapiro_wilk(data) {
            results.push(result);
            }
        }

        // Anderson-Darling (good general purpose test, n ≥ 2)
        if let Ok(result) = Self::anderson_darling(data) {
            results.push(result);
        }

        // Jarque-Bera (good for detecting skewness/kurtosis, n ≥ 2)
        if let Ok(result) = Self::jarque_bera(data) {
            results.push(result);
        }

        // Lilliefors (good for small samples, n ≥ 4)
        if n >= 4 {
            if let Ok(result) = Self::lilliefors(data) {
                results.push(result);
            }
        }

        // Pearson Chi-squared (good for larger samples, n ≥ 5)
        if n >= 5 {
            if let Ok(result) = Self::pearson_chi_squared(data) {
                results.push(result);
            }
        }

        // Anscombe-Glynn kurtosis test (n ≥ 5)
        if n >= 5 {
            if let Ok(result) = Self::anscombe_glynn(data) {
                results.push(result);
            }
        }

        // D'Agostino's K-squared (requires larger samples, n ≥ 8)
        if n >= 8 {
            if let Ok(result) = Self::dagostino_k_squared(data) {
                results.push(result);
            }
        }

        Ok(results)
    }
}