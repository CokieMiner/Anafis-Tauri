//! Goodness of fit tests
//!
//! This module provides comprehensive goodness of fit tests
//! for comparing empirical distributions to theoretical models.

/// Result of a goodness of fit test
#[derive(Debug, Clone)]
pub struct GoodnessOfFitResult {
    /// Name of the test
    pub test_name: String,
    /// Test statistic value
    pub statistic: f64,
    /// Degrees of freedom
    pub degrees_of_freedom: usize,
    /// P-value for the test
    pub p_value: f64,
    /// Whether the fit is acceptable at alpha=0.05
    pub good_fit: bool,
    /// Significance level used
    pub alpha: f64,
}

/// Goodness of fit testing engine
pub struct GoodnessOfFitTests;

impl GoodnessOfFitTests {
    /// Kolmogorov-Smirnov test for goodness of fit
    pub fn kolmogorov_smirnov_test<F>(
        data: &[f64],
        cdf: F,
    ) -> Result<GoodnessOfFitResult, String>
    where
        F: Fn(f64) -> Result<f64, String>,
    {
        let n = data.len();
        if n == 0 {
            return Err("Cannot perform KS test on empty data".to_string());
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        let mut max_diff = 0.0f64;

        for (i, &x) in sorted_data.iter().enumerate() {
            let empirical_cdf_plus = (i + 1) as f64 / n as f64;
            let empirical_cdf_minus = i as f64 / n as f64;
            let theoretical_cdf = cdf(x)?;
            let diff_plus = (empirical_cdf_plus - theoretical_cdf).abs();
            let diff_minus = (theoretical_cdf - empirical_cdf_minus).abs();
            max_diff = max_diff.max(diff_plus).max(diff_minus);
        }

        // KS statistic
        let ks_statistic = max_diff;

        // Critical value approximation for p-value
        let p_value = Self::kolmogorov_p_value(ks_statistic, n);

        Ok(GoodnessOfFitResult {
            test_name: "Kolmogorov-Smirnov".to_string(),
            statistic: ks_statistic,
            degrees_of_freedom: n,
            p_value,
            good_fit: p_value > 0.05,
            alpha: 0.05,
        })
    }

    /// Anderson-Darling test for goodness of fit
    pub fn anderson_darling_test<F>(
        data: &[f64],
        cdf: F,
    ) -> Result<GoodnessOfFitResult, String>
    where
        F: Fn(f64) -> Result<f64, String>,
    {
        let n = data.len();
        if n < 2 {
            return Err("Need at least 2 observations for Anderson-Darling test".to_string());
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        let mut sum = 0.0;
        for (i, &x) in sorted_data.iter().enumerate() {
            let i_float = i as f64 + 1.0;
            let theoretical_cdf = cdf(x)?;

            let term1 = (2.0 * i_float - 1.0) * (theoretical_cdf).ln();
            let term2 = (2.0 * (n as f64 - i_float) + 1.0) * (1.0 - theoretical_cdf).ln();

            sum += term1 + term2;
        }

        let ad_statistic = -(n as f64) - sum / n as f64;

        // Adjusted statistic for better small sample performance
        let adjusted_ad = ad_statistic * (1.0 + 0.75 / n as f64 + 2.25 / (n as f64 * n as f64));

        // P-value approximation
        let p_value = Self::anderson_darling_p_value(adjusted_ad, n);

        Ok(GoodnessOfFitResult {
            test_name: "Anderson-Darling".to_string(),
            statistic: adjusted_ad,
            degrees_of_freedom: n,
            p_value,
            good_fit: p_value > 0.05,
            alpha: 0.05,
        })
    }

    /// Cramér-von Mises test for goodness of fit
    pub fn cramer_von_mises_test<F>(
        data: &[f64],
        cdf: F,
    ) -> Result<GoodnessOfFitResult, String>
    where
        F: Fn(f64) -> Result<f64, String>,
    {
        let n = data.len();
        if n == 0 {
            return Err("Cannot perform Cramér-von Mises test on empty data".to_string());
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        let mut sum = 0.0;
        for (i, &x) in sorted_data.iter().enumerate() {
            let i_float = i as f64 + 1.0;
            let empirical_cdf = i_float / n as f64;
            let theoretical_cdf = cdf(x)?;

            let diff = empirical_cdf - theoretical_cdf;
            sum += diff * diff;
        }

        let cvm_statistic = sum / n as f64;

        // P-value approximation (simplified)
        let p_value = Self::cramer_von_mises_p_value(cvm_statistic, n);

        Ok(GoodnessOfFitResult {
            test_name: "Cramér-von Mises".to_string(),
            statistic: cvm_statistic,
            degrees_of_freedom: n,
            p_value,
            good_fit: p_value > 0.05,
            alpha: 0.05,
        })
    }

    /// Chi-squared goodness of fit test for binned data
    pub fn chi_squared_test(
        observed: &[f64],
        expected: &[f64],
        degrees_of_freedom: usize,
    ) -> Result<GoodnessOfFitResult, String> {
        if observed.len() != expected.len() {
            return Err("Observed and expected frequencies must have the same length".to_string());
        }

        if observed.len() <= degrees_of_freedom {
            return Err("Number of bins must be greater than degrees of freedom".to_string());
        }

        let mut chi_squared = 0.0;
        for (&obs, &exp) in observed.iter().zip(expected.iter()) {
            if exp > 0.0 {
                chi_squared += (obs - exp).powi(2) / exp;
            } else if obs > 0.0 {
                return Err("Expected frequency is zero but observed frequency is non-zero".to_string());
            }
        }

        // P-value from chi-squared distribution
        let p_value = Self::chi_squared_cdf(chi_squared, degrees_of_freedom as f64);

        Ok(GoodnessOfFitResult {
            test_name: "Chi-squared".to_string(),
            statistic: chi_squared,
            degrees_of_freedom,
            p_value,
            good_fit: p_value > 0.05,
            alpha: 0.05,
        })
    }



    /// Run comprehensive goodness of fit tests
    pub fn comprehensive_goodness_of_fit<F>(
        data: &[f64],
        cdf: F,
        distribution_name: &str,
    ) -> Result<Vec<GoodnessOfFitResult>, String>
    where
        F: Fn(f64) -> Result<f64, String> + Copy,
    {
        let mut results = Vec::new();

        // Kolmogorov-Smirnov test
        if let Ok(result) = Self::kolmogorov_smirnov_test(data, cdf) {
            results.push(result);
        }

        // Anderson-Darling test
        if let Ok(result) = Self::anderson_darling_test(data, cdf) {
            results.push(result);
        }

        // Cramér-von Mises test
        if let Ok(result) = Self::cramer_von_mises_test(data, cdf) {
            results.push(result);
        }

        // For normality-specific tests, use the consolidated normality_tests module
        if distribution_name.to_lowercase().contains("normal") {
            // Note: Normality tests are now handled by the consolidated NormalityTests module
            // in distributions::normality_tests
        }

        Ok(results)
    }

    // P-value approximations

    fn kolmogorov_p_value(ks: f64, n: usize) -> f64 {
        // Simplified approximation for large n
        let n_float = n as f64;
        let lambda = ks * (n_float).sqrt();

        if lambda < 1.0 {
            1.0
        } else {
            2.0 * (-2.0 * lambda * lambda).exp()
        }
    }

    fn anderson_darling_p_value(ad: f64, n: usize) -> f64 {
        let _n_float = n as f64;
        let a2 = ad;

        // Approximation for p-value
        if a2 < 0.6 {
            1.0 - (-13.436 + 101.14 * a2 - 223.73 * a2 * a2).exp()
        } else {
            (-0.012 * a2 + 0.528).exp()
        }
    }

    fn cramer_von_mises_p_value(cvm: f64, n: usize) -> f64 {
        // Simplified approximation
        let n_float = n as f64;
        (-cvm * n_float).exp()
    }

    fn chi_squared_cdf(x: f64, df: f64) -> f64 {
        // Use the precise implementation from distribution_functions
        crate::scientific::statistics::distributions::distribution_functions::chi_squared_cdf(x, df)
    }
}