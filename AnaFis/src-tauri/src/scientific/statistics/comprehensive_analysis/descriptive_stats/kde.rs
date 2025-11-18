//! Kernel Density Estimation
//!
//! This module provides kernel density estimation functionality
//! for continuous data analysis.

/// Simple Gaussian Kernel Density Estimator
pub struct SimpleKDE {
    data: Vec<f64>,
    bandwidth: f64,
}

impl SimpleKDE {
    /// Create a new KDE from data using Silverman's rule of thumb for bandwidth
    pub fn new(data: &[f64]) -> Self {
        let valid_data: Vec<f64> = data.iter()
            .filter(|x| x.is_finite())
            .cloned()
            .collect();

        // Use Silverman's rule of thumb for bandwidth
        let n = valid_data.len() as f64;
        let std_dev = if n > 1.0 {
            let mean = valid_data.iter().sum::<f64>() / n;
            let variance = valid_data.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / (n - 1.0);
            variance.sqrt()
        } else {
            1.0
        };

        let bandwidth = 0.9 * std_dev * n.powf(-0.2);

        SimpleKDE {
            data: valid_data,
            bandwidth: bandwidth.max(1e-10), // Avoid zero bandwidth
        }
    }

    /// Evaluate the KDE at a given point
    pub fn evaluate(&self, x: f64) -> f64 {
        let n = self.data.len() as f64;
        let sum: f64 = self.data.iter()
            .map(|&xi| {
                let diff = (x - xi) / self.bandwidth;
                (-0.5 * diff * diff).exp() / (self.bandwidth * std::f64::consts::PI.sqrt())
            })
            .sum();

        sum / n
    }

    /// Get the bandwidth used by this KDE
    pub fn bandwidth(&self) -> f64 {
        self.bandwidth
    }

    /// Get the data points used by this KDE
    pub fn data(&self) -> &[f64] {
        &self.data
    }
}