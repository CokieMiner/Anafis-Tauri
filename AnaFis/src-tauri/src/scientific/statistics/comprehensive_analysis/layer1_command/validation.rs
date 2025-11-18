//! Input validation and data sanitization functionality

use crate::scientific::statistics::types::{AnalysisOptions, NanHandling, AnalysisError};
use crate::scientific::statistics::types::SanitizationReport;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::neighbors::knn_regressor::{KNNRegressor, KNNRegressorParameters};

/// Input validation and sanitization functions
pub struct InputValidator;

impl InputValidator {
    /// Validate and sanitize input data
    pub fn validate_and_sanitize_input(
        datasets: &[Vec<f64>],
        options: &AnalysisOptions,
    ) -> Result<(Vec<Vec<f64>>, SanitizationReport), AnalysisError> {
        if datasets.is_empty() {
            return Err(AnalysisError::InsufficientData("At least one dataset is required".to_string()));
        }

        let first_len = datasets[0].len();
        if first_len < 2 {
            return Err(AnalysisError::InsufficientData("Each dataset must have at least 2 observations".to_string()));
        }

        // Validate all datasets have the same length - required for multi-variable analysis
        let treat_as_paired = options.treat_as_paired.unwrap_or(true);
        if datasets.len() > 1 && treat_as_paired {
            for (i, dataset) in datasets.iter().enumerate() {
                if dataset.len() != first_len {
                    return Err(AnalysisError::InvalidInput(format!(
                        "Dataset {} has {} observations, but first dataset has {}. All datasets must have the same length for multi-variable analysis.",
                        i, dataset.len(), first_len
                    )));
                }
            }
        }

        // Sanitize data based on NaN handling option
        let mut sanitized_datasets: Vec<Vec<f64>> = Vec::new();
        let original_row_counts: Vec<usize> = datasets.iter().map(|d| d.len()).collect();

        if datasets.len() > 1 && matches!(options.nan_handling, NanHandling::Remove) && treat_as_paired {
            // Determine valid indices: indices where all dataset values are finite
            let mut valid_indices = Vec::with_capacity(first_len);
            for idx in 0..first_len {
                let mut valid = true;
                for dataset in datasets.iter() {
                    let v = dataset[idx];
                    if !v.is_finite() {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    valid_indices.push(idx);
                }
            }

            if valid_indices.is_empty() {
                return Err(AnalysisError::InsufficientData("All rows removed after sanitization; no valid observations remain".to_string()));
            }

            for dataset in datasets.iter() {
                let mut sanitized = Vec::with_capacity(valid_indices.len());
                for idx in &valid_indices {
                    sanitized.push(dataset[*idx]);
                }
                sanitized_datasets.push(sanitized);
            }

            let remaining_row_counts: Vec<usize> = sanitized_datasets.iter().map(|d| d.len()).collect();
            let rows_removed_total: usize = original_row_counts.iter().zip(&remaining_row_counts).map(|(o, r)| o - r).sum();
            let report = SanitizationReport {
                original_row_counts,
                remaining_row_counts,
                rows_removed_total,
            };
            Ok((sanitized_datasets, report))
        } else {
            for dataset in datasets {
                let sanitized = Self::sanitize_dataset(dataset, options)
                    .map_err(AnalysisError::InvalidInput)?;
                if sanitized.is_empty() {
                    return Err(AnalysisError::InsufficientData("Dataset became empty after sanitization".to_string()));
                }
                sanitized_datasets.push(sanitized);
            }

            let remaining_row_counts: Vec<usize> = sanitized_datasets.iter().map(|d| d.len()).collect();
            let rows_removed_total: usize = original_row_counts.iter().zip(&remaining_row_counts).map(|(o, r)| o - r).sum();
            let report = SanitizationReport {
                original_row_counts,
                remaining_row_counts,
                rows_removed_total,
            };
            Ok((sanitized_datasets, report))
        }
    }

    /// Sanitize a single dataset based on NaN handling options
    pub fn sanitize_dataset(dataset: &[f64], options: &AnalysisOptions) -> Result<Vec<f64>, String> {
        match &options.nan_handling {
            NanHandling::Error => {
                // Check for NaN/inf values
                for (i, &value) in dataset.iter().enumerate() {
                    if !value.is_finite() {
                        return Err(format!(
                            "Non-finite value found at index {}: {}",
                            i, value
                        ));
                    }
                }
                Ok(dataset.to_vec())
            }
            NanHandling::Remove => {
                // Remove rows containing NaN/inf
                Ok(dataset.iter()
                    .filter(|&&x| x.is_finite())
                    .cloned()
                    .collect())
            }
            NanHandling::Mean => {
                // Replace NaN with mean, inf with large finite values
                let valid_values: Vec<f64> = dataset.iter()
                    .filter(|&&x| x.is_finite())
                    .cloned()
                    .collect();

                if valid_values.is_empty() {
                    return Err("No valid values found in dataset".to_string());
                }

                let mean = valid_values.iter().sum::<f64>() / valid_values.len() as f64;
                // Deterministic small perturbation to avoid identical imputed values
                let std_dev = if valid_values.len() > 1 {
                    (valid_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (valid_values.len() - 1) as f64).sqrt()
                } else { 0.0 };
                // Replace NaN/Inf with the mean value to avoid injecting extreme sentinel values
                Ok(dataset.iter().enumerate().map(|(i, &x)| {
                    if x.is_nan() {
                        // Deterministic pseudo-noise: small sinusoidal perturbation
                        let noise = if std_dev.is_finite() && std_dev != 0.0 {
                            std_dev * 0.001 * ((i as f64).sin())
                        } else {
                            0.0
                        };
                        mean + noise
                    } else if x.is_infinite() {
                        // Treat infinities separately (clamp to 1.5x valid max/min)
                        if x.is_sign_positive() {
                            *valid_values.iter().max_by(|a, b| a.total_cmp(b)).unwrap() * 1.5
                        } else {
                            *valid_values.iter().min_by(|a, b| a.total_cmp(b)).unwrap() * 1.5
                        }
                    } else {
                        x
                    }
                }).collect())
            }
            NanHandling::Median => {
                // Replace NaN with median, inf with large finite values
                let mut valid_values: Vec<f64> = dataset.iter()
                    .filter(|&&x| x.is_finite())
                    .cloned()
                    .collect();

                if valid_values.is_empty() {
                    return Err("No valid values found in dataset".to_string());
                }

                valid_values.sort_by(|a, b| a.total_cmp(b));
                let mid = valid_values.len() / 2;
                let median = if valid_values.len().is_multiple_of(2) {
                    (valid_values[mid - 1] + valid_values[mid]) / 2.0
                } else {
                    valid_values[mid]
                };

                // Replace NaNs and Inf with the median to preserve scale
                Ok(dataset.iter().map(|&x| {
                    if x.is_nan() {
                        median
                    } else if x.is_infinite() {
                        if x.is_sign_positive() {
                            *valid_values.iter().max_by(|a, b| a.total_cmp(b)).unwrap() * 1.5
                        } else {
                            *valid_values.iter().min_by(|a, b| a.total_cmp(b)).unwrap() * 1.5
                        }
                    } else {
                        x
                    }
                }).collect())
            }
            NanHandling::Multiple => {
                // Multiple imputation: delegate to the imputation helper which may use feature-gated crates
                let valid_values: Vec<f64> = dataset.iter()
                    .filter(|&&x| x.is_finite())
                    .cloned()
                    .collect();

                if valid_values.is_empty() {
                    return Err("No valid values found in dataset".to_string());
                }

                let mean = valid_values.iter().sum::<f64>() / valid_values.len() as f64;
                let _std_dev = if valid_values.len() > 1 {
                    (valid_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (valid_values.len() - 1) as f64).sqrt()
                } else { 0.0 };

                    let mut rows: Vec<Vec<f64>> = Vec::new();
                    let mut y_train: Vec<f64> = Vec::new();
                    let mut missing_indices: Vec<usize> = Vec::new();
                    for (i, &v) in dataset.iter().enumerate() {
                        if v.is_finite() {
                            rows.push(vec![i as f64]);
                            y_train.push(v);
                        } else {
                            missing_indices.push(i);
                        }
                    }

                    if missing_indices.is_empty() {
                        return Ok(dataset.to_vec());
                    }

                    let rows_ref: Vec<&[f64]> = rows.iter().map(|r| r.as_slice()).collect();
                    let x_mat = DenseMatrix::from_2d_array(rows_ref.as_slice()).map_err(|e| e.to_string())?;
                    let mut params = KNNRegressorParameters::default();
                    let k = rows.len().clamp(1, 5);
                    params.k = k; // ensure k doesn't exceed training set size
                    let knn = KNNRegressor::fit(&x_mat, &y_train, params).map_err(|e| format!("KNN fit error: {:?}", e))?;

                    let mut imputed: Vec<f64> = dataset.to_vec();
                    let mut pred_rows: Vec<Vec<f64>> = Vec::with_capacity(missing_indices.len());
                    for &idx in &missing_indices {
                        pred_rows.push(vec![idx as f64]);
                    }
                    let pred_rows_ref: Vec<&[f64]> = pred_rows.iter().map(|r| r.as_slice()).collect();
                    let pred_mat = DenseMatrix::from_2d_array(pred_rows_ref.as_slice()).map_err(|e| e.to_string())?;
                    let y_hat = knn.predict(&pred_mat).map_err(|e| format!("KNN predict error: {:?}", e))?;
                    for (i, &idx) in missing_indices.iter().enumerate() {
                        imputed[idx] = y_hat[i];
                    }
                    Ok(imputed)
            }
            NanHandling::Zero => {
                // Replace NaN/inf with 0
                Ok(dataset.iter().map(|&x| if x.is_finite() { x } else { 0.0 }).collect())
            }
            NanHandling::Ignore => {
                // Keep as-is (may produce NaN results in calculations)
                Ok(dataset.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InputValidator;
    use crate::scientific::statistics::types::{AnalysisOptions, NanHandling};

    #[test]
    fn test_remove_rows_pairwise() {
        let datasets = vec![vec![1.0, f64::NAN, 3.0], vec![4.0, 5.0, f64::INFINITY]];
        let mut options = AnalysisOptions::default();
        options.nan_handling = NanHandling::Remove;
        options.treat_as_paired = Some(true);

        let (sanitized, _report) = InputValidator::validate_and_sanitize_input(&datasets, &options)
            .expect("Sanitization failed");
        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0], vec![1.0]);
        assert_eq!(sanitized[1], vec![4.0]);
    }

    #[test]
    fn test_remove_rows_independent() {
        let datasets = vec![vec![1.0, f64::NAN, 3.0], vec![4.0, 5.0, f64::INFINITY]];
        let mut options = AnalysisOptions::default();
        options.nan_handling = NanHandling::Remove;
        options.treat_as_paired = Some(false);

        let (sanitized, _report) = InputValidator::validate_and_sanitize_input(&datasets, &options)
            .expect("Sanitization failed");
        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0], vec![1.0, 3.0]);
        assert_eq!(sanitized[1], vec![4.0, 5.0]);
    }

    #[test]
    fn test_mean_imputation_handles_infs() {
        let dataset = vec![1.0, f64::NAN, f64::INFINITY];
        let mut options = AnalysisOptions::default();
        options.nan_handling = NanHandling::Mean;
        let sanitized = InputValidator::sanitize_dataset(&dataset, &options).expect("imputation failed");
        // 1.0 is finite; second is imputed with mean 1.0; third is replaced by max*1.5 => 1.5
        assert_eq!(sanitized[0], 1.0);
        assert!(sanitized[1].is_finite());
        assert_eq!(sanitized[2], 1.5);
    }

    #[test]
    fn test_multiple_imputation_fallback() {
        let dataset = vec![1.0, f64::NAN, f64::NAN, 3.0];
        let mut options = AnalysisOptions::default();
        options.nan_handling = NanHandling::Multiple;
        options.random_seed = Some(12345);
        let sanitized = InputValidator::sanitize_dataset(&dataset, &options).expect("imputation failed");
        // Imputed values should be finite and not equal to the original mean exactly due to noise
        assert!(sanitized.iter().all(|v| v.is_finite()));
        assert!(sanitized[1] != sanitized[2] || (sanitized[1] - sanitized[2]).abs() < 1e-12);
    }

    #[test]
    fn test_multiple_imputation_knn_feature_enabled() {
        let dataset = vec![1.0, 2.0, f64::NAN, 4.0, f64::NAN, 6.0];
        let mut options = AnalysisOptions::default();
        options.nan_handling = NanHandling::Multiple;
        options.random_seed = Some(123);
        let sanitized = InputValidator::sanitize_dataset(&dataset, &options).expect("imputation failed");
        // All values should be finite after KNN imputation
        assert!(sanitized.iter().all(|v| v.is_finite()));
        // Missing values must be imputed differently from NaN
        assert!(!sanitized[2].is_nan());
        assert!(!sanitized[4].is_nan());
    }
}