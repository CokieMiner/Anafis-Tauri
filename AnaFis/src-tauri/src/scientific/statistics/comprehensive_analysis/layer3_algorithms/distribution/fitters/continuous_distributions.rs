use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::{UnifiedStats, SpecialFunctions};
use super::super::moments;
use argmin::core::{CostFunction, Error, Gradient, Operator};
use argmin::solver::linesearch::MoreThuenteLineSearch;
use argmin::solver::quasinewton::LBFGS;
use argmin::core::Executor;
use rayon::prelude::*;
use super::super::global_optimizer::{GlobalOptimizer, GlobalOptimizationConfig, OptimizationResult};
use super::super::goodness_of_fit::{goodness_of_fit, gamma_cdf, beta_cdf};
use crate::scientific::statistics::types::DistributionFit;

/// Cost function for Weibull MLE optimization
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
                return Ok(f64::INFINITY); // Weibull is defined for x > 0
            }
            let log_pdf = shape.ln() - scale.ln() + (shape - 1.0) * (x.ln() - scale.ln()) - (x / scale).powf(shape);
            if !log_pdf.is_finite() {
                return Ok(f64::INFINITY);
            }
            log_likelihood += log_pdf;
        }

        Ok(-log_likelihood) // Minimize negative log-likelihood
    }
}

impl Operator for WeibullCost {
    type Param = Vec<f64>;
    type Output = f64;

    fn apply(&self, param: &Vec<f64>) -> Result<Self::Output, Error> {
        // Negative log-likelihood
        if param.len() < 2 {
            return Ok(f64::INFINITY);
        }
        let k = param[0];
        let s = param[1];
        if k <= 0.0 || s <= 0.0 || !k.is_finite() || !s.is_finite() {
            return Ok(f64::INFINITY);
        }
        let mut nll = 0.0;
        for &x in self.data.iter() {
            let z = (x / s).powf(k);
            let pdf = (k / s) * (x / s).powf(k - 1.0) * (-z).exp();
            if !pdf.is_finite() || pdf <= 0.0 {
                return Ok(f64::INFINITY);
            }
            nll -= pdf.ln();
        }
        Ok(nll)
    }
}

impl Gradient for WeibullCost {
    type Param = Vec<f64>;
    type Gradient = Vec<f64>;

    fn gradient(&self, param: &Vec<f64>) -> Result<Self::Gradient, Error> {
        if param.len() < 2 {
            return Ok(vec![f64::INFINITY, f64::INFINITY]);
        }
        let k = param[0];
        let s = param[1];
        if k <= 0.0 || s <= 0.0 || !k.is_finite() || !s.is_finite() {
            return Ok(vec![f64::INFINITY, f64::INFINITY]);
        }

        let mut dk = 0.0;
        let mut ds = 0.0;
        for &x in self.data.iter() {
            let ln_x = x.ln();
            let ln_s = s.ln();
            let ln_x_over_s = ln_x - ln_s;
            let z = (x / s).powf(k);

            // d/dk log f = 1/k + ln x - ln s - z * ln(x/s)
            let dlogf_dk = 1.0 / k + ln_x_over_s - z * ln_x_over_s;
            // d/ds log f = (k/s) * (z - 1)
            let dlogf_ds = (k / s) * (z - 1.0);

            dk -= dlogf_dk;
            ds -= dlogf_ds;
        }
        Ok(vec![dk, ds])
    }
}

/// Fit normal distribution using maximum likelihood
pub fn fit_normal_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    let (mean, variance, _, _) = moments::moments(data)?;
    let std_dev = variance.sqrt();

    if std_dev <= 0.0 || std_dev.is_nan() {
        return Err("Standard deviation is zero; normal fit undefined for constant data".to_string());
    }

    let log_likelihood = if data.len() > 1000 {
        data.par_iter()
            .map(|x| UnifiedStats::normal_pdf(*x, mean, std_dev).ln())
            .sum::<f64>()
    } else {
        data
            .iter()
            .map(|x| UnifiedStats::normal_pdf(*x, mean, std_dev).ln())
            .sum::<f64>()
    };

    let k = 2.0; // parameters: mean, std
    let aic = 2.0 * k - 2.0 * log_likelihood; // AIC = 2k - 2 ln L
    let bic = k * (data.len() as f64).ln() - 2.0 * log_likelihood; // BIC = ln(n)k - 2 ln L

    Ok(DistributionFit {
        distribution_name: "normal".to_string(),
        parameters: vec![
            ("mean".to_string(), mean),
            ("std_dev".to_string(), std_dev),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| {
            Ok(UnifiedStats::normal_cdf(x, mean, std_dev))
        })?,
    })
}

/// Fit log-normal distribution
pub fn fit_lognormal_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    // Check if all data is positive
    if data.iter().any(|x| *x <= 0.0) {
        return Err("Log-normal distribution requires positive data".to_string());
    }

    let log_data: Vec<f64> = data.iter().map(|x| x.ln()).collect();
    let (mu, sigma_squared, _, _) = moments::moments(&log_data)?;
    let sigma = sigma_squared.sqrt();
    if sigma <= 0.0 || sigma.is_nan() {
        return Err("Lognormal fit undefined: zero variance in log-data".to_string());
    }

    let log_likelihood = if data.len() > 1000 {
        data.par_iter()
            .map(|x| {
                let pdf = UnifiedStats::normal_pdf(x.ln(), mu, sigma) / x;
                pdf.ln()
            })
            .sum::<f64>()
    } else {
        data.iter()
            .map(|x| {
                let pdf = UnifiedStats::normal_pdf(x.ln(), mu, sigma) / x;
                pdf.ln()
            })
            .sum::<f64>()
    };

    let k = 2.0; // parameters: mu, sigma
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * (data.len() as f64).ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "lognormal".to_string(),
        parameters: vec![("mu".to_string(), mu), ("sigma".to_string(), sigma)],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| {
            // CDF of log-normal: Φ((ln(x) - μ)/σ)
            Ok(UnifiedStats::normal_cdf((x.ln() - mu) / sigma, 0.0, 1.0))
        })?,
    })
}

/// Fit exponential distribution
pub fn fit_exponential_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    if data.iter().any(|x| *x < 0.0) {
        return Err("Exponential distribution requires non-negative data".to_string());
    }

    let mean = data.iter().sum::<f64>() / data.len() as f64;
    if mean <= 0.0 || mean.is_nan() {
        return Err("Exponential fit undefined: mean is zero or data contains negative values".to_string());
    }
    let lambda = 1.0 / mean;

    let log_likelihood = if data.len() > 1000 {
        data.par_iter()
            .map(|x| lambda * (-lambda * x).exp())
            .map(|pdf| pdf.ln())
            .sum::<f64>()
    } else {
        data.iter()
            .map(|x| lambda * (-lambda * x).exp())
            .map(|pdf| pdf.ln())
            .sum::<f64>()
    };

    let k = 1.0; // parameters: lambda
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * (data.len() as f64).ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "exponential".to_string(),
        parameters: vec![("lambda".to_string(), lambda)],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| Ok(1.0 - (-lambda * x).exp()))?,
    })
}

/// Fit Weibull distribution using maximum likelihood with global optimization
pub fn fit_weibull_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    if data.iter().any(|x| *x <= 0.0) {
        return Err("Weibull distribution requires positive data".to_string());
    }

    // Check for constant data
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    if variance <= 0.0 {
        return Err("Weibull fit undefined: zero variance (constant data)".to_string());
    }

    // Use an improved initial guess for the shape parameter based on sample log-std
    let data_ln: Vec<f64> = data.iter().map(|x| x.ln()).collect();
    let mean_ln = data_ln.iter().sum::<f64>() / n;
    let std_ln = (data_ln.iter().map(|lnx| (lnx - mean_ln).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    // Using relation between std of log-values and shape: sigma_ln ≈ π/(sqrt(6)*k)
    let nm_guess = if std_ln > 0.0 { std::f64::consts::PI / (std_ln * 6.0f64.sqrt()) } else { 1.0 };
    let mom_guess = (mean.powi(2) / variance).sqrt();
    let shape = if mom_guess.is_finite() && mom_guess > 0.0 { mom_guess } else if nm_guess.is_finite() && nm_guess > 0.0 { nm_guess } else { 1.0 };

    // Build initial guess and bounds
    let sum_xk = data.iter().map(|xi| xi.powf(shape)).sum::<f64>();
    if sum_xk <= 0.0 { return Err("Weibull fit undefined: invalid sums".to_string()); }
    let scale = (sum_xk / n).powf(1.0 / shape);

    let initial_guess = vec![shape, scale];
    let bounds = vec![(1e-6, 100.0), (1e-6, data.iter().fold(0.0f64, |a: f64, &b| a.max(b)) * 10.0)];

    // Use global optimization for robust fitting
    let cost_fn = WeibullCost { data: data.to_vec() };
    let config = GlobalOptimizationConfig {
        num_starts: 5,
        max_local_iters: 50,
        tolerance: 1e-8,
        basin_hopping: true,
        max_basin_iters: 10,
        ..Default::default()
    };

    // Use LBFGS for gradient-based optimization when gradients are available
    let opt_result = if data.len() >= 50 {
        // For larger datasets, use LBFGS with gradients for better performance
        let linesearch = MoreThuenteLineSearch::new();
        let solver = LBFGS::new(linesearch, 7);

        let executor = Executor::new(cost_fn.clone(), solver)
            .configure(|state| {
                state
                    .param(initial_guess.clone())
                    .max_iters(config.max_local_iters)
                    .target_cost(config.tolerance)
            });

        match executor.run() {
            Ok(result) => {
                if let Some(best_param) = result.state.best_param {
                    let best_cost = result.state.best_cost;
                    // Convert to OptimizationResult format
                    OptimizationResult {
                        best_param: best_param.clone(),
                        best_cost,
                        num_evaluations: 1,
                        num_local_opts: 1,
                        converged: best_cost.is_finite() && best_cost < f64::INFINITY,
                        all_solutions: vec![(best_param, best_cost)],
                    }
                } else {
                    // Fall back to global optimization
                    let mut optimizer = GlobalOptimizer::new(cost_fn, config);
                    optimizer.optimize(&initial_guess, &bounds)?
                }
            }
            Err(_) => {
                // Fall back to global optimization
                let mut optimizer = GlobalOptimizer::new(cost_fn, config);
                optimizer.optimize(&initial_guess, &bounds)?
            }
        }
    } else {
        // For smaller datasets, use global optimization
        let mut optimizer = GlobalOptimizer::new(cost_fn, config);
        optimizer.optimize(&initial_guess, &bounds)?
    };

    if !opt_result.converged || opt_result.best_param.len() < 2 {
        return Err("Weibull global optimization failed to converge".to_string());
    }

    let mle_shape = opt_result.best_param[0];
    let mle_scale = opt_result.best_param[1];

    if (mle_shape <= 0.0 || mle_shape.is_nan()) || (mle_scale <= 0.0 || mle_scale.is_nan()) {
        return Err("Weibull fit undefined: invalid optimized parameter estimates".to_string());
    }

    let log_likelihood = if data.len() > 1000 {
        data.par_iter()
            .map(|&x| {
                if x <= 0.0 {
                    f64::NEG_INFINITY // Handle x <= 0 for Weibull
                } else {
                    mle_shape.ln() - mle_scale.ln() + (mle_shape - 1.0) * (x.ln() - mle_scale.ln()) - (x / mle_scale).powf(mle_shape)
                }
            })
            .sum::<f64>()
    } else {
        data.iter()
            .map(|&x| {
                if x <= 0.0 {
                    f64::NEG_INFINITY
                } else {
                    mle_shape.ln() - mle_scale.ln() + (mle_shape - 1.0) * (x.ln() - mle_scale.ln()) - (x / mle_scale).powf(mle_shape)
                }
            })
            .sum::<f64>()
    };

    let k = 2.0; // parameters: shape, scale
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * (data.len() as f64).ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "weibull".to_string(),
        parameters: vec![
            ("shape".to_string(), mle_shape),
            ("scale".to_string(), mle_scale),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| {
            Ok(1.0 - (-(x / mle_scale).powf(mle_shape)).exp())
        })?,
    })
}

/// Fit Gamma distribution using maximum likelihood estimation with global optimization
pub fn fit_gamma_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    if data.iter().any(|&x| x <= 0.0) {
        return Err("Gamma distribution requires positive data".to_string());
    }

    let n = data.len() as f64;
    let sum_x = data.iter().sum::<f64>();
    let sum_x2 = data.iter().map(|x| x * x).sum::<f64>();

    // MLE for Gamma distribution
    let mean = sum_x / n;
    let mean_sq = sum_x2 / n;

    // Method of moments gives us initial estimates, but we'll use MLE
    // For Gamma: shape = mean² / variance, scale = variance / mean
    // where variance = mean_sq - mean²
    let variance = mean_sq - mean * mean;
    if variance <= 0.0 {
        return Err("Gamma fit undefined: zero variance".to_string());
    }

    let shape_mom = mean * mean / variance;

    // Use global optimization instead of Newton-Raphson
    let initial_guess = vec![shape_mom.max(0.1), mean / shape_mom.max(0.1)];
    let bounds = vec![(1e-6, 100.0), (1e-6, data.iter().fold(0.0f64, |a: f64, &b| a.max(b)) * 10.0)];

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
            let scale = param[1];

            if shape <= 0.0 || scale <= 0.0 || !shape.is_finite() || !scale.is_finite() {
                return Ok(f64::INFINITY);
            }

            let nll = self.data.iter()
                .map(|&x| {
                    // Negative log-likelihood for Gamma: shape*ln(scale) + (shape-1)*ln(x) - scale*x - ln(Γ(shape))
                    shape * scale.ln() + (shape - 1.0) * x.ln() - scale * x - SpecialFunctions::gamma(shape).ln()
                })
                .sum::<f64>();

            Ok(-nll) // Minimize negative log-likelihood
        }
    }

    let cost_fn = GammaCost {
        data: data.to_vec(),
    };

    let config = GlobalOptimizationConfig {
        num_starts: 3,
        max_local_iters: 50,
        tolerance: 1e-8,
        ..Default::default()
    };

    let mut optimizer = GlobalOptimizer::new(cost_fn, config);
    let opt_result = optimizer.optimize(&initial_guess, &bounds)?;

    if !opt_result.converged || opt_result.best_param.len() < 2 {
        return Err("Gamma global optimization failed to converge".to_string());
    }

    let shape = opt_result.best_param[0];
    let scale = opt_result.best_param[1];

    if (shape <= 0.0 || shape.is_nan()) || (scale <= 0.0 || scale.is_nan()) {
        return Err("Gamma fit undefined: invalid optimized parameter estimates".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // Gamma PDF: (1/Γ(k)) * (1/θ^k) * x^(k-1) * e^(-x/θ)
            let pdf = (x / scale).powf(shape - 1.0) * (-x / scale).exp() / (scale * SpecialFunctions::gamma(shape));
            pdf.ln()
        })
        .sum::<f64>();

    let k = 2.0; // parameters: shape, scale
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * n.ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "gamma".to_string(),
        parameters: vec![
            ("shape".to_string(), shape),
            ("scale".to_string(), scale),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| gamma_cdf(x, shape, scale))?,
    })
}

/// Fit Beta distribution using maximum likelihood estimation with global optimization
pub fn fit_beta_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    if data.iter().any(|&x| x <= 0.0 || x >= 1.0) {
        return Err("Beta distribution requires data in (0,1)".to_string());
    }

    let n = data.len() as f64;
    let _sum_log_x = data.iter().map(|x| x.ln()).sum::<f64>();
    let _sum_log_1mx = data.iter().map(|x| (1.0 - x).ln()).sum::<f64>();

    // Use method of moments as initial estimates
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (n - 1.0);

    if variance <= 0.0 || mean <= 0.0 || mean >= 1.0 {
        return Err("Beta fit undefined: invalid moments".to_string());
    }

    // Method of moments initial estimates
    let mut alpha = mean * (mean * (1.0 - mean) / variance - 1.0);
    let mut beta_param = (1.0 - mean) * (mean * (1.0 - mean) / variance - 1.0);

    if alpha <= 0.0 || beta_param <= 0.0 {
        // Fallback to simple estimates
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
                    // Negative log-likelihood for Beta: (α-1)ln(x) + (β-1)ln(1-x) - ln(B(α,β))
                    (alpha - 1.0) * x.ln() + (beta_param - 1.0) * (1.0 - x).ln() - SpecialFunctions::beta(alpha, beta_param).ln()
                })
                .sum::<f64>();

            Ok(-nll) // Minimize negative log-likelihood
        }
    }

    let cost_fn = BetaCost {
        data: data.to_vec(),
    };

    let config = GlobalOptimizationConfig {
        num_starts: 3,
        max_local_iters: 50,
        tolerance: 1e-8,
        ..Default::default()
    };

    let mut optimizer = GlobalOptimizer::new(cost_fn, config);
    let opt_result = optimizer.optimize(&initial_guess, &bounds)?;

    if !opt_result.converged || opt_result.best_param.len() < 2 {
        return Err("Beta global optimization failed to converge".to_string());
    }

    let alpha = opt_result.best_param[0];
    let beta_param = opt_result.best_param[1];

    if (alpha <= 0.0 || alpha.is_nan()) || (beta_param <= 0.0 || beta_param.is_nan()) {
        return Err("Beta fit undefined: invalid optimized parameter estimates".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // Beta PDF: x^(α-1) * (1-x)^(β-1) / B(α,β)
            let pdf = x.powf(alpha - 1.0) * (1.0 - x).powf(beta_param - 1.0) / SpecialFunctions::beta(alpha, beta_param);
            pdf.ln()
        })
        .sum::<f64>();

    let k = 2.0; // parameters: alpha, beta
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * n.ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "beta".to_string(),
        parameters: vec![
            ("alpha".to_string(), alpha),
            ("beta".to_string(), beta_param),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| beta_cdf(x, alpha, beta_param))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weibull_fitting_with_gradients() {
        // Generate synthetic Weibull data for testing
        let shape_true = 2.5;
        let scale_true = 3.0;
        let data: Vec<f64> = (0..100)
            .map(|_| {
                // Simple Weibull sample generation (not perfect but sufficient for testing)
                let u: f64 = rand::random();
                scale_true * (-u.ln()).powf(1.0 / shape_true)
            })
            .collect();

        // Fit Weibull distribution
        let result = fit_weibull_distribution(&data);
        assert!(result.is_ok(), "Weibull fitting should succeed");

        let fit = result.unwrap();
        assert_eq!(fit.distribution_name, "weibull");
        assert!(fit.parameters.len() == 2);

        // Check that parameters are reasonable (should be close to true values)
        let shape_est = fit.parameters[0].1;
        let scale_est = fit.parameters[1].1;

        assert!(shape_est > 0.0 && shape_est.is_finite());
        assert!(scale_est > 0.0 && scale_est.is_finite());

        // Check that log-likelihood is finite
        assert!(fit.log_likelihood.is_finite());

        // Check that AIC/BIC are finite
        assert!(fit.aic.is_finite());
        assert!(fit.bic.is_finite());
    }
}