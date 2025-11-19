//! Hypothesis testing functionality for correlations and normality

use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::distribution::moments;
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::RandomSampling;
use crate::scientific::statistics::types::NormalityTestResult;
use crate::scientific::statistics::comprehensive_analysis::traits::{ProgressCallback, NoOpProgressCallback};
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::correlation_methods::CorrelationMethods;
use crate::scientific::statistics::types::CorrelationTestResult;
use rand_pcg::Pcg64;
use rand::{Rng, SeedableRng};
use statrs::distribution::ContinuousCDF;
use rayon::prelude::*;

/// Hypothesis testing engine
pub struct CorrelationHypothesisTestingEngine;

impl CorrelationHypothesisTestingEngine {
    /// Perform multiple normality tests using robust library implementations
    pub fn normality_tests(data: &[f64]) -> Result<Vec<NormalityTestResult>, String> {
        if data.len() < 3 {
            return Err("Normality tests require at least 3 observations".to_string());
        }

        // Deduplicate data for certain tests
        let mut unique_data = data.to_vec();
        unique_data.sort_by(|a, b| a.total_cmp(b));
        unique_data.dedup();
        if unique_data.len() < 3 {
            // Data is constant or has very few unique values, not suitable for normality testing
            return Ok(vec![]);
        }

        let data_vec = data.to_vec();

        // Run normality tests in parallel
        use rayon::prelude::*;
        let test_results: Vec<Option<NormalityTestResult>> = [
            // Shapiro-Wilk test
            (data.len() <= 5000).then(|| {
                normality::shapiro_wilk(data_vec.clone()).ok().map(|result| {
                    NormalityTestResult {
                        test_name: "Shapiro-Wilk".to_string(),
                        statistic: result.statistic,
                        p_value: result.p_value,
                        is_normal: result.p_value > 0.05,
                        method: "Shapiro-Wilk W test".to_string(),
                    }
                })
            }),
            // Anderson-Darling test
            Some(normality::anderson_darling(data_vec.clone()).ok().map(|result| {
                NormalityTestResult {
                    test_name: "Anderson-Darling".to_string(),
                    statistic: result.statistic,
                    p_value: result.p_value,
                    is_normal: result.p_value > 0.05,
                    method: "Anderson-Darling A² test".to_string(),
                }
            })),
            // Jarque-Bera test
            Some(normality::jarque_bera(data_vec.clone()).ok().map(|result| {
                NormalityTestResult {
                    test_name: "Jarque-Bera".to_string(),
                    statistic: result.statistic,
                    p_value: result.p_value,
                    is_normal: result.p_value > 0.05,
                    method: "Jarque-Bera test".to_string(),
                }
            })),
            // Lilliefors test
            (data.len() >= 4).then(|| {
                normality::lilliefors(data_vec.clone()).ok().map(|result| {
                    NormalityTestResult {
                        test_name: "Lilliefors".to_string(),
                        statistic: result.statistic,
                        p_value: result.p_value,
                        is_normal: result.p_value > 0.05,
                        method: "Lilliefors test".to_string(),
                    }
                })
            }),
            // D'Agostino-Pearson test
            (data.len() >= 8).then(|| {
                Self::dagostino_pearson_test(data).ok()
            }),
        ].into_par_iter()
        .map(|test_fn| test_fn.flatten())
        .collect();

        // Filter out None results and collect successful tests
        let results: Vec<NormalityTestResult> = test_results.into_iter().flatten().collect();

        Ok(results)
    }

    /// D'Agostino-Pearson omnibus normality test
    /// This test is not directly in `statrs`, so we implement it using `statrs` components.
    fn dagostino_pearson_test(data: &[f64]) -> Result<NormalityTestResult, String> {
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();
        let skew = data.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum::<f64>() / n;
        let kurt = data.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum::<f64>() / n - 3.0;

        // Transform skewness to Z-score
        let y = skew * (((n + 1.0) * (n + 3.0)) / (6.0 * (n - 2.0))).sqrt();
        let beta2_skew = (3.0 * (n.powi(2) + 27.0*n - 70.0) * (n + 1.0) * (n + 3.0)) / ((n - 2.0) * (n + 5.0) * (n + 7.0) * (n + 9.0));
        let w2 = -1.0 + (2.0 * (beta2_skew - 1.0)).sqrt();
        let delta = 1.0 / (w2.ln()).sqrt();
        let alpha = (2.0 / (w2 - 1.0)).sqrt();
        let z_skew = delta * (y / alpha).asinh();

        // Transform kurtosis to Z-score
        let mean_kurt = (3.0 * (n - 1.0)) / (n + 1.0);
        let var_kurt = (24.0 * n * (n - 2.0) * (n - 3.0)) / ((n + 1.0).powi(2) * (n + 3.0) * (n + 5.0));
        let _x_kurt = (kurt - mean_kurt) / var_kurt.sqrt();
        let beta1_kurt = (6.0 * (n.powi(2) - 5.0*n + 2.0) / ((n + 7.0) * (n + 9.0))) * (6.0 * (n + 3.0) * (n + 5.0) / (n * (n - 2.0) * (n - 3.0))).sqrt();
        let a_kurt = 6.0 + (8.0 / beta1_kurt) * (2.0 / beta1_kurt + (1.0 + 4.0 / beta1_kurt.powi(2)).sqrt());
        let z_kurt = ((1.0 - 2.0 / a_kurt).powf(1.0/3.0) - 1.0 + (2.0 / (9.0 * a_kurt))) / (2.0 / (9.0 * a_kurt)).sqrt();

        // K^2 statistic
        let k_squared = z_skew.powi(2) + z_kurt.powi(2);

        // P-value from chi-squared distribution with 2 degrees of freedom
        let chi_squared = statrs::distribution::ChiSquared::new(2.0)
            .map_err(|e| e.to_string())?;
        let p_value = 1.0 - chi_squared.cdf(k_squared);

        Ok(NormalityTestResult {
            test_name: "D'Agostino-Pearson".to_string(),
            statistic: k_squared,
            p_value: p_value,
            is_normal: p_value > 0.05,
            method: "D'Agostino-Pearson omnibus test".to_string(),
        })
    }

    /// Correlation hypothesis tests
    pub fn correlation_tests(
        x: &[f64],
        y: &[f64],
        var1: usize,
        var2: usize,
        alpha: Option<f64>,
        n_permutations: usize,
        rng: &mut Pcg64,
    ) -> Result<Vec<CorrelationTestResult>, String> {
        if x.len() != y.len() || x.len() < 3 {
            return Err("Correlation tests require paired data with at least 3 observations".to_string());
        }

        let mut results = Vec::new();

        // Pearson correlation test
        if let Ok(pearson_result) = Self::pearson_correlation_test(x, y, var1, var2, alpha, n_permutations, rng) {
            results.push(pearson_result);
        }

        // Spearman correlation test
        if let Ok(spearman_result) = Self::spearman_correlation_test(x, y, var1, var2, alpha, n_permutations, rng) {
            results.push(spearman_result);
        }

        // Kendall correlation test (approximate p-value using normal approximation)
        if let Ok(kendall_result) = Self::kendall_correlation_test(x, y, var1, var2, alpha, n_permutations, rng) {
            results.push(kendall_result);
        }

        Ok(results)
    }

    /// Pearson correlation hypothesis test
    fn pearson_correlation_test(
        x: &[f64],
        y: &[f64],
        var1: usize,
        var2: usize,
        alpha: Option<f64>,
        n_permutations: usize,
        rng: &mut Pcg64,
    ) -> Result<CorrelationTestResult, String> {
        let n = x.len() as f64;
        let r = CorrelationMethods::pearson_correlation(x, y)?;

        // t-statistic for correlation coefficient
        let t_statistic = if (1.0 - r * r) > 0.0 {
            r * ((n - 2.0) / (1.0 - r * r)).sqrt()
        } else {
            // Handle perfect correlation
            if r > 0.0 { f64::INFINITY } else { f64::NEG_INFINITY }
        };

        let df = n - 2.0;

        // Two-tailed p-value using statrs
        let t_dist = statrs::distribution::StudentsT::new(0.0, 1.0, df)
            .map_err(|e| e.to_string())?;
        // For small samples, prefer permutation p-value for robustness
        let p_value = if x.len() <= 50 {
            Self::permutation_p_value(x, y, |a, b| CorrelationMethods::pearson_correlation(a, b).unwrap_or(0.0), n_permutations, rng)?
        } else {
            2.0 * (1.0 - t_dist.cdf(t_statistic.abs()))
        };

        let significance_level = alpha.unwrap_or(0.05);
        Ok(CorrelationTestResult {
            method: "Pearson".to_string(),
            variable_1: var1,
            variable_2: var2,
            correlation: r,
            statistic: t_statistic,
            p_value,
            significant: p_value < significance_level,
        })
    }

    /// Spearman correlation hypothesis test
    fn spearman_correlation_test(
        x: &[f64],
        y: &[f64],
        var1: usize,
        var2: usize,
        alpha: Option<f64>,
        n_permutations: usize,
        rng: &mut Pcg64,
    ) -> Result<CorrelationTestResult, String> {
        let x_ranks = moments::rank_transformation(x);
        let y_ranks = moments::rank_transformation(y);

        Self::pearson_correlation_test(&x_ranks, &y_ranks, var1, var2, alpha, n_permutations, rng).map(|mut result| {
            result.method = "Spearman".to_string();
            result
        })
    }

    /// Kendall tau correlation hypothesis test (approximate)
    fn kendall_correlation_test(
        x: &[f64],
        y: &[f64],
        var1: usize,
        var2: usize,
        alpha: Option<f64>,
        n_permutations: usize,
        rng: &mut Pcg64,
    ) -> Result<CorrelationTestResult, String> {
        let n = x.len();
        if n < 2 { return Err("Need at least 2 observations".to_string()); }

        let tau = CorrelationMethods::kendall_correlation(x, y)?;

        // Approximate variance for Kendall's tau under H0 (no ties correction)
        let n_f = n as f64;
        let var_tau = 2.0 * (2.0 * n_f + 5.0) / (9.0 * n_f * (n_f - 1.0));
        let z = tau / var_tau.sqrt();

        // Use permutation p-value for Kendall when n small
        let p_value = if n <= 50 {
            Self::permutation_p_value(x, y, |a, b| CorrelationMethods::kendall_correlation(a, b).unwrap_or(0.0), n_permutations, rng)?
        } else {
            let normal = statrs::distribution::Normal::new(0.0, 1.0).map_err(|e| e.to_string())?;
            2.0 * (1.0 - normal.cdf(z.abs()))
        };

        let significance_level = alpha.unwrap_or(0.05);
        Ok(CorrelationTestResult {
            method: "Kendall".to_string(),
            variable_1: var1,
            variable_2: var2,
            correlation: tau,
            statistic: z,
            p_value,
            significant: p_value < significance_level,
        })
    }

    /// Monte Carlo / permutation test for correlation-based statistics
    fn permutation_p_value<F>(
        x: &[f64],
        y: &[f64],
        statistic_fn: F,
        n_permutations: usize,
        rng: &mut Pcg64,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64], &[f64]) -> f64 + Send + Sync,
    {
        Self::permutation_p_value_with_progress(x, y, statistic_fn, n_permutations, rng, &NoOpProgressCallback)
    }

    /// Monte Carlo / permutation test for correlation-based statistics with progress reporting
    fn permutation_p_value_with_progress<F, P>(
        x: &[f64],
        y: &[f64],
        statistic_fn: F,
        n_permutations: usize,
        rng: &mut Pcg64,
        progress_callback: &P,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64], &[f64]) -> f64 + Send + Sync,
        P: ProgressCallback,
    {
        if x.len() != y.len() {
            return Err("Paired data required for permutation test".to_string());
        }

        let observed = statistic_fn(x, y);

        // Generate seeds for each permutation
        let seeds: Vec<u64> = (0..n_permutations).map(|_| rng.random::<u64>()).collect();

        // Generate permutation statistics in parallel
        let permutation_results: Vec<f64> = seeds
            .into_par_iter()
            .enumerate()
            .map(|(i, seed)| {
                if i % 1000 == 0 { // Report progress every 1000 iterations
                    progress_callback.report_progress(i, n_permutations, "Running permutation tests...");
                }
                // Create a new RNG for each permutation using the pre-generated seed
                let mut local_rng = Pcg64::seed_from_u64(seed);
                let mut y_permuted = y.to_vec();
                RandomSampling::shuffle(&mut local_rng, &mut y_permuted);
                statistic_fn(x, &y_permuted)
            })
            .collect();

        progress_callback.report_progress(n_permutations, n_permutations, "Permutation tests completed");

        // Count how many permutation statistics are at least as extreme as observed
        let count = permutation_results
            .iter()
            .filter(|&&stat| stat.abs() >= observed.abs())
            .count();

        // Add 1 to numerator and denominator to avoid p=0 (Phipson & Smyth, 2010)
        Ok((count as f64 + 1.0) / (n_permutations as f64 + 1.0))
    }
}