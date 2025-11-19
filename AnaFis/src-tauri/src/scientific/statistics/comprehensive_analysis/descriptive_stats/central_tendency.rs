//! Central tendency measures
//!
//! This module provides calculations for measures of central tendency:
//! mean, median, and mode.

use super::kde::SimpleKDE;
use super::quantiles::Quantiles;

/// Central tendency calculations
pub struct CentralTendency;

impl CentralTendency {
    /// Compute median
    pub fn median(data: &[f64]) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        Quantiles::median(&sorted)
    }

    /// Compute modes (values that appear most frequently)
    /// For continuous data, uses kernel density estimation to find peaks
    /// For discrete data, uses binning approach
    pub fn modes(data: &[f64]) -> Vec<f64> {
        if data.len() < 2 {
            return Vec::new();
        }

        // Check if data appears discrete (few unique values relative to sample size)
        let unique_values: std::collections::HashSet<u64> = data.iter()
            .map(|&x| x.to_bits())
            .collect();

        let is_discrete = (unique_values.len() as f64) / (data.len() as f64) < 0.1;

        if is_discrete {
            Self::modes_discrete(data, 1e-10)
        } else {
            Self::modes_continuous(data)
        }
    }

    /// Compute modes for discrete/categorical data using binning
    fn modes_discrete(data: &[f64], tolerance: f64) -> Vec<f64> {
        let mut frequency: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();

        // Round to tolerance for binning
        for &value in data {
            if !value.is_finite() { continue; }
            let binned = (value / tolerance).round() as i64;
            *frequency.entry(binned).or_insert(0) += 1;
        }

        let max_freq = frequency.values().max().copied().unwrap_or(0);
        if max_freq <= 1 {
            return Vec::new();
        }

        frequency.iter()
            .filter(|(_, &freq)| freq == max_freq)
            .map(|(&key, _)| key as f64 * tolerance)
            .collect()
    }

    /// Compute modes for continuous data using kernel density estimation
    fn modes_continuous(data: &[f64]) -> Vec<f64> {
        if data.len() < 3 {
            return Vec::new();
        }

        // Simple Gaussian KDE implementation
        let kde = SimpleKDE::new(data);

        // Evaluate KDE on a grid to find local maxima
        let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        if !range.is_finite() || range <= 0.0 {
            return Vec::new();
        }

        let n_grid = 200;
        let grid_spacing = range / (n_grid - 1) as f64;

        let mut densities = Vec::with_capacity(n_grid);
        let mut grid_points = Vec::with_capacity(n_grid);

        for i in 0..n_grid {
            let x = min_val + i as f64 * grid_spacing;
            let density = kde.evaluate(x);
            densities.push(density);
            grid_points.push(x);
        }

        // Find local maxima
        let mut modes = Vec::new();
        for i in 1..(n_grid - 1) {
            let prev_density = densities[i - 1];
            let curr_density = densities[i];
            let next_density = densities[i + 1];

            // Local maximum: density increases then decreases
            if curr_density > prev_density && curr_density > next_density {
                modes.push(grid_points[i]);
            }
        }

        // If no local maxima found, return empty (unimodal or uniform)
        modes
    }
}