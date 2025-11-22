//! Tests for data preprocessing functions
//!
//! Validates outlier detection, smoothing, and transformation methods.

use super::test_utils::approx_eq;
use crate::scientific::statistics::preprocessing::*;
use crate::scientific::statistics::CorrelationMethods;

/// Test data imputation methods
#[cfg(test)]
mod tests {
    use super::*;

    /// Test KNN imputation with complete data
    #[test]
    fn test_knn_impute_complete_data() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::knn_impute(&data, 2, f64::NAN).unwrap();

        // Should return original data unchanged
        assert_eq!(result.imputed_data, data);
        assert_eq!(result.imputed_count, 0);
        assert!(result.quality_metrics.correlation_preservation >= 0.99);
        assert!(result.quality_metrics.variance_preservation >= 0.99);
    }

    /// Test KNN imputation with missing values
    #[test]
    fn test_knn_impute_with_missing_values() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, f64::NAN, 6.0], // Missing value in middle
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::knn_impute(&data, 2, f64::NAN).unwrap();

        // Should have imputed one value
        assert_eq!(result.imputed_count, 1);
        assert!(!result.imputed_data[1][1].is_nan());
        assert!(result.imputed_data[1][1].is_finite());

        // Check quality metrics are reasonable
        assert!(result.quality_metrics.correlation_preservation >= 0.0);
        assert!(result.quality_metrics.correlation_preservation <= 1.0);
        assert!(result.quality_metrics.variance_preservation >= 0.0);
        assert!(result.quality_metrics.variance_preservation <= 1.0);
    }

    /// Test KNN imputation with multiple missing values
    #[test]
    fn test_knn_impute_multiple_missing() {
        let data = vec![
            vec![1.0, f64::NAN, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::knn_impute(&data, 1, f64::NAN).unwrap();

        assert_eq!(result.imputed_count, 2);
        assert!(!result.imputed_data[0][1].is_nan());
        assert!(!result.imputed_data[1][1].is_nan());
        assert!(result.imputed_data[0][1].is_finite());
        assert!(result.imputed_data[1][1].is_finite());
    }

    /// Test mean imputation
    #[test]
    fn test_mean_impute() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::mean_impute(&data, f64::NAN).unwrap();

        assert_eq!(result.imputed_count, 1);
        assert!(!result.imputed_data[1][1].is_nan());

        // Check that imputed value is the mean of the column (2.0, 8.0) = 5.0
        assert!(approx_eq(result.imputed_data[1][1], 5.0, 1e-10));
    }

    /// Test median imputation
    #[test]
    fn test_median_impute() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::median_impute(&data, f64::NAN).unwrap();

        assert_eq!(result.imputed_count, 1);
        assert!(!result.imputed_data[1][1].is_nan());

        // Check that imputed value is the median of the column (2.0, 8.0) = 5.0
        assert!(approx_eq(result.imputed_data[1][1], 5.0, 1e-10));
    }

    /// Test regression imputation
    #[test]
    fn test_regression_impute() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
            vec![10.0, 11.0, 12.0],
        ];

        let result = DataImputationEngine::regression_impute(&data, f64::NAN).unwrap();

        assert_eq!(result.imputed_count, 1);
        assert!(!result.imputed_data[1][1].is_nan());
        assert!(result.imputed_data[1][1].is_finite());
    }

    /// Test auto imputation selection
    #[test]
    fn test_auto_impute() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::auto_impute(&data, f64::NAN).unwrap();

        assert_eq!(result.imputed_count, 1);
        assert!(!result.imputed_data[1][1].is_nan());
        assert!(result.imputed_data[1][1].is_finite());
        assert!(!result.method.is_empty());
    }

    /// Test imputation with all missing values in a column
    #[test]
    fn test_impute_all_missing_column() {
        let data = vec![
            vec![1.0, f64::NAN, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, f64::NAN, 9.0],
        ];

        let result = DataImputationEngine::mean_impute(&data, f64::NAN).unwrap();

        // Should fall back to 0.0 for columns with all missing values
        assert_eq!(result.imputed_count, 3);
        for row in &result.imputed_data {
            assert!(approx_eq(row[1], 0.0, 1e-10));
        }
    }

    /// Test KNN imputation with different k values
    #[test]
    fn test_knn_impute_different_k() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
            vec![10.0, 11.0, 12.0],
        ];

        // Test with k=1
        let result1 = DataImputationEngine::knn_impute(&data, 1, f64::NAN).unwrap();
        assert_eq!(result1.imputed_count, 1);

        // Test with k=2
        let result2 = DataImputationEngine::knn_impute(&data, 2, f64::NAN).unwrap();
        assert_eq!(result2.imputed_count, 1);

        // Results should be different
        assert_ne!(result1.imputed_data[1][1], result2.imputed_data[1][1]);
    }

    /// Test imputation quality metrics calculation
    #[test]
    fn test_imputation_quality_metrics() {
        let original = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let _imputed = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.1, 6.0], // Slight change
            vec![7.0, 8.0, 9.0],
        ];

        // Test that imputation functions return quality metrics
        let result = DataImputationEngine::knn_impute(&original, 2, f64::NAN).unwrap();
        assert!(result.quality_metrics.correlation_preservation >= 0.0);
        assert!(result.quality_metrics.correlation_preservation <= 1.0);
        assert!(result.quality_metrics.variance_preservation >= 0.0);
        assert!(result.quality_metrics.variance_preservation <= 1.0);
    }

    /// Test KNN imputation with edge case: k larger than available neighbors
    #[test]
    fn test_knn_impute_k_too_large() {
        let data = vec![
            vec![1.0, f64::NAN, 3.0],
            vec![4.0, 5.0, 6.0],
        ];

        // k=2 but only 1 complete row available - should return error
        assert!(DataImputationEngine::knn_impute(&data, 2, f64::NAN).is_err());
    }

    /// Test imputation with empty dataset
    #[test]
    fn test_impute_empty_dataset() {
        let data: Vec<Vec<f64>> = vec![];

        assert!(DataImputationEngine::knn_impute(&data, 2, f64::NAN).is_err());
        assert!(DataImputationEngine::mean_impute(&data, f64::NAN).is_err());
        assert!(DataImputationEngine::median_impute(&data, f64::NAN).is_err());
        assert!(DataImputationEngine::regression_impute(&data, f64::NAN).is_err());
        assert!(DataImputationEngine::auto_impute(&data, f64::NAN).is_err());
    }

    /// Test imputation with single row
    #[test]
    fn test_impute_single_row() {
        let data = vec![vec![1.0, f64::NAN, 3.0]];

        let result = DataImputationEngine::mean_impute(&data, f64::NAN).unwrap();
        assert_eq!(result.imputed_count, 1);
        // Should fall back to 0.0
        assert!(approx_eq(result.imputed_data[0][1], 0.0, 1e-10));
    }

    /// Test KNN imputation with custom missing value indicator
    #[test]
    fn test_knn_impute_custom_missing_value() {
        let data = vec![
            vec![1.0, -999.0, 3.0], // -999.0 as missing value
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::knn_impute(&data, 2, -999.0).unwrap();

        assert_eq!(result.imputed_count, 1);
        assert!(!result.imputed_data[0][1].is_nan());
        assert!(result.imputed_data[0][1] != -999.0);
    }

    /// Test regression imputation with insufficient data
    #[test]
    fn test_regression_impute_insufficient_data() {
        let data = vec![
            vec![1.0, f64::NAN, 3.0],
            vec![4.0, 5.0, 6.0],
        ];

        let result = DataImputationEngine::regression_impute(&data, f64::NAN).unwrap();
        assert_eq!(result.imputed_count, 1);
        // Should fall back to mean imputation
        assert!(!result.imputed_data[0][1].is_nan());
    }

    /// Test auto impute with no missing values
    #[test]
    fn test_auto_impute_no_missing() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::auto_impute(&data, f64::NAN).unwrap();

        assert_eq!(result.imputed_count, 0);
        assert_eq!(result.imputed_data, data);
        assert_eq!(result.method, "none");
        assert!(approx_eq(result.quality_metrics.correlation_preservation, 1.0, 1e-10));
        assert!(approx_eq(result.quality_metrics.variance_preservation, 1.0, 1e-10));
    }

    /// Test imputation quality with highly correlated data
    #[test]
    fn test_imputation_quality_high_correlation() {
        let original = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 4.0, 6.0],
            vec![3.0, 6.0, 9.0],
        ];

        let imputed = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 4.1, 6.0], // Small change
            vec![3.0, 6.0, 9.0],
        ];

        let quality = DataImputationEngine::calculate_imputation_quality(
            &original,
            &imputed,
            f64::NAN,
        ).unwrap();

        // Should have high correlation preservation due to linear relationships
        assert!(quality.correlation_preservation > 0.9);
    }

    /// Test KNN imputation precision with known expected values
    #[test]
    fn test_knn_impute_precision() {
        // Create a dataset where we can predict exact imputation values
        let data = vec![
            vec![1.0, 2.0, 3.0],  // Row 0: complete
            vec![4.0, f64::NAN, 6.0],  // Row 1: missing middle value
            vec![7.0, 8.0, 9.0],  // Row 2: complete
        ];

        let result = DataImputationEngine::knn_impute(&data, 2, f64::NAN).unwrap();

        // With k=2, the missing value at (1,1) should be imputed as weighted average
        // of neighbors (0,0) and (2,2) which have values 2.0 and 8.0
        // Distance to row 0: sqrt((4-1)^2 + (6-3)^2) = sqrt(9+9) = sqrt(18) ≈ 4.24
        // Distance to row 2: sqrt((4-7)^2 + (6-9)^2) = sqrt(9+9) = sqrt(18) ≈ 4.24
        // Equal distances, so weighted average: (2.0 + 8.0) / 2 = 5.0
        assert!(approx_eq(result.imputed_data[1][1], 5.0, 1e-10));
        assert_eq!(result.imputed_count, 1);
    }

    /// Test mean imputation precision
    #[test]
    fn test_mean_impute_precision() {
        let data = vec![
            vec![1.0, 2.0, f64::NAN],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::mean_impute(&data, f64::NAN).unwrap();

        // Column 0 mean: (1+4+7)/3 = 4.0
        // Column 1 mean: (2+8)/2 = 5.0
        // Column 2 mean: (6+9)/2 = 7.5
        assert!(approx_eq(result.imputed_data[0][2], 7.5, 1e-10)); // Missing at (0,2)
        assert!(approx_eq(result.imputed_data[1][1], 5.0, 1e-10)); // Missing at (1,1)
        assert_eq!(result.imputed_count, 2);
    }

    /// Test median imputation precision
    #[test]
    fn test_median_impute_precision() {
        let data = vec![
            vec![1.0, 2.0, f64::NAN],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
            vec![10.0, 11.0, 12.0],
        ];

        let result = DataImputationEngine::median_impute(&data, f64::NAN).unwrap();

        // Column 0 median: [1,4,7,10] -> sorted [1,4,7,10], median = (4+7)/2 = 5.5
        // Column 1 median: [2,8,11] -> sorted [2,8,11], median = 8.0
        // Column 2 median: [6,9,12] -> sorted [6,9,12], median = 9.0
        assert!(approx_eq(result.imputed_data[0][2], 9.0, 1e-10)); // Missing at (0,2)
        assert!(approx_eq(result.imputed_data[1][1], 8.0, 1e-10)); // Missing at (1,1)
        assert_eq!(result.imputed_count, 2);
    }

    /// Test regression imputation precision
    #[test]
    fn test_regression_impute_precision() {
        // Simple case: y = 2x + 1, with one missing y value
        let data = vec![
            vec![1.0, 3.0],    // x=1, y=3 (3 = 2*1 + 1)
            vec![2.0, 5.0],    // x=2, y=5 (5 = 2*2 + 1)
            vec![3.0, f64::NAN], // x=3, y=? (should be 7 = 2*3 + 1)
            vec![4.0, 9.0],    // x=4, y=9 (9 = 2*4 + 1)
        ];

        let result = DataImputationEngine::regression_impute(&data, f64::NAN).unwrap();

        // Should predict y = 2*3 + 1 = 7 for the missing value
        assert!(approx_eq(result.imputed_data[2][1], 7.0, 1e-6)); // Allow small numerical error
        assert_eq!(result.imputed_count, 1);
    }

    /// Test quality metrics precision
    #[test]
    fn test_quality_metrics_precision() {
        // Perfect correlation preservation case
        let original = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 4.0, 6.0],
            vec![3.0, 6.0, 9.0],
        ];
        let imputed = original.clone(); // No changes

        let quality = DataImputationEngine::calculate_imputation_quality(&original, &imputed, f64::NAN).unwrap();

        // Should have perfect preservation
        assert!(approx_eq(quality.correlation_preservation, 1.0, 1e-10));
        assert!(approx_eq(quality.variance_preservation, 1.0, 1e-10));

        // Test with some degradation
        let degraded = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 4.1, 6.0], // Slight change
            vec![3.0, 6.0, 9.0],
        ];

        let quality_degraded = DataImputationEngine::calculate_imputation_quality(&original, &degraded, f64::NAN).unwrap();

        // Should have high but not perfect preservation
        assert!(quality_degraded.correlation_preservation > 0.95);
        assert!(quality_degraded.correlation_preservation < 1.0);
        assert!(quality_degraded.variance_preservation > 0.95);
        assert!(quality_degraded.variance_preservation < 1.0);
    }

    /// Test auto imputation method selection
    #[test]
    fn test_auto_impute_method_selection() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::auto_impute(&data, f64::NAN).unwrap();

        // Should select a method and impute successfully
        assert!(!result.method.is_empty());
        assert_ne!(result.method, "none");
        assert_eq!(result.imputed_count, 1);
        assert!(result.imputed_data[1][1].is_finite());
        assert!(!result.imputed_data[1][1].is_nan());

        // Quality metrics should be reasonable
        assert!(result.quality_metrics.correlation_preservation >= 0.0);
        assert!(result.quality_metrics.correlation_preservation <= 1.0);
        assert!(result.quality_metrics.variance_preservation >= 0.0);
        assert!(result.quality_metrics.variance_preservation <= 1.0);
    }

    /// Test KNN with different distance calculations
    #[test]
    fn test_knn_distance_calculations() {
        let data = vec![
            vec![0.0, 0.0, 1.0],  // Row 0
            vec![1.0, f64::NAN, 2.0], // Row 1: missing middle value
            vec![2.0, 2.0, 3.0],  // Row 2
        ];

        let result = DataImputationEngine::knn_impute(&data, 1, f64::NAN).unwrap();

        // With k=1, should use closest neighbor
        // Distance from row 1 to row 0: sqrt((1-0)^2 + (2-1)^2) = sqrt(1+1) = 1.414
        // Distance from row 1 to row 2: sqrt((1-2)^2 + (2-3)^2) = sqrt(1+1) = 1.414
        // Equal distance, but algorithm should pick one consistently
        assert!(result.imputed_data[1][1].is_finite());
        assert!(!result.imputed_data[1][1].is_nan());
        assert_eq!(result.imputed_count, 1);
    }

    /// Test imputation with mixed NaN and custom missing values
    #[test]
    fn test_mixed_missing_values() {
        let data = vec![
            vec![1.0, f64::NAN, 3.0],     // NaN
            vec![4.0, -999.0, 6.0],       // Custom missing
            vec![7.0, 8.0, f64::NAN],     // NaN
        ];

        // Test with NaN as missing (imputes NaN values only)
        let result_nan = DataImputationEngine::mean_impute(&data, f64::NAN).unwrap();
        assert_eq!(result_nan.imputed_count, 2); // Two NaN values

        // Test with custom value as missing (imputes both NaN and custom missing)
        let result_custom = DataImputationEngine::mean_impute(&data, -999.0).unwrap();
        assert_eq!(result_custom.imputed_count, 3); // All NaN and -999.0 values
    }

    /// Test correlation calculation precision
    #[test]
    fn test_correlation_calculation_precision() {
        // Test perfect positive correlation
        let pairs_pos = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
        let x_vals: Vec<f64> = pairs_pos.iter().map(|(x, _)| *x).collect();
        let y_vals: Vec<f64> = pairs_pos.iter().map(|(_, y)| *y).collect();
        let corr_pos = CorrelationMethods::pearson_correlation(&x_vals, &y_vals, None, None).map(|(r, _)| r).unwrap_or(0.0);
        assert!(approx_eq(corr_pos, 1.0, 1e-10));

        // Test perfect negative correlation
        let pairs_neg = vec![(1.0, 3.0), (2.0, 2.0), (3.0, 1.0)];
        let x_vals: Vec<f64> = pairs_neg.iter().map(|(x, _)| *x).collect();
        let y_vals: Vec<f64> = pairs_neg.iter().map(|(_, y)| *y).collect();
        let corr_neg = CorrelationMethods::pearson_correlation(&x_vals, &y_vals, None, None).map(|(r, _)| r).unwrap_or(0.0);
        assert!(approx_eq(corr_neg, -1.0, 1e-10));

        // Test zero correlation
        let pairs_zero = vec![(1.0, 1.0), (2.0, 1.0), (3.0, 1.0)];
        let x_vals: Vec<f64> = pairs_zero.iter().map(|(x, _)| *x).collect();
        let y_vals: Vec<f64> = pairs_zero.iter().map(|(_, y)| *y).collect();
        let corr_zero = CorrelationMethods::pearson_correlation(&x_vals, &y_vals, None, None).map(|(r, _)| r).unwrap_or(0.0);
        assert!(approx_eq(corr_zero, 0.0, 1e-10));

        // Test insufficient data
        let pairs_small = vec![(1.0, 2.0)];
        let x_vals: Vec<f64> = pairs_small.iter().map(|(x, _)| *x).collect();
        let y_vals: Vec<f64> = pairs_small.iter().map(|(_, y)| *y).collect();
        let corr_small = CorrelationMethods::pearson_correlation(&x_vals, &y_vals, None, None).map(|(r, _)| r).unwrap_or(0.0);
        assert_eq!(corr_small, 0.0);
    }

    /// Test large dataset handling
    #[test]
    fn test_large_dataset_handling() {
        // Create a moderately large dataset
        let mut data = Vec::new();
        for i in 0..100 {
            let row = vec![
                i as f64,
                (i * 2) as f64,
                if i % 10 == 0 { f64::NAN } else { (i * 3) as f64 }, // Every 10th value missing
            ];
            data.push(row);
        }

        let result = DataImputationEngine::knn_impute(&data, 5, f64::NAN).unwrap();

        // Should handle large dataset
        assert_eq!(result.imputed_count, 10); // 10 missing values
        for row in &result.imputed_data {
            assert!(row[2].is_finite()); // All values should be imputed
        }

        // Quality should be reasonable
        assert!(result.quality_metrics.correlation_preservation > 0.8);
        assert!(result.quality_metrics.variance_preservation > 0.8);
    }

    /// Test regression imputation with multiple predictors
    #[test]
    fn test_regression_multiple_predictors() {
        // y = x1 + 2*x2 + 1
        let data = vec![
            vec![1.0, 1.0, 4.0],    // x1=1, x2=1, y=4 (1+2+1=4)
            vec![2.0, 1.0, 5.0],    // x1=2, x2=1, y=5 (2+2+1=5)
            vec![1.0, 2.0, f64::NAN], // x1=1, x2=2, y=? (1+4+1=6)
            vec![2.0, 2.0, 7.0],    // x1=2, x2=2, y=7 (2+4+1=7)
        ];

        let result = DataImputationEngine::regression_impute(&data, f64::NAN).unwrap();

        // Should predict close to 6.0 (though simple regression uses only first predictor)
        assert!(result.imputed_data[2][2].is_finite());
        assert!(!result.imputed_data[2][2].is_nan());
        assert_eq!(result.imputed_count, 1);
    }

    /// Test edge case: all values missing in a column
    #[test]
    fn test_all_missing_column_fallback() {
        let data = vec![
            vec![1.0, f64::NAN, 3.0],
            vec![2.0, f64::NAN, 4.0],
            vec![3.0, f64::NAN, 5.0],
        ];

        let result = DataImputationEngine::mean_impute(&data, f64::NAN).unwrap();

        // Column 1 has all missing values, should fall back to 0.0
        for row in &result.imputed_data {
            assert!(approx_eq(row[1], 0.0, 1e-10));
        }
        assert_eq!(result.imputed_count, 3);
    }

    /// Test imputation result structure
    #[test]
    fn test_imputation_result_structure() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, f64::NAN, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = DataImputationEngine::mean_impute(&data, f64::NAN).unwrap();

        // Check result structure
        assert_eq!(result.imputed_data.len(), 3);
        assert_eq!(result.imputed_data[0].len(), 3);
        assert_eq!(result.method, "mean");
        assert_eq!(result.imputed_count, 1);

        // Check quality metrics structure
        assert!(result.quality_metrics.correlation_preservation.is_finite());
        assert!(result.quality_metrics.variance_preservation.is_finite());
        assert!(result.quality_metrics.mae.is_none()); // Not implemented yet
        assert!(result.quality_metrics.rmse.is_none()); // Not implemented yet
    }

    /// Test parallel processing behavior
    #[test]
    fn test_parallel_processing() {
        // Create larger dataset to trigger parallel processing
        let mut data = Vec::new();
        for i in 0..50 {
            let row = vec![
                i as f64,
                (i as f64 * 1.5),
                if i % 5 == 0 { f64::NAN } else { i as f64 * 2.0 },
            ];
            data.push(row);
        }

        let result = DataImputationEngine::knn_impute(&data, 3, f64::NAN).unwrap();

        // Should process in parallel and produce valid results
        assert_eq!(result.imputed_count, 10); // Every 5th value missing
        for row in &result.imputed_data {
            assert!(row[2].is_finite());
        }
    }
}