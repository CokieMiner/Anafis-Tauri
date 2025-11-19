use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats;
use crate::scientific::statistics::types::DistributionFit;
use super::super::goodness_of_fit::goodness_of_fit;
use argmin::core::{CostFunction, Error};
use super::super::global_optimizer::{GlobalOptimizer, GlobalOptimizationConfig};

/// Cost function for Gumbel MLE optimization
#[derive(Clone)]
struct GumbelCost {
    data: Vec<f64>,
}

impl CostFunction for GumbelCost {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        let mu = param[0];    // location
        let beta = param[1];  // scale

        if beta <= 0.0 || !mu.is_finite() || !beta.is_finite() {
            return Ok(f64::INFINITY);
        }

        let nll: f64 = self.data.iter()
            .map(|&x| {
                let z = (x - mu) / beta;
                // log(PDF) = -log(β) - z - exp(-z)
                let log_pdf = -beta.ln() - z - (-z).exp();
                -log_pdf // Return negative log-pdf for one sample
            })
            .sum();

        if !nll.is_finite() {
            return Ok(f64::INFINITY);
        }

        Ok(nll)
    }
}


/// Cost function for Burr Type XII MLE optimization
#[derive(Clone)]
struct BurrCost {
    data: Vec<f64>,
}

impl CostFunction for BurrCost {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        let c = param[0]; // shape
        let k = param[1]; // shape

        if c <= 0.0 || k <= 0.0 || !c.is_finite() || !k.is_finite() {
            return Ok(f64::INFINITY);
        }

        let nll: f64 = self.data.iter()
            .map(|&x| {
                // PDF = c*k * x^(c-1) / (1 + x^c)^(k+1)
                // log(PDF) = log(c) + log(k) + (c-1)*log(x) - (k+1)*log(1 + x^c)
                if x <= 0.0 {
                    // Return a large value for points outside the domain, effectively -log(0)
                    f64::INFINITY
                } else {
                    let log_pdf = c.ln() + k.ln() + (c - 1.0) * x.ln() - (k + 1.0) * (1.0 + x.powf(c)).ln();
                    -log_pdf // Return negative log-pdf for one sample
                }
            })
            .sum();

        if !nll.is_finite() {
            return Ok(f64::INFINITY);
        }

        Ok(nll)
    }
}


/// Fit Gumbel distribution (extreme value type I) using maximum likelihood
pub fn fit_gumbel_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    let n = data.len() as f64;
    if n < 2.0 {
        return Err("Gumbel fit requires at least 2 data points".to_string());
    }

    // Use method of moments as initial estimates
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (n - 1.0);

    if variance <= 0.0 {
        return Err("Gumbel fit undefined: zero variance (constant data)".to_string());
    }

    // For Gumbel: variance = (π²/6) * β², so β = sqrt(6*variance/π²)
    let beta_mom = (variance * 6.0 / std::f64::consts::PI.powi(2)).sqrt();
    // Euler-Mascheroni constant γ ≈ 0.5772156649
    let euler_gamma = 0.577_215_664_901_532_9;
    let mu_mom = mean - euler_gamma * beta_mom;

    let initial_guess = vec![mu_mom, beta_mom];
    let bounds = vec![ (data.iter().cloned().fold(f64::NEG_INFINITY, f64::max) * -10.0, data.iter().cloned().fold(f64::NEG_INFINITY, f64::max) * 10.0), (1e-6, variance.sqrt() * 10.0) ];

    let cost_fn = GumbelCost { data: data.to_vec() };
    let config = GlobalOptimizationConfig {
        num_starts: 5,
        max_local_iters: 50,
        ..Default::default()
    };

    let mut optimizer = GlobalOptimizer::new(cost_fn, config);
    let opt_result = optimizer.optimize(&initial_guess, &bounds)?;
    
    if !opt_result.converged || opt_result.best_param.len() < 2 {
        return Err("Gumbel global optimization failed to converge".to_string());
    }

    let mu = opt_result.best_param[0];
    let beta = opt_result.best_param[1];
    let log_likelihood = -opt_result.best_cost;


    if (beta <= 0.0 || beta.is_nan()) || !mu.is_finite() {
        return Err("Gumbel fit undefined: invalid MLE parameter estimates".to_string());
    }

    let k_params = 2.0;
    let aic = 2.0 * k_params - 2.0 * log_likelihood;
    let bic = k_params * n.ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "gumbel".to_string(),
        parameters: vec![
            ("location".to_string(), mu),
            ("scale".to_string(), beta),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| gumbel_cdf(x, mu, beta))?,
    })
}

/// Fit Pareto distribution (power-law tail)
pub fn fit_pareto_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    if data.iter().any(|&x| x <= 0.0) {
        return Err("Pareto distribution requires positive data".to_string());
    }

    let n = data.len() as f64;
    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);

    // Correct MLE for Pareto shape parameter
    let sum_log_x = data.iter().map(|&x| (x / min_val).ln()).sum::<f64>();
    if sum_log_x <= 0.0 {
        return Err("Pareto fit undefined: cannot compute shape from log-data.".to_string());
    }
    let shape = n / sum_log_x;

    if shape <= 0.0 || shape.is_nan() {
        return Err("Pareto fit undefined: invalid shape parameter".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // Pareto PDF: α * xₘ^α / x^(α+1)
            let pdf = shape * min_val.powf(shape) / x.powf(shape + 1.0);
            if pdf > 0.0 && pdf.is_finite() { pdf.ln() } else { f64::NEG_INFINITY }
        })
        .sum::<f64>();

    let k = 2.0; // parameters: shape, scale
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * n.ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "pareto".to_string(),
        parameters: vec![
            ("shape".to_string(), shape),
            ("scale".to_string(), min_val),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| pareto_cdf(x, shape, min_val))?,
    })
}

/// Gumbel CDF
fn gumbel_cdf(x: f64, location: f64, scale: f64) -> Result<f64, String> {
    let z = (x - location) / scale;
    Ok((-(-z).exp()).exp())
}

/// Pareto CDF
fn pareto_cdf(x: f64, shape: f64, scale: f64) -> Result<f64, String> {
    if x < scale {
        Ok(0.0)
    } else {
        Ok(1.0 - (scale / x).powf(shape))
    }
}

/// Fit Johnson's SU distribution (for data transformation to normality)
pub fn fit_johnson_su_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    let n = data.len() as f64;
    if n < 3.0 {
        return Err("Johnson SU fit requires at least 3 data points".to_string());
    }

    // Use method of moments to estimate parameters
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (n - 1.0);

    let skewness = data.iter()
        .map(|x| ((x - mean) / variance.sqrt()).powi(3))
        .sum::<f64>() / n;

    let kurtosis = data.iter()
        .map(|x| ((x - mean) / variance.sqrt()).powi(4))
        .sum::<f64>() / n - 3.0; // Excess kurtosis

    if variance <= 0.0 {
        return Err("Johnson SU fit undefined: insufficient variation.".to_string());
    }

    // This is a simplified estimation - full MLE would be more complex
    // Using a library or a proper numerical optimization is recommended for production
    let w = (2.0 * kurtosis - 3.0 * skewness.powi(2) + 2.0).sqrt();
    let omega = (w.powi(2) - 1.0).sqrt().asinh() / 2.0;
    if !omega.is_finite() || omega <= 0.0 { return Err("Failed to calculate Johnson SU omega parameter".to_string()); }

    let delta = 1.0 / omega.sinh().sqrt();
    let gamma = -skewness.signum() * delta * ((w.powi(2) - 1.0) / (2.0 * w.powi(2))).sqrt();
    let lambda = variance.sqrt() / (0.5 * (delta.powi(2) - 1.0) * (1.0 + 2.0 * gamma.powi(2))).sqrt();
    let xi = mean - lambda * (0.5 * (delta.powi(2) - 1.0)).sqrt() * gamma;
    
    if !gamma.is_finite() || !delta.is_finite() || !lambda.is_finite() || !xi.is_finite() || lambda <= 0.0 || delta <= 0.0 {
        return Err("Johnson SU fit undefined: invalid parameter estimates".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // Johnson SU PDF involves normal distribution after transformation
            let z = gamma + delta * ((x - xi) / lambda).asinh();
            let jacobian = delta / (lambda * (1.0 + ((x - xi) / lambda).powi(2)).sqrt());
            let normal_pdf = (-0.5 * z.powi(2)).exp() / (2.0 * std::f64::consts::PI).sqrt();
            (normal_pdf * jacobian).ln()
        })
        .sum::<f64>();

    let k = 4.0; // parameters: gamma, delta, lambda, xi
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * n.ln() - 2.0 * log_likelihood;

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
        goodness_of_fit: goodness_of_fit(data, |x| johnson_su_cdf(x, gamma, delta, lambda, xi))?,
    })
}

/// Fit Burr distribution (heavy-tail, flexible shape)
pub fn fit_burr_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    if data.iter().any(|&x| x <= 0.0) {
        return Err("Burr distribution requires positive data".to_string());
    }

    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    if variance <= 0.0 {
        return Err("Burr fit undefined: zero variance".to_string());
    }

    // Heuristic initial guess
    let initial_guess = vec![2.0, 1.0];
    let bounds = vec![(1e-6, 100.0), (1e-6, 100.0)];

    let cost_fn = BurrCost { data: data.to_vec() };
    let config = GlobalOptimizationConfig {
        num_starts: 5,
        max_local_iters: 50,
        tolerance: 1e-6,
        ..Default::default()
    };

    let mut optimizer = GlobalOptimizer::new(cost_fn, config);
    let opt_result = optimizer.optimize(&initial_guess, &bounds)?;

    if !opt_result.converged || opt_result.best_param.len() < 2 {
        return Err("Burr global optimization failed to converge".to_string());
    }
    
    let c = opt_result.best_param[0];
    let k = opt_result.best_param[1];

    let log_likelihood = -opt_result.best_cost;

    let k_params = 2.0;
    let aic = 2.0 * k_params - 2.0 * log_likelihood;
    let bic = k_params * n.ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "burr".to_string(),
        parameters: vec![
            ("c".to_string(), c),
            ("k".to_string(), k),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| burr_cdf(x, c, k))?,
    })
}

/// Johnson SU CDF
fn johnson_su_cdf(x: f64, gamma: f64, delta: f64, lambda: f64, xi: f64) -> Result<f64, String> {
    let z = gamma + delta * ((x - xi) / lambda).asinh();
    Ok(UnifiedStats::normal_cdf(z, 0.0, 1.0))
}

/// Burr CDF
fn burr_cdf(x: f64, c: f64, k: f64) -> Result<f64, String> {
    if x < 0.0 { Ok(0.0) } else { Ok(1.0 - (1.0 + x.powf(c)).powf(-k)) }
}