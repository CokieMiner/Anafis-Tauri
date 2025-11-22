//! Reliability Analysis Module
//!
//! This module provides comprehensive reliability analysis for psychometric scales and measurements:
//! - Cronbach's alpha
//! - Item-total correlations
//! - McDonald's omega
//! - Scale reliability measures

use crate::scientific::statistics::correlation::CorrelationMethods;
use crate::scientific::statistics::primitives::LinearAlgebra;
use crate::scientific::statistics::descriptive::StatisticalMoments;
use crate::scientific::statistics::distributions::distribution_functions;
use ndarray::Array2;
use rayon::prelude::*;

/// Reliability analysis results
#[derive(Debug, Clone)]
pub struct ReliabilityAnalysis {
    pub cronbach_alpha: f64,
    pub mcdonald_omega: f64,
    pub item_total_correlations: Vec<f64>,
    pub scale_reliability: ScaleReliability,
    pub reliability_interpretation: String,
}

/// Scale reliability measures
#[derive(Debug, Clone)]
pub struct ScaleReliability {
    pub average_interitem_correlation: f64,
    pub split_half_reliability: Option<f64>,
    pub standard_error_of_measurement: f64,
}

/// Reliability Analysis Engine
/// Main engine for reliability analysis
pub struct ReliabilityEngine;

impl ReliabilityEngine {
    /// Perform comprehensive reliability analysis
    pub fn analyze_scale_reliability(datasets: &[Vec<f64>]) -> Result<ReliabilityAnalysis, String> {
        if datasets.len() < 2 {
            return Err("Reliability analysis requires at least 2 items".to_string());
        }

        // Validate data lengths
        Self::validate_data_lengths(datasets)?;

        // Cronbach's alpha
        let cronbach_alpha = Self::calculate_cronbach_alpha(datasets)?;

        // McDonald's omega
        let mcdonald_omega = Self::calculate_mcdonald_omega(datasets)?;

        // Item-total correlations
        let item_total_correlations = Self::calculate_item_total_correlations(datasets)?;

        // Scale reliability measures
        let scale_reliability = Self::calculate_scale_reliability(datasets)?;

        // Interpretation
        let reliability_interpretation = Self::interpret_reliability(cronbach_alpha, mcdonald_omega);

        Ok(ReliabilityAnalysis {
            cronbach_alpha,
            mcdonald_omega,
            item_total_correlations,
            scale_reliability,
            reliability_interpretation,
        })
    }

    /// Validate that all datasets have the same length
    fn validate_data_lengths(datasets: &[Vec<f64>]) -> Result<(), String> {
        if datasets.is_empty() {
            return Err("No datasets provided".to_string());
        }

        let first_len = datasets[0].len();
        for (i, dataset) in datasets.iter().enumerate() {
            if dataset.len() != first_len {
                return Err(format!(
                    "Dataset {} has length {}, expected {}",
                    i, dataset.len(), first_len
                ));
            }
        }

        Ok(())
    }

    /// Calculate Cronbach's alpha
    fn calculate_cronbach_alpha(datasets: &[Vec<f64>]) -> Result<f64, String> {
        let k = datasets.len() as f64;

        // Calculate variance of each item
        let item_variances: Vec<f64> = datasets.iter()
            .map(|item| item.variance())
            .collect();

        // Calculate variance of total scores
        let total_scores: Vec<f64> = (0..datasets[0].len())
            .map(|i| datasets.iter().map(|item| item[i]).sum::<f64>())
            .collect();

        let total_variance = total_scores.variance();

        // Sum of item variances
        let sum_item_variances: f64 = item_variances.iter().sum();

        // Cronbach's alpha formula: α = (k / (k-1)) * (1 - Σσ²ᵢ / σ²ₜ)
        let alpha = if total_variance > 0.0 {
            (k / (k - 1.0)) * (1.0 - sum_item_variances / total_variance)
        } else {
            0.0
        };

        // Clamp to valid range
        Ok(alpha.clamp(0.0, 1.0))
    }

    /// Calculate McDonald's omega using principal component analysis
    fn calculate_mcdonald_omega(datasets: &[Vec<f64>]) -> Result<f64, String> {
        let num_items = datasets.len();
        if num_items < 2 {
            return Ok(0.0); // Omega not well-defined for single item
        }

        // Create correlation matrix
        let mut corr_matrix = Array2::<f64>::zeros((num_items, num_items));
        for i in 0..num_items {
            for j in i..num_items {
                if i == j {
                    corr_matrix[[i, j]] = 1.0;
                } else {
                    let corr = CorrelationMethods::pearson_correlation(&datasets[i], &datasets[j], None, None).map(|(r, _)| r)?;
                    corr_matrix[[i, j]] = corr;
                    corr_matrix[[j, i]] = corr;
                }
            }
        }

        // Perform eigendecomposition (PCA)
        let (eigenvalues, eigenvectors) = LinearAlgebra::eigenvalue_decomposition(&corr_matrix)?;

        // Find the largest eigenvalue and corresponding eigenvector
        let (max_eigenvalue_index, &max_eigenvalue) = eigenvalues.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .ok_or("Could not find maximum eigenvalue")?;

        let principal_component = eigenvectors.column(max_eigenvalue_index);

        // Calculate factor loadings (standardized regression coefficients)
        let loadings: Vec<f64> = principal_component.iter()
            .map(|&loading| loading * max_eigenvalue.sqrt())
            .collect();

        // Calculate omega: ω = Σλ² / (Σλ² + Σ(1 - λ²))
        let sum_loadings_squared: f64 = loadings.iter().map(|&l| l * l).sum();
        let sum_uniquenesses: f64 = loadings.iter().map(|&l| 1.0 - l * l).sum();

        let omega = if sum_loadings_squared + sum_uniquenesses > 0.0 {
            sum_loadings_squared / (sum_loadings_squared + sum_uniquenesses)
        } else {
            0.0
        };

        Ok(omega.clamp(0.0, 1.0))
    }

    /// Calculate item-total correlations (corrected for overlap)
    fn calculate_item_total_correlations(datasets: &[Vec<f64>]) -> Result<Vec<f64>, String> {
        let n_items = datasets.len();

        // Compute total scores for each observation (excluding each item in parallel)
        let total_scores_per_item: Vec<Vec<f64>> = (0..n_items).into_par_iter()
            .map(|exclude_idx| {
                (0..datasets[0].len())
                    .map(|obs_idx| {
                        datasets.iter().enumerate()
                            .filter(|(i, _)| *i != exclude_idx)
                            .map(|(_, item)| item[obs_idx])
                            .sum::<f64>()
                    })
                    .collect()
            })
            .collect();

        // Compute correlations in parallel
        let correlations: Vec<f64> = (0..n_items).into_par_iter()
            .map(|i| {
                CorrelationMethods::pearson_correlation(&datasets[i], &total_scores_per_item[i], None, None)
                    .map(|(r, _)| r)
                    .unwrap_or(0.0)
            })
            .collect();

        Ok(correlations)
    }

    /// Calculate comprehensive scale reliability measures
    fn calculate_scale_reliability(datasets: &[Vec<f64>]) -> Result<ScaleReliability, String> {
        let n_items = datasets.len();
        let n_observations = datasets[0].len();

        // Average inter-item correlation
        let inter_item_correlations: Vec<f64> = (0..n_items).into_par_iter()
            .flat_map(|i| {
                (i + 1..n_items).into_par_iter().map(move |j| {
                    CorrelationMethods::pearson_correlation(&datasets[i], &datasets[j], None, None)
                        .map(|(r, _)| r)
                        .unwrap_or(0.0)
                })
            })
            .collect();

        let average_interitem_correlation = if inter_item_correlations.is_empty() {
            0.0
        } else {
            inter_item_correlations.iter().sum::<f64>() / inter_item_correlations.len() as f64
        };

        // Split-half reliability (Spearman-Brown prophecy formula)
        let split_half_reliability = if n_items >= 4 {
            Some(Self::calculate_split_half_reliability(datasets)?)
        } else {
            None
        };

        // Standard error of measurement
        let cronbach_alpha = Self::calculate_cronbach_alpha(datasets)?;
        let total_scores: Vec<f64> = (0..n_observations)
            .map(|i| datasets.iter().map(|item| item[i]).sum::<f64>())
            .collect();

        let total_sd = total_scores.std_dev();
        let standard_error_of_measurement = total_sd * (1.0 - cronbach_alpha).sqrt();

        Ok(ScaleReliability {
            average_interitem_correlation,
            split_half_reliability,
            standard_error_of_measurement,
        })
    }

    /// Calculate split-half reliability using Spearman-Brown prophecy formula
    fn calculate_split_half_reliability(datasets: &[Vec<f64>]) -> Result<f64, String> {
        let n_items = datasets.len();
        if n_items < 4 {
            return Err("Split-half reliability requires at least 4 items".to_string());
        }

        // Split items into two halves (odd vs even indices)
        let odd_items: Vec<&Vec<f64>> = datasets.iter().enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, item)| item)
            .collect();

        let even_items: Vec<&Vec<f64>> = datasets.iter().enumerate()
            .filter(|(i, _)| i % 2 == 1)
            .map(|(_, item)| item)
            .collect();

        // Calculate sum scores for each half
        let odd_scores: Vec<f64> = (0..datasets[0].len())
            .map(|i| odd_items.iter().map(|item| item[i]).sum::<f64>())
            .collect();

        let even_scores: Vec<f64> = (0..datasets[0].len())
            .map(|i| even_items.iter().map(|item| item[i]).sum::<f64>())
            .collect();

        // Correlation between halves
        let half_correlation = CorrelationMethods::pearson_correlation(&odd_scores, &even_scores, None, None).map(|(r, _)| r)?;

        // Spearman-Brown prophecy formula
        let split_half = (2.0 * half_correlation) / (1.0 + half_correlation);

        Ok(split_half.clamp(0.0, 1.0))
    }

    /// Interpret reliability coefficients
    fn interpret_reliability(alpha: f64, omega: f64) -> String {
        let primary_measure = if omega > 0.0 { omega } else { alpha };

        if primary_measure >= 0.9 {
            "Excellent reliability".to_string()
        } else if primary_measure >= 0.8 {
            "Good reliability".to_string()
        } else if primary_measure >= 0.7 {
            "Acceptable reliability".to_string()
        } else if primary_measure >= 0.6 {
            "Questionable reliability".to_string()
        } else if primary_measure >= 0.5 {
            "Poor reliability".to_string()
        } else {
            "Unacceptable reliability".to_string()
        }
    }

    /// Calculate confidence interval for Cronbach's alpha
    pub fn cronbach_alpha_confidence_interval(
        alpha: f64,
        n_items: usize,
        n_observations: usize,
        confidence_level: f64,
    ) -> Result<(f64, f64), String> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err("Alpha must be between 0 and 1".to_string());
        }

        // Feldt's confidence interval for Cronbach's alpha
        let n = n_observations as f64;
        let k = n_items as f64;

        // Fisher transformation for alpha
        let fisher_alpha = ((1.0 - alpha) / alpha).ln();

        // Standard error of Fisher transformed alpha
        let se_fisher = ((2.0 * (1.0 - alpha) * (1.0 - alpha)) / ((k - 1.0) * (n - 1.0))).sqrt();

        // Confidence interval in Fisher space
        let z = distribution_functions::normal_quantile(1.0 - (1.0 - confidence_level) / 2.0);
        let lower_fisher = fisher_alpha - z * se_fisher;
        let upper_fisher = fisher_alpha + z * se_fisher;

        // Transform back to alpha scale
        let lower_alpha: f64 = 1.0 / (1.0 + lower_fisher.exp());
        let upper_alpha: f64 = 1.0 / (1.0 + upper_fisher.exp());

        Ok((lower_alpha.clamp(0.0, 1.0), upper_alpha.clamp(0.0, 1.0)))
    }
}
