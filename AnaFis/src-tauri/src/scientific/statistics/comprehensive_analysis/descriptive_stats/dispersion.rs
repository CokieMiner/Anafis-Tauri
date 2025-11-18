//! Measures of dispersion
//!
//! This module provides calculations for measures of data spread:
//! variance, standard deviation, range, interquartile range, MAD.

use super::quantiles::Quantiles;

/// Dispersion measures
pub struct Dispersion;

impl Dispersion {
    /// Compute variance
    pub fn variance(data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let sum_sq_diff: f64 = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum();

        sum_sq_diff / (data.len() - 1) as f64
    }

    /// Compute standard deviation
    pub fn std_dev(data: &[f64]) -> f64 {
        Self::variance(data).sqrt()
    }

    /// Compute range (max - min)
    pub fn range(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if min.is_infinite() || max.is_infinite() {
            0.0
        } else {
            max - min
        }
    }

    /// Compute minimum value
    pub fn min(data: &[f64]) -> f64 {
        data.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// Compute maximum value
    pub fn max(data: &[f64]) -> f64 {
        data.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    /// Compute interquartile range
    pub fn iqr(data: &[f64]) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        Quantiles::iqr(&sorted)
    }

    /// Compute median absolute deviation
    pub fn median_absolute_deviation(data: &[f64], median: f64) -> f64 {
        let mut deviations: Vec<f64> = data.iter()
            .map(|x| (x - median).abs())
            .collect();

        deviations.sort_by(|a, b| a.total_cmp(b));

        Quantiles::quantile(&deviations, 0.50, super::quantiles::QuantileMethod::Type8)
    }

    /// Compute quartiles
    pub fn quartiles(data: &[f64]) -> (f64, f64) {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        Quantiles::quartiles(&sorted)
    }
}