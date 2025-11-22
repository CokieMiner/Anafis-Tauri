#[cfg(test)]
mod tests {
    use anafis_lib::scientific::statistics::descriptive::{CentralTendency, Quantiles};
    use anafis_lib::scientific::statistics::StatisticalMoments;

    #[test]
    fn test_welford_variance() {
        // Test with data that causes catastrophic cancellation in naive algorithms
        // Data: 1e9 + 1, 1e9 + 2, 1e9 + 3
        // Mean: 1e9 + 2
        // Variance: 1.0
        let data = vec![1_000_000_001.0, 1_000_000_002.0, 1_000_000_003.0];
        let var = data.variance();
        assert!((var - 1.0).abs() < 1e-10, "Variance should be 1.0, got {}", var);
        
        let std = data.std_dev();
        assert!((std - 1.0).abs() < 1e-10, "Std dev should be 1.0, got {}", std);
    }

    #[test]
    fn test_median_mut() {
        let mut data = vec![5.0, 1.0, 3.0, 2.0, 4.0];
        let median = Quantiles::median_mut(&mut data);
        assert_eq!(median, 3.0);
        
        let mut data_even = vec![1.0, 2.0, 3.0, 4.0];
        let median_even = Quantiles::median_mut(&mut data_even);
        assert_eq!(median_even, 2.5);
    }

    #[test]
    fn test_nan_safe_median() {
        let data = vec![1.0, f64::NAN, 3.0, 2.0, f64::NAN];
        let median = Quantiles::nan_safe_median(&data);
        assert_eq!(median, 2.0);
    }

    #[test]
    fn test_central_tendency_median() {
        let data = vec![10.0, 2.0, 5.0];
        // Should not modify original data (it takes a slice)
        let (median, _) = CentralTendency::median(&data, None);
        assert_eq!(median, 5.0);
        assert_eq!(data, vec![10.0, 2.0, 5.0]); // Verify original data is untouched
    }

    #[test]
    fn test_pipeline_moments_consistency() {
        // Check that moments calculated manually match
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = data.mean();
        let var = data.variance();
        let skew = data.skewness();
        let kurt = data.kurtosis();

        assert_eq!(mean, 3.0);
        assert_eq!(var, 2.5);
        assert_eq!(skew, 0.0);
        // Kurtosis for discrete uniform {1,2,3,4,5} using sample std dev (n-1)
        // m4 = 6.8, s^4 = 2.5^2 = 6.25
        // K = 6.8 / 6.25 - 3 = 1.088 - 3 = -1.912
        assert!((kurt - (-1.912)).abs() < 0.001, "Kurtosis expected -1.912, got {}", kurt);
    }
}
