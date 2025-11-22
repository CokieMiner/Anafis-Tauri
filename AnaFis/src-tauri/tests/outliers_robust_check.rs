
#[cfg(test)]
mod tests {
    use anafis_lib::scientific::statistics::outliers::{OutlierDetectionEngine, AnalysisOptions};
    use anafis_lib::scientific::statistics::robust_regression::RobustRegressionEngine;

    #[test]
    fn test_iqr_outliers_optimization() {
        // Data with obvious outliers
        let mut data = vec![10.0; 20];
        data[0] = 100.0; // Outlier
        data[1] = -50.0; // Outlier
        
        let options = AnalysisOptions {
            iqr_multiplier: Some(1.5),
            ..Default::default()
        };
        
        let result = OutlierDetectionEngine::detect_outliers(&data, &options).unwrap();
        
        // Check if IQR detected outliers
        let iqr_outliers = result.methods.iter().find(|(name, _)| name == "IQR").unwrap();
        assert!(iqr_outliers.1.len() >= 2);
        
        // Check indices
        let indices: Vec<usize> = iqr_outliers.1.iter().map(|o| o.index).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }

    #[test]
    fn test_lof_outliers_optimization() {
        // Data: cluster around 10, and one point at 100
        let mut data = vec![10.0, 10.1, 9.9, 10.2, 9.8, 10.0, 10.1, 9.9, 10.2, 9.8];
        data.push(100.0); // Outlier
        
        let options = AnalysisOptions {
            lof_k: Some(5),
            lof_threshold: Some(1.5),
            ..Default::default()
        };
        
        let result = OutlierDetectionEngine::detect_outliers(&data, &options).unwrap();
        
        // Check if LOF detected outlier
        let lof_outliers = result.methods.iter().find(|(name, _)| name == "Local Outlier Factor").unwrap();
        assert!(!lof_outliers.1.is_empty());
        
        // The last point (index 10) should be an outlier
        let indices: Vec<usize> = lof_outliers.1.iter().map(|o| o.index).collect();
        assert!(indices.contains(&10));
    }

    #[test]
    fn test_huber_regression_convergence() {
        // y = 2x + 1 + noise + outliers
        let n = 50;
        let mut x_vec = Vec::new();
        let mut y_vec = Vec::new();
        let mut rng = rand::rng();
        use rand_distr::{Normal, Distribution};
        let normal = Normal::new(0.0, 0.1).unwrap();
        
        for i in 0..n {
            let x = i as f64;
            let y = 2.0 * x + 1.0 + normal.sample(&mut rng);
            x_vec.push(vec![x]);
            y_vec.push(y);
        }
        
        // Add outliers
        y_vec[10] += 50.0;
        y_vec[20] -= 50.0;
        
        let result = RobustRegressionEngine::huber_regression(&x_vec, &y_vec, 1.345, 100, 1e-6).unwrap();
        
        assert!(result.converged);
        // Slope should be close to 2.0
        assert!((result.coefficients[0] - 2.0).abs() < 0.5, "Slope {} too far from 2.0", result.coefficients[0]);
    }
}
