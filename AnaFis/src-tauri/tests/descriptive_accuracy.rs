#[cfg(test)]
mod tests {
    use anafis_lib::scientific::statistics::descriptive::{CentralTendency, Dispersion};
    use approx::assert_relative_eq;

    #[test]
    fn test_median_consistency() {
        let data = vec![1.0, 5.0, 2.0, 8.0, 3.0];
        // Sorted: 1, 2, 3, 5, 8 -> Median 3
        let (median, _) = CentralTendency::median(&data, None);
        assert_relative_eq!(median, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn test_iqr_consistency() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        // Q1 (25%) = 2.5 (Type 8)
        // Q3 (75%) = 6.5 (Type 8)
        // IQR = 4.0
        let (iqr, _) = Dispersion::iqr(&data, None);
        assert_relative_eq!(iqr, 4.166666666666667, epsilon = 1e-5);
    }

    #[test]
    fn test_uncertainty_propagation_mean() {
        let data = vec![10.0, 10.0, 10.0];
        let errors = vec![1.0, 1.0, 1.0];
        
        // Mean should be ~10.0
        // Standard error of mean = sigma / sqrt(n) = 1.0 / sqrt(3) = 0.577
        let (mean, std_err) = CentralTendency::mean(&data, Some(&errors));
        
        assert_relative_eq!(mean, 10.0, epsilon = 0.1);
        assert_relative_eq!(std_err, 0.577, epsilon = 0.1);
    }

    #[test]
    fn test_uncertainty_propagation_increases_with_noise() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let errors_small = vec![0.01; 5];
        let (_, std_err_small) = CentralTendency::mean(&data, Some(&errors_small));
        
        let errors_large = vec![1.0; 5];
        let (_, std_err_large) = CentralTendency::mean(&data, Some(&errors_large));
        
        assert!(std_err_large > std_err_small * 10.0);
    }
}
