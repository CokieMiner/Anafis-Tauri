//! Data transformation utilities
//!
//! This module provides data transformation methods for normalizing
//! distributions and stabilizing variance, including Box-Cox and Yeo-Johnson transformations.

use serde::{Deserialize, Serialize};
use crate::scientific::statistics::descriptive::StatisticalMoments;
use argmin::core::{CostFunction, Error};

/// Result of a data transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationResult {
    /// Name of the transformation applied
    pub transformation_name: String,
    /// Transformed data
    pub transformed_data: Vec<f64>,
    /// Optimal transformation parameter (lambda)
    pub lambda: f64,
    /// Log-likelihood of the transformation
    pub log_likelihood: f64,
    /// Shapiro-Wilk normality test statistic for transformed data
    pub normality_test_statistic: f64,
}

/// Data transformation engine
pub struct DataTransformationEngine;

impl DataTransformationEngine {
    /// Apply Yeo-Johnson transformation to normalize data
    ///
    /// The Yeo-Johnson transformation is a power transformation that can handle
    /// both positive and negative values, unlike the Box-Cox transformation.
    ///
    /// Y(λ) = {(y+1)^λ - 1}/λ    if λ ≠ 0 and y ≥ 0
    ///         ln(y+1)            if λ = 0 and y ≥ 0
    ///         -{(-y+1)^(2-λ) - 1}/(2-λ)  if λ ≠ 2 and y < 0
    ///         -ln(-y+1)          if λ = 2 and y < 0
    pub fn yeo_johnson_transform(data: &[f64], lambda: f64) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Cannot transform empty dataset".to_string());
        }

        let mut transformed = Vec::with_capacity(data.len());

        for &y in data {
            let t = if lambda != 0.0 && y >= 0.0 {
                ((y + 1.0).powf(lambda) - 1.0) / lambda
            } else if lambda == 0.0 && y >= 0.0 {
                (y + 1.0).ln()
            } else if lambda != 2.0 && y < 0.0 {
                -(((-y + 1.0).powf(2.0 - lambda) - 1.0) / (2.0 - lambda))
            } else if lambda == 2.0 && y < 0.0 {
                -(-y + 1.0).ln()
            } else {
                return Err(format!("Invalid transformation for y={}, lambda={}", y, lambda));
            };

            if !t.is_finite() {
                return Err(format!("Transformation resulted in non-finite value for y={}, lambda={}", y, lambda));
            }

            transformed.push(t);
        }

        Ok(transformed)
    }

    /// Find optimal Yeo-Johnson transformation parameter using maximum likelihood
    pub fn optimize_yeo_johnson(data: &[f64]) -> Result<TransformationResult, String> {
        if data.is_empty() {
            return Err("Cannot optimize transformation for empty dataset".to_string());
        }

        // Check if data contains zeros or negative values
        let has_negative = data.iter().any(|&x| x < 0.0);
        let lambda_range = if has_negative {
            (-2.0, 2.0) // Wider range for negative data
        } else {
            (-2.0, 3.0) // Standard range for positive data
        };

        // Create cost function for Yeo-Johnson MLE
        #[derive(Clone)]
        struct YeoJohnsonCost {
            data: Vec<f64>,
        }

        impl CostFunction for YeoJohnsonCost {
            type Param = Vec<f64>;
            type Output = f64;

            fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
                let lambda = param[0];

                if !lambda.is_finite() {
                    return Ok(f64::INFINITY);
                }

                // Transform data
                let transformed = match DataTransformationEngine::yeo_johnson_transform(&self.data, lambda) {
                    Ok(t) => t,
                    Err(_) => return Ok(f64::INFINITY),
                };

                // Compute log-likelihood assuming normality after transformation
                let mean = transformed.mean();
                let variance = transformed.variance();

                if !mean.is_finite() || !variance.is_finite() || variance <= 0.0 {
                    return Ok(f64::INFINITY);
                }

                let log_likelihood = transformed.iter()
                    .map(|&x| {
                        let z = (x - mean) / variance.sqrt();
                        -0.5 * (2.0 * std::f64::consts::PI).ln() - variance.ln() / 2.0 - 0.5 * z * z
                    })
                    .sum::<f64>();

                // Add Jacobian adjustment for transformation
                let jacobian_sum = self.data.iter()
                    .map(|&y| Self::yeo_johnson_jacobian(y, lambda))
                    .sum::<f64>();

                Ok(-(log_likelihood + jacobian_sum))
            }
        }

        impl YeoJohnsonCost {
            fn yeo_johnson_jacobian(y: f64, lambda: f64) -> f64 {
                if lambda != 0.0 && y >= 0.0 {
                    (y + 1.0).powf(lambda - 1.0)
                } else if lambda == 0.0 && y >= 0.0 {
                    1.0 / (y + 1.0)
                } else if lambda != 2.0 && y < 0.0 {
                    (-y + 1.0).powf(1.0 - lambda)
                } else if lambda == 2.0 && y < 0.0 {
                    1.0 / (-y + 1.0)
                } else {
                    0.0
                }
            }
        }

        let cost_fn = YeoJohnsonCost { data: data.to_vec() };

        // Simple grid search followed by local optimization
        let lambda_candidates = if has_negative {
            vec![-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0]
        } else {
            vec![-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0]
        };

        let mut best_lambda = 1.0;
        let mut best_cost = f64::INFINITY;

        for &lambda in &lambda_candidates {
            if let Ok(cost) = cost_fn.cost(&vec![lambda]) {
                if cost < best_cost && cost.is_finite() {
                    best_cost = cost;
                    best_lambda = lambda;
                }
            }
        }

        // Fine-tune around best lambda
        let mut fine_lambda = best_lambda;
        let mut fine_cost = best_cost;

        for offset in [-0.1, -0.05, 0.0, 0.05, 0.1] {
            let test_lambda = best_lambda + offset;
            if test_lambda >= lambda_range.0 && test_lambda <= lambda_range.1 {
                if let Ok(cost) = cost_fn.cost(&vec![test_lambda]) {
                    if cost < fine_cost && cost.is_finite() {
                        fine_cost = cost;
                        fine_lambda = test_lambda;
                    }
                }
            }
        }

        // Apply optimal transformation
        let transformed_data = Self::yeo_johnson_transform(data, fine_lambda)?;

        // Compute final log-likelihood
        let mean = transformed_data.mean();
        let variance = transformed_data.variance();
        let log_likelihood = transformed_data.iter()
            .map(|&x| {
                let z = (x - mean) / variance.sqrt();
                -0.5 * (2.0 * std::f64::consts::PI).ln() - variance.ln() / 2.0 - 0.5 * z * z
            })
            .sum::<f64>();

        // Simple normality test (simplified Shapiro-Wilk approximation)
        let normality_stat = Self::normality_test_statistic(&transformed_data);

        Ok(TransformationResult {
            transformation_name: "yeo_johnson".to_string(),
            transformed_data,
            lambda: fine_lambda,
            log_likelihood,
            normality_test_statistic: normality_stat,
        })
    }

    /// Apply Box-Cox transformation (for positive data only)
    ///
    /// The Box-Cox transformation is: (y^λ - 1)/λ for λ ≠ 0, ln(y) for λ = 0
    pub fn box_cox_transform(data: &[f64], lambda: f64) -> Result<Vec<f64>, String> {
        if data.iter().any(|&x| x <= 0.0) {
            return Err("Box-Cox transformation requires positive data".to_string());
        }

        if data.is_empty() {
            return Err("Cannot transform empty dataset".to_string());
        }

        let mut transformed = Vec::with_capacity(data.len());

        for &y in data {
            let t = if lambda != 0.0 {
                (y.powf(lambda) - 1.0) / lambda
            } else {
                y.ln()
            };

            if !t.is_finite() {
                return Err(format!("Box-Cox transformation resulted in non-finite value for y={}, lambda={}", y, lambda));
            }

            transformed.push(t);
        }

        Ok(transformed)
    }

    /// Find optimal Box-Cox transformation parameter
    pub fn optimize_box_cox(data: &[f64]) -> Result<TransformationResult, String> {
        if data.iter().any(|&x| x <= 0.0) {
            return Err("Box-Cox transformation requires positive data".to_string());
        }

        if data.is_empty() {
            return Err("Cannot optimize transformation for empty dataset".to_string());
        }

        // Grid search for optimal lambda
        let lambda_candidates = vec![-2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0];

        let mut best_lambda = 1.0;
        let mut best_log_likelihood = f64::NEG_INFINITY;

        for &lambda in &lambda_candidates {
            if let Ok(transformed) = Self::box_cox_transform(data, lambda) {
                let mean = transformed.mean();
                let variance = transformed.variance();

                if mean.is_finite() && variance.is_finite() && variance > 0.0 {
                    let log_likelihood = transformed.iter()
                        .map(|&x| {
                            let z = (x - mean) / variance.sqrt();
                            -0.5 * (2.0 * std::f64::consts::PI).ln() - variance.ln() / 2.0 - 0.5 * z * z
                        })
                        .sum::<f64>();

                    // Add Jacobian adjustment
                    let jacobian_sum = data.iter()
                        .map(|&y| (lambda - 1.0) * y.ln())
                        .sum::<f64>();

                    let adjusted_ll = log_likelihood + jacobian_sum;

                    if adjusted_ll > best_log_likelihood && adjusted_ll.is_finite() {
                        best_log_likelihood = adjusted_ll;
                        best_lambda = lambda;
                    }
                }
            }
        }

        // Apply optimal transformation
        let transformed_data = Self::box_cox_transform(data, best_lambda)?;
        let normality_stat = Self::normality_test_statistic(&transformed_data);

        Ok(TransformationResult {
            transformation_name: "box_cox".to_string(),
            transformed_data,
            lambda: best_lambda,
            log_likelihood: best_log_likelihood,
            normality_test_statistic: normality_stat,
        })
    }

    /// Simple normality test statistic (approximation of Shapiro-Wilk)
    fn normality_test_statistic(data: &[f64]) -> f64 {
        if data.len() < 3 {
            return 0.0;
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted_data.len();
        let mean = sorted_data.mean();
        let std_dev = sorted_data.variance().sqrt();

        if std_dev == 0.0 {
            return 1.0; // Perfectly normal (constant)
        }

        // Compute b2 statistic (simplified Shapiro-Wilk approximation)
        let mut b2 = 0.0;
        for &x in &sorted_data {
            let z = (x - mean) / std_dev;
            b2 += z * z;
        }

        // For normal distribution, b2 should be approximately n-1
        // Return p-value approximation
        let expected_b2 = (n - 1) as f64;
        1.0 - (b2 - expected_b2).abs() / (2.0 * expected_b2.sqrt())
    }

    /// Apply multiple transformations and return the best one
    #[allow(unused_assignments)]
    pub fn find_best_transformation(data: &[f64]) -> Result<TransformationResult, String> {
        if data.is_empty() {
            return Err("Cannot transform empty dataset".to_string());
        }

        let mut best_result = None;
        let mut best_score = f64::NEG_INFINITY;

        // Try Yeo-Johnson (works with any data)
        if let Ok(yj_result) = Self::optimize_yeo_johnson(data) {
            let score = yj_result.log_likelihood + yj_result.normality_test_statistic;
            if score > best_score {
                best_score = score;
                best_result = Some(yj_result);
            }
        }

        // Try Box-Cox if data is positive
        if data.iter().all(|&x| x > 0.0) {
            if let Ok(bc_result) = Self::optimize_box_cox(data) {
                let score = bc_result.log_likelihood + bc_result.normality_test_statistic;
                if score > best_score {
                    best_score = score;
                    best_result = Some(bc_result);
                }
            }
        }

        best_result.ok_or_else(|| "No suitable transformation found".to_string())
    }
}