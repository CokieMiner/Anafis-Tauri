/// Integration tests for statistical analysis pipelines
///
/// Tests end-to-end workflows combining multiple statistical methods.

use crate::scientific::statistics::pipeline::*;

/// Test comprehensive statistical analysis pipeline
#[cfg(test)]
mod tests {
    use super::*;

    /// Test comprehensive analysis with large dataset
    #[test]
    fn test_comprehensive_analysis_large_dataset() {
        // Generate larger dataset
        let mut data = Vec::new();
        for i in 0..50 {
            let row = vec![
                i as f64,
                (i * 2) as f64,
                (i as f64).sin(),
                (i as f64).cos(),
            ];
            data.push(row);
        }
        let var_names = vec!["Linear".to_string(), "Double".to_string(), "Sin".to_string(), "Cos".to_string()];

        let result = StatisticalAnalysisPipeline::comprehensive_analysis(
            &data,
            Some(var_names),
            false,
        ).unwrap();

        assert_eq!(result.metadata.n_observations, 50);
        assert_eq!(result.metadata.n_variables, 4);

        // Should handle larger datasets without issues
        assert_eq!(result.descriptive_stats.univariate_stats.len(), 4);
        assert_eq!(result.correlation_analysis.correlation_tests.len(), 6); // C(4,2) = 6 pairs
    }

    /// Test comprehensive analysis with non-normal data
    #[test]
    fn test_comprehensive_analysis_non_normal() {
        // Generate skewed data
        let data = vec![
            vec![1.0, 1.0, 1.0],
            vec![2.0, 2.0, 2.0],
            vec![3.0, 3.0, 3.0],
            vec![10.0, 10.0, 10.0], // Skewed values
            vec![20.0, 20.0, 20.0],
        ];
        let var_names = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        let result = StatisticalAnalysisPipeline::comprehensive_analysis(
            &data,
            Some(var_names),
            false,
        ).unwrap();

        // Should run normality tests
        assert!(result.data_quality.normality_tests.len() > 0);

        // Should attempt distribution fitting
        assert!(result.distribution_analysis.fitted_distributions.len() > 0);
    }

    /// Test edge cases
    #[test]
    fn test_pipeline_edge_cases() {
        // Empty data should fail
        let empty_data: Vec<Vec<f64>> = vec![];
        assert!(StatisticalAnalysisPipeline::comprehensive_analysis(&empty_data, None, false).is_err());

        // Single variable with multiple observations - should work
        let single_var = vec![
            vec![1.0],
            vec![2.0],
            vec![3.0],
            vec![4.0],
            vec![5.0],
        ];
        let result = StatisticalAnalysisPipeline::comprehensive_analysis(&single_var, None, false).unwrap();
        assert_eq!(result.metadata.n_variables, 1);
        assert_eq!(result.metadata.n_observations, 5);
    }
}