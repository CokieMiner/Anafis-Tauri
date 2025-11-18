//! Interpolation methods
//!
//! This module provides various interpolation techniques
//! including linear interpolation and cubic splines.

use splines::{Interpolation as SplineInterpolation, Key, Spline};

/// Interpolation methods
pub struct Interpolation;

impl Interpolation {
    /// Linear interpolation
    pub fn linear_interpolate(x: f64, x_values: &[f64], y_values: &[f64]) -> f64 {
        if x_values.len() != y_values.len() || x_values.is_empty() {
            return f64::NAN;
        }

        if x <= x_values[0] {
            return y_values[0];
        }
        if x >= x_values[x_values.len() - 1] {
            return y_values[y_values.len() - 1];
        }

        // Find the interval containing x
        for i in 0..(x_values.len() - 1) {
            if x >= x_values[i] && x <= x_values[i + 1] {
                let t = (x - x_values[i]) / (x_values[i + 1] - x_values[i]);
                return y_values[i] + t * (y_values[i + 1] - y_values[i]);
            }
        }

        f64::NAN
    }

    /// Cubic spline interpolation using the splines library for better accuracy
    pub fn cubic_spline_interpolate(x: f64, x_values: &[f64], y_values: &[f64]) -> f64 {
        if x_values.len() != y_values.len() || x_values.len() < 2 {
            return f64::NAN;
        }

        // For small datasets, use linear interpolation
        if x_values.len() < 4 {
            return Self::linear_interpolate(x, x_values, y_values);
        }

        // Create keys from the data points using CatmullRom for cubic interpolation
        let keys: Vec<Key<f64, f64>> = x_values.iter()
            .zip(y_values.iter())
            .map(|(&x, &y)| Key::new(x, y, SplineInterpolation::CatmullRom))
            .collect();

        // Create the spline
        let spline = Spline::from_vec(keys);

        // Sample the spline, falling back to linear if out of bounds
        match spline.sample(x) {
            Some(value) => value,
            None => Self::linear_interpolate(x, x_values, y_values),
        }
    }
}