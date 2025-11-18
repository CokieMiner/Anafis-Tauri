

use crate::scientific::statistics::types::DistributionFit;

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

    // Use MLE for better estimates
    let mut mu = mu_mom;
    let mut beta = beta_mom;

    // Newton-Raphson for MLE
    let tolerance = 1e-8;
    let max_iter = 100;

    for _ in 0..max_iter {
        let mut sum_exp_term = 0.0;
        let mut sum_exp_z = 0.0;
        let mut sum_z_exp_z = 0.0;

        for &x in data {
            let z = (x - mu) / beta;
            let exp_z = z.exp();
            sum_exp_term += exp_z;
            sum_exp_z += z * exp_z;
            sum_z_exp_z += z * z * exp_z;
        }

        // MLE equations for Gumbel:
        // ∂lnL/∂μ = (n/β) - (1/β) * sum(exp((x-μ)/β)) = 0
        // ∂lnL/∂β = -n/β + (1/β²) * sum((x-μ)*exp((x-μ)/β)) = 0

        let g_mu = n / beta - sum_exp_term / beta;
        let g_beta = -n / beta + sum_exp_z / beta.powi(2);

        // Jacobian
        let dg_mu_dmu = -sum_exp_term / beta.powi(2);
        let dg_mu_dbeta = -n / beta.powi(2) + sum_exp_z / beta.powi(3);
        let dg_beta_dmu = -sum_exp_z / beta.powi(3);
        let dg_beta_dbeta = n / beta.powi(2) - sum_z_exp_z / beta.powi(4);

        let det = dg_mu_dmu * dg_beta_dbeta - dg_mu_dbeta * dg_beta_dmu;
        if det.abs() < 1e-12 {
            break;
        }

        let delta_mu = (g_mu * dg_beta_dbeta - g_beta * dg_mu_dbeta) / det;
        let delta_beta = (dg_mu_dmu * g_beta - dg_beta_dmu * g_mu) / det;

        mu -= delta_mu;
        beta -= delta_beta;

        if delta_mu.abs() < tolerance && delta_beta.abs() < tolerance {
            break;
        }

        // Prevent invalid parameters
        if beta <= 1e-6 {
            beta = 1e-6;
        }
    }

    if (beta <= 0.0 || beta.is_nan()) || !mu.is_finite() {
        return Err("Gumbel fit undefined: invalid MLE parameter estimates".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            let z = (x - mu) / beta;
            // Gumbel PDF: (1/β) * exp(-(z + exp(-z)))
            let pdf = (1.0 / beta) * (-z - (-z).exp()).exp();
            pdf.ln()
        })
        .sum::<f64>();

    let k = 2.0; // parameters: location, scale
    let aic = 2.0 * k - 2.0 * log_likelihood;
    let bic = k * n.ln() - 2.0 * log_likelihood;

    Ok(DistributionFit {
        distribution_name: "gumbel".to_string(),
        parameters: vec![
            ("location".to_string(), mu),
            ("scale".to_string(), beta),
        ],
        log_likelihood,
        aic,
        bic,
        goodness_of_fit: goodness_of_fit(data, |x| gumbel_cdf(x, mu, beta)).unwrap(),
    })
}

/// Fit Pareto distribution (power-law tail)
pub fn fit_pareto_distribution(data: &[f64]) -> Result<DistributionFit, String> {
    if data.iter().any(|&x| x <= 0.0) {
        return Err("Pareto distribution requires positive data".to_string());
    }

    let n = data.len() as f64;
    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);

    // Method of moments for Pareto: shape = 1 + 1/ln(max/min)
    // But we'll use a simpler approach
    let mean = data.iter().sum::<f64>() / n;

    if mean <= min_val {
        return Err("Pareto fit undefined: data doesn't follow power-law".to_string());
    }

    let shape = 1.0 + 1.0 / ((mean / min_val) - 1.0).ln();

    if shape <= 0.0 || shape.is_nan() {
        return Err("Pareto fit undefined: invalid shape parameter".to_string());
    }

    let log_likelihood = data.iter()
        .map(|&x| {
            // Pareto PDF: α * xₘ^α / x^(α+1)
            let pdf = shape * min_val.powf(shape) / x.powf(shape + 1.0);
            pdf.ln()
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
        goodness_of_fit: goodness_of_fit(data, |x| pareto_cdf(x, shape, min_val)).unwrap(),
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

/// Gumbel CDF
fn gumbel_cdf(x: f64, location: f64, scale: f64) -> f64 {
    let z = (x - location) / scale;
    (-(-z).exp()).exp()
}

/// Pareto CDF
fn pareto_cdf(x: f64, shape: f64, scale: f64) -> f64 {
    if x < scale {
        0.0
    } else {
        1.0 - (scale / x).powf(shape)
    }
}