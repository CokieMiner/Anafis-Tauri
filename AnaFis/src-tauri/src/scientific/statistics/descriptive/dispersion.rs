//! Measures of dispersion
//!
//! This module provides calculations for measures of data spread:
//! range, interquartile range, and median absolute deviation.
//! Variance and standard deviation are provided via the `StatisticalMoments` trait.


use crate::scientific::statistics::descriptive::uncertainty::DescriptiveUncertainty;

/// Dispersion measures
pub struct Dispersion;

impl Dispersion {
    /// Compute range (max - min).
    pub fn range(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if min.is_finite() && max.is_finite() {
            max - min
        } else {
            f64::INFINITY
        }
    }

    /// Compute the minimum value in the dataset.
    /// Returns `f64::INFINITY` for an empty dataset.
    pub fn min(data: &[f64]) -> f64 {
        data.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// Compute the maximum value in the dataset.
    /// Returns `f64::NEG_INFINITY` for an empty dataset.
    pub fn max(data: &[f64]) -> f64 {
        data.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    /// Compute interquartile range (IQR).
    /// This function will sort the data.
    /// Compute interquartile range (IQR).
    /// If errors are provided, computes uncertainty using Monte Carlo simulation.
    pub fn iqr(data: &[f64], errors: Option<&[f64]>) -> (f64, f64) {
        if let Some(errs) = errors {
            let (q1, q1_err) = DescriptiveUncertainty::quantile_uncertainty(data, errs, 0.25, 1000).unwrap_or((f64::NAN, f64::NAN));
            let (q3, q3_err) = DescriptiveUncertainty::quantile_uncertainty(data, errs, 0.75, 1000).unwrap_or((f64::NAN, f64::NAN));
            // Error propagation for subtraction: sqrt(err1^2 + err2^2)
            let iqr_err = (q1_err.powi(2) + q3_err.powi(2)).sqrt();
            return (q3 - q1, iqr_err);
        }

        // Use centralized Quantiles implementation
        // We sort here because Quantiles::iqr expects sorted data
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        (super::Quantiles::iqr(&sorted), 0.0)
    }

    /// Compute standard deviation.
    /// If errors are provided, computes uncertainty using Monte Carlo simulation.
    pub fn std_dev(data: &[f64], errors: Option<&[f64]>) -> (f64, f64) {
        if let Some(errs) = errors {
            return DescriptiveUncertainty::std_dev_uncertainty(data, errs, 1000)
                .unwrap_or((f64::NAN, f64::NAN));
        }

        use super::moments::StatisticalMoments;
        (data.std_dev(), 0.0)
    }

    /// Compute Median Absolute Deviation (MAD).
    /// A robust measure of spread.
    pub fn median_absolute_deviation(data: &[f64], median: f64) -> f64 {
        let deviations: Vec<f64> = data.iter()
            .map(|x| (x - median).abs())
            .collect();

        // The MAD is the median of the absolute deviations
        let mut sorted_deviations = deviations;
        sorted_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        super::Quantiles::median(&sorted_deviations)
    }
}
