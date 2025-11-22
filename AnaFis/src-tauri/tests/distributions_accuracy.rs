#[cfg(test)]
mod tests {
    use anafis_lib::scientific::statistics::distributions::fitting::StatisticalDistributionEngine;
    use approx::assert_relative_eq;

    #[test]
    fn test_normal_fit_without_errors() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let fit = StatisticalDistributionEngine::fit_normal_distribution(&data, None).unwrap();
        
        assert_eq!(fit.distribution_name, "normal");
        assert_eq!(fit.parameters.len(), 2);
        assert!(fit.parameter_uncertainties.is_none());
        
        // Mean should be ~3.0
        assert_relative_eq!(fit.parameters[0].1, 3.0, epsilon = 0.01);
    }

    #[test]
    fn test_normal_fit_with_errors() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let errors = vec![0.1, 0.1, 0.1, 0.1, 0.1];
        let fit = StatisticalDistributionEngine::fit_normal_distribution(&data, Some(&errors)).unwrap();
        
        assert_eq!(fit.distribution_name, "normal");
        assert!(fit.parameter_uncertainties.is_some());
        
        let uncertainties = fit.parameter_uncertainties.unwrap();
        assert_eq!(uncertainties.len(), 2);
        
        // Uncertainties should be non-zero
        assert!(uncertainties[0].1 > 0.0); // mean uncertainty
        assert!(uncertainties[1].1 > 0.0); // std_dev uncertainty
    }

    #[test]
    fn test_weibull_fit_without_errors() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let fit = StatisticalDistributionEngine::fit_weibull_distribution(&data, None).unwrap();
        
        assert_eq!(fit.distribution_name, "weibull");
        assert_eq!(fit.parameters.len(), 2);
        assert!(fit.parameter_uncertainties.is_none());
    }

    #[test]
    fn test_weibull_fit_with_errors() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let errors = vec![0.1, 0.1, 0.1, 0.1, 0.1];
        let fit = StatisticalDistributionEngine::fit_weibull_distribution(&data, Some(&errors)).unwrap();
        
        assert_eq!(fit.distribution_name, "weibull");
        assert!(fit.parameter_uncertainties.is_some());
        
        let uncertainties = fit.parameter_uncertainties.unwrap();
        assert_eq!(uncertainties.len(), 2);
        
        // Uncertainties should be non-zero
        assert!(uncertainties[0].1 > 0.0); // shape uncertainty
        assert!(uncertainties[1].1 > 0.0); // scale uncertainty
    }

    #[test]
    fn test_larger_errors_increase_uncertainty() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let small_errors = vec![0.01, 0.01, 0.01, 0.01, 0.01];
        let large_errors = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        
        let fit_small = StatisticalDistributionEngine::fit_normal_distribution(&data, Some(&small_errors)).unwrap();
        let fit_large = StatisticalDistributionEngine::fit_normal_distribution(&data, Some(&large_errors)).unwrap();
        
        let unc_small = fit_small.parameter_uncertainties.unwrap();
        let unc_large = fit_large.parameter_uncertainties.unwrap();
        
        // Larger input errors should lead to larger parameter uncertainties
        assert!(unc_large[0].1 > unc_small[0].1); // mean
        assert!(unc_large[1].1 > unc_small[1].1); // std_dev
    }

    #[test]
    fn test_fit_distributions_with_errors() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let errors = vec![0.1, 0.1, 0.1, 0.1, 0.1];
        
        let fits = StatisticalDistributionEngine::fit_distributions(&data, Some(&errors)).unwrap();
        
        // Should return multiple fits
        assert!(!fits.is_empty());
        
        // At least some fits should have parameter_uncertainties
        let has_uncertainties = fits.iter().any(|fit| fit.parameter_uncertainties.is_some());
        assert!(has_uncertainties, "At least some distributions should have uncertainty estimates");
    }

    #[test]
    fn test_fit_distributions_without_errors() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let fits = StatisticalDistributionEngine::fit_distributions(&data, None).unwrap();
        
        // Should return multiple fits
        assert!(!fits.is_empty());
        
        // All fits should have None for parameter_uncertainties
        for fit in &fits {
            assert!(fit.parameter_uncertainties.is_none());
        }
    }
}
