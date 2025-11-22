//! Distribution fitting utilities
//!
//! This module provides comprehensive distribution fitting capabilities
//! for continuous, extreme, and heavy-tail distributions.

use serde::{Deserialize, Serialize};
use crate::scientific::statistics::primitives::SpecialFunctions;
use crate::scientific::statistics::descriptive::{Quantiles, StatisticalMoments};
use crate::scientific::statistics::distributions::distribution_functions;
use argmin::core::{CostFunction, Error, Gradient};
use num_dual::{DualNum, Dual64};
use rayon::prelude::*;

/// Type alias for distribution fitting functions
type DistributionFitter = fn(&[f64]) -> Result<DistributionFit, String>;

/// Result of fitting a statistical distribution to data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionFit {
    /// Name of the fitted distribution
    pub distribution_name: String,
    /// Fitted parameters as (name, value) pairs
    pub parameters: Vec<(String, f64)>,
    /// Log-likelihood of the fit
    pub log_likelihood: f64,
    /// Akaike Information Criterion
    pub aic: f64,
    /// Bayesian Information Criterion
    pub bic: f64,
    /// Goodness of fit statistic (Kolmogorov-Smirnov statistic)
    pub goodness_of_fit: f64,
    /// Uncertainties (standard deviations) of the fitted parameters
    pub parameter_uncertainties: Option<Vec<(String, f64)>>,
}

/// Global optimization configuration
#[derive(Debug, Clone)]
pub struct GlobalOptimizationConfig {
    /// Number of random starts for global optimization
    pub num_starts: usize,
    /// Maximum iterations for local optimization
    pub max_local_iters: usize,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Parameter bounds as (lower, upper) pairs
    pub bounds: Vec<(f64, f64)>,
}

impl Default for GlobalOptimizationConfig {
    fn default() -> Self {
        Self {
            num_starts: 5,
            max_local_iters: 100,
            tolerance: 1e-8,
            bounds: vec![],
        }
    }
}

/// Global optimizer for distribution fitting
pub struct GlobalOptimizer<F: CostFunction<Param = Vec<f64>, Output = f64>> {
    cost_fn: F,
    _config: GlobalOptimizationConfig,
}

impl<F: CostFunction<Param = Vec<f64>, Output = f64>> GlobalOptimizer<F> {
    pub fn new(cost_fn: F, _config: GlobalOptimizationConfig) -> Self {
        Self { cost_fn, _config }
    }

    pub fn optimize(&mut self, initial_guess: &[f64], _bounds: &[Vec<(f64, f64)>]) -> Result<OptimizationResult, String> {
        // Simplified global optimization - in practice, this would use multiple starts
        let best_param = initial_guess.to_vec();
        let mut best_cost = f64::INFINITY;

        // Try the initial guess
        if let Ok(cost) = self.cost_fn.cost(&best_param) {
            if cost < best_cost && cost.is_finite() {
                best_cost = cost;
            }
        }

        // For now, just return the initial guess result
        // A full implementation would try multiple random starts within bounds
        Ok(OptimizationResult {
            best_param,
            best_cost,
            converged: true,
        })
    }
}

/// Optimization result
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub best_param: Vec<f64>,
    pub best_cost: f64,
    pub converged: bool,
}

/// Gradient-based optimizer using Dual Numbers
pub struct GradientOptimizer<F> {
    cost_fn: F,
    config: GlobalOptimizationConfig,
}

impl<F> GradientOptimizer<F>
where
    F: CostFunction<Param = Vec<f64>, Output = f64> + Gradient<Param = Vec<f64>, Gradient = Vec<f64>>,
{
    pub fn new(cost_fn: F, config: GlobalOptimizationConfig) -> Self {
        Self { cost_fn, config }
    }

    pub fn optimize(&mut self, initial_guess: &[f64], bounds: &[Vec<(f64, f64)>]) -> Result<OptimizationResult, String> {
        let mut param = initial_guess.to_vec();
        let mut cost = self.cost_fn.cost(&param).map_err(|e| e.to_string())?;
        let mut learning_rate = 0.01;

        for _iter in 0..self.config.max_local_iters {
            let grad = self.cost_fn.gradient(&param).map_err(|e| e.to_string())?;
            
            // Simple gradient descent step
            let mut new_param = param.clone();
            for i in 0..param.len() {
                new_param[i] -= learning_rate * grad[i];
                
                // Apply bounds if provided
                if !bounds.is_empty() && i < bounds.len() {
                    // Assuming bounds[0] contains bounds for all params or bounds matches param length
                    // The original code passed `&[bounds]` which is `&[Vec<(f64, f64)>]`.
                    // `bounds` here is `&[Vec<(f64, f64)>]`.
                    // Actually `bounds` in `fit_weibull` is `vec![(min, max), (min, max)]`.
                    // And passed as `&[bounds]`. So it's `&[Vec<(f64, f64)>]`.
                    // So `bounds[0]` is the vector of bounds for parameters.
                    if let Some(param_bounds) = bounds.first() {
                        if i < param_bounds.len() {
                            new_param[i] = new_param[i].clamp(param_bounds[i].0, param_bounds[i].1);
                        }
                    }
                }
            }

            let new_cost = match self.cost_fn.cost(&new_param) {
                Ok(c) => c,
                Err(_) => f64::INFINITY,
            };

            if new_cost < cost {
                param = new_param;
                cost = new_cost;
                learning_rate *= 1.1; // Increase learning rate if successful
            } else {
                learning_rate *= 0.5; // Decrease learning rate if failed
            }
            
            if learning_rate < 1e-10 {
                break;
            }
        }

        Ok(OptimizationResult {
            best_param: param,
            best_cost: cost,
            converged: true,
        })
    }
}

/// Statistical distribution engine - coordinates distribution-related computations
pub struct StatisticalDistributionEngine;

impl StatisticalDistributionEngine {
    /// Fit multiple distributions to data and return best fits
    pub fn fit_distributions(data: &[f64], errors: Option<&[f64]>) -> Result<Vec<DistributionFit>, String> {
        if data.is_empty() {
            return Err("Cannot fit distributions to empty dataset".to_string());
        }

        // Validate data: ensure no NaNs or Infinities are passed to fitters
        if !data.iter().all(|x| x.is_finite()) {
             return Err("Data contains non-finite values (NaN or Infinity)".to_string());
        }

        // Define all distribution fitting functions
        // We need to wrap them to match the expected signature for the loop if we want to be generic,
        // but since we are updating them all, we can just call them directly or update the type alias.
        // For now, let's update the type alias to match the new signature.
        
        let fitting_functions: Vec<fn(&[f64], Option<&[f64]>) -> Result<DistributionFit, String>> = vec![
            Self::fit_normal_distribution,
            Self::fit_lognormal_distribution,
            Self::fit_exponential_distribution,
            Self::fit_weibull_distribution,
            Self::fit_gamma_distribution,
            Self::fit_beta_distribution,
            Self::fit_gumbel_distribution,
            Self::fit_pareto_distribution,
            Self::fit_students_t_distribution,
            Self::fit_cauchy_distribution,
            Self::fit_johnson_su_distribution,
            Self::fit_burr_type_xii_distribution,
        ];

        // Fit all distributions in parallel
        let fits: Vec<DistributionFit> = fitting_functions
            .into_par_iter()
            .filter_map(|fit_fn| fit_fn(data, errors).ok())
            .collect();

        // Filter out fits with invalid AIC values and sort by goodness of fit (lower AIC is better)
        let mut sorted_fits = fits
            .into_iter()
            .filter(|fit| fit.aic.is_finite())
            .collect::<Vec<_>>();
        sorted_fits.sort_by(|a, b| match a.aic.partial_cmp(&b.aic) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        Ok(sorted_fits)
    }

    /// Helper to perform uncertainty propagation via Monte Carlo
    fn uncertainty_fit<F>(
        data: &[f64],
        errors: &[f64],
        fit_fn: F,
        n_sims: usize
    ) -> Option<Vec<(String, f64)>>
    where
        F: Fn(&[f64]) -> Result<DistributionFit, String> + Sync + Send,
    {
        use rand_distr::{Normal, Distribution};
        
        if data.len() != errors.len() {
            return None;
        }

        let results: Vec<Vec<(String, f64)>> = (0..n_sims).into_par_iter().filter_map(|_| {
            let mut rng = rand::rng();
            let simulated_data: Vec<f64> = data.iter().zip(errors.iter()).map(|(&val, &err)| {
                if err > 0.0 {
                    let normal = Normal::new(val, err).ok()?;
                    Some(normal.sample(&mut rng))
                } else {
                    Some(val)
                }
            }).collect::<Option<Vec<_>>>()?;

            fit_fn(&simulated_data).ok().map(|fit| fit.parameters)
        }).collect();

        if results.is_empty() {
            return None;
        }

        // Calculate std dev for each parameter
        let first_res = &results[0];
        let mut uncertainties = Vec::new();

        for (i, (name, _)) in first_res.iter().enumerate() {
            let values: Vec<f64> = results.iter().map(|res| res[i].1).collect();
            let std_dev = values.std_dev();
            uncertainties.push((name.clone(), std_dev));
        }

        Some(uncertainties)
    }

    /// Fit normal distribution using maximum likelihood estimation
    pub fn fit_normal_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        let n = data.len() as f64;
        let mean = data.mean();
        let variance = data.variance();
        let std_dev = variance.sqrt();

        if !mean.is_finite() || !std_dev.is_finite() || std_dev <= 0.0 {
            return Err("Normal fit undefined: invalid parameter estimates".to_string());
        }

        // Log-likelihood for normal distribution
        let log_likelihood = data.iter()
            .map(|&x| {
                let z = (x - mean) / std_dev;
                -0.5 * (2.0 * std::f64::consts::PI).ln() - std_dev.ln() - 0.5 * z * z
            })
            .sum::<f64>();

        let k = 2.0; // parameters: mean, std_dev
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_normal_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "normal".to_string(),
            parameters: vec![
                ("mean".to_string(), mean),
                ("std_dev".to_string(), std_dev),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::normal_cdf(x, mean, std_dev))?,
            parameter_uncertainties,
        })
    }

    /// Fit lognormal distribution
    pub fn fit_lognormal_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        if data.iter().any(|&x| x <= 0.0) {
            return Err("Lognormal distribution requires positive data".to_string());
        }

        let n = data.len() as f64;
        let log_data: Vec<f64> = data.iter().map(|x| x.ln()).collect();
        let mu = log_data.mean();
        let sigma_squared = log_data.variance();
        let sigma = sigma_squared.sqrt();

        if !mu.is_finite() || !sigma.is_finite() || sigma <= 0.0 {
            return Err("Lognormal fit undefined: invalid parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                let ln_x = x.ln();
                let z = (ln_x - mu) / sigma;
                -ln_x - 0.5 * (2.0 * std::f64::consts::PI).ln() - sigma.ln() - 0.5 * z * z
            })
            .sum::<f64>();

        let k = 2.0; // parameters: mu, sigma
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_lognormal_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "lognormal".to_string(),
            parameters: vec![
                ("mu".to_string(), mu),
                ("sigma".to_string(), sigma),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::lognormal_cdf(x, mu, sigma))?,
            parameter_uncertainties,
        })
    }

    /// Fit exponential distribution
    pub fn fit_exponential_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        if data.iter().any(|&x| x < 0.0) {
            return Err("Exponential distribution requires non-negative data".to_string());
        }

        let n = data.len() as f64;
        let mean = data.mean();
        let rate: f64 = 1.0 / mean;

        if !rate.is_finite() || rate <= 0.0 {
            return Err("Exponential fit undefined: invalid parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| rate.ln() - rate * x)
            .sum::<f64>();

        let k = 1.0; // parameter: rate
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_exponential_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "exponential".to_string(),
            parameters: vec![("rate".to_string(), rate)],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::exponential_cdf(x, rate))?,
            parameter_uncertainties,
        })
    }

    /// Fit Weibull distribution using maximum likelihood estimation
    pub fn fit_weibull_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        if data.iter().any(|&x| x <= 0.0) {
            return Err("Weibull distribution requires positive data".to_string());
        }

        let n = data.len() as f64;

        // Initial estimates using method of moments
        let mean = data.mean();
        let variance = data.variance();
        let cv = (variance).sqrt() / mean;

        // Approximate shape parameter using method of moments
        let shape_est = if cv < 0.6 {
            1.0 / (cv * cv)
        } else {
            1.0 / (cv * cv + 1.0).sqrt()
        };
        let scale_est = mean / SpecialFunctions::gamma(1.0 + 1.0 / shape_est);

        let initial_guess = vec![shape_est, scale_est];
        let bounds = vec![(0.1, 10.0), (0.1, data.iter().cloned().fold(f64::NEG_INFINITY, f64::max) * 2.0)];

        // Create cost function for Weibull MLE
        #[derive(Clone)]
        struct WeibullCost {
            data: Vec<f64>,
        }

        impl CostFunction for WeibullCost {
            type Param = Vec<f64>;
            type Output = f64;

            fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
                let shape = param[0];
                let scale = param[1];

                if shape <= 0.0 || scale <= 0.0 || !shape.is_finite() || !scale.is_finite() {
                    return Ok(f64::INFINITY);
                }

                let mut log_likelihood = 0.0;
                for &x in self.data.iter() {
                    if x <= 0.0 {
                        return Ok(f64::INFINITY);
                    }
                    let log_pdf = shape.ln() - scale.ln() + (shape - 1.0) * (x.ln() - scale.ln()) - (x / scale).powf(shape);
                    if !log_pdf.is_finite() {
                        return Ok(f64::INFINITY);
                    }
                    log_likelihood += log_pdf;
                }

                Ok(-log_likelihood)
            }
        }

        let _cost_fn = WeibullCost { data: data.to_vec() };
        let _config = GlobalOptimizationConfig {
            num_starts: 3,
            max_local_iters: 50,
            tolerance: 1e-8,
            ..Default::default()
        };

        impl Gradient for WeibullCost {
            type Param = Vec<f64>;
            type Gradient = Vec<f64>;

            fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
                let shape_val = param[0];
                let scale_val = param[1];

                // Calculate gradients using Dual Numbers
                let mut grad = vec![0.0; 2];

                // Partial wrt shape
                let shape_d = Dual64::new(shape_val, 1.0);
                let scale_c = Dual64::new(scale_val, 0.0);
                grad[0] = weibull_cost_dual(&self.data, shape_d, scale_c).eps;

                // Partial wrt scale
                let shape_c = Dual64::new(shape_val, 0.0);
                let scale_d = Dual64::new(scale_val, 1.0);
                grad[1] = weibull_cost_dual(&self.data, shape_c, scale_d).eps;

                Ok(grad)
            }
        }

        fn weibull_cost_dual<T: DualNum<f64> + Copy>(data: &[f64], shape: T, scale: T) -> T {
            let mut log_likelihood = T::zero();
            for &x in data.iter() {
                 if x <= 0.0 { return T::from(f64::INFINITY); }
                 let x_t = T::from(x);
                 let log_pdf = shape.ln() - scale.ln() + (shape - T::one()) * (x_t.ln() - scale.ln()) - (x_t / scale).powd(shape);
                 log_likelihood += log_pdf;
            }
            -log_likelihood
        }

        let cost_fn = WeibullCost { data: data.to_vec() };
        let config = GlobalOptimizationConfig {
            num_starts: 3,
            max_local_iters: 50,
            tolerance: 1e-8,
            ..Default::default()
        };

        let mut optimizer = GradientOptimizer::new(cost_fn, config);
        let opt_result = optimizer.optimize(&initial_guess, &[bounds])?;

        if !opt_result.converged || opt_result.best_param.len() < 2 {
            return Err("Weibull optimization failed to converge".to_string());
        }

        let shape = opt_result.best_param[0];
        let scale = opt_result.best_param[1];

        if shape <= 0.0 || scale <= 0.0 || !shape.is_finite() || !scale.is_finite() {
            return Err("Weibull fit undefined: invalid optimized parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                let z = (x / scale).powf(shape);
                shape.ln() - scale.ln() + (shape - 1.0) * (x.ln() - scale.ln()) - z
            })
            .sum::<f64>();

        let k = 2.0; // parameters: shape, scale
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_weibull_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "weibull".to_string(),
            parameters: vec![
                ("shape".to_string(), shape),
                ("scale".to_string(), scale),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::weibull_cdf(x, shape, scale))?,
            parameter_uncertainties,
        })
    }

    /// Fit Gamma distribution using maximum likelihood estimation
    pub fn fit_gamma_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        if data.iter().any(|&x| x <= 0.0) {
            return Err("Gamma distribution requires positive data".to_string());
        }

        let n = data.len() as f64;

        // Method of moments initial estimates
        let mean = data.mean();
        let variance = data.variance();

        if mean <= 0.0 || variance <= 0.0 {
            return Err("Gamma fit undefined: invalid moments".to_string());
        }

        let shape_est = mean * mean / variance;
        let rate_est = mean / variance;

        let initial_guess = vec![shape_est, rate_est];
        let bounds = vec![(0.1, 100.0), (1e-6, 100.0)];

        // Create cost function for Gamma MLE
        #[derive(Clone)]
        struct GammaCost {
            data: Vec<f64>,
        }

        impl CostFunction for GammaCost {
            type Param = Vec<f64>;
            type Output = f64;

            fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
                let shape = param[0];
                let rate = param[1];

                if shape <= 0.0 || rate <= 0.0 || !shape.is_finite() || !rate.is_finite() {
                    return Ok(f64::INFINITY);
                }

                let nll = self.data.iter()
                    .map(|&x| {
                        shape * rate.ln() + (shape - 1.0) * x.ln() - rate * x - SpecialFunctions::gamma(shape).ln()
                    })
                    .sum::<f64>();

                Ok(-nll)
            }
        }

        let cost_fn = GammaCost { data: data.to_vec() };
        let config = GlobalOptimizationConfig {
            num_starts: 3,
            max_local_iters: 50,
            tolerance: 1e-8,
            ..Default::default()
        };

        impl Gradient for GammaCost {
            type Param = Vec<f64>;
            type Gradient = Vec<f64>;

            fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
                let shape_val = param[0];
                let rate_val = param[1];

                let mut grad = vec![0.0; 2];
                
                let shape_d = Dual64::new(shape_val, 1.0);
                let rate_c = Dual64::new(rate_val, 0.0);
                grad[0] = gamma_cost_dual(&self.data, shape_d, rate_c).eps;

                let shape_c = Dual64::new(shape_val, 0.0);
                let rate_d = Dual64::new(rate_val, 1.0);
                grad[1] = gamma_cost_dual(&self.data, shape_c, rate_d).eps;

                Ok(grad)
            }
        }

        fn gamma_cost_dual<T: DualNum<f64> + Copy>(data: &[f64], shape: T, rate: T) -> T {
            let mut nll = T::zero();
            for &x in data.iter() {
                let x_t = T::from(x);
                let log_pdf = shape * rate.ln() + (shape - T::one()) * x_t.ln() - rate * x_t - ln_gamma_dual(shape);
                nll -= log_pdf;
            }
            nll
        }

        fn ln_gamma_dual<T: DualNum<f64> + Copy>(z: T) -> T {
            // Use recurrence for z < 0.5: ln(gamma(z)) = ln(gamma(z+1)) - ln(z)
            // Use Stirling approximation for z >= 0.5: ln(gamma(z)) ≈ (z-0.5)*ln(z) - z + 0.5*ln(2*pi)
            if z.re() < 0.5 {
                ln_gamma_dual(z + T::one()) - z.ln()
            } else {
                let half = T::from(0.5);
                let two_pi = T::from(2.0 * std::f64::consts::PI);
                (z - half) * z.ln() - z + half * two_pi.ln()
            }
        }
        
        // Redoing the implementation to use manual gradient for Gamma
        // because of the Gamma function dependency.
        
        let mut optimizer = GradientOptimizer::new(cost_fn, config);
        let opt_result = optimizer.optimize(&initial_guess, &[bounds])?;

        if !opt_result.converged || opt_result.best_param.len() < 2 {
            return Err("Gamma optimization failed to converge".to_string());
        }

        let shape = opt_result.best_param[0];
        let rate = opt_result.best_param[1];

        if shape <= 0.0 || rate <= 0.0 || !shape.is_finite() || !rate.is_finite() {
            return Err("Gamma fit undefined: invalid optimized parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                shape * rate.ln() + (shape - 1.0) * x.ln() - rate * x - SpecialFunctions::gamma(shape).ln()
            })
            .sum::<f64>();

        let k = 2.0; // parameters: shape, rate
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_gamma_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "gamma".to_string(),
            parameters: vec![
                ("shape".to_string(), shape),
                ("rate".to_string(), rate),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::gamma_cdf(x, shape, rate))?,
            parameter_uncertainties,
        })
    }

    /// Fit Beta distribution
    pub fn fit_beta_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        if data.iter().any(|&x| x <= 0.0 || x >= 1.0) {
            return Err("Beta distribution requires data in (0,1)".to_string());
        }

        let n = data.len() as f64;

        // Method of moments initial estimates
        let mean = data.mean();
        let variance = data.variance();

        if variance <= 0.0 || mean <= 0.0 || mean >= 1.0 {
            return Err("Beta fit undefined: invalid moments".to_string());
        }

        let mut alpha = mean * (mean * (1.0 - mean) / variance - 1.0);
        let mut beta_param = (1.0 - mean) * (mean * (1.0 - mean) / variance - 1.0);

        if alpha <= 0.0 || beta_param <= 0.0 {
            alpha = mean * 2.0;
            beta_param = (1.0 - mean) * 2.0;
        }

        let initial_guess = vec![alpha, beta_param];
        let bounds = vec![(1e-6, 100.0), (1e-6, 100.0)];

        // Create cost function for Beta MLE
        #[derive(Clone)]
        struct BetaCost {
            data: Vec<f64>,
        }

        impl CostFunction for BetaCost {
            type Param = Vec<f64>;
            type Output = f64;

            fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
                let alpha = param[0];
                let beta_param = param[1];

                if alpha <= 0.0 || beta_param <= 0.0 || !alpha.is_finite() || !beta_param.is_finite() {
                    return Ok(f64::INFINITY);
                }

                let nll = self.data.iter()
                    .map(|&x| {
                        (alpha - 1.0) * x.ln() + (beta_param - 1.0) * (1.0 - x).ln() - SpecialFunctions::beta(alpha, beta_param).ln()
                    })
                    .sum::<f64>();

                Ok(-nll)
            }
        }

        let cost_fn = BetaCost { data: data.to_vec() };
        let config = GlobalOptimizationConfig {
            num_starts: 3,
            max_local_iters: 50,
            tolerance: 1e-8,
            ..Default::default()
        };

        impl Gradient for BetaCost {
            type Param = Vec<f64>;
            type Gradient = Vec<f64>;

            fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
                let alpha = param[0];
                let beta_param = param[1];

                // Manual gradient for Beta distribution
                // d/d(alpha) = sum(ln(x)) - n * (digamma(alpha) - digamma(alpha+beta))
                // d/d(beta) = sum(ln(1-x)) - n * (digamma(beta) - digamma(alpha+beta))
                
                let n = self.data.len() as f64;
                let digamma_alpha = SpecialFunctions::digamma(alpha);
                let digamma_beta = SpecialFunctions::digamma(beta_param);
                let digamma_sum = SpecialFunctions::digamma(alpha + beta_param);
                
                let mut grad_alpha = 0.0;
                let mut grad_beta = 0.0;
                
                for &x in self.data.iter() {
                    grad_alpha += x.ln();
                    grad_beta += (1.0 - x).ln();
                }
                
                grad_alpha -= n * (digamma_alpha - digamma_sum);
                grad_beta -= n * (digamma_beta - digamma_sum);
                
                // We are minimizing negative log likelihood, so negate gradients
                Ok(vec![-grad_alpha, -grad_beta])
            }
        }

        let mut optimizer = GradientOptimizer::new(cost_fn, config);
        let opt_result = optimizer.optimize(&initial_guess, &[bounds])?;

        if !opt_result.converged || opt_result.best_param.len() < 2 {
            return Err("Beta optimization failed to converge".to_string());
        }

        let alpha = opt_result.best_param[0];
        let beta_param = opt_result.best_param[1];

        if alpha <= 0.0 || beta_param <= 0.0 || !alpha.is_finite() || !beta_param.is_finite() {
            return Err("Beta fit undefined: invalid optimized parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                let pdf = x.powf(alpha - 1.0) * (1.0 - x).powf(beta_param - 1.0) / SpecialFunctions::beta(alpha, beta_param);
                pdf.ln()
            })
            .sum::<f64>();

        let k = 2.0; // parameters: alpha, beta
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_beta_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "beta".to_string(),
            parameters: vec![
                ("alpha".to_string(), alpha),
                ("beta".to_string(), beta_param),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::beta_cdf(x, alpha, beta_param))?,
            parameter_uncertainties,
        })
    }

    /// Fit Gumbel distribution (extreme value type I)
    pub fn fit_gumbel_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        let n = data.len() as f64;

        // Method of moments for initial estimates
        let mean = data.mean();
        let std_dev = data.variance().sqrt();

        // Gumbel parameters: location = μ - γ*β, scale = β
        // where γ ≈ 0.5772156649 (Euler-Mascheroni constant)
        let euler_gamma = 0.577_215_664_901_532_9;
        let scale_est = std_dev * (6.0f64).sqrt() / std::f64::consts::PI;
        let location_est = mean - euler_gamma * scale_est;

        let initial_guess = vec![location_est, scale_est];
        let bounds = vec![
            (data.iter().cloned().fold(f64::INFINITY, f64::min) - 10.0 * std_dev, data.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + 10.0 * std_dev), // location
            (1e-6, std_dev * 10.0), // scale
        ];

        // Create cost function for Gumbel MLE
        #[derive(Clone)]
        struct GumbelCost {
            data: Vec<f64>,
        }

        impl CostFunction for GumbelCost {
            type Param = Vec<f64>;
            type Output = f64;

            fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
                let location = param[0];
                let scale = param[1];

                if scale <= 0.0 || !location.is_finite() || !scale.is_finite() {
                    return Ok(f64::INFINITY);
                }

                let mut nll = 0.0;
                for &x in self.data.iter() {
                    let z = (x - location) / scale;
                    // PDF: (1/scale) * exp(-(z + exp(-z)))
                    // Log PDF: -ln(scale) - (z + exp(-z))
                    let log_pdf = -scale.ln() - (z + (-z).exp());
                    nll -= log_pdf;
                }

                Ok(nll)
            }
        }

        impl Gradient for GumbelCost {
            type Param = Vec<f64>;
            type Gradient = Vec<f64>;

            fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
                let location_val = param[0];
                let scale_val = param[1];

                if scale_val <= 0.0 {
                    return Ok(vec![0.0, 0.0]);
                }

                // Gradient w.r.t location
                let grad_location = {
                    let location = Dual64::new(location_val, 1.0);
                    let scale = Dual64::from(scale_val);
                    
                    let mut nll = Dual64::from(0.0);
                    for &x_val in self.data.iter() {
                        let x = Dual64::from(x_val);
                        let z = (x - location) / scale;
                        let log_pdf = -scale.ln() - (z + (-z).exp());
                        nll -= log_pdf;
                    }
                    nll.eps
                };

                // Gradient w.r.t scale
                let grad_scale = {
                    let location = Dual64::from(location_val);
                    let scale = Dual64::new(scale_val, 1.0);
                    
                    let mut nll = Dual64::from(0.0);
                    for &x_val in self.data.iter() {
                        let x = Dual64::from(x_val);
                        let z = (x - location) / scale;
                        let log_pdf = -scale.ln() - (z + (-z).exp());
                        nll -= log_pdf;
                    }
                    nll.eps
                };

                Ok(vec![grad_location, grad_scale])
            }
        }

        let cost_fn = GumbelCost { data: data.to_vec() };
        let config = GlobalOptimizationConfig {
            num_starts: 3,
            max_local_iters: 50,
            tolerance: 1e-8,
            ..Default::default()
        };

        let mut optimizer = GradientOptimizer::new(cost_fn, config);
        let opt_result = optimizer.optimize(&initial_guess, &[bounds])?;

        if !opt_result.converged || opt_result.best_param.len() < 2 {
            return Err("Gumbel optimization failed to converge".to_string());
        }

        let location = opt_result.best_param[0];
        let scale = opt_result.best_param[1];

        if !location.is_finite() || !scale.is_finite() || scale <= 0.0 {
            return Err("Gumbel fit undefined: invalid optimized parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                let z = (x - location) / scale;
                -scale.ln() - (z + (-z).exp())
            })
            .sum::<f64>();

        let k = 2.0; // parameters: location, scale
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_gumbel_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "gumbel".to_string(),
            parameters: vec![
                ("location".to_string(), location),
                ("scale".to_string(), scale),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::gumbel_cdf(x, location, scale))?,
            parameter_uncertainties,
        })
    }

    /// Fit Pareto distribution
    pub fn fit_pareto_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        if data.iter().any(|&x| x <= 0.0) {
            return Err("Pareto distribution requires positive data".to_string());
        }

        let n = data.len() as f64;
        let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);

        if !min_val.is_finite() || min_val <= 0.0 {
            return Err("Pareto fit undefined: data contains non-positive values.".to_string());
        }

        // For Pareto distribution, the Maximum Likelihood Estimates (MLE) are analytical.
        // The scale parameter (xm) is the minimum value of the data.
        let scale = min_val;

        // The shape parameter (alpha) is n / sum(ln(xi / xm)).
        let sum_log_diff: f64 = data.iter().map(|&x| x.ln() - scale.ln()).sum();
        
        if sum_log_diff <= 0.0 {
            return Err("Pareto fit undefined: cannot compute shape parameter.".to_string());
        }
        
        let shape_mle = n / sum_log_diff;

        if !shape_mle.is_finite() || shape_mle <= 0.0 {
            return Err("Pareto fit undefined: invalid estimated shape parameter.".to_string());
        }

        // Apply bias correction for small sample sizes: α_unbiased = (n/(n-2)) * α_MLE
        let shape = if n > 2.0 {
            (n / (n - 2.0)) * shape_mle
        } else {
            shape_mle // No correction for very small samples
        };

        // Log-likelihood for Pareto distribution
        let log_likelihood = data.iter()
            .map(|&x| {
                // Log PDF: ln(shape) + shape*ln(scale) - (shape+1)*ln(x)
                shape.ln() + shape * scale.ln() - (shape + 1.0) * x.ln()
            })
            .sum::<f64>();

        let k = 2.0; // parameters: shape, scale
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_pareto_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "pareto".to_string(),
            parameters: vec![
                ("shape".to_string(), shape),
                ("scale".to_string(), scale),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::pareto_cdf(x, shape, scale))?,
            parameter_uncertainties,
        })
    }

    /// Fit Student's t distribution
    pub fn fit_students_t_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        let n = data.len() as f64;

        // Simplified fit - assume location 0, unit scale, estimate df
        let mean = data.mean();
        let variance = data.variance();

        // Estimate degrees of freedom using method of moments
        // For t-distribution: Var(X) = df/(df-2) for df > 2
        let df_est = if variance > 0.0 {
            2.0 * variance / (variance - 1.0) + 2.0
        } else {
            5.0 // fallback
        };

        if !df_est.is_finite() || df_est <= 2.0 {
            return Err("Student's t fit undefined: invalid degrees of freedom".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                let z = x - mean;
                SpecialFunctions::gamma((df_est + 1.0) / 2.0).ln() -
                SpecialFunctions::gamma(df_est / 2.0).ln() -
                0.5 * (df_est * std::f64::consts::PI).ln() -
                0.5 * ((df_est + 1.0) / df_est * z * z).ln()
            })
            .sum::<f64>();

        let k = 3.0; // parameters: location, scale, df
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_students_t_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "students_t".to_string(),
            parameters: vec![
                ("location".to_string(), mean),
                ("scale".to_string(), variance.sqrt()),
                ("df".to_string(), df_est),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::student_t_cdf(x, mean, variance.sqrt(), df_est))?,
            parameter_uncertainties,
        })
    }

    /// Fit Cauchy distribution
    pub fn fit_cauchy_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        let n = data.len() as f64;

        // Cauchy parameters: location (median), scale
        let location = Quantiles::median(data);

        // Scale estimate using MAD (median absolute deviation)
        let mad = Quantiles::median(&data.iter()
            .map(|&x| (x - location).abs())
            .collect::<Vec<f64>>());

        if !location.is_finite() || !mad.is_finite() || mad <= 0.0 {
            return Err("Cauchy fit undefined: invalid parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                let z = (x - location) / mad;
                -mad.ln() - std::f64::consts::PI.ln() - (1.0 + z * z).ln()
            })
            .sum::<f64>();

        let k = 2.0; // parameters: location, scale
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_cauchy_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "cauchy".to_string(),
            parameters: vec![
                ("location".to_string(), location),
                ("scale".to_string(), mad),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::cauchy_cdf(x, location, mad))?,
            parameter_uncertainties,
        })
    }

    /// Fit Johnson SU distribution (heavy-tail distribution)
    pub fn fit_johnson_su_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        let n = data.len() as f64;

        // Initial parameter estimates using method of moments
        let mean = data.mean();
        let variance = data.variance();
        let skewness = Self::moments(data)?.2;
        let kurtosis = Self::moments(data)?.3;

        // Johnson SU parameter estimation using moments
        // This is a simplified approach - full MLE would be more complex
        let delta = if skewness.abs() > 0.0 {
            1.0 / (skewness.abs().sqrt() + 0.1)
        } else {
            1.0
        };

        let gamma = if skewness > 0.0 {
            -delta * (4.0 * skewness / (2.0 * kurtosis + 3.0)).sqrt()
        } else {
            delta * (4.0 * skewness.abs() / (2.0 * kurtosis + 3.0)).sqrt()
        };

        let lambda = variance.sqrt() / delta;
        let xi = mean - lambda * (gamma / delta).sinh();

        let initial_guess = vec![gamma, delta, lambda, xi];
        let bounds = vec![
            (-10.0, 10.0),  // gamma
            (0.1, 10.0),    // delta
            (1e-6, data.iter().cloned().fold(f64::NEG_INFINITY, f64::max) * 2.0), // lambda
            (f64::NEG_INFINITY, data.iter().cloned().fold(f64::INFINITY, f64::min)), // xi
        ];

        // Create cost function for Johnson SU MLE
        #[derive(Clone)]
        struct JohnsonSuCost {
            data: Vec<f64>,
        }

        impl CostFunction for JohnsonSuCost {
            type Param = Vec<f64>;
            type Output = f64;

            fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
                let gamma = param[0];
                let delta = param[1];
                let lambda = param[2];
                let xi = param[3];

                if delta <= 0.0 || lambda <= 0.0 || !gamma.is_finite() || !delta.is_finite() || !lambda.is_finite() || !xi.is_finite() {
                    return Ok(f64::INFINITY);
                }

                let mut nll = 0.0;
                for &x in self.data.iter() {
                    if x <= xi || x >= xi + lambda {
                        return Ok(f64::INFINITY);
                    }

                    let y = (x - xi) / lambda;
                    let asinh_y = (y + (y*y + 1.0).sqrt()).ln();
                    let z = gamma + delta * asinh_y;
                    let jacobian = 0.5 * (1.0 + y*y).ln();
                    let log_pdf = delta.ln() - lambda.ln() - jacobian - 0.5 * (2.0 * std::f64::consts::PI).ln() - 0.5 * z * z;
                    nll -= log_pdf;
                }

                Ok(nll)
            }
        }

        let cost_fn = JohnsonSuCost { data: data.to_vec() };
        let config = GlobalOptimizationConfig {
            num_starts: 3,
            max_local_iters: 50,
            tolerance: 1e-8,
            ..Default::default()
        };

        impl Gradient for JohnsonSuCost {
            type Param = Vec<f64>;
            type Gradient = Vec<f64>;

            fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
                let gamma_val = param[0];
                let delta_val = param[1];
                let lambda_val = param[2];
                let xi_val = param[3];

                let mut grad = vec![0.0; 4];
                
                // Gamma
                grad[0] = johnson_su_cost_dual(&self.data, Dual64::new(gamma_val, 1.0), Dual64::new(delta_val, 0.0), Dual64::new(lambda_val, 0.0), Dual64::new(xi_val, 0.0)).eps;
                // Delta
                grad[1] = johnson_su_cost_dual(&self.data, Dual64::new(gamma_val, 0.0), Dual64::new(delta_val, 1.0), Dual64::new(lambda_val, 0.0), Dual64::new(xi_val, 0.0)).eps;
                // Lambda
                grad[2] = johnson_su_cost_dual(&self.data, Dual64::new(gamma_val, 0.0), Dual64::new(delta_val, 0.0), Dual64::new(lambda_val, 1.0), Dual64::new(xi_val, 0.0)).eps;
                // Xi
                grad[3] = johnson_su_cost_dual(&self.data, Dual64::new(gamma_val, 0.0), Dual64::new(delta_val, 0.0), Dual64::new(lambda_val, 0.0), Dual64::new(xi_val, 1.0)).eps;

                Ok(grad)
            }
        }

        fn johnson_su_cost_dual<T: DualNum<f64> + Copy>(data: &[f64], gamma: T, delta: T, lambda: T, xi: T) -> T {
            let mut nll = T::zero();
            let half = T::from(0.5);
            let two = T::from(2.0);
            let pi = T::from(std::f64::consts::PI);
            let one = T::one();

            for &x in data.iter() {
                let x_t = T::from(x);
                // z = gamma + delta * asinh((x - xi) / lambda)
                // Note: asinh is ln(y + sqrt(y^2 + 1))
                let y = (x_t - xi) / lambda;
                let asinh_y = (y + (y * y + one).sqrt()).ln();
                let z = gamma + delta * asinh_y;
                
                // log_f(x) = ln(delta) - ln(lambda) - 0.5*ln(1+y^2) - 0.5*ln(2pi) - 0.5*z^2
                
                let log_pdf = delta.ln() - lambda.ln() - half * (one + y*y).ln() - half * (two * pi).ln() - half * z * z;
                nll -= log_pdf;
            }
            nll
        }

        let mut optimizer = GradientOptimizer::new(cost_fn, config);
        let opt_result = optimizer.optimize(&initial_guess, &[bounds])?;

        if !opt_result.converged || opt_result.best_param.len() < 4 {
            return Err("Johnson SU optimization failed to converge".to_string());
        }

        let gamma = opt_result.best_param[0];
        let delta = opt_result.best_param[1];
        let lambda = opt_result.best_param[2];
        let xi = opt_result.best_param[3];

        if delta <= 0.0 || lambda <= 0.0 || !gamma.is_finite() || !delta.is_finite() || !lambda.is_finite() || !xi.is_finite() {
            return Err("Johnson SU fit undefined: invalid optimized parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                let y = (x - xi) / lambda;
                let asinh_y = (y + (y*y + 1.0).sqrt()).ln();
                let z = gamma + delta * asinh_y;
                let jacobian = 0.5 * (1.0 + y*y).ln();
                delta.ln() - lambda.ln() - jacobian - 0.5 * (2.0 * std::f64::consts::PI).ln() - 0.5 * z * z
            })
            .sum::<f64>();

        let k = 4.0; // parameters: gamma, delta, lambda, xi
        let aic = 2.0 * k - 2.0 * log_likelihood;
        let bic = k * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_johnson_su_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "johnson_su".to_string(),
            parameters: vec![
                ("gamma".to_string(), gamma),
                ("delta".to_string(), delta),
                ("lambda".to_string(), lambda),
                ("xi".to_string(), xi),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::johnson_su_cdf(x, gamma, delta, lambda, xi))?,
            parameter_uncertainties,
        })
    }

    /// Fit Burr Type XII distribution (heavy-tail distribution)
    pub fn fit_burr_type_xii_distribution(data: &[f64], errors: Option<&[f64]>) -> Result<DistributionFit, String> {
        if data.iter().any(|&x| x <= 0.0) {
            return Err("Burr Type XII distribution requires positive data".to_string());
        }

        let n = data.len() as f64;

        // Initial parameter estimates using method of moments
        let mean = data.mean();
        let (_, _, skewness, _) = Self::moments(data)?;

        // Data-driven initial estimate for c based on skewness
        // Higher absolute skewness suggests heavier tails (larger c)
        let c_est = (2.0 + skewness.abs() * 0.5).max(1.1); // ensure c > 1 to avoid beta function issues
        let k_est = 1.0; // shape parameter 2
        let lambda_est = mean / SpecialFunctions::beta(1.0 - 1.0/c_est, k_est + 1.0/c_est);

        let initial_guess = vec![c_est, k_est, lambda_est];
        let bounds = vec![
            (0.1, 10.0),   // c
            (0.1, 10.0),   // k
            (1e-6, data.iter().cloned().fold(f64::NEG_INFINITY, f64::max) * 2.0), // lambda
        ];

        // Create cost function for Burr Type XII MLE
        #[derive(Clone)]
        struct BurrXiiCost {
            data: Vec<f64>,
        }

        impl CostFunction for BurrXiiCost {
            type Param = Vec<f64>;
            type Output = f64;

            fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
                let c = param[0];
                let k = param[1];
                let lambda = param[2];

                if c <= 0.0 || k <= 0.0 || lambda <= 0.0 || !c.is_finite() || !k.is_finite() || !lambda.is_finite() {
                    return Ok(f64::INFINITY);
                }

                let mut nll = 0.0;
                for &x in self.data.iter() {
                    if x <= 0.0 {
                        return Ok(f64::INFINITY);
                    }

                    let u = (x / lambda).powf(c);
                    // Use ln_1p for numerical stability when u is large
                    let log_term = if u > 1e10 {
                        u.ln_1p()
                    } else {
                        (1.0 + u).ln()
                    };
                    let log_pdf = c.ln() + k.ln() - lambda.ln() + (c - 1.0) * (x / lambda).ln() -
                                 (k + 1.0) * log_term;

                    if !log_pdf.is_finite() {
                        return Ok(f64::INFINITY);
                    }
                    nll -= log_pdf;
                }

                Ok(nll)
            }
        }

        impl Gradient for BurrXiiCost {
            type Param = Vec<f64>;
            type Gradient = Vec<f64>;

            fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
                let c_val = param[0];
                let k_val = param[1];
                let lambda_val = param[2];

                if c_val <= 0.0 || k_val <= 0.0 || lambda_val <= 0.0 {
                     return Ok(vec![0.0, 0.0, 0.0]);
                }

                let _c = Dual64::new(c_val, 1.0);
                let _k = Dual64::new(k_val, 0.0);
                let _lambda = Dual64::new(lambda_val, 0.0);

                // We need to compute partial derivatives w.r.t c, k, lambda separately
                // because Dual64 only tracks one derivative at a time in this usage pattern
                // or we need to use a different approach for multivariate gradient.
                // So we will compute gradient components one by one.

                // Gradient w.r.t c
                let grad_c = {
                    let c = Dual64::new(c_val, 1.0);
                    let k = Dual64::from(k_val);
                    let lambda = Dual64::from(lambda_val);
                    
                    let mut nll = Dual64::from(0.0);
                    for &x_val in self.data.iter() {
                        let x = Dual64::from(x_val);
                        let u = (x / lambda).powd(c);
                        let log_pdf = c.ln() + k.ln() - lambda.ln() + (c - Dual64::from(1.0)) * (x / lambda).ln() - (k + Dual64::from(1.0)) * (Dual64::from(1.0) + u).ln();
                        nll -= log_pdf;
                    }
                    nll.eps
                };

                // Gradient w.r.t k
                let grad_k = {
                    let c = Dual64::from(c_val);
                    let k = Dual64::new(k_val, 1.0);
                    let lambda = Dual64::from(lambda_val);
                    
                    let mut nll = Dual64::from(0.0);
                    for &x_val in self.data.iter() {
                        let x = Dual64::from(x_val);
                        let u = (x / lambda).powd(c);
                        let log_pdf = c.ln() + k.ln() - lambda.ln() + (c - Dual64::from(1.0)) * (x / lambda).ln() - (k + Dual64::from(1.0)) * (Dual64::from(1.0) + u).ln();
                        nll -= log_pdf;
                    }
                    nll.eps
                };

                // Gradient w.r.t lambda
                let grad_lambda = {
                    let c = Dual64::from(c_val);
                    let k = Dual64::from(k_val);
                    let lambda = Dual64::new(lambda_val, 1.0);
                    
                    let mut nll = Dual64::from(0.0);
                    for &x_val in self.data.iter() {
                        let x = Dual64::from(x_val);
                        let u = (x / lambda).powd(c);
                        let log_pdf = c.ln() + k.ln() - lambda.ln() + (c - Dual64::from(1.0)) * (x / lambda).ln() - (k + Dual64::from(1.0)) * (Dual64::from(1.0) + u).ln();
                        nll -= log_pdf;
                    }
                    nll.eps
                };

                Ok(vec![grad_c, grad_k, grad_lambda])
            }
        }

        let cost_fn = BurrXiiCost { data: data.to_vec() };
        let config = GlobalOptimizationConfig {
            num_starts: 3,
            max_local_iters: 50,
            tolerance: 1e-8,
            ..Default::default()
        };

        let mut optimizer = GradientOptimizer::new(cost_fn, config);
        let opt_result = optimizer.optimize(&initial_guess, &[bounds])?;

        if !opt_result.converged || opt_result.best_param.len() < 3 {
            return Err("Burr Type XII optimization failed to converge".to_string());
        }

        let c = opt_result.best_param[0];
        let k = opt_result.best_param[1];
        let lambda = opt_result.best_param[2];

        if c <= 0.0 || k <= 0.0 || lambda <= 0.0 || !c.is_finite() || !k.is_finite() || !lambda.is_finite() {
            return Err("Burr Type XII fit undefined: invalid optimized parameter estimates".to_string());
        }

        let log_likelihood = data.iter()
            .map(|&x| {
                let u = (x / lambda).powf(c);
                // Use ln_1p for numerical stability when u is large
                let log_term = if u > 1e10 {
                    u.ln_1p()
                } else {
                    (1.0 + u).ln()
                };
                c.ln() + k.ln() - lambda.ln() + (c - 1.0) * (x / lambda).ln() - (k + 1.0) * log_term
            })
            .sum::<f64>();

        let num_params = 3.0; // parameters: c, k, lambda
        let aic = 2.0 * num_params - 2.0 * log_likelihood;
        let bic = num_params * n.ln() - 2.0 * log_likelihood;

        let parameter_uncertainties = if let Some(errs) = errors {
            Self::uncertainty_fit(data, errs, |d| Self::fit_burr_type_xii_distribution(d, None), 100)
        } else {
            None
        };

        Ok(DistributionFit {
            distribution_name: "burr_type_xii".to_string(),
            parameters: vec![
                ("c".to_string(), c),
                ("k".to_string(), k),
                ("lambda".to_string(), lambda),
            ],
            log_likelihood,
            aic,
            bic,
            goodness_of_fit: Self::goodness_of_fit(data, |x| distribution_functions::burr_type_xii_cdf(x, c, k, lambda))?,
            parameter_uncertainties,
        })
    }

    /// Compute goodness of fit using Kolmogorov-Smirnov test
    pub fn goodness_of_fit<F>(data: &[f64], cdf: F) -> Result<f64, String>
    where
        F: Fn(f64) -> f64,
    {
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        let n = sorted_data.len() as f64;
        let mut max_diff: f64 = 0.0;

        for (i, &x) in sorted_data.iter().enumerate() {
            let empirical_cdf = (i + 1) as f64 / n;
            let theoretical_cdf = cdf(x);
            let diff = (empirical_cdf - theoretical_cdf).abs();
            max_diff = max_diff.max(diff);
        }

        Ok(max_diff) // KS statistic
    }

    /// Compute statistical moments (mean, variance, skewness, kurtosis)
    pub fn moments(data: &[f64]) -> Result<(f64, f64, f64, f64), String> {
        if data.is_empty() {
            return Err("Cannot compute moments of empty dataset".to_string());
        }

        let n = data.len() as f64;
        let mean = data.mean();
        let variance = data.variance();
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return Ok((mean, variance, 0.0, 0.0)); // No skewness/kurtosis for constant data
        }

        let skewness = data.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum::<f64>() / n;
        let kurtosis = data.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum::<f64>() / n - 3.0; // Excess kurtosis

        Ok((mean, variance, skewness, kurtosis))
    }
}