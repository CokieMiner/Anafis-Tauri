//! Goodness of fit calculations and CDF functions

use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::*;
use statrs::distribution::ContinuousCDF;

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
pub fn gamma(z: f64) -> f64 {
    statrs::function::gamma::gamma(z)
}

/// Beta function
pub fn beta(a: f64, b: f64) -> f64 {
    use statrs::function::beta;
    beta::beta(a, b)
}

/// Digamma function (derivative of log gamma)
pub fn digamma(x: f64) -> f64 {
    use statrs::function::gamma;
    gamma::digamma(x)
}

/// Trigamma function (second derivative of log gamma)
pub fn trigamma(x: f64) -> f64 {
    // Approximation using derivative of digamma
    // trigamma(x) ≈ (digamma(x + h) - digamma(x - h)) / (2h) for small h
    let h = 1e-5;
    use statrs::function::gamma;
    (gamma::digamma(x + h) - gamma::digamma(x - h)) / (2.0 * h)
}

/// Incomplete gamma function (simplified approximation)
pub fn gamma_cdf(x: f64, shape: f64, scale: f64) -> f64 {
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
pub fn beta_cdf(x: f64, alpha: f64, beta: f64) -> f64 {
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

/// Student's t CDF using statrs
pub fn student_t_cdf(x: f64, location: f64, scale: f64, df: f64) -> f64 {
    // Standardize to location 0, scale 1
    let standardized = (x - location) / scale;

    // Use statrs StudentsT CDF
    if let Ok(t_dist) = statrs::distribution::StudentsT::new(0.0, 1.0, df) {
        t_dist.cdf(standardized)
    } else {
        // Fallback to approximation
        if df > 30.0 {
            StatisticalDistributions::normal_cdf(standardized, 0.0, 1.0)
        } else {
            0.5 + 0.5 * (standardized / (1.0 + standardized * standardized / df).sqrt()).atan() / std::f64::consts::PI * 2.0
        }
    }
}

/// Cauchy CDF
pub fn cauchy_cdf(x: f64, location: f64, scale: f64) -> f64 {
    0.5 + (1.0 / std::f64::consts::PI) * ((x - location) / scale).atan()
}

/// Pareto CDF
pub fn pareto_cdf(x: f64, shape: f64, scale: f64) -> f64 {
    if x < scale {
        0.0
    } else {
        1.0 - (scale / x).powf(shape)
    }
}

/// Gumbel CDF
pub fn gumbel_cdf(x: f64, location: f64, scale: f64) -> f64 {
    let z = (x - location) / scale;
    (-(-z).exp()).exp()
}