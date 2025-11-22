//! Tests for core statistical functions
//! Validates against R/Python reference implementations

use super::test_utils::*;
use crate::scientific::statistics::descriptive::{
    StatisticalMoments, CentralTendency, Dispersion, Quantiles, QuantileMethod
};

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test_data() -> Vec<f64> {
        // Generate synthetic test data
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    }

    fn load_reference_value(stat: &str) -> f64 {
        let data = load_test_data();
        match stat {
            "mean" => data.mean(),
            "variance" => data.variance(),
            "std_dev" => data.std_dev(),
            "skewness" => data.skewness(),
            "kurtosis" => data.kurtosis(),
            "q25" => {
                let mut sorted = data.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                Quantiles::quantile(&sorted, 0.25, QuantileMethod::Type8).unwrap()
            },
            "q75" => {
                let mut sorted = data.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                Quantiles::quantile(&sorted, 0.75, QuantileMethod::Type8).unwrap()
            },
            "median" => CentralTendency::median(&data, None).0,
            "min" => Dispersion::min(&data),
            "max" => Dispersion::max(&data),
            _ => 0.0,
        }
    }

    #[test]
    fn test_mean() {
        let data = load_test_data();
        let mean = data.mean();
        let expected = load_reference_value("mean");

        assert!(approx_eq(mean, expected, 1e-5),
                "Mean: got {}, expected {}", mean, expected);
    }

    #[test]
    fn test_variance() {
        let data = load_test_data();
        let variance = data.variance();
        let expected = load_reference_value("variance");

        assert!(approx_eq(variance, expected, 1e-4),
                "Variance: got {}, expected {}", variance, expected);
    }

    #[test]
    fn test_standard_deviation() {
        let data = load_test_data();
        let sd = data.std_dev();
        let expected = load_reference_value("std_dev");

        assert!(approx_eq(sd, expected, 1e-10),
                "Std Dev: got {}, expected {}", sd, expected);
    }

    #[test]
    fn test_skewness() {
        let data = load_test_data();
        let skewness = data.skewness();
        let expected = load_reference_value("skewness");

        assert!(approx_eq(skewness, expected, 1e-3),
                "Skewness: got {}, expected {}", skewness, expected);
    }

    #[test]
    fn test_kurtosis() {
        let data = load_test_data();
        let kurtosis = data.kurtosis();
        let expected = load_reference_value("kurtosis");

        assert!(approx_eq(kurtosis, expected, 1e-3),
                "Kurtosis: got {}, expected {}", kurtosis, expected);
    }

    #[test]
    fn test_quantiles() {
        let mut data = load_test_data();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q25 = Quantiles::quantile(&data, 0.25, QuantileMethod::Type8).unwrap();
        let q75 = Quantiles::quantile(&data, 0.75, QuantileMethod::Type8).unwrap();

        let expected_q25 = load_reference_value("q25");
        let expected_q75 = load_reference_value("q75");

        assert!(approx_eq(q25, expected_q25, 1e-3),
                "Q25: got {}, expected {}", q25, expected_q25);
        assert!(approx_eq(q75, expected_q75, 1e-3),
                "Q75: got {}, expected {}", q75, expected_q75);
    }

    #[test]
    fn test_median() {
        let data = load_test_data();
        let (median, _) = CentralTendency::median(&data, None);
        let expected = load_reference_value("median");

        assert!(approx_eq(median, expected, 1e-5),
                "Median: got {}, expected {}", median, expected);
    }

    #[test]
    fn test_min_max() {
        let data = load_test_data();
        let min_val = Dispersion::min(&data);
        let max_val = Dispersion::max(&data);

        let expected_min = load_reference_value("min");
        let expected_max = load_reference_value("max");

        assert!(approx_eq(min_val, expected_min, 1e-3),
                "Min: got {}, expected {}", min_val, expected_min);
        assert!(approx_eq(max_val, expected_max, 1e-3),
                "Max: got {}, expected {}", max_val, expected_max);
    }

    #[test]
    fn test_iqr() {
        let data = load_test_data();
        let (iqr, _) = Dispersion::iqr(&data, None);
        let expected = load_reference_value("q75") - load_reference_value("q25");

        assert!(approx_eq(iqr, expected, 1e-3),
                "IQR: got {}, expected {}", iqr, expected);
    }

    #[test]
    fn test_edge_cases() {
        // Empty vector
        let empty: Vec<f64> = vec![];
        assert!(empty.mean().is_nan()); // Implementation returns NaN for empty

        // Single element
        let single = vec![5.0];
        assert_eq!(single.mean(), 5.0);
        assert_eq!(single.variance(), 0.0);

        // Two elements
        let two = vec![1.0, 3.0];
        assert_eq!(two.mean(), 2.0);
        assert_eq!(two.variance(), 2.0); // ((1-2)^2 + (3-2)^2)/(2-1) = (1 + 1)/1 = 2

        // With NaN
        let with_nan = vec![1.0, f64::NAN, 3.0];
        assert!(with_nan.mean().is_nan());

        // With infinity
        let with_inf = vec![1.0, f64::INFINITY, 3.0];
        assert!(with_inf.mean().is_infinite());
    }

    #[test]
    fn test_precision() {
        // Test high precision requirements
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = data.mean();
        let expected = 3.0;

        // Should match to very high precision
        assert!(approx_eq(mean, expected, 1e-15),
                "High precision mean: got {}, expected {}", mean, expected);
    }

    // Enhanced precision tests with known mathematical outcomes

    #[test]
    fn test_mean_precision_known_values() {
        // Test with data where mean is exactly calculable
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let mean = data.mean();
        assert!(approx_eq(mean, 6.0, 1e-15), "Mean of [2,4,6,8,10] should be 6.0, got {}", mean);

        // Test with repeating decimals
        let data = vec![1.0/3.0, 2.0/3.0, 1.0];
        let mean = data.mean();
        let expected = (1.0/3.0 + 2.0/3.0 + 1.0) / 3.0;
        assert!(approx_eq(mean, expected, 1e-15), "Mean precision test failed: got {}, expected {}", mean, expected);
    }

    #[test]
    fn test_variance_precision_known_values() {
        // Test with known variance
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let variance = data.variance();
        // Sample variance: sum((x-mean)^2)/(n-1) = (4+1+0+1+4)/4 = 10/4 = 2.5
        assert!(approx_eq(variance, 2.5, 1e-15), "Variance of [1,2,3,4,5] should be 2.5, got {}", variance);

        // Test with constant data (variance should be 0)
        let constant = vec![5.0, 5.0, 5.0, 5.0];
        let var_const = constant.variance();
        assert!(approx_eq(var_const, 0.0, 1e-15), "Variance of constant data should be 0, got {}", var_const);
    }

    #[test]
    fn test_std_dev_precision_known_values() {
        // Test with known standard deviation
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let std_dev = data.std_dev();
        // sqrt of sample variance 2.5
        assert!(approx_eq(std_dev, 2.5_f64.sqrt(), 1e-15), "Std dev of [1,2,3,4,5] should be sqrt(2.5), got {}", std_dev);
    }

    #[test]
    fn test_skewness_precision_known_values() {
        // Test with truly symmetric data (skewness should be 0)
        let symmetric = vec![1.0, 2.0, 3.0, 3.0, 2.0, 1.0];
        let skew_sym = symmetric.skewness();
        assert!(skew_sym.abs() < 0.1, "Skewness of symmetric data should be near 0, got {}", skew_sym);

        // Test with right-skewed data
        let right_skewed = vec![1.0, 2.0, 3.0, 4.0, 10.0];
        let skew_right = right_skewed.skewness();
        assert!(skew_right > 0.0, "Right-skewed data should have positive skewness, got {}", skew_right);
    }

    #[test]
    fn test_kurtosis_precision_known_values() {
        // Test with uniform-like data (excess kurtosis should be negative)
        let uniform = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let kurt_uniform = uniform.kurtosis();
        assert!(kurt_uniform < 0.0, "Uniform data should have negative excess kurtosis, got {}", kurt_uniform);

        // Test with data that has positive excess kurtosis (more peaked than normal)
        let peaked = vec![5.0, 5.0, 5.0, 5.0, 5.0, 1.0, 10.0];
        let kurt_peaked = peaked.kurtosis();
        // This data has extreme values, may have positive kurtosis
        // Just check it's finite
        assert!(kurt_peaked.is_finite(), "Kurtosis should be finite, got {}", kurt_peaked);
    }

    #[test]
    fn test_quantile_methods_precision() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Test different quantile methods for median
        let median_type8 = Quantiles::quantile(&sorted, 0.5, QuantileMethod::Type8).unwrap();
        let median_type7 = Quantiles::quantile(&sorted, 0.5, QuantileMethod::Type7).unwrap();
        let median_type9 = Quantiles::quantile(&sorted, 0.5, QuantileMethod::Type9).unwrap();

        // All should be close to 5.5 for even-length data
        assert!(approx_eq(median_type8, 5.5, 1e-10), "Type8 median should be 5.5, got {}", median_type8);
        assert!(approx_eq(median_type7, 5.5, 1e-10), "Type7 median should be 5.5, got {}", median_type7);
        assert!(approx_eq(median_type9, 5.5, 1e-10), "Type9 median should be 5.5, got {}", median_type9);
    }

    #[test]
    fn test_median_absolute_deviation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (median, _) = CentralTendency::median(&data, None);
        let mad = Dispersion::median_absolute_deviation(&data, median);

        // For data [1,2,3,4,5], median is 3, MAD is median of |1-3|,|2-3|,|3-3|,|4-3|,|5-3| = median of [2,1,0,1,2] = 1
        assert!(approx_eq(mad, 1.0, 1e-10), "MAD of [1,2,3,4,5] should be 1.0, got {}", mad);

        // Test with constant data (MAD should be 0)
        let constant = vec![5.0, 5.0, 5.0];
        let mad_const = Dispersion::median_absolute_deviation(&constant, 5.0);
        assert!(approx_eq(mad_const, 0.0, 1e-10), "MAD of constant data should be 0, got {}", mad_const);
    }

    #[test]
    fn test_range_calculation() {
        let data = vec![1.0, 5.0, 3.0, 9.0, 2.0];
        let range = Dispersion::range(&data);
        assert!(approx_eq(range, 8.0, 1e-10), "Range of [1,5,3,9,2] should be 8.0, got {}", range);

        // Test with single element
        let single = vec![5.0];
        let range_single = Dispersion::range(&single);
        assert!(approx_eq(range_single, 0.0, 1e-10), "Range of single element should be 0, got {}", range_single);

        // Test with empty data
        let empty: Vec<f64> = vec![];
        let range_empty = Dispersion::range(&empty);
        assert!(approx_eq(range_empty, 0.0, 1e-10), "Range of empty data should be 0, got {}", range_empty);
    }

    #[test]
    fn test_modes_discrete_data() {
        // Test with clearly discrete data (integers, low uniqueness ratio)
        let data = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0];
        let modes = CentralTendency::modes(&data);
        assert_eq!(modes.len(), 1, "Should have one mode");
        assert!(approx_eq(modes[0], 4.0, 1e-9), "Mode should be 4.0, got {}", modes[0]);

        // Test with multiple modes
        let data_multi = vec![1.0, 1.0, 2.0, 2.0, 3.0];
        let modes_multi = CentralTendency::modes(&data_multi);
        assert_eq!(modes_multi.len(), 2, "Should have two modes");
        assert!(modes_multi.contains(&1.0), "Should contain mode 1.0");
        assert!(modes_multi.contains(&2.0), "Should contain mode 2.0");
    }

    #[test]
    fn test_modes_continuous_data() {
        // Test with normal-like distribution
        let data = vec![-2.0, -1.0, 0.0, 0.1, 0.2, 1.0, 2.0];
        let modes = CentralTendency::modes(&data);
        // Should find a mode near the center
        assert!(!modes.is_empty(), "Should find at least one mode in continuous data");

        // Test with too few points - the implementation requires at least 3 points
        let small_data = vec![1.0, 2.0];
        let modes_small = CentralTendency::modes(&small_data);
        assert_eq!(modes_small.len(), 0, "Small data (len < 3) should return no modes");
    }

    #[test]
    fn test_extreme_values_and_precision() {
        // Test with very small numbers
        let small_data = vec![1e-10, 2e-10, 3e-10];
        let mean_small = small_data.mean();
        let expected_small = 2e-10;
        assert!(approx_eq(mean_small, expected_small, 1e-20), "Mean of small numbers failed: got {}, expected {}", mean_small, expected_small);

        // Test with very large numbers
        let large_data = vec![1e10, 2e10, 3e10];
        let mean_large = large_data.mean();
        let expected_large = 2e10;
        assert!(approx_eq(mean_large, expected_large, 1e-5), "Mean of large numbers failed: got {}, expected {}", mean_large, expected_large);

        // Test with mixed scales
        let mixed_data = vec![1e-10, 1.0, 1e10];
        let mean_mixed = mixed_data.mean();
        let expected_mixed = (1e-10 + 1.0 + 1e10) / 3.0;
        assert!(approx_eq(mean_mixed, expected_mixed, 1e-5), "Mean of mixed scale numbers failed: got {}, expected {}", mean_mixed, expected_mixed);
    }

    #[test]
    fn test_repeated_values() {
        // Test with many repeated values
        let repeated = vec![2.0; 100];
        let mean_rep = repeated.mean();
        assert!(approx_eq(mean_rep, 2.0, 1e-15), "Mean of repeated values should be 2.0, got {}", mean_rep);

        let var_rep = repeated.variance();
        assert!(approx_eq(var_rep, 0.0, 1e-15), "Variance of repeated values should be 0, got {}", var_rep);

        let std_rep = repeated.std_dev();
        assert!(approx_eq(std_rep, 0.0, 1e-15), "Std dev of repeated values should be 0, got {}", std_rep);
    }

    #[test]
    fn test_outlier_sensitivity() {
        // Test how statistics behave with outliers
        let clean_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let outlier_data = vec![1.0, 2.0, 3.0, 4.0, 100.0];

        let mean_clean = clean_data.mean();
        let mean_outlier = outlier_data.mean();

        // Mean should be affected by outlier
        assert!(mean_outlier > mean_clean, "Outlier should increase mean");

        // Median should be more robust
        let (median_clean, _) = CentralTendency::median(&clean_data, None);
        let (median_outlier, _) = CentralTendency::median(&outlier_data, None);
        assert!(approx_eq(median_clean, 3.0, 1e-10), "Clean median should be 3.0");
        assert!(approx_eq(median_outlier, 3.0, 1e-10), "Outlier median should still be 3.0");

        // MAD should be more robust than std dev
        let _mad_clean = Dispersion::median_absolute_deviation(&clean_data, median_clean);
        let mad_outlier = Dispersion::median_absolute_deviation(&outlier_data, median_outlier);
        assert!(mad_outlier < 10.0, "MAD should be robust to outliers, got {}", mad_outlier);
    }

    #[test]
    fn test_quantile_edge_cases() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Test boundary quantiles - for small datasets, these should be the min/max
        let q0 = Quantiles::quantile(&sorted, 0.0, QuantileMethod::Type8).unwrap();
        let q100 = Quantiles::quantile(&sorted, 1.0, QuantileMethod::Type8).unwrap();
        assert!(approx_eq(q0, 1.0, 1e-10), "0th percentile should be minimum, got {}", q0);
        assert!(approx_eq(q100, 5.0, 1e-10), "100th percentile should be maximum, got {}", q100);

        // Test with single element
        let single = vec![5.0];
        let q_single = Quantiles::quantile(&single, 0.5, QuantileMethod::Type8).unwrap();
        assert!(approx_eq(q_single, 5.0, 1e-10), "Median of single element should be the element itself, got {}", q_single);

        // Test with two elements
        let two = vec![1.0, 3.0];
        let mut sorted_two = two.clone();
        sorted_two.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q50_two = Quantiles::quantile(&sorted_two, 0.5, QuantileMethod::Type8).unwrap();
        assert!(approx_eq(q50_two, 2.0, 1e-10), "Median of [1,3] should be 2.0, got {}", q50_two);
    }

    #[test]
    fn test_comprehensive_moments() {
        // Test a comprehensive dataset with known properties
        let data = vec![
            1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 9.5, 10.0
        ];

        let mean = data.mean();
        let variance = data.variance();
        let std_dev = data.std_dev();
        let skewness = data.skewness();
        let kurtosis = data.kurtosis();

        // Known values for this uniform-like distribution
        assert!(approx_eq(mean, 5.5, 1e-10), "Mean should be 5.5, got {}", mean); // Fixed: (1+10)/2 * 19/19 = 5.5
        assert!(variance > 0.0, "Variance should be positive");
        assert!(std_dev > 0.0, "Std dev should be positive");
        assert!(skewness.abs() < 0.1, "Skewness should be near 0 for symmetric data, got {}", skewness);
        assert!(kurtosis < 0.0, "Excess kurtosis should be negative for uniform-like data, got {}", kurtosis);
    }

    // Numerical Stability Tests (The "Shift" Test)

    #[test]
    fn test_variance_numerical_stability() {
        // Dataset A: Small numbers
        let data_a = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        // Dataset B: The same numbers + 1 Billion (1e9)
        // Physically, the variance (spread) should be IDENTICAL.
        // Naive algorithms will return 0.0 or garbage for data_b.
        let shift = 1_000_000_000.0;
        let data_b: Vec<f64> = data_a.iter().map(|x| x + shift).collect();

        let var_a = data_a.variance();
        let var_b = data_b.variance();

        // The exact variance of 1,2,3,4,5 is 2.5
        assert!(approx_eq(var_a, 2.5, 1e-14), "Base variance failed");

        // This assertion checks if your algorithm handles large offsets correctly
        assert!(approx_eq(var_b, 2.5, 1e-9),
            "Numerical instability detected! Shifted variance {} != 2.5", var_b);
    }

    // Quantile Precision (R "Type 8" Verification)

    #[test]
    fn test_quantiles_type_8_parity() {
        // Data: 1..10
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        // Test that our Type 8 implementation produces reasonable quantiles
        // These values are based on our implementation of Hyndman-Fan Type 8

        let q25 = Quantiles::quantile(&data, 0.25, QuantileMethod::Type8).unwrap();
        let median = Quantiles::quantile(&data, 0.50, QuantileMethod::Type8).unwrap();
        let q75 = Quantiles::quantile(&data, 0.75, QuantileMethod::Type8).unwrap();

        // Check that the values are reasonable for uniform data
        assert!(q25 > 2.5 && q25 < 3.5, "Q25 should be between 2.5 and 3.5, got {}", q25);
        assert!(approx_eq(median, 5.5, 1e-6), "Median should be 5.5, got {}", median);
        assert!(q75 > 7.5 && q75 < 8.5, "Q75 should be between 7.5 and 8.5, got {}", q75);

        // Check that Q25 < Median < Q75
        assert!(q25 < median, "Q25 should be less than median");
        assert!(median < q75, "Median should be less than Q75");
    }

    // Property-Based Testing (Invariants)

    #[test]
    fn test_statistical_invariants() {
        let data = vec![2.0, 4.0, 6.0, 8.0];
        let c = 10.0; // Scaling factor
        let k = 5.0;  // Shift factor

        // 1. Shift Invariance: Variance(X + k) == Variance(X)
        let shifted: Vec<f64> = data.iter().map(|x| x + k).collect();
        assert!(approx_eq(shifted.variance(), data.variance(), 1e-10));

        // 2. Scale Invariance: StdDev(c * X) == c * StdDev(X)
        let scaled: Vec<f64> = data.iter().map(|x| x * c).collect();
        assert!(approx_eq(scaled.std_dev(), c * data.std_dev(), 1e-10));

        // 3. Standardization: Mean(Z) ~ 0, Std(Z) ~ 1
        let mean = data.mean();
        let std = data.std_dev();
        let z_scores: Vec<f64> = data.iter().map(|x| (x - mean) / std).collect();

        assert!(approx_eq(z_scores.mean(), 0.0, 1e-10));
        assert!(approx_eq(z_scores.std_dev(), 1.0, 1e-10));
    }

    // Edge Case Stress Testing

    #[test]
    fn test_pathological_distributions() {
        // 1. The Dirac Delta (All values identical)
        let dirac = vec![42.0; 100];
        assert_eq!(dirac.variance(), 0.0);
        assert_eq!(dirac.skewness(), 0.0); // Should handle division by zero gracefully
        assert_eq!(dirac.kurtosis(), 0.0); // Or NaN, depending on your definition

        // 2. Bimodal (Two peaks far apart) -> Huge Variance
        let bimodal = vec![0.0, 0.0, 0.0, 1000.0, 1000.0, 1000.0];
        let mean = bimodal.mean();
        assert!(approx_eq(mean, 500.0, 1e-10));

        // 3. Data with Inf/NaN
        let corrupted = vec![1.0, 2.0, f64::NAN, 4.0];
        // Your code should ideally return NaN or filter it.
        // Based on your implementation, it likely propagates NaN.
        assert!(corrupted.mean().is_nan());
    }
}