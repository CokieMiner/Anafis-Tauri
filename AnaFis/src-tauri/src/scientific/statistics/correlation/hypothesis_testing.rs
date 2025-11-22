//! Hypothesis testing functionality for correlations and normality

use super::types::*;
use crate::scientific::statistics::primitives::RandomSampling;
use crate::scientific::statistics::correlation::CorrelationMethods;
use crate::scientific::statistics::distributions::normality_tests::NormalityTests;
use crate::scientific::statistics::distributions::distribution_functions;
use rand_pcg::Pcg64;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

/// Configuration for hypothesis testing parameters
#[derive(Debug, Clone)]
pub struct TestConfiguration {
    /// Maximum sample size for Shapiro-Wilk test (computationally expensive)
    pub max_sample_size_shapiro_wilk: usize,
    /// Minimum sample size for D'Agostino-Pearson test
    pub min_sample_size_dagostino: usize,
    /// Sample size threshold for using permutation tests instead of parametric
    pub permutation_threshold: usize,
    /// Default significance level
    pub alpha: f64,
    /// Whether to use continuity correction in permutation tests
    pub use_continuity_correction: bool,
    /// Number of permutations for Monte Carlo tests
    pub default_n_permutations: usize,
}

impl Default for TestConfiguration {
    fn default() -> Self {
        Self {
            max_sample_size_shapiro_wilk: 5000,
            min_sample_size_dagostino: 20, // More conservative
            permutation_threshold: 100,    // Use permutations for n ≤ 100
            alpha: 0.05,
            use_continuity_correction: true,
            default_n_permutations: 10000,
        }
    }
}

/// Hypothesis testing engine for correlations
pub struct CorrelationHypothesisTestingEngine {
    config: TestConfiguration,
}

impl Default for CorrelationHypothesisTestingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration parameters for correlation hypothesis tests
#[derive(Debug, Clone)]
struct CorrelationTestConfig {
    var1: usize,
    var2: usize,
    alpha: f64,
    n_permutations: usize,
}

impl CorrelationHypothesisTestingEngine {
    /// Create a new testing engine with default configuration
    pub fn new() -> Self {
        Self {
            config: TestConfiguration::default(),
        }
    }

    /// Create a new testing engine with custom configuration
    pub fn with_config(config: TestConfiguration) -> Self {
        Self { config }
    }

    /// Perform multiple normality tests using consolidated library implementations
    pub fn normality_tests(&self, data: &[f64]) -> Result<Vec<NormalityTestResult>, String> {
        NormalityTests::comprehensive_normality_tests(data)
    }

    /// Correlation hypothesis tests
    #[allow(clippy::too_many_arguments)]
    pub fn correlation_tests(
        &self,
        x: &[f64],
        y: &[f64],
        var1: usize,
        var2: usize,
        alpha: Option<f64>,
        n_permutations: Option<usize>,
        rng: &mut Pcg64,
    ) -> Result<Vec<CorrelationTestResult>, String> {
        if x.len() != y.len() || x.len() < 3 {
            return Err("Correlation tests require paired data with at least 3 observations".to_string());
        }

        let alpha = alpha.unwrap_or(self.config.alpha);
        let n_permutations = n_permutations.unwrap_or(self.config.default_n_permutations);
        let test_config = CorrelationTestConfig {
            var1,
            var2,
            alpha,
            n_permutations,
        };

        let mut results = Vec::new();

        // Pearson correlation test
        if let Ok(pearson_result) = self.pearson_correlation_test(x, y, &test_config, rng) {
            results.push(pearson_result);
        }

        // Spearman correlation test
        if let Ok(spearman_result) = self.spearman_correlation_test(x, y, &test_config, rng) {
            results.push(spearman_result);
        }

        // Kendall correlation test (approximate p-value using normal approximation)
        if let Ok(kendall_result) = self.kendall_correlation_test(x, y, &test_config, rng) {
            results.push(kendall_result);
        }

        Ok(results)
    }

    /// Pearson correlation hypothesis test
    fn pearson_correlation_test(
        &self,
        x: &[f64],
        y: &[f64],
        config: &CorrelationTestConfig,
        rng: &mut Pcg64,
    ) -> Result<CorrelationTestResult, String> {
        let n = x.len() as f64;
        let r = CorrelationMethods::pearson_correlation(x, y, None, None).map(|(r, _)| r)?;

        // t-statistic for correlation coefficient
        let t_statistic = if (1.0 - r * r) > 0.0 {
            r * ((n - 2.0) / (1.0 - r * r)).sqrt()
        } else {
            // Handle perfect correlation
            if r > 0.0 { f64::INFINITY } else { f64::NEG_INFINITY }
        };

        let df = n - 2.0;

        // Two-tailed p-value using centralized distribution functions
        // For small samples, prefer permutation p-value for robustness
        let p_value = if x.len() <= self.config.permutation_threshold {
            self.improved_permutation_p_value(x, y, |a, b| CorrelationMethods::pearson_correlation(a, b, None, None).map(|(r, _)| r).unwrap_or(0.0), config.n_permutations, rng)?
        } else {
            2.0 * (1.0 - distribution_functions::t_cdf(t_statistic.abs(), df))
        };

        Ok(CorrelationTestResult {
            method: "Pearson".to_string(),
            variable_1: config.var1,
            variable_2: config.var2,
            correlation: r,
            statistic: t_statistic,
            p_value,
            significant: p_value < config.alpha,
        })
    }

    /// Spearman correlation hypothesis test
    fn spearman_correlation_test(
        &self,
        x: &[f64],
        y: &[f64],
        config: &CorrelationTestConfig,
        rng: &mut Pcg64,
    ) -> Result<CorrelationTestResult, String> {
        // Convert to ranks using centralized ranking function
        let x_ranks = crate::scientific::statistics::correlation::utils::rank_data(x);
        let y_ranks = crate::scientific::statistics::correlation::utils::rank_data(y);

        self.pearson_correlation_test(&x_ranks, &y_ranks, config, rng).map(|mut result| {
            result.method = "Spearman".to_string();
            result
        })
    }

    /// Kendall tau correlation hypothesis test (approximate)
    fn kendall_correlation_test(
        &self,
        x: &[f64],
        y: &[f64],
        config: &CorrelationTestConfig,
        rng: &mut Pcg64,
    ) -> Result<CorrelationTestResult, String> {
        let n = x.len();
        if n < 2 { return Err("Need at least 2 observations".to_string()); }

        let (tau, var_tau) = self.kendall_correlation_improved(x, y)?;

        let z = tau / var_tau.sqrt();

        // Use permutation p-value for Kendall when n small
        let p_value = if n <= self.config.permutation_threshold {
            self.improved_permutation_p_value(x, y, |a, b| self.kendall_correlation_improved(a, b).map(|(t, _)| t).unwrap_or(0.0), config.n_permutations, rng)?
        } else {
            2.0 * (1.0 - distribution_functions::normal_cdf(z.abs(), 0.0, 1.0))
        };

        Ok(CorrelationTestResult {
            method: "Kendall".to_string(),
            variable_1: config.var1,
            variable_2: config.var2,
            correlation: tau,
            statistic: z,
            p_value,
            significant: p_value < config.alpha,
        })
    }

    /// Improved Monte Carlo / permutation test for correlation-based statistics
    /// Uses more accurate p-value calculation by including observed statistic in distribution
    fn improved_permutation_p_value<F>(
        &self,
        x: &[f64],
        y: &[f64],
        statistic_fn: F,
        n_permutations: usize,
        rng: &mut Pcg64,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64], &[f64]) -> f64 + Send + Sync,
    {
        if x.len() != y.len() {
            return Err("Paired data required for permutation test".to_string());
        }

        let observed = statistic_fn(x, y);

        // Generate seeds for each permutation
        let seeds: Vec<u64> = (0..n_permutations).map(|_| rng.random::<u64>()).collect();

        // Generate permutation statistics in parallel
        let mut permutation_results: Vec<f64> = seeds
            .into_par_iter()
            .map(|seed| {
                // Create a new RNG for each permutation using the pre-generated seed
                let mut local_rng = Pcg64::seed_from_u64(seed);
                let mut y_permuted = y.to_vec();
                RandomSampling::shuffle(&mut local_rng, &mut y_permuted);
                statistic_fn(x, &y_permuted)
            })
            .collect();

        // Include the observed statistic in the permutation distribution
        permutation_results.push(observed);

        // Sort by absolute value to find rank of observed statistic
        permutation_results.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap());

        // Find the rank of the observed statistic
        let rank = permutation_results
            .iter()
            .position(|&x| (x - observed).abs() < 1e-10)
            .unwrap_or(permutation_results.len());

        // Calculate p-value as proportion of statistics at least as extreme
        let p_value = rank as f64 / permutation_results.len() as f64;

        // Apply continuity correction if configured
        if self.config.use_continuity_correction && n_permutations > 10 {
            // Continuity correction for better small sample performance
            let correction = 0.5 / (n_permutations + 1) as f64;
            Ok((p_value - correction).max(0.0))
        } else {
            Ok(p_value)
        }
    }

    /// Improved Kendall tau correlation with proper ties handling
    fn kendall_correlation_improved(&self, x: &[f64], y: &[f64]) -> Result<(f64, f64), String> {
        let n = x.len();
        if n < 2 {
            return Err("Need at least 2 observations".to_string());
        }

        let mut concordant = 0i64;
        let mut discordant = 0i64;
        let mut x_ties = 0i64;
        let mut y_ties = 0i64;

        // Count concordant/discordant pairs and ties
        for i in 0..n {
            for j in (i + 1)..n {
                match (x[i].partial_cmp(&x[j]), y[i].partial_cmp(&y[j])) {
                    (Some(Ordering::Greater), Some(Ordering::Greater)) => concordant += 1,
                    (Some(Ordering::Less), Some(Ordering::Less)) => concordant += 1,
                    (Some(Ordering::Greater), Some(Ordering::Less)) => discordant += 1,
                    (Some(Ordering::Less), Some(Ordering::Greater)) => discordant += 1,
                    (Some(Ordering::Equal), _) => x_ties += 1,
                    (_, Some(Ordering::Equal)) => y_ties += 1,
                    _ => {} // Handle NaN cases
                }
            }
        }

        let total_pairs = (n * (n - 1) / 2) as i64;
        let tau = if total_pairs - x_ties > 0 && total_pairs - y_ties > 0 {
            (concordant - discordant) as f64 /
                ((total_pairs - x_ties) * (total_pairs - y_ties)) as f64
        } else {
            0.0
        };

        // Better variance estimation with ties (Kendall, 1970)
        let n_f = n as f64;
        let proportion_x_ties = x_ties as f64 / total_pairs as f64;
        let proportion_y_ties = y_ties as f64 / total_pairs as f64;

        let var_tau = (4.0 * n_f + 10.0) / (9.0 * n_f * (n_f - 1.0)) *
                     (1.0 + proportion_x_ties) * (1.0 + proportion_y_ties);

        Ok((tau, var_tau))
    }

    /// Calculate confidence interval for correlation coefficient
    pub fn correlation_confidence_interval(
        &self,
        r: f64,
        n: usize,
        alpha: Option<f64>,
    ) -> Result<(f64, f64), String> {
        if n < 3 {
            return Err("Need at least 3 observations for confidence interval".to_string());
        }

        let alpha = alpha.unwrap_or(self.config.alpha);
        let z = (1.0 + r).ln() / 2.0; // Fisher z-transformation
        let se = 1.0 / (n as f64 - 3.0).sqrt(); // Standard error
        let z_critical = distribution_functions::normal_quantile(1.0 - alpha / 2.0);

        let z_lower = z - z_critical * se;
        let z_upper = z + z_critical * se;

        // Transform back to correlation scale
        let r_lower = (2.0 * z_lower).exp().tanh();
        let r_upper = (2.0 * z_upper).exp().tanh();

        Ok((r_lower, r_upper))
    }

    /// Calculate effect size measures for correlation
    pub fn correlation_effect_size(&self, r: f64) -> CorrelationEffectSize {
        let r_squared = r * r;

        // Cohen's guidelines for correlation effect sizes
        let magnitude = if r_squared >= 0.25 {
            "large"
        } else if r_squared >= 0.09 {
            "medium"
        } else if r_squared >= 0.01 {
            "small"
        } else {
            "negligible"
        };

        CorrelationEffectSize {
            r_squared,
            magnitude: magnitude.to_string(),
            shared_variance_percent: r_squared * 100.0,
        }
    }

    /// Apply multiple testing correction (Bonferroni)
    pub fn bonferroni_correction(&self, p_values: &[f64]) -> Vec<f64> {
        crate::scientific::statistics::correlation::correction_methods::bonferroni_correction(p_values)
    }

    /// Apply Benjamini-Hochberg FDR correction
    pub fn benjamini_hochberg_correction(&self, p_values: &[f64]) -> Vec<f64> {
        crate::scientific::statistics::correlation::correction_methods::benjamini_hochberg_correction(p_values)
    }

    /// Streaming normality tests for large datasets (memory efficient)
    pub fn streaming_normality_tests(
        &self,
        data_iter: impl Iterator<Item = f64>,
        sample_size: Option<usize>,
    ) -> Result<Vec<NormalityTestResult>, String> {
        // Use reservoir sampling for large datasets
        let max_sample = sample_size.unwrap_or(5000);
        let mut reservoir: Vec<f64> = Vec::with_capacity(max_sample);
        let mut count = 0u64;

        // Welford's algorithm for online mean/variance calculation
        let mut mean = 0.0;
        let mut m2 = 0.0;
        let mut skewness = 0.0;
        let mut kurtosis = 0.0;

        for value in data_iter {
            count += 1;

            // Reservoir sampling
            if reservoir.len() < max_sample {
                reservoir.push(value);
            } else if rand::random::<f64>() < max_sample as f64 / count as f64 {
                let replace_idx = (rand::random::<f64>() * max_sample as f64) as usize;
                reservoir[replace_idx] = value;
            }

            // Online moment calculation
            let delta = value - mean;
            let delta_n = delta / count as f64;
            let term1 = delta * delta_n * (count - 1) as f64;

            mean += delta_n;
            m2 += term1;

            // Update higher moments (approximate)
            if count > 1 {
                let variance = m2 / (count - 1) as f64;
                if variance > 0.0 {
                    let std_val = (value - mean) / variance.sqrt();
                    skewness += (std_val.powi(3) - skewness) / count as f64;
                    kurtosis += (std_val.powi(4) - kurtosis) / count as f64;
                }
            }
        }

        if reservoir.len() < 3 {
            return Err("Insufficient data for normality testing".to_string());
        }

        // Use consolidated normality tests on reservoir sample
        NormalityTests::comprehensive_normality_tests(&reservoir)
    }

    /// Comprehensive correlation analysis combining tests, effect sizes and confidence intervals
    #[allow(clippy::too_many_arguments)]
    pub fn comprehensive_correlation_analysis(
        &self,
        x: &[f64],
        y: &[f64],
        var1: usize,
        var2: usize,
        alpha: Option<f64>,
        n_permutations: Option<usize>,
        rng: &mut Pcg64,
    ) -> Result<ComprehensiveCorrelationResult, String> {
        if x.len() != y.len() || x.len() < 3 {
            return Err("Correlation analysis requires paired data with at least 3 observations".to_string());
        }

        let alpha = alpha.unwrap_or(self.config.alpha);

        // Run correlation tests
        let tests = self.correlation_tests(x, y, var1, var2, Some(alpha), n_permutations, rng)?;

        // Calculate comprehensive results
        let mut comprehensive_tests = Vec::new();

        for test in tests {
            // Calculate confidence interval for this correlation
            let ci = self.correlation_confidence_interval(test.correlation, x.len(), Some(alpha))?;

            // Calculate effect size
            let effect_size = self.correlation_effect_size(test.correlation);

            comprehensive_tests.push(ComprehensiveCorrelationTestResult {
                test_result: test,
                confidence_interval: ci,
                effect_size,
            });
        }

        Ok(ComprehensiveCorrelationResult {
            variable_1: var1,
            variable_2: var2,
            sample_size: x.len(),
            tests: comprehensive_tests,
        })
    }

    /// Power analysis for correlation tests
    pub fn correlation_power_analysis(
        &self,
        effect_size: f64,
        n: usize,
        alpha: Option<f64>,
        power: Option<f64>,
    ) -> Result<PowerAnalysisResult, String> {
        use crate::scientific::statistics::PowerAnalysisEngine;

        let alpha = alpha.unwrap_or(self.config.alpha);

        if let Some(desired_power) = power {
            // Calculate required sample size
            let sample_size = PowerAnalysisEngine::t_test_sample_size(effect_size, desired_power, alpha, "two.sided")?;
            let achieved_power = PowerAnalysisEngine::correlation_power(effect_size, sample_size, alpha, "two.sided")?;

            Ok(PowerAnalysisResult {
                effect_size,
                current_power: achieved_power,
                required_sample_size: sample_size,
                alpha,
                desired_power,
            })
        } else {
            // Calculate achieved power
            let achieved_power = PowerAnalysisEngine::correlation_power(effect_size, n, alpha, "two.sided")?;
            Ok(PowerAnalysisResult {
                effect_size,
                current_power: achieved_power,
                required_sample_size: n,
                alpha,
                desired_power: achieved_power,
            })
        }
    }
}

/// Effect size measures for correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEffectSize {
    pub r_squared: f64,
    pub magnitude: String,
    pub shared_variance_percent: f64,
}

/// Comprehensive correlation test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveCorrelationTestResult {
    pub test_result: CorrelationTestResult,
    pub confidence_interval: (f64, f64),
    pub effect_size: CorrelationEffectSize,
}

/// Comprehensive correlation analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveCorrelationResult {
    pub variable_1: usize,
    pub variable_2: usize,
    pub sample_size: usize,
    pub tests: Vec<ComprehensiveCorrelationTestResult>,
}

/// Power analysis result for correlation tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysisResult {
    pub effect_size: f64,
    pub current_power: f64,
    pub required_sample_size: usize,
    pub alpha: f64,
    pub desired_power: f64,
}

// Output methods for ComprehensiveCorrelationResult
impl ComprehensiveCorrelationResult {
    /// Export results to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export results to CSV format
    pub fn to_csv(&self) -> String {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write header
        let _ = wtr.write_record([
            "Variable1", "Variable2", "SampleSize", "Method", "Correlation",
            "Statistic", "P_Value", "Significant", "CI_Lower", "CI_Upper",
            "R_Squared", "Effect_Size", "Shared_Variance_Percent"
        ]);

        // Write data rows
        for test in &self.tests {
            let _ = wtr.write_record(&[
                self.variable_1.to_string(),
                self.variable_2.to_string(),
                self.sample_size.to_string(),
                test.test_result.method.clone(),
                test.test_result.correlation.to_string(),
                test.test_result.statistic.to_string(),
                test.test_result.p_value.to_string(),
                test.test_result.significant.to_string(),
                test.confidence_interval.0.to_string(),
                test.confidence_interval.1.to_string(),
                test.effect_size.r_squared.to_string(),
                test.effect_size.magnitude.clone(),
                test.effect_size.shared_variance_percent.to_string(),
            ]);
        }

        String::from_utf8(wtr.into_inner().unwrap()).unwrap()
    }
}