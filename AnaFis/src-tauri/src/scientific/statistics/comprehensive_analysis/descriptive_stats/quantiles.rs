//! Quantile calculation methods
//!
//! This module provides various quantile calculation methods
//! following the Hyndman & Fan (1996) taxonomy.

/// Quantile calculation methods (Hyndman & Fan, 1996)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuantileMethod {
    /// Type 1: Inverse of empirical CDF
    Type1,
    /// Type 2: Inverse of empirical CDF with averaging
    Type2,
    /// Type 3: Nearest even order statistic
    Type3,
    /// Type 4: Linear interpolation of empirical CDF
    Type4,
    /// Type 5: Piecewise linear (Hazen)
    Type5,
    /// Type 6: Linear interpolation (Weibull)
    Type6,
    /// Type 7: Linear interpolation (Excel, current implementation)
    Type7,
    /// Type 8: Linear interpolation (median unbiased) - RECOMMENDED
    Type8,
    /// Type 9: Approximate median unbiased
    Type9,
}

/// Quantile calculation utilities
pub struct Quantiles;

impl Quantiles {
    /// Compute quantile from sorted data using specified method
    pub fn quantile(sorted_data: &[f64], p: f64, method: QuantileMethod) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }

        if !(0.0..=1.0).contains(&p) {
            return f64::NAN;
        }

        let n = sorted_data.len() as f64;

        // Handle edge cases
        if p <= 0.0 {
            return *sorted_data.first().unwrap();
        }
        if p >= 1.0 {
            return *sorted_data.last().unwrap();
        }

        let (m, j) = match method {
            QuantileMethod::Type1 => {
                let h = n * p;
                (0.0, h.ceil() as usize)
            },
            QuantileMethod::Type2 => {
                let h = n * p + 0.5;
                (0.0, h.ceil() as usize)
            },
            QuantileMethod::Type5 => {
                let h = n * p + 0.5;
                (0.5, h.floor() as usize)
            },
            QuantileMethod::Type6 => {
                let h = (n + 1.0) * p;
                (0.0, h.floor() as usize)
            },
            QuantileMethod::Type7 => {
                let h = (n - 1.0) * p + 1.0;
                (1.0, h.floor() as usize)
            },
            QuantileMethod::Type8 => {
                let h = (n + 1.0/3.0) * p + 1.0/3.0;
                (1.0/3.0, h.floor() as usize)
            },
            QuantileMethod::Type9 => {
                let h = (n + 1.0/4.0) * p + 3.0/8.0;
                (3.0/8.0, h.floor() as usize)
            },
            _ => unimplemented!("Quantile method not implemented"),
        };

        let j = j.min(sorted_data.len() - 1).max(0);
        let g = (n * p + m) - j as f64;

        if g.abs() < 1e-10 {
            sorted_data[j]
        } else {
            let j_next = (j + 1).min(sorted_data.len() - 1);
            sorted_data[j] + g * (sorted_data[j_next] - sorted_data[j])
        }
    }

    /// Legacy quantile method (Type 7) for backward compatibility
    pub fn quantile_legacy(sorted_data: &[f64], p: f64) -> f64 {
        Self::quantile(sorted_data, p, QuantileMethod::Type7)
    }

    /// Compute median (50th percentile) using recommended method
    pub fn median(sorted_data: &[f64]) -> f64 {
        Self::quantile(sorted_data, 0.50, QuantileMethod::Type8)
    }

    /// Compute quartiles (25th, 75th percentiles)
    pub fn quartiles(sorted_data: &[f64]) -> (f64, f64) {
        let q1 = Self::quantile(sorted_data, 0.25, QuantileMethod::Type8);
        let q3 = Self::quantile(sorted_data, 0.75, QuantileMethod::Type8);
        (q1, q3)
    }

    /// Compute interquartile range
    pub fn iqr(sorted_data: &[f64]) -> f64 {
        let (q1, q3) = Self::quartiles(sorted_data);
        q3 - q1
    }
}