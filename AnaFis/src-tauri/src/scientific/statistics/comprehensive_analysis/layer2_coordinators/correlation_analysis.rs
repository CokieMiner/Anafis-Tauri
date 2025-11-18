use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::{
    CorrelationEngine, CorrelationTestResult, HypothesisTestingEngine,
};
use crate::scientific::statistics::types::AnalysisOptions;
use ndarray::Array2;
use rand_pcg::Pcg64;

#[derive(Debug, Clone)]
pub struct CorrelationAnalysis {
    pub correlation_matrix: Array2<f64>,
    pub correlation_tests: Vec<CorrelationTestResult>,
    pub strongest_correlations: Vec<StrongestCorrelation>,
    pub method: String,
}

#[derive(Debug, Clone)]
pub struct StrongestCorrelation {
    pub variable_1: usize,
    pub variable_2: usize,
    pub correlation: f64,
    pub p_value: Option<f64>,
    pub significant: Option<bool>,
    pub strength: String,
}

/// Correlation Analysis Coordinator
/// Coordinates correlation analysis between multiple datasets
pub struct CorrelationAnalysisCoordinator;

impl CorrelationAnalysisCoordinator {
    /// Analyze correlations between multiple datasets
    pub fn analyze(datasets: &[Vec<f64>], options: &AnalysisOptions, rng: &mut Pcg64) -> Result<CorrelationAnalysis, String> {
        if datasets.len() < 2 {
            return Err("Need at least 2 datasets for correlation analysis".to_string());
        }

        // Validate dataset lengths
        let first_len = datasets[0].len();
        for (i, dataset) in datasets.iter().enumerate() {
            if dataset.len() != first_len {
                return Err(format!("Dataset {} has different length ({}) than first dataset ({})",
                    i, dataset.len(), first_len));
            }
        }

        // Compute correlation matrix using requested method
        let method_str = options.correlation_method.as_deref().unwrap_or("pearson");
        let correlation_matrix = CorrelationEngine::compute_matrix_with_method(datasets, method_str, options.biweight_tuning_constant.unwrap_or(9.0))?;

        // Compute correlation tests for all pairs
        let alpha = options.statistical_confidence_level.map(|c| 1.0 - c).or(Some(0.05));
        let n_permutations = options.n_permutations.unwrap_or(5000);
        let mut correlation_tests = Vec::new();
        for i in 0..datasets.len() {
            for j in (i + 1)..datasets.len() {
                let tests = HypothesisTestingEngine::correlation_tests(&datasets[i], &datasets[j], i, j, alpha, n_permutations, rng)?;
                correlation_tests.extend(tests);
            }
        }

        // Identify strongest correlations
        let mut correlations_with_indices: Vec<(usize, usize, f64)> = Vec::new();
        for i in 0..datasets.len() {
            for j in (i + 1)..datasets.len() {
                let corr = correlation_matrix[[i, j]];
                correlations_with_indices.push((i, j, corr));
            }
        }

        correlations_with_indices.sort_by(|a, b| b.2.abs().total_cmp(&a.2.abs()));

        let strongest_correlations = correlations_with_indices.into_iter()
            .take(5) // Top 5 correlations
            .map(|(i, j, corr)| {
                // Find associated test for the chosen method if available
                let matching_test = correlation_tests.iter().find(|t| {
                    ((t.variable_1 == i && t.variable_2 == j) || (t.variable_1 == j && t.variable_2 == i))
                    && t.method.to_lowercase() == method_str.to_lowercase()
                });
                let (p_value, significant) = matching_test.map(|t| (Some(t.p_value), Some(t.significant))).unwrap_or((None, None));
                StrongestCorrelation {
                variable_1: i,
                variable_2: j,
                correlation: corr,
                p_value,
                significant,
                strength: Self::correlation_strength(corr),
            }})
            .collect();

        Ok(CorrelationAnalysis {
            correlation_matrix,
            correlation_tests,
            strongest_correlations,
            method: method_str.to_string(),
        })
    }

    /// Classify correlation strength
    fn correlation_strength(correlation: f64) -> String {
        let abs_corr = correlation.abs();
        if abs_corr >= 0.9 {
            "Very Strong".to_string()
        } else if abs_corr >= 0.7 {
            "Strong".to_string()
        } else if abs_corr >= 0.5 {
            "Moderate".to_string()
        } else if abs_corr >= 0.3 {
            "Weak".to_string()
        } else {
            "Very Weak".to_string()
        }
    }
}
