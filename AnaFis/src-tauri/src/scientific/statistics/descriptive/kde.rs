//! Simple Kernel Density Estimation
//!
//! Provides a basic implementation of Gaussian Kernel Density Estimation (KDE),
//! primarily for mode estimation in continuous data.

/// A simple Gaussian Kernel Density Estimator.
pub(super) struct SimpleKDE<'a> {
    data: &'a [f64],
    bandwidth: f64,
}

impl<'a> SimpleKDE<'a> {
    /// Create a new KDE instance with a specified bandwidth.
    /// If bandwidth is not provided, it's estimated using Silverman's rule of thumb.
    pub fn new(data: &'a [f64]) -> Self {
        let bandwidth = Self::silvermans_rule(data);
        Self { data, bandwidth }
    }

    /// Evaluate the KDE at a given point `x`.
    pub fn evaluate(&self, x: f64) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }

        let n = self.data.len() as f64;
        let h = self.bandwidth;

        let sum: f64 = self.data.iter()
            .map(|&xi| {
                let z = (x - xi) / h;
                // Standard normal PDF
                (-0.5 * z * z).exp() / (2.0 * std::f64::consts::PI).sqrt()
            })
            .sum();

        sum / (n * h)
    }

    /// Estimate bandwidth using Silverman's rule of thumb.
    /// This rule is optimal for Gaussian-like data but serves as a good default.
    fn silvermans_rule(data: &[f64]) -> f64 {
        use super::moments::StatisticalMoments;
        
        if data.len() < 2 {
            return 1.0;
        }
        let n = data.len() as f64;

        // Calculate standard deviation using centralized function
        let std_dev = data.std_dev();
        if std_dev == 0.0 {
            return 1.0;
        }

        // Calculate IQR using centralized Quantiles module
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        
        let (q1, q3) = super::Quantiles::quartiles(&sorted);
        let iqr = q3 - q1;

        // Silverman's factor uses the minimum of std_dev and IQR/1.34
        let robust_std = std_dev.min(iqr / 1.349);

        // Silverman's rule of thumb formula
        1.06 * robust_std * n.powf(-0.2)
    }
}
