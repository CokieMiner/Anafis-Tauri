//! Numerical integration methods
//!
//! This module provides numerical integration algorithms
//! including adaptive quadrature and singularity handling.

use integrate::adaptive_quadrature::adaptive_simpson_method;

/// Numerical integration using adaptive quadrature
pub struct NumericalIntegration;

impl NumericalIntegration {
    /// Adaptive numerical integration using the integrate crate's adaptive Simpson method
    pub fn adaptive_quadrature<F>(f: F, a: f64, b: f64, tol: f64) -> Result<f64, String>
    where
        F: Fn(f64) -> f64 + Send + Sync + Copy + 'static,
    {
        // Use the integrate crate's adaptive_simpson_method.
        // We map signature to that of integrate::adaptive_quadrature::adaptive_simpson_method,
        // which expects min_h and tolerance. We'll set a minimal subinterval size relative to the interval.
        let span = (b - a).abs();
        if span == 0.0 || !span.is_finite() {
            return Err("Invalid integration interval".to_string());
        }

        // min_h is minimum subinterval length to try; pick a sensible small fraction of the interval.
        let min_h = (span * 1e-6).max(1e-12);

        match adaptive_simpson_method(f, a, b, min_h, tol) {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("Integration error: {:?}", err)),
        }
    }

    /// Numerical integration with singularity handling
    pub fn integrate_singular<F>(f: F, a: f64, b: f64, tol: f64) -> Result<f64, String>
    where
        F: Fn(f64) -> f64 + Send + Sync + Copy + 'static,
    {
        // Use transformation for singularities
        let transformed_f = move |t: f64| {
            let x = a + (b - a) * t * t; // t² transformation for endpoint singularities
            f(x) * 2.0 * t * (b - a)
        };

        Self::adaptive_quadrature(transformed_f, 0.0, 1.0, tol)
    }
}