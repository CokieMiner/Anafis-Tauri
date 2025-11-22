//! Uncertainty propagation for descriptive statistics
//!
//! This module provides methods to estimate the uncertainty of descriptive statistics
//! (mean, standard deviation, quantiles) given uncertainties in the input data.
//! It uses Monte Carlo simulation to propagate errors.

use super::moments::StatisticalMoments;
use super::quantiles::{Quantiles, QuantileMethod};
use rand_distr::{Normal, Distribution};
use rayon::prelude::*;

/// Uncertainty calculations for descriptive statistics
pub struct DescriptiveUncertainty;

impl DescriptiveUncertainty {
    /// Compute mean with uncertainty propagation.
    ///
    /// # Arguments
    /// * `data` - Vector of data values
    /// * `errors` - Vector of uncertainties (standard deviations) for each data point
    /// * `n_sims` - Number of Monte Carlo simulations
    ///
    /// # Returns
    /// * `(mean_val, std_err)` - The mean of the means and the standard deviation of the means
    pub fn mean_uncertainty(data: &[f64], errors: &[f64], n_sims: usize) -> Result<(f64, f64), String> {
        if data.len() != errors.len() {
            return Err("Data and errors must have the same length".to_string());
        }
        if data.is_empty() {
            return Err("Data cannot be empty".to_string());
        }

        let means: Vec<f64> = (0..n_sims).into_par_iter().map(|_| {
            let mut rng = rand::rng();
            let mut sum = 0.0;
            for (&val, &err) in data.iter().zip(errors.iter()) {
                let sample = if err > 0.0 {
                    let normal = Normal::new(val, err).unwrap();
                    normal.sample(&mut rng)
                } else {
                    val
                };
                sum += sample;
            }
            sum / data.len() as f64
        }).collect();

        Ok((means.mean(), means.std_dev()))
    }

    /// Compute standard deviation with uncertainty propagation.
    ///
    /// # Returns
    /// * `(mean_std_dev, std_err_std_dev)`
    pub fn std_dev_uncertainty(data: &[f64], errors: &[f64], n_sims: usize) -> Result<(f64, f64), String> {
        if data.len() != errors.len() {
            return Err("Data and errors must have the same length".to_string());
        }
        if data.len() < 2 {
            return Err("Need at least 2 data points for standard deviation".to_string());
        }

        let std_devs: Vec<f64> = (0..n_sims).into_par_iter().map(|_| {
            let mut rng = rand::rng();
            let simulated_data: Vec<f64> = data.iter().zip(errors.iter()).map(|(&val, &err)| {
                if err > 0.0 {
                    let normal = Normal::new(val, err).unwrap();
                    normal.sample(&mut rng)
                } else {
                    val
                }
            }).collect();
            simulated_data.std_dev()
        }).collect();

        Ok((std_devs.mean(), std_devs.std_dev()))
    }

    /// Compute quantile with uncertainty propagation.
    ///
    /// # Returns
    /// * `(mean_quantile, std_err_quantile)`
    pub fn quantile_uncertainty(data: &[f64], errors: &[f64], p: f64, n_sims: usize) -> Result<(f64, f64), String> {
        if data.len() != errors.len() {
            return Err("Data and errors must have the same length".to_string());
        }
        if !(0.0..=1.0).contains(&p) {
            return Err("Probability must be between 0 and 1".to_string());
        }

        let quantiles: Vec<f64> = (0..n_sims).into_par_iter().map(|_| {
            let mut rng = rand::rng();
            let mut simulated_data: Vec<f64> = data.iter().zip(errors.iter()).map(|(&val, &err)| {
                if err > 0.0 {
                    let normal = Normal::new(val, err).unwrap();
                    normal.sample(&mut rng)
                } else {
                    val
                }
            }).collect();
            
            // Sort for quantile calculation
            simulated_data.sort_by(|a, b| a.total_cmp(b));
            
            Quantiles::quantile(&simulated_data, p, QuantileMethod::Type8).unwrap_or(f64::NAN)
        }).collect();

        Ok((quantiles.mean(), quantiles.std_dev()))
    }
}
