//! Quantile and percentile calculations.
//!
//! This module provides robust quantile estimation using various methods,
//! with a focus on the Hyndman & Fan Type 8 algorithm, which is recommended
//! for its properties of being approximately unbiased for the expected value
//! of the order statistics.



/// Enum representing the different quantile estimation methods
/// as defined by Hyndman & Fan (1996).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QuantileMethod {
    /// Inverse of empirical CDF.
    Type1,
    /// Like Type 1, but with averaging at discontinuities.
    Type2,
    /// Nearest order statistic.
    Type3,
    /// Linear interpolation of the empirical CDF.
    Type4,
    /// Linear interpolation of the expectations for the order statistics for the uniform distribution on [0, 1].
    Type5,
    /// Like Type 5, but for the uniform distribution on [0, 1].
    Type6,
    /// Like Type 1, but with linear interpolation between order statistics.
    Type7,
    /// Recommended method: Linear interpolation of the approximate medians for order statistics.
    Type8,
    /// Like Type 8, but for the uniform distribution on [0, 1].
    Type9,
}

/// Quantile calculations. Assumes data is pre-sorted.
pub struct Quantiles;

impl Quantiles {
    /// Generic quantile function. Data must be sorted.
    pub fn quantile(sorted_data: &[f64], p: f64, method: QuantileMethod) -> Result<f64, String> {
        let n = sorted_data.len();
        if n == 0 {
            return Ok(f64::NAN);
        }
        if !(0.0..=1.0).contains(&p) {
            return Err(format!("Quantile probability must be between 0 and 1, got {}", p));
        }

        let real_idx = Self::get_real_index(p, n as f64, method);
        let k = real_idx.floor() as usize;
        let h = real_idx - k as f64;

        if h == 0.0 {
            // If index is integer, return the corresponding value
            let index = if k == 0 { 0 } else { k - 1 };
            return Ok(sorted_data[index]);
        }

        // Linear interpolation
        let idx1 = if k == 0 { 0 } else { k - 1 };
        let idx2 = k.min(n - 1);
        
        Ok(sorted_data[idx1] + h * (sorted_data[idx2] - sorted_data[idx1]))
    }
    
    /// Get the real-valued index `k + h` based on the method.
    fn get_real_index(p: f64, n: f64, method: QuantileMethod) -> f64 {
        match method {
            QuantileMethod::Type1 => n * p,
            QuantileMethod::Type2 => n * p,
            QuantileMethod::Type3 => n * p - 0.5,
            QuantileMethod::Type4 => n * p,
            QuantileMethod::Type5 => n * p + 0.5,
            QuantileMethod::Type6 => (n + 1.0) * p,
            QuantileMethod::Type7 => (n - 1.0) * p + 1.0,
            QuantileMethod::Type8 => (n + 1.0 / 3.0) * p + 1.0 / 3.0,
            QuantileMethod::Type9 => (n + 1.0 / 4.0) * p + 3.0 / 8.0,
        }
    }

    /// Calculate median (50th percentile).
    /// Assumes data is sorted.
    pub fn median(sorted_data: &[f64]) -> f64 {
        let n = sorted_data.len();
        if n == 0 {
            return f64::NAN;
        }
        if n.is_multiple_of(2) {
            (sorted_data[n / 2 - 1] + sorted_data[n / 2]) / 2.0
        } else {
            sorted_data[n / 2]
        }
    }

    /// Calculate median by partially sorting the data.
    /// This is more efficient than full sort if only median is needed.
    /// WARNING: This modifies the input slice!
    pub fn median_mut(data: &mut [f64]) -> f64 {
        let n = data.len();
        if n == 0 {
            return f64::NAN;
        }
        
        let mid = n / 2;
        
        if n % 2 == 1 {
            // Odd length: select the middle element
            let (_, median, _) = data.select_nth_unstable_by(mid, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            *median
        } else {
            // Even length: need average of two middle elements
            // First select the upper middle element
            let (_, upper, _) = data.select_nth_unstable_by(mid, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let upper_val = *upper;
            
            // Then find the max of the lower half (which is now partitioned to the left of mid)
            let lower_val = data[..mid].iter()
                .fold(f64::NEG_INFINITY, |max, &x| if x > max { x } else { max });
                
            (lower_val + upper_val) / 2.0
        }
    }

    /// Calculate quartiles (25th and 75th percentiles).
    pub fn quartiles(sorted_data: &[f64]) -> (f64, f64) {
        (
            Self::quantile(sorted_data, 0.25, QuantileMethod::Type8).unwrap_or(f64::NAN),
            Self::quantile(sorted_data, 0.75, QuantileMethod::Type8).unwrap_or(f64::NAN),
        )
    }

    /// Calculate the interquartile range (IQR).
    pub fn iqr(sorted_data: &[f64]) -> f64 {
        let (q1, q3) = Self::quartiles(sorted_data);
        q3 - q1
    }

    /// Calculate median, ignoring NaN values.
    /// This function handles sorting internally and minimizes allocations.
    pub fn nan_safe_median(data: &[f64]) -> f64 {
        // We need to copy to filter NaNs and sort, but we can do it efficiently
        let mut valid_values: Vec<f64> = data.iter()
            .filter(|&&x| !x.is_nan())
            .cloned()
            .collect();

        if valid_values.is_empty() {
            return f64::NAN;
        }

        Self::median_mut(&mut valid_values)
    }
}
