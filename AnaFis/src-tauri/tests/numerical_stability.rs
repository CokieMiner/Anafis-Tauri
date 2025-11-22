//! Numerical stability tests
//! Validates that algorithms handle edge cases and extreme values correctly

use anafis_lib::scientific::statistics::descriptive::{StatisticalMoments, Quantiles, QuantileMethod};

fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance || (a - b).abs() / b.abs().max(1.0) < tolerance
}

#[test]
fn test_variance_numerical_stability_large_offset() {
    // Test the "shift" problem: variance should be invariant to location shifts
    let data_small = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let shift = 1_000_000_000.0; // 1 billion
    let data_large: Vec<f64> = data_small.iter().map(|x| x + shift).collect();
    
    let var_small = data_small.variance();
    let var_large = data_large.variance();
    
    // Variance should be identical (2.5 for this dataset)
    assert!(
        approx_eq(var_small, 2.5, 1e-14),
        "Base variance incorrect: got {}, expected 2.5",
        var_small
    );
    
    assert!(
        approx_eq(var_large, 2.5, 1e-9),
        "Numerical instability detected! Shifted variance {} != 2.5",
        var_large
    );
}

#[test]
fn test_mean_with_extreme_values() {
    // Test mean calculation with very large and very small numbers
    let data = vec![1e-10, 1.0, 1e10];
    let mean = data.mean();
    let expected = (1e-10 + 1.0 + 1e10) / 3.0;
    
    assert!(
        approx_eq(mean, expected, 1e-5),
        "Mean of mixed scale numbers failed: got {}, expected {}",
        mean,
        expected
    );
}

#[test]
fn test_quantile_with_repeated_values() {
    // Test quantile calculation when many values are identical
    let data = vec![1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0];
    let mut sorted = data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let q25 = Quantiles::quantile(&sorted, 0.25, QuantileMethod::Type8).unwrap();
    let median = Quantiles::quantile(&sorted, 0.5, QuantileMethod::Type8).unwrap();
    let q75 = Quantiles::quantile(&sorted, 0.75, QuantileMethod::Type8).unwrap();
    
    // Should handle repeated values gracefully
    assert!(q25.is_finite() && q25 > 0.0);
    assert!(median.is_finite() && median > 0.0);
    assert!(q75.is_finite() && q75 > 0.0);
    assert!(q25 <= median && median <= q75);
}

#[test]
fn test_statistics_with_nan() {
    // Test that NaN is handled appropriately
    let data_with_nan = vec![1.0, 2.0, f64::NAN, 4.0, 5.0];
    
    let mean = data_with_nan.mean();
    let variance = data_with_nan.variance();
    
    // NaN should propagate through calculations
    assert!(mean.is_nan(), "Mean should be NaN when data contains NaN");
    assert!(variance.is_nan(), "Variance should be NaN when data contains NaN");
}

#[test]
fn test_statistics_with_infinity() {
    // Test that infinity is handled appropriately
    let data_with_inf = vec![1.0, 2.0, f64::INFINITY, 4.0, 5.0];
    
    let mean = data_with_inf.mean();
    
    // Infinity should propagate
    assert!(mean.is_infinite(), "Mean should be infinite when data contains infinity");
}

#[test]
fn test_empty_dataset_handling() {
    let empty: Vec<f64> = vec![];
    
    let mean = empty.mean();
    let variance = empty.variance();
    
    // Mean returns NaN for empty datasets
    assert!(mean.is_nan(), "Mean of empty dataset should be NaN");
    // Variance returns 0.0 for datasets with < 2 elements (by design)
    assert_eq!(variance, 0.0, "Variance of empty dataset should be 0.0");
}

#[test]
fn test_single_value_dataset() {
    let single = vec![42.0];
    
    let mean = single.mean();
    let variance = single.variance();
    let std_dev = single.std_dev();
    
    assert_eq!(mean, 42.0, "Mean of single value should be that value");
    assert_eq!(variance, 0.0, "Variance of single value should be 0");
    assert_eq!(std_dev, 0.0, "Std dev of single value should be 0");
}

#[test]
fn test_constant_dataset() {
    // All values identical
    let constant = vec![5.0; 100];
    
    let mean = constant.mean();
    let variance = constant.variance();
    let std_dev = constant.std_dev();
    let skewness = constant.skewness();
    let kurtosis = constant.kurtosis();
    
    assert_eq!(mean, 5.0, "Mean of constant data should be the constant");
    assert_eq!(variance, 0.0, "Variance of constant data should be 0");
    assert_eq!(std_dev, 0.0, "Std dev of constant data should be 0");
    assert_eq!(skewness, 0.0, "Skewness of constant data should be 0");
    assert_eq!(kurtosis, 0.0, "Kurtosis of constant data should be 0");
}

#[test]
fn test_nearly_identical_values() {
    // Values that differ only in the last few decimal places
    let nearly_identical = vec![
        1.0000000001,
        1.0000000002,
        1.0000000003,
        1.0000000004,
        1.0000000005,
    ];
    
    let mean = nearly_identical.mean();
    let variance = nearly_identical.variance();
    
    // Should handle tiny differences without catastrophic cancellation
    assert!(mean.is_finite() && mean > 1.0 && mean < 1.001);
    assert!(variance.is_finite() && variance >= 0.0);
}

#[test]
fn test_alternating_signs() {
    // Test with alternating positive and negative values
    let alternating = vec![-1.0, 1.0, -2.0, 2.0, -3.0, 3.0];
    
    let mean = alternating.mean();
    let variance = alternating.variance();
    
    assert!(approx_eq(mean, 0.0, 1e-10), "Mean of alternating signs should be near 0");
    assert!(variance > 0.0 && variance.is_finite(), "Variance should be positive and finite");
}

#[test]
fn test_quantile_boundary_values() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let mut sorted = data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    // Test boundary quantiles
    let q0 = Quantiles::quantile(&sorted, 0.0, QuantileMethod::Type8).unwrap();
    let q100 = Quantiles::quantile(&sorted, 1.0, QuantileMethod::Type8).unwrap();
    
    assert_eq!(q0, 1.0, "0th percentile should be minimum");
    assert_eq!(q100, 5.0, "100th percentile should be maximum");
}

#[test]
fn test_quantile_invalid_probability() {
    let data = vec![1.0, 2.0, 3.0];
    let mut sorted = data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    // Test that invalid probabilities return errors
    let result_negative = Quantiles::quantile(&sorted, -0.1, QuantileMethod::Type8);
    let result_too_large = Quantiles::quantile(&sorted, 1.5, QuantileMethod::Type8);
    
    assert!(result_negative.is_err(), "Negative probability should return error");
    assert!(result_too_large.is_err(), "Probability > 1 should return error");
}

#[test]
fn test_very_large_dataset() {
    // Test with a large dataset to ensure no performance issues
    let large_data: Vec<f64> = (0..10000).map(|i| i as f64).collect();
    
    let mean = large_data.mean();
    let variance = large_data.variance();
    
    // Mean should be around 4999.5
    assert!(approx_eq(mean, 4999.5, 1e-6), "Mean of large dataset incorrect");
    assert!(variance > 0.0 && variance.is_finite(), "Variance should be positive and finite");
}
