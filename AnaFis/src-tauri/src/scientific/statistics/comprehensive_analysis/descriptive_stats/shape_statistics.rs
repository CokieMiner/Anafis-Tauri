//! Shape statistics
//!
//! This module provides calculations for shape-related statistics:
//! skewness, kurtosis, and coefficient of variation.

/// Shape statistics calculations
pub struct ShapeStatistics;

impl ShapeStatistics {
    /// Compute coefficient of variation with proper error handling
    pub fn coefficient_of_variation(mean: f64, std_dev: f64) -> f64 {
        if mean.abs() < 1e-10 {
            // Mean is effectively zero - CV is undefined
            f64::NAN
        } else if mean < 0.0 {
            // CV is typically undefined for negative means
            f64::NAN
        } else {
            std_dev / mean
        }
    }

    /// Compute robust coefficient of variation using median and MAD
    pub fn robust_coefficient_of_variation(data: &[f64]) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));

        let median = super::quantiles::Quantiles::median(&sorted);

        if median.abs() < 1e-10 {
            return f64::NAN;
        }

        if median < 0.0 {
            return f64::NAN;
        }

        // Use MAD as robust scale estimator
        let mad = super::dispersion::Dispersion::median_absolute_deviation(data, median);

        // Convert MAD to SD equivalent for normal distribution (MAD ≈ 0.6745 * SD)
        let robust_sd = mad / 0.6745;

        robust_sd / median
    }

    /// Compute skewness (third standardized moment)
    pub fn skewness(data: &[f64]) -> f64 {
        if data.len() < 3 {
            return 0.0;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = super::dispersion::Dispersion::variance(data);
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return 0.0; // No variation, skewness undefined
        }

        let n = data.len() as f64;
        let skewness_sum: f64 = data.iter()
            .map(|x| ((x - mean) / std_dev).powi(3))
            .sum();

        // Use unbiased estimator: divide by n-1 for variance, but n for skewness
        skewness_sum / n
    }

    /// Compute kurtosis (fourth standardized moment)
    pub fn kurtosis(data: &[f64]) -> f64 {
        if data.len() < 4 {
            return 0.0;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = super::dispersion::Dispersion::variance(data);
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return 0.0; // No variation, kurtosis undefined
        }

        let n = data.len() as f64;
        let kurtosis_sum: f64 = data.iter()
            .map(|x| ((x - mean) / std_dev).powi(4))
            .sum();

        // Excess kurtosis (subtract 3 for normal distribution)
        kurtosis_sum / n - 3.0
    }

    /// Compute all moments (mean, variance, skewness, kurtosis)
    pub fn moments(data: &[f64]) -> (f64, f64, f64, f64) {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = super::dispersion::Dispersion::variance(data);
        let skewness = Self::skewness(data);
        let kurtosis = Self::kurtosis(data);

        (mean, variance, skewness, kurtosis)
    }
}