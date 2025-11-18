use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::StatisticalDistributions;
use super::super::moments;
use argmin::core::{CostFunction, Error, Executor, Gradient, Operator};
use argmin::solver::neldermead::NelderMead;
use argmin::solver::quasinewton::LBFGS;
use argmin::solver::linesearch::MoreThuenteLineSearch;
use rayon::prelude::*;
use statrs::distribution::ContinuousCDF;

use crate::scientific::statistics::types::DistributionFit;

/// Cost function for Weibull MLE optimization
#[derive(Clone)]
struct WeibullCost<'a> {
    data: &'a [f64],
}

impl<'a> CostFunction for WeibullCost<'a> {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        let shape = param[0];
        let scale = param[1];

        if shape <= 0.0 || scale <= 0.0 || !shape.is_finite() || !scale.is_finite() {
            return Ok(f64::INFINITY);
        }

        let log_likelihood = self.data.iter()
            .map(|&x| {
                let pdf = (shape / scale)
                    * (x / scale).powf(shape - 1.0)
                    * (-(x / scale).powf(shape)).exp();
                pdf.ln()
            })
            .sum::<f64>();

        Ok(-log_likelihood) // Minimize negative log-likelihood
    }
}

impl<'a> Operator for WeibullCost<'a> {
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

impl<'a> Gradient for WeibullCost<'a> {
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
            .map(|x| StatisticalDistributions::normal_pdf(*x, mean, std_dev).ln())
            .sum::<f64>()
    } else {
        data
            .iter()
            .map(|x| StatisticalDistributions::normal_pdf(*x, mean, std_dev).ln())
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
            StatisticalDistributions::normal_cdf(x, mean, std_dev)
        }).unwrap(),
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
                let pdf = StatisticalDistributions::normal_pdf(x.ln(), mu, sigma) / x;
                pdf.ln()
            })
            .sum::<f64>()
    } else {
        data.iter()
            .map(|x| {
                let pdf = StatisticalDistributions::normal_pdf(x.ln(), mu, sigma) / x;
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
            StatisticalDistributions::normal_cdf((x.ln() - mu) / sigma, 0.0, 1.0)
        }).unwrap(),
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
        goodness_of_fit: goodness_of_fit(data, |x| 1.0 - (-lambda * x).exp()).unwrap(),
    })
}

/// Fit Weibull distribution using maximum likelihood via Newton-Raphson
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
    let mom_guess = (variance / mean.powi(2)).sqrt().recip();
    let shape = if nm_guess.is_finite() && nm_guess > 0.0 { nm_guess } else if mom_guess.is_finite() && mom_guess > 0.0 { mom_guess } else { 1.0 };

    // Build Nelder-Mead optimization around initial guess and rely on argmin crate (well-tested) rather than a custom Newton-Raphson.
    let mut mle_shape = if shape.is_finite() && shape > 0.0 { shape } else { 1.0 };
    let sum_xk = data.iter().map(|xi| xi.powf(mle_shape)).sum::<f64>();
    if sum_xk <= 0.0 { return Err("Weibull fit undefined: invalid sums".to_string()); }
    let mut mle_scale = (sum_xk / n).powf(1.0 / mle_shape);

    // Use argmin's Nelder-Mead as the primary numeric optimizer for MLE (robust / well-tested)
    let cost = WeibullCost { data };

    // initial param: [shape, scale]
    let init_param = vec![mle_shape, mle_scale];
    // Build a small simplex (dim + 1 points) for Nelder-Mead
    let eps = 1e-6;
    let simplex: Vec<Vec<f64>> = vec![
        init_param.clone(),
        vec![init_param[0] * (1.0 + eps), init_param[1]],
        vec![init_param[0], init_param[1] * (1.0 + eps)],
    ];

    // Try LBFGS using the analytic gradient first (fast & robust), then fall back to Nelder-Mead
    // Additional solvers available in argmin crate for future enhancement:
    // - BFGS: Good for medium-sized problems with analytic gradients
    // - NonlinearConjugateGradient: Efficient for large-scale problems
    // - TrustRegion methods: Robust for ill-conditioned problems (requires Hessian)
    let linesearch = MoreThuenteLineSearch::new();
    let lbfgs = LBFGS::new(linesearch, 10);
    let mut accepted = false;
    if let Ok(res) = Executor::new(cost.clone(), lbfgs).configure(|state| state.param(init_param.clone()).max_iters(50)).run() {
        if let Some(best) = res.state.best_param {
            if best.len() >= 2 && best[0].is_finite() && best[1].is_finite() && best[0] > 0.0 && best[1] > 0.0 {
                mle_shape = best[0];
                mle_scale = best[1];
                accepted = true;
            }
        }
    }

    if !accepted {
        // Fall back to Nelder-Mead if LBFGS fails
        let solver = NelderMead::<Vec<f64>, f64>::new(simplex);
        if let Ok(res) = Executor::new(cost, solver).configure(|state| state.max_iters(50)).run() {
            if let Some(best) = res.state.best_param {
                if best.len() >= 2 && best[0].is_finite() && best[1].is_finite() && best[0] > 0.0 && best[1] > 0.0 {
                    mle_shape = best[0];
                    mle_scale = best[1];
                }
            }
        }
    }

    if (mle_shape <= 0.0 || mle_shape.is_nan()) || (mle_scale <= 0.0 || mle_scale.is_nan()) {
        return Err("Weibull fit undefined: invalid initial parameter estimates".to_string());
    }

    let log_likelihood = if data.len() > 1000 {
        data.par_iter()
            .map(|&x| {
                let pdf = (mle_shape / mle_scale)
                    * (x / mle_scale).powf(mle_shape - 1.0)
                    * (-(x / mle_scale).powf(mle_shape)).exp();
                pdf.ln()
            })
            .sum::<f64>()
    } else {
        data.iter()
            .map(|&x| {
                let pdf = (mle_shape / mle_scale)
                    * (x / mle_scale).powf(mle_shape - 1.0)
                    * (-(x / mle_scale).powf(mle_shape)).exp();
                pdf.ln()
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
            1.0 - (-(x / mle_scale).powf(mle_shape)).exp()
        }).unwrap(),
    })
}

/// Fit Gamma distribution using maximum likelihood estimation
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

    // Use Newton-Raphson to solve for shape parameter using digamma function
    let mut shape = shape_mom;
    let tolerance = 1e-8;
    let max_iter = 100;

    for _ in 0..max_iter {
        let digamma_k = digamma(shape);
        let trigamma_k = trigamma(shape);

        let g = digamma_k - (sum_x.ln() - (sum_x2 / n).ln()) / n;
        let g_prime = trigamma_k;

        if g_prime.abs() < 1e-12 {
            break;
        }

        let delta = g / g_prime;
        shape -= delta;

        if delta.abs() < tolerance {
            break;
        }

        if shape <= 0.0 {
            shape = 1e-6; // Prevent negative shape
        }
    }

    let scale = mean / shape; // MLE scale = mean / shape

    if (shape <= 0.0 || shape.is_nan()) || (scale <= 0.0 || scale.is_nan()) {
        return Err("Gamma fit undefined: invalid MLE parameter estimates".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // Gamma PDF: (1/Γ(k)) * (1/θ^k) * x^(k-1) * e^(-x/θ)
            let pdf = (x / scale).powf(shape - 1.0) * (-x / scale).exp() / (scale * gamma(shape));
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
        goodness_of_fit: goodness_of_fit(data, |x| gamma_cdf(x, shape, scale)).unwrap(),
    })
}

/// Fit Beta distribution using maximum likelihood estimation
pub fn fit_beta_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    if data.iter().any(|&x| x <= 0.0 || x >= 1.0) {
        return Err("Beta distribution requires data in (0,1)".to_string());
    }

    let n = data.len() as f64;
    let sum_log_x = data.iter().map(|x| x.ln()).sum::<f64>();
    let sum_log_1mx = data.iter().map(|x| (1.0 - x).ln()).sum::<f64>();

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

    // Newton-Raphson for MLE
    let tolerance = 1e-8;
    let max_iter = 100;

    for _ in 0..max_iter {
        let digamma_alpha = digamma(alpha);
        let digamma_beta = digamma(beta_param);
        let digamma_sum = digamma(alpha + beta_param);

        let trigamma_alpha = trigamma(alpha);
        let trigamma_beta = trigamma(beta_param);
        let trigamma_sum = trigamma(alpha + beta_param);

        // MLE equations:
        // ψ(α) - ψ(α+β) = (1/n)∑ln(x_i)
        // ψ(β) - ψ(α+β) = (1/n)∑ln(1-x_i)
        let g1 = digamma_alpha - digamma_sum - sum_log_x / n;
        let g2 = digamma_beta - digamma_sum - sum_log_1mx / n;

        // Jacobian
        let dg1_dalpha = trigamma_alpha - trigamma_sum;
        let dg1_dbeta = -trigamma_sum;
        let dg2_dalpha = -trigamma_sum;
        let dg2_dbeta = trigamma_beta - trigamma_sum;

        let det = dg1_dalpha * dg2_dbeta - dg1_dbeta * dg2_dalpha;
        if det.abs() < 1e-12 {
            break;
        }

        let delta_alpha = (g1 * dg2_dbeta - g2 * dg1_dbeta) / det;
        let delta_beta = (dg1_dalpha * g2 - dg2_dalpha * g1) / det;

        alpha -= delta_alpha;
        beta_param -= delta_beta;

        if delta_alpha.abs() < tolerance && delta_beta.abs() < tolerance {
            break;
        }

        // Prevent negative parameters
        if alpha <= 1e-6 {
            alpha = 1e-6;
        }
        if beta_param <= 1e-6 {
            beta_param = 1e-6;
        }
    }

    if (alpha <= 0.0 || alpha.is_nan()) || (beta_param <= 0.0 || beta_param.is_nan()) {
        return Err("Beta fit undefined: invalid MLE parameter estimates".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // Beta PDF: x^(α-1) * (1-x)^(β-1) / B(α,β)
            let pdf = x.powf(alpha - 1.0) * (1.0 - x).powf(beta_param - 1.0) / beta(alpha, beta_param);
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
        goodness_of_fit: goodness_of_fit(data, |x| beta_cdf(x, alpha, beta_param)).unwrap(),
    })
}

/// Compute goodness of fit using Kolmogorov-Smirnov test
pub fn goodness_of_fit<F>(data: &[f64], cdf: F) -> Result<f64, String>
    where F: Fn(f64) -> f64,
    {
    let mut sorted_data = data.to_vec();
    sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = sorted_data.len() as f64;
    let mut max_diff: f64 = 0.0;

    for (i, &x) in sorted_data.iter().enumerate() {
        let empirical_cdf = (i + 1) as f64 / n;
        let theoretical_cdf = cdf(x);
        max_diff = max_diff.max((empirical_cdf - theoretical_cdf).abs());
    }

    Ok(max_diff) // KS statistic
}

/// Gamma function approximation
fn gamma(z: f64) -> f64 {
    statrs::function::gamma::gamma(z)
}

/// Beta function
fn beta(a: f64, b: f64) -> f64 {
    use statrs::function::beta;
    beta::beta(a, b)
}

/// Digamma function (derivative of log gamma)
fn digamma(x: f64) -> f64 {
    use statrs::function::gamma;
    gamma::digamma(x)
}

/// Trigamma function (second derivative of log gamma)
fn trigamma(x: f64) -> f64 {
    // Approximation using derivative of digamma
    // trigamma(x) ≈ (digamma(x + h) - digamma(x - h)) / (2h) for small h
    let h = 1e-5;
    use statrs::function::gamma;
    (gamma::digamma(x + h) - gamma::digamma(x - h)) / (2.0 * h)
}

/// Incomplete gamma function (simplified approximation)
fn gamma_cdf(x: f64, shape: f64, scale: f64) -> f64 {
    // Use statrs Gamma CDF if available, otherwise fall back to approximation
    if let Ok(gamma_dist) = statrs::distribution::Gamma::new(shape, scale) {
        gamma_dist.cdf(x)
    } else {
        // Fallback to normal approximation for large shape
        if x <= 0.0 {
            0.0
        } else {
            let mean = shape * scale;
            let variance = shape * scale * scale;
            let std_dev = variance.sqrt();
            StatisticalDistributions::normal_cdf(x, mean, std_dev)
        }
    }
}

/// Beta CDF using statrs
fn beta_cdf(x: f64, alpha: f64, beta: f64) -> f64 {
    // Use statrs Beta CDF if available
    if let Ok(beta_dist) = statrs::distribution::Beta::new(alpha, beta) {
        beta_dist.cdf(x)
    } else {
        // Fallback to normal approximation
        if x <= 0.0 {
            0.0
        } else if x >= 1.0 {
            1.0
        } else {
            let mean = alpha / (alpha + beta);
            let variance = alpha * beta / ((alpha + beta).powi(2) * (alpha + beta + 1.0));
            let std_dev = variance.sqrt();
            StatisticalDistributions::normal_cdf(x, mean, std_dev)
        }
    }
}