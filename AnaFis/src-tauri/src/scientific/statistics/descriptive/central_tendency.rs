//! Central tendency measures
//!
//! This module provides calculations for measures of central tendency:
//! median, and mode. The mean is provided via the `StatisticalMoments` trait.

use super::kde::SimpleKDE;
use crate::scientific::statistics::descriptive::uncertainty::DescriptiveUncertainty;


/// Central tendency calculations
pub struct CentralTendency;

impl CentralTendency {
    /// Compute median.
    /// If errors are provided, computes uncertainty using Monte Carlo simulation.
    pub fn median(data: &[f64], errors: Option<&[f64]>) -> (f64, f64) {
        if let Some(errs) = errors {
            return DescriptiveUncertainty::quantile_uncertainty(data, errs, 0.5, 1000)
                .unwrap_or((f64::NAN, f64::NAN));
        }

        // Use centralized Quantiles implementation
        // We need to clone because median_mut modifies the data
        let mut work_data = data.to_vec();
        (super::Quantiles::median_mut(&mut work_data), 0.0)
    }

    /// Compute mean.
    /// If errors are provided, computes uncertainty using Monte Carlo simulation.
    pub fn mean(data: &[f64], errors: Option<&[f64]>) -> (f64, f64) {
        if let Some(errs) = errors {
            return DescriptiveUncertainty::mean_uncertainty(data, errs, 1000)
                .unwrap_or((f64::NAN, f64::NAN));
        }

        use super::moments::StatisticalMoments;
        (data.mean(), 0.0)
    }

    /// Compute modes (values that appear most frequently).
    /// For continuous data, uses kernel density estimation to find peaks.
    /// For discrete data, uses a binning approach.
    pub fn modes(data: &[f64]) -> Vec<f64> {
        if data.len() < 2 {
            return data.to_vec();
        }

        // Heuristic to check if data appears discrete (all integer values)
        let is_discrete = data.iter().all(|&x| x.fract() == 0.0 && x.is_finite());

        if is_discrete {
            Self::modes_discrete(data, 1e-9)
        } else {
            Self::modes_continuous(data)
        }
    }

    /// Compute modes for discrete/categorical data using binning.
    fn modes_discrete(data: &[f64], tolerance: f64) -> Vec<f64> {
        let mut frequency = std::collections::HashMap::new();

        // Round to tolerance for binning
        for &value in data {
            if !value.is_finite() { continue; }
            let binned = (value / tolerance).round() as i64;
            *frequency.entry(binned).or_insert(0) += 1;
        }

        let max_freq = match frequency.values().max() {
            Some(&max) => max,
            None => return Vec::new(),
        };

        if max_freq <= 1 {
            return Vec::new();
        }

        frequency.iter()
            .filter(|(_, &freq)| freq == max_freq)
            .map(|(&key, _)| key as f64 * tolerance)
            .collect()
    }

    /// Compute modes for continuous data using kernel density estimation.
    fn modes_continuous(data: &[f64]) -> Vec<f64> {
        if data.len() < 3 {
            return Vec::new();
        }

        let kde = SimpleKDE::new(data);

        let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        if !range.is_finite() || range <= 0.0 {
            return Vec::new();
        }

        const N_GRID: usize = 512;
        let grid_spacing = range / (N_GRID - 1) as f64;

        let densities: Vec<f64> = (0..N_GRID)
            .map(|i| {
                let x = min_val + i as f64 * grid_spacing;
                kde.evaluate(x)
            })
            .collect();

        // Find local maxima in the density grid
        let mut modes = Vec::new();
        for i in 1..(N_GRID - 1) {
            if densities[i] > densities[i - 1] && densities[i] > densities[i + 1] {
                let x = min_val + i as f64 * grid_spacing;
                modes.push(x);
            }
        }
        modes
    }
}
