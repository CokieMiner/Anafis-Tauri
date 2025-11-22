//! Prophet Forecasting Module
//!
//! This module implements a simplified Prophet-like forecasting model:
//! - Trend changepoint detection
//! - Seasonal decomposition
//! - Holiday effects
//! - Bayesian inference for uncertainty

pub mod types;

pub use types::*;
use ndarray::{Array1, Array2};
use rand::prelude::*;
use rand_distr::{Normal, Distribution};
use rand_pcg::Pcg64;
use std::collections::HashMap;
use rayon::prelude::*;
use crate::scientific::statistics::primitives::DesignMatrixBuilder;
use crate::scientific::statistics::time_series::SpectralEngine;

type McmcSample = (Vec<f64>, Vec<f64>, f64);

/// Prophet forecasting engine
pub struct ProphetEngine;

#[derive(Debug)]
struct LogLikelihoodInputs<'a> {
    y: &'a [f64],
    timestamps: &'a [f64],
    trend_params: &'a [f64],
    seasonal_params: &'a [f64],
    sigma: f64,
    changepoints: &'a [usize],
    seasonal_model: &'a SeasonalModel,
}

impl ProphetEngine {
    /// Fit Prophet model to time series data
    pub fn fit(
        &self,
        time_series: &[f64],
        timestamps: Option<&[f64]>,
        config: &ProphetConfig,
    ) -> Result<ProphetModel, String> {
        if time_series.is_empty() {
            return Err("Empty time series".to_string());
        }

        let n = time_series.len();
        let default_timestamps: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let timestamps = timestamps.unwrap_or(&default_timestamps);

        if timestamps.len() != n {
            return Err("Timestamps and time series must have same length".to_string());
        }

        // Detect changepoints
        let changepoints = self.detect_changepoints(time_series, config)?;

        // Fit trend component
        let trend_model = self.fit_trend(time_series, timestamps, &changepoints, config)?;

        // Fit seasonal component
        let seasonal_model = self.fit_seasonal(time_series, timestamps, config)?;

        // Fit holiday effects if provided
        let holiday_model = if let Some(holidays) = &config.holidays {
            let holiday_map: HashMap<String, Vec<f64>> = holidays.clone();
            Some(self.fit_holidays(time_series, timestamps, &holiday_map, config)?)
        } else {
            None
        };

        Ok(ProphetModel {
            trend_model,
            seasonal_model,
            holiday_model,
            changepoints,
            config: config.clone(),
            training_data: time_series.to_vec(),
            timestamps: timestamps.to_vec(),
        })
    }

    /// Make predictions with the fitted model
    pub fn predict(
        &self,
        model: &ProphetModel,
        future_timestamps: &[f64],
        include_uncertainty: bool,
    ) -> Result<ProphetPrediction, String> {
        let n_future = future_timestamps.len();

        // 1. Predict Trend (Parallelized)
        let trend_components: Vec<f64> = future_timestamps.par_iter()
            .map(|&t| self.predict_trend(&model.trend_model, t))
            .collect::<Result<Vec<_>, String>>()?;

        // 2. Predict Seasonal (Batch)
        // Generate Fourier basis for all timestamps at once
        let seasonal_components = if model.seasonal_model.n_harmonics > 0 {
             let fourier_basis = SpectralEngine::generate_fourier_basis(
                future_timestamps,
                model.seasonal_model.period,
                model.seasonal_model.n_harmonics,
            )?;
            
            // Matrix multiplication: basis * coeffs
            let coeffs = Array1::from_vec(model.seasonal_model.coefficients.clone());
            let seasonal_pred = fourier_basis.dot(&coeffs);
            seasonal_pred.to_vec()
        } else {
            vec![0.0; n_future]
        };

        // 3. Predict Holidays (Parallelized)
        let holiday_components: Vec<f64> = if let Some(holiday_model) = &model.holiday_model {
            future_timestamps.par_iter()
                .map(|&t| self.predict_holidays(holiday_model, t))
                .collect::<Result<Vec<_>, String>>()?
        } else {
            vec![0.0; n_future]
        };

        // 4. Combine
        let predictions: Vec<f64> = trend_components.iter()
            .zip(seasonal_components.iter())
            .zip(holiday_components.iter())
            .map(|((t, s), h)| t + s + h)
            .collect();

        let mut prediction_intervals = None;
        if include_uncertainty {
            prediction_intervals = Some(self.compute_uncertainty_intervals(
                model,
                future_timestamps,
                &model.config,
            )?);
        }

        Ok(ProphetPrediction {
            predictions,
            trend_components,
            seasonal_components,
            holiday_components,
            prediction_intervals,
        })
    }

    /// Detect changepoints in the trend
    fn detect_changepoints(
        &self,
        series: &[f64],
        config: &ProphetConfig,
    ) -> Result<Vec<usize>, String> {
        let n = series.len();
        let n_changepoints = config.n_changepoints.min(n / 2);

        if n_changepoints == 0 {
            return Ok(Vec::new());
        }

        // Compute differences
        let mut differences = Vec::with_capacity(n - 1);
        for i in 1..n {
            differences.push((series[i] - series[i - 1]).abs());
        }

        // Find indices of largest differences
        let mut indices: Vec<usize> = (0..differences.len()).collect();
        indices.sort_by(|&a, &b| differences[b].partial_cmp(&differences[a]).unwrap());

        let mut changepoints: Vec<usize> = indices
            .into_iter()
            .take(n_changepoints)
            .map(|i| i + 1) // Convert to original indices
            .collect();

        changepoints.sort();
        Ok(changepoints)
    }

    /// Fit piecewise linear trend with changepoints
    fn fit_trend(
        &self,
        series: &[f64],
        timestamps: &[f64],
        changepoints: &[usize],
        _config: &ProphetConfig,
    ) -> Result<TrendModel, String> {
        // Create design matrix using centralized builder
        let time_data: Vec<Vec<f64>> = timestamps.iter().map(|&t| vec![t]).collect();
        let x_trend = DesignMatrixBuilder::build_trend(
            &time_data,
            changepoints,
            timestamps,
        )?;
        let y_trend = Array1::from_vec(series.to_vec());

        // Fit linear regression
        let trend_coeffs = Self::linear_regression(&x_trend, &y_trend)?;

        Ok(TrendModel {
            coefficients: trend_coeffs.to_vec(),
            changepoints: changepoints.to_vec(),
            base_timestamps: timestamps.to_vec(),
        })
    }

    fn fit_seasonal(
        &self,
        series: &[f64],
        timestamps: &[f64],
        _config: &ProphetConfig,
    ) -> Result<SeasonalModel, String> {
        let n = series.len();

        // Remove trend first (simple linear detrending)
        let trend_coeffs = Self::linear_regression(
            &Array2::from_shape_vec(
                (n, 2),
                timestamps.iter()
                    .cloned()
                    .chain(vec![1.0; n])
                    .collect(),
            )
            .map_err(|e| format!("Failed to create design matrix: {:?}", e))?,
            &Array1::from_vec(series.to_vec()),
        )?;

        let mut detrended = Vec::with_capacity(n);
        for i in 0..n {
            let trend = trend_coeffs[0] * timestamps[i] + trend_coeffs[1];
            detrended.push(series[i] - trend);
        }

        // Fit Fourier series for seasonality using centralized function
        let period = _config.seasonality_period.unwrap_or(365.25); // Default to yearly
        let n_harmonics = _config.seasonality_harmonics;

        let x_seasonal = SpectralEngine::generate_fourier_basis(timestamps, period, n_harmonics)?;

        let seasonal_coeffs = Self::linear_regression(&x_seasonal, &Array1::from_vec(detrended))?;

        Ok(SeasonalModel {
            coefficients: seasonal_coeffs.to_vec(),
            period,
            n_harmonics,
        })
    }

    fn fit_holidays(
        &self,
        series: &[f64],
        timestamps: &[f64],
        holidays: &HashMap<String, Vec<f64>>,
        config: &ProphetConfig,
    ) -> Result<HolidayModel, String> {
        let n = series.len();
        let mut holiday_names = Vec::new();
        let mut x_holidays = Array2::zeros((n, holidays.len()));

        for (i, (name, dates)) in holidays.iter().enumerate() {
            holiday_names.push(name.clone());

            for &date in dates {
                // Find closest timestamp
                let mut min_dist = f64::INFINITY;
                let mut closest_idx = 0;

                for (j, &t) in timestamps.iter().enumerate() {
                    let dist = (t - date).abs();
                    if dist < min_dist {
                        min_dist = dist;
                        closest_idx = j;
                    }
                }

                // Add holiday effect (simple indicator)
                x_holidays[[closest_idx, i]] = 1.0;
            }
        }

        // Detrend and deseasonalize first
        let detrended = self.detrend_and_deseasonalize(series, timestamps, config)?;
        let holiday_coeffs = Self::linear_regression(&x_holidays, &Array1::from_vec(detrended))?;

        Ok(HolidayModel {
            coefficients: holiday_coeffs.to_vec(),
            holiday_names,
        })
    }

    /// Predict trend component
    fn predict_trend(&self, model: &TrendModel, timestamp: f64) -> Result<f64, String> {
        let mut prediction = model.coefficients[0] * timestamp;

        for (i, &cp_idx) in model.changepoints.iter().enumerate() {
            let cp_time = model.base_timestamps[cp_idx];
            if timestamp >= cp_time {
                prediction += model.coefficients[i + 1] * (timestamp - cp_time);
            }
        }

        Ok(prediction)
    }

    /// Predict seasonal component
    fn _predict_seasonal(&self, model: &SeasonalModel, timestamp: f64) -> Result<f64, String> {
        // Generate Fourier basis for this single timestamp
        let fourier_basis = SpectralEngine::generate_fourier_basis(
            &[timestamp],
            model.period,
            model.n_harmonics,
        )?;

        // Compute dot product of coefficients with Fourier basis
        let mut prediction = 0.0;
        for (i, &coeff) in model.coefficients.iter().enumerate() {
            prediction += coeff * fourier_basis[[0, i]];
        }

        Ok(prediction)
    }

    /// Predict holiday component
    fn predict_holidays(&self, _model: &HolidayModel, _timestamp: f64) -> Result<f64, String> {
        // For prediction, we don't know future holidays
        // This is a simplified implementation
        Ok(0.0)
    }

    /// Compute uncertainty intervals using MCMC sampling of model parameters
    /// This implements proper Bayesian uncertainty quantification by sampling from the
    /// posterior distribution of Prophet model parameters using Metropolis-Hastings MCMC.
    fn compute_uncertainty_intervals(
        &self,
        model: &ProphetModel,
        future_timestamps: &[f64],
        config: &ProphetConfig,
    ) -> Result<PredictionIntervals, String> {
        let n_sims = config.uncertainty_samples;
        let n_future = future_timestamps.len();

        // MCMC parameters
        let n_mcmc_samples = 1000; // Number of MCMC iterations
        let burn_in = 200; // Burn-in period
        let thinning = 5; // Thinning factor

        // Generate seeds for parallel MCMC chains
        let mut seed_rng = rand::rng();
        let n_chains = 4; // Run multiple chains in parallel
        let chain_seeds: Vec<u64> = (0..n_chains).map(|_| seed_rng.random::<u64>()).collect();

        // Run MCMC chains in parallel
        let all_samples: Vec<Vec<McmcSample>> = chain_seeds.into_par_iter()
            .map(|chain_seed| {
                let mut rng = Pcg64::seed_from_u64(chain_seed);

                // Initialize MCMC chain with current parameter estimates
                let mut current_trend_params = model.trend_model.coefficients.clone();
                let mut current_seasonal_params = model.seasonal_model.coefficients.clone();
                let mut current_sigma = config.uncertainty_scale;

                // Compute initial likelihood
                let inputs = LogLikelihoodInputs {
                    y: &model.training_data,
                    timestamps: &model.timestamps,
                    trend_params: &current_trend_params,
                    seasonal_params: &current_seasonal_params,
                    sigma: current_sigma,
                    changepoints: &model.changepoints,
                    seasonal_model: &model.seasonal_model,
                };
                let mut current_log_likelihood = self.compute_log_likelihood(&inputs)?;

                let mut chain_samples = Vec::new();

                // MCMC sampling for this chain
                for iteration in 0..(n_mcmc_samples + burn_in) {
                    // Propose new parameters
                    let (new_trend_params, new_seasonal_params, new_sigma) = self.propose_parameters(
                        &current_trend_params,
                        &current_seasonal_params,
                        current_sigma,
                        &mut rng,
                    );

                    // Compute new likelihood
                    let inputs = LogLikelihoodInputs {
                        y: &model.training_data,
                        timestamps: &model.timestamps,
                        trend_params: &new_trend_params,
                        seasonal_params: &new_seasonal_params,
                        sigma: new_sigma,
                        changepoints: &model.changepoints,
                        seasonal_model: &model.seasonal_model,
                    };
                    let new_log_likelihood = self.compute_log_likelihood(&inputs)?;

                    // Compute priors
                    let current_log_prior = self.compute_log_prior(&current_trend_params, &current_seasonal_params, current_sigma);
                    let new_log_prior = self.compute_log_prior(&new_trend_params, &new_seasonal_params, new_sigma);

                    // Compute proposal probabilities (symmetric proposals, so they cancel)
                    let log_acceptance_ratio = (new_log_likelihood + new_log_prior) -
                                             (current_log_likelihood + current_log_prior);

                    // Accept or reject
                    let acceptance_prob = log_acceptance_ratio.exp().min(1.0);
                    if rng.random::<f64>() < acceptance_prob {
                        current_trend_params = new_trend_params;
                        current_seasonal_params = new_seasonal_params;
                        current_sigma = new_sigma;
                        current_log_likelihood = new_log_likelihood;
                    }

                    // Store samples after burn-in and thinning
                    if iteration >= burn_in && (iteration - burn_in) % thinning == 0 {
                        chain_samples.push((current_trend_params.clone(), current_seasonal_params.clone(), current_sigma));
                    }
                }

                Ok(chain_samples)
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Combine samples from all chains
        let mut trend_samples = Vec::new();
        let mut seasonal_samples = Vec::new();
        let mut sigma_samples = Vec::new();

        for chain_samples in all_samples {
            for (trend, seasonal, sigma) in chain_samples {
                trend_samples.push(trend);
                seasonal_samples.push(seasonal);
                sigma_samples.push(sigma);
            }
        }

        // Generate predictions from MCMC samples
        let mut all_predictions = vec![Vec::new(); n_future];

        for sample_idx in 0..trend_samples.len().min(n_sims) {
            let trend_params = &trend_samples[sample_idx];
            let seasonal_params = &seasonal_samples[sample_idx];
            let sigma = sigma_samples[sample_idx];

            for (i, &t) in future_timestamps.iter().enumerate() {
                // Generate prediction with sampled parameters
                let trend_pred = self.predict_trend_with_params(
                    trend_params,
                    t,
                    &model.changepoints,
                    &model.trend_model.base_timestamps,
                )?;

                let seasonal_pred = self.predict_seasonal_with_params(
                    seasonal_params,
                    t,
                    &model.seasonal_model,
                )?;

                let holiday_pred = if let Some(holiday_model) = &model.holiday_model {
                    self.predict_holidays(holiday_model, t)?
                } else {
                    0.0
                };

                // Add residual noise
                let mut rng = rand::rng();
                let noise = rng.sample(rand_distr::Normal::new(0.0, sigma).map_err(|e| format!("Normal distribution error: {}", e))?);
                let prediction = trend_pred + seasonal_pred + holiday_pred + noise;

                all_predictions[i].push(prediction);
            }
        }

        // Compute quantiles for uncertainty intervals
        let mut lower_80 = Vec::new();
        let mut upper_80 = Vec::new();
        let mut lower_95 = Vec::new();
        let mut upper_95 = Vec::new();

        for pred in all_predictions.iter().take(n_future) {
            let mut predictions = pred.clone();
            predictions.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let n_samples = predictions.len();
            lower_95.push(predictions[(0.025 * n_samples as f64) as usize]);
            upper_95.push(predictions[(0.975 * n_samples as f64) as usize]);
            lower_80.push(predictions[(0.10 * n_samples as f64) as usize]);
            upper_80.push(predictions[(0.90 * n_samples as f64) as usize]);
        }

        Ok(PredictionIntervals {
            lower_80,
            upper_80,
            lower_95,
            upper_95,
        })
    }

    /// Compute log likelihood for MCMC
    fn compute_log_likelihood(&self, inputs: &LogLikelihoodInputs) -> Result<f64, String> {
        if inputs.sigma <= 0.0 {
            return Ok(f64::NEG_INFINITY);
        }

        let mut log_likelihood = 0.0;
        let two_pi = 2.0 * std::f64::consts::PI;
        let log_sigma = inputs.sigma.ln();

        for (i, &observed) in inputs.y.iter().enumerate() {
            let t = inputs.timestamps[i];

            let trend_pred = self.predict_trend_with_params(
                inputs.trend_params,
                t,
                inputs.changepoints,
                inputs.timestamps,
            )?;

            let seasonal_pred = self.predict_seasonal_with_params(
                inputs.seasonal_params,
                t,
                inputs.seasonal_model,
            )?;

            let predicted = trend_pred + seasonal_pred;
            let residual = observed - predicted;

            // Gaussian log likelihood
            let log_density = -0.5 * two_pi.ln() - log_sigma - 0.5 * (residual / inputs.sigma).powi(2);
            log_likelihood += log_density;
        }

        Ok(log_likelihood)
    }

    /// Compute log prior for MCMC
    fn compute_log_prior(&self, trend_params: &[f64], seasonal_params: &[f64], sigma: f64) -> f64 {
        let mut log_prior = 0.0;

        // Weak Gaussian priors on trend parameters
        for &param in trend_params {
            log_prior += -0.5 * (param / 10.0).powi(2); // N(0, 100) prior
        }

        // Weak Gaussian priors on seasonal parameters
        for &param in seasonal_params {
            log_prior += -0.5 * (param / 1.0).powi(2); // N(0, 1) prior
        }

        // Inverse gamma prior on sigma (conjugate for Gaussian likelihood)
        if sigma > 0.0 {
            log_prior += -2.0 * sigma.ln() - 1.0 / sigma; // Roughly IG(1, 1)
        } else {
            return f64::NEG_INFINITY;
        }

        log_prior
    }

    /// Propose new parameters for MCMC
    fn propose_parameters(
        &self,
        current_trend: &[f64],
        current_seasonal: &[f64],
        current_sigma: f64,
        rng: &mut Pcg64,
    ) -> (Vec<f64>, Vec<f64>, f64) {
        let normal = Normal::new(0.0, 0.1).unwrap();

        // Propose new trend parameters
        let new_trend = current_trend.iter()
            .map(|&x| x + normal.sample(rng))
            .collect();

        // Propose new seasonal parameters
        let new_seasonal = current_seasonal.iter()
            .map(|&x| x + normal.sample(rng))
            .collect();

        // Propose new sigma (log space to ensure positivity)
        let log_sigma_proposal = current_sigma.ln() + normal.sample(rng);
        let new_sigma = log_sigma_proposal.exp();

        (new_trend, new_seasonal, new_sigma)
    }

    /// Predict trend with given parameters
    fn predict_trend_with_params(
        &self,
        trend_params: &[f64],
        timestamp: f64,
        changepoints: &[usize],
        base_timestamps: &[f64],
    ) -> Result<f64, String> {
        if trend_params.is_empty() {
            return Ok(0.0);
        }

        let mut prediction = trend_params[0] * timestamp;

        for (i, &cp_idx) in changepoints.iter().enumerate() {
            if cp_idx < base_timestamps.len() {
                let cp_time = base_timestamps[cp_idx];
                if timestamp >= cp_time && i + 1 < trend_params.len() {
                    prediction += trend_params[i + 1] * (timestamp - cp_time);
                }
            }
        }

        Ok(prediction)
    }

    /// Predict seasonal with given parameters
    fn predict_seasonal_with_params(
        &self,
        seasonal_params: &[f64],
        timestamp: f64,
        seasonal_model: &SeasonalModel,
    ) -> Result<f64, String> {
        // Generate Fourier basis for this single timestamp
        let fourier_basis = SpectralEngine::generate_fourier_basis(
            &[timestamp],
            seasonal_model.period,
            seasonal_model.n_harmonics,
        )?;

        // Compute dot product of parameters with Fourier basis
        let mut prediction = 0.0;
        for (i, &param) in seasonal_params.iter().enumerate() {
            prediction += param * fourier_basis[[0, i]];
        }

        Ok(prediction)
    }

    /// Detrend and deseasonalize series for holiday fitting
    fn detrend_and_deseasonalize(
        &self,
        series: &[f64],
        timestamps: &[f64],
        config: &ProphetConfig,
    ) -> Result<Vec<f64>, String> {
        let n = series.len();

        // Simple linear detrending
        let trend_coeffs = Self::linear_regression(
            &Array2::from_shape_vec(
                (n, 2),
                timestamps.iter()
                    .cloned()
                    .chain(vec![1.0; n])
                    .collect(),
            )
            .map_err(|e| format!("Failed to create design matrix: {:?}", e))?,
            &Array1::from_vec(series.to_vec()),
        )?;

        let mut detrended = Vec::with_capacity(n);
        for i in 0..n {
            let trend = trend_coeffs[0] * timestamps[i] + trend_coeffs[1];
            detrended.push(series[i] - trend);
        }

        // Simple seasonal adjustment (assuming yearly seasonality)
        let period = config.seasonality_period.unwrap_or(365.25);
        let mut seasonal_means = vec![0.0; period.ceil() as usize];

        for (&timestamp, &detrended_val) in timestamps.iter().zip(detrended.iter()) {
            let seasonal_idx = (timestamp % period) as usize;
            seasonal_means[seasonal_idx] += detrended_val;
        }

        // Count observations per seasonal bin
        let mut seasonal_counts = vec![0; seasonal_means.len()];
        for &timestamp in timestamps.iter() {
            let seasonal_idx = (timestamp % period) as usize;
            seasonal_counts[seasonal_idx] += 1;
        }

        // Compute seasonal means
        for i in 0..seasonal_means.len() {
            if seasonal_counts[i] > 0 {
                seasonal_means[i] /= seasonal_counts[i] as f64;
            }
        }

        // Remove seasonality
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            let seasonal_idx = (timestamps[i] % period) as usize;
            result.push(detrended[i] - seasonal_means[seasonal_idx]);
        }

        Ok(result)
    }

    /// Simple linear regression utility
    fn linear_regression(x: &Array2<f64>, y: &Array1<f64>) -> Result<Array1<f64>, String> {
        use crate::scientific::statistics::primitives::LinearRegression;
        LinearRegression::ols_fit(x, y)
    }
}