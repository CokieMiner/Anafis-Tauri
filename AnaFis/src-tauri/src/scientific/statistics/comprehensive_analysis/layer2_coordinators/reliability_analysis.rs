use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::correlation_methods::CorrelationMethods;
use crate::scientific::statistics::comprehensive_analysis::utils;
use crate::scientific::statistics::types::{ReliabilityAnalysisResult, ScaleReliability};
use ndarray::Array2;

/// Reliability Analysis Coordinator
/// Coordinates scale reliability and consistency analysis
pub struct ReliabilityAnalysisCoordinator;

impl ReliabilityAnalysisCoordinator {
    /// Analyze reliability of multiple variables (e.g., psychometric scales)
    pub fn analyze(datasets: &[Vec<f64>]) -> Result<ReliabilityAnalysisResult, String> {
        if datasets.len() < 2 {
            return Err("Need at least 2 variables for reliability analysis".to_string());
        }

        // Validate data
        utils::validate_variable_lengths(datasets)?;

        // Cronbach's alpha
        let cronbach_alpha = Self::compute_cronbach_alpha(datasets)?;

        // Item-total correlations
        let item_total_correlations = Self::compute_item_total_correlations(datasets)?;

        // Scale reliability measures
        let scale_reliability = Self::compute_scale_reliability(datasets)?;

        Ok(ReliabilityAnalysisResult {
            cronbach_alpha,
            item_total_correlations,
            scale_reliability,
        })
    }

    /// Compute Cronbach's alpha
    fn compute_cronbach_alpha(datasets: &[Vec<f64>]) -> Result<f64, String> {
        let k = datasets.len() as f64;
        let _n = datasets[0].len() as f64;

        // Compute variance of each item
        let item_variances: Vec<f64> = datasets.iter()
            .map(|item| crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats::variance(item))
            .collect();

        // Compute variance of total scores
        let total_scores: Vec<f64> = (0..datasets[0].len())
            .map(|i| datasets.iter().map(|item| item[i]).sum::<f64>())
            .collect();

        let total_variance = crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats::variance(&total_scores);

        // Cronbach's alpha formula
        let sum_item_variances: f64 = item_variances.iter().sum();
        let alpha = (k / (k - 1.0)) * (1.0 - sum_item_variances / total_variance);

        Ok(alpha.clamp(0.0, 1.0)) // Clamp to [0,1]
    }

    /// Compute item-total correlations
    fn compute_item_total_correlations(datasets: &[Vec<f64>]) -> Result<Vec<f64>, String> {
        let mut correlations = Vec::new();

        for i in 0..datasets.len() {
            // Total score without this item
            let total_without_item: Vec<f64> = (0..datasets[0].len())
                .map(|j| {
                    datasets.iter().enumerate()
                        .filter(|(idx, _)| *idx != i)
                        .map(|(_, item)| item[j])
                        .sum::<f64>()
                })
                .collect();

            let correlation = CorrelationMethods::pearson_correlation(&datasets[i], &total_without_item)?;
            correlations.push(correlation);
        }

        Ok(correlations)
    }

    /// Compute McDonald's Omega using Principal Component Analysis for factor loadings.
    /// This is a common method for unidimensional models.
    fn compute_mcdonalds_omega(datasets: &[Vec<f64>]) -> Result<f64, String> {
        let num_items = datasets.len();
        if num_items < 2 {
            // Omega is not well-defined for a single item.
            // Returning 1.0 might be misleading. An error or specific handling is better.
            return Err("McDonald's Omega requires at least 2 items.".to_string());
        }

        // 1. Create a correlation matrix
        let mut corr_matrix = Array2::<f64>::zeros((num_items, num_items));
        for i in 0..num_items {
            for j in i..num_items {
                if i == j {
                    corr_matrix[[i, j]] = 1.0;
                } else {
                    let corr = CorrelationMethods::pearson_correlation(&datasets[i], &datasets[j])?;
                    corr_matrix[[i, j]] = corr;
                    corr_matrix[[j, i]] = corr; // Symmetric matrix
                }
            }
        }

        // 2. Perform eigendecomposition (PCA on correlation matrix)
        let (eigenvalues, eigenvectors) = crate::scientific::statistics::comprehensive_analysis::layer4_primitives::LinearAlgebra::eigenvalue_decomposition(&corr_matrix)?;
        
        // Find the index of the largest eigenvalue
        let (max_eigenvalue_index, _) = eigenvalues.iter().enumerate()
            .fold((0, eigenvalues[0]), |(idx_max, val_max), (idx, &val)| {
                if val > val_max { (idx, val) } else { (idx_max, val_max) }
            });

        let max_eigenvalue = eigenvalues[max_eigenvalue_index];
        let principal_component = eigenvectors.column(max_eigenvalue_index);

        // 3. Calculate factor loadings from the first principal component
        let loadings = principal_component.mapv(|x| x * max_eigenvalue.sqrt());

        // 4. Calculate Omega
        let sum_loadings: f64 = loadings.iter().map(|&l| l.abs()).sum();
        let sum_squared_loadings: f64 = loadings.iter().map(|&l| l * l).sum();
        
        let numerator = sum_loadings.powi(2);
        // Communality for an item is its squared loading on the factor.
        // Uniqueness is 1 - communality.
        // Sum of uniquenesses = num_items - sum_of_communalities (sum_squared_loadings)
        let denominator = numerator + (num_items as f64 - sum_squared_loadings);

        if denominator == 0.0 {
            return Err("Cannot compute Omega: denominator is zero. This may happen if all items are perfectly correlated.".to_string());
        }

        let omega = numerator / denominator;

        Ok(omega.clamp(0.0, 1.0))
    }


    /// Compute scale reliability measures
    fn compute_scale_reliability(datasets: &[Vec<f64>]) -> Result<ScaleReliability, String> {
        // McDonald's omega
        let omega = Self::compute_mcdonalds_omega(datasets)?;

        // Average inter-item correlation
        let mut inter_item_correlations = Vec::new();
        for i in 0..datasets.len() {
            for j in (i + 1)..datasets.len() {
                let corr = CorrelationMethods::pearson_correlation(&datasets[i], &datasets[j])?;
                inter_item_correlations.push(corr);
            }
        }

        let average_interitem_corr = if inter_item_correlations.is_empty() {
            0.0
        } else {
            inter_item_correlations.iter().sum::<f64>() / inter_item_correlations.len() as f64
        };

        Ok(ScaleReliability {
            omega,
            average_interitem_corr,
        })
    }
}
