//! Correlation computation engine facade

use ndarray::Array2;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::{
    correlation_methods::CorrelationMethods,
    correlation_matrix::CorrelationMatrix,
};

/// Correlation computation engine - facade for correlation operations
pub struct CorrelationEngine;

impl CorrelationEngine {
    // Delegate correlation methods
    pub fn pearson_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        CorrelationMethods::pearson_correlation(x, y)
    }

    pub fn spearman_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        CorrelationMethods::spearman_correlation(x, y)
    }

    pub fn kendall_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        CorrelationMethods::kendall_correlation(x, y)
    }

    pub fn biweight_midcorrelation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        CorrelationMethods::biweight_midcorrelation(x, y)
    }

    pub fn biweight_midcorrelation_tuned(x: &[f64], y: &[f64], tuning_constant: f64) -> Result<f64, String> {
        CorrelationMethods::biweight_midcorrelation_tuned(x, y, tuning_constant)
    }

    // Delegate matrix operations
    pub fn correlation_matrix(data: &[Vec<f64>]) -> Result<Array2<f64>, String> {
        CorrelationMatrix::correlation_matrix(data)
    }

    pub fn compute_matrix_with_method(datasets: &[Vec<f64>], method: &str, biweight_tuning: f64) -> Result<Array2<f64>, String> {
        CorrelationMatrix::compute_matrix_with_method(datasets, method, biweight_tuning)
    }

    pub fn compute_matrix_with_method_unchecked(datasets: &[Vec<f64>], method: &str, biweight_tuning: f64) -> Result<Array2<f64>, String> {
        CorrelationMatrix::compute_matrix_with_method_unchecked(datasets, method, biweight_tuning)
    }

    pub fn partial_correlations(data: &[Vec<f64>]) -> Result<Array2<f64>, String> {
        CorrelationMatrix::partial_correlations(data)
    }

    pub fn distance_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        CorrelationMatrix::distance_correlation(x, y)
    }

    pub fn autocorrelation(data: &[f64], max_lag: usize) -> Result<Vec<f64>, String> {
        CorrelationMatrix::autocorrelation(data, max_lag)
    }

    pub fn cross_correlation(x: &[f64], y: &[f64], max_lag: usize) -> Result<Vec<f64>, String> {
        CorrelationMatrix::cross_correlation(x, y, max_lag)
    }

    pub fn correlation_matrix_with_significance(
        data: &[Vec<f64>],
        alpha: f64,
    ) -> Result<(Array2<f64>, Array2<bool>), String> {
        CorrelationMatrix::correlation_matrix_with_significance(data, alpha)
    }
}