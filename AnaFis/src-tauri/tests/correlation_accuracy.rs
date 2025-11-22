#[cfg(test)]
mod tests {
    use anafis_lib::scientific::statistics::correlation::CorrelationMethods;
    use approx::assert_relative_eq;

    #[test]
    fn test_pearson_perfect_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let (pearson, _) = CorrelationMethods::pearson_correlation(&x, &y, None, None).unwrap();
        assert_relative_eq!(pearson, 1.0);
        
        let (spearman, _) = CorrelationMethods::spearman_correlation(&x, &y, None, None).unwrap();
        assert_relative_eq!(spearman, 1.0);
        
        let (kendall, _) = CorrelationMethods::kendall_correlation(&x, &y, None, None).unwrap();
        assert_relative_eq!(kendall, 1.0);
    }

    #[test]
    fn test_negative_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        
        let (pearson, _) = CorrelationMethods::pearson_correlation(&x, &y, None, None).unwrap();
        assert_relative_eq!(pearson, -1.0);
    }

    #[test]
    fn test_monotonic_relationship() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 4.0, 9.0, 16.0, 25.0]; // y = x^2 (monotonic)
        
        let (spearman, _) = CorrelationMethods::spearman_correlation(&x, &y, None, None).unwrap();
        assert_relative_eq!(spearman, 1.0); // Should be 1.0 for monotonic
        
        let (pearson, _) = CorrelationMethods::pearson_correlation(&x, &y, None, None).unwrap();
        assert!(pearson < 1.0); // Pearson checks linear, so < 1.0
    }

    #[test]
    fn test_outliers_biweight() {
        let mut x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        // Add outlier
        x.push(100.0);
        y.push(0.0);
        
        let (pearson, _) = CorrelationMethods::pearson_correlation(&x, &y, None, None).unwrap();
        let biweight = CorrelationMethods::biweight_midcorrelation(&x, &y).unwrap();
        
        // Biweight should be higher (closer to true correlation of main data) than Pearson
        assert!(biweight > pearson);
    }

    #[test]
    fn test_zero_variance_error() {
        let x = vec![1.0, 1.0, 1.0];
        let y = vec![1.0, 2.0, 3.0];
        
        let result = CorrelationMethods::pearson_correlation(&x, &y, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_uncertainty_propagation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        // Case 1: Tiny errors -> Low uncertainty in correlation
        let x_err_small = vec![0.001; 5];
        let y_err_small = vec![0.001; 5];
        
        // Use the new unified API
        let (mean_corr_small, std_corr_small) = CorrelationMethods::pearson_correlation(
            &x, &y, Some(&x_err_small), Some(&y_err_small)
        ).unwrap();
        
        assert!(mean_corr_small > 0.99); 
        assert!(std_corr_small < 0.01);  

        // Case 2: Large errors -> Higher uncertainty
        let x_err_large = vec![2.0; 5]; 
        let y_err_large = vec![2.0; 5];

        let (_mean_corr_large, std_corr_large) = CorrelationMethods::pearson_correlation(
            &x, &y, Some(&x_err_large), Some(&y_err_large)
        ).unwrap();

        assert!(std_corr_large > std_corr_small * 10.0); 
    }
}
