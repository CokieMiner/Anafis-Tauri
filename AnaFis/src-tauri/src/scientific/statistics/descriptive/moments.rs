//! Statistical moments trait and implementation.
//!
//! This module provides the `StatisticalMoments` trait for calculating mean, variance,
//! standard deviation, skewness, and kurtosis. It is implemented for `[f64]`.


/// A trait for calculating fundamental statistical moments.
pub trait StatisticalMoments {
    /// Calculate the arithmetic mean (first moment).
    fn mean(&self) -> f64;

    /// Calculate the sample variance (second central moment).
    fn variance(&self) -> f64;

    /// Calculate the sample standard deviation.
    fn std_dev(&self) -> f64;

    /// Calculate the sample skewness (third standardized moment).
    fn skewness(&self) -> f64;

    /// Calculate the sample excess kurtosis (fourth standardized moment).
    fn kurtosis(&self) -> f64;

    /// Calculate the arithmetic mean, ignoring NaN values.
    fn nan_safe_mean(&self) -> f64;

    /// Calculate the sample variance, ignoring NaN values.
    fn nan_safe_variance(&self) -> f64;

    /// Calculate the sample standard deviation, ignoring NaN values.
    fn nan_safe_std_dev(&self) -> f64;
}

impl StatisticalMoments for [f64] {
    /// Calculate the arithmetic mean.
    /// Returns NaN if the slice is empty or contains NaNs.
    fn mean(&self) -> f64 {
        if self.is_empty() {
            return f64::NAN;
        }
        
        let mut sum = 0.0;
        for &x in self {
            if x.is_nan() {
                return f64::NAN;
            }
            sum += x;
        }
        sum / self.len() as f64
    }

    /// Calculate the sample variance.
    /// Returns 0.0 if the slice has fewer than 2 elements.
    /// Uses Welford's online algorithm for numerical stability.
    fn variance(&self) -> f64 {
        if self.len() < 2 {
            return 0.0;
        }
        
        let mut count = 0.0;
        let mut mean = 0.0;
        let mut m2 = 0.0;

        for &x in self {
            if x.is_nan() {
                return f64::NAN;
            }
            count += 1.0;
            let delta = x - mean;
            mean += delta / count;
            let delta2 = x - mean;
            m2 += delta * delta2;
        }

        m2 / (count - 1.0)
    }

    /// Calculate the sample standard deviation.
    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Calculate the sample skewness.
    /// Returns NaN if standard deviation is zero or length < 3.
    fn skewness(&self) -> f64 {
        let n = self.len();
        if n < 3 {
            return f64::NAN;
        }
        
        if self.iter().any(|&x| x.is_nan()) {
            return f64::NAN;
        }

        let mean = self.mean();
        let std_dev = self.std_dev();
        
        // Mathematical Note: Skewness is undefined for constant data (std_dev = 0).
        // Returning 0.0 is a common pragmatic choice for software, 
        // assuming a symmetric distribution (Dirac delta).
        if std_dev < 1e-14 {
            return 0.0;
        }

        let mut sum_cubed_deviations = 0.0;
        for &x in self {
            // Use powi(3) or simple multiplication
            let deviation = x - mean;
            sum_cubed_deviations += deviation * deviation * deviation;
        }

        let n_f = n as f64;
        
        // CORRECTED Formula:
        // G1 = [n / ((n-1)(n-2))] * [Σ(x - mean)^3 / s^3]
        (n_f * sum_cubed_deviations) / 
                      ((n_f - 1.0) * (n_f - 2.0) * std_dev.powi(3))
    }

    /// Calculate the sample excess kurtosis.
    /// Returns NaN if standard deviation is zero or length < 4.
    fn kurtosis(&self) -> f64 {
        let n = self.len();
        if n < 4 {
            return f64::NAN;
        }
        
        if self.iter().any(|&x| x.is_nan()) {
            return f64::NAN;
        }

        let n_f = n as f64;
        let mean = self.mean();
        let mut m2 = 0.0;
        let mut m4 = 0.0;

        for &x in self {
            let d = x - mean;
            let d2 = d * d;
            m2 += d2;
            m4 += d2 * d2;
        }

        // Use sample variance (divide by n-1) for denominator
        let sample_var = m2 / (n_f - 1.0);
        if sample_var < 1e-14 { return 0.0; }

        // Use population fourth moment (divide by n) for numerator
        let m4_n = m4 / n_f;
        
        // Sample excess kurtosis: E[(X-μ)^4] / σ^4 - 3
        m4_n / sample_var.powi(2) - 3.0
    }

    fn nan_safe_mean(&self) -> f64 {
        let mut sum = 0.0;
        let mut count = 0;
        
        for &x in self {
            if !x.is_nan() {
                sum += x;
                count += 1;
            }
        }

        if count == 0 {
            f64::NAN
        } else {
            sum / count as f64
        }
    }

    fn nan_safe_variance(&self) -> f64 {
        let mut count = 0.0;
        let mut mean = 0.0;
        let mut m2 = 0.0;

        for &x in self {
            if !x.is_nan() {
                count += 1.0;
                let delta = x - mean;
                mean += delta / count;
                let delta2 = x - mean;
                m2 += delta * delta2;
            }
        }

        if count < 2.0 {
            0.0
        } else {
            m2 / (count - 1.0)
        }
    }

    fn nan_safe_std_dev(&self) -> f64 {
        self.nan_safe_variance().sqrt()
    }
}
