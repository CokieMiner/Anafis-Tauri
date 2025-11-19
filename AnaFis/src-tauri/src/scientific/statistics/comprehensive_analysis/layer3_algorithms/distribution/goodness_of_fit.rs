//! Goodness of fit calculations and CDF functions
use statrs::distribution::ContinuousCDF;

/// Compute goodness of fit using Kolmogorov-Smirnov test
pub fn goodness_of_fit<F>(data: &[f64], cdf: F) -> Result<f64, String>
    where F: Fn(f64) -> Result<f64, String>,
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
        let theoretical_cdf = cdf(x)?;
        max_diff = max_diff.max((empirical_cdf - theoretical_cdf).abs());
    }

    Ok(max_diff) // KS statistic
}

/// Incomplete gamma function (simplified approximation)
pub fn gamma_cdf(x: f64, shape: f64, scale: f64) -> Result<f64, String> {
    // Use statrs Gamma CDF
    let gamma_dist = statrs::distribution::Gamma::new(shape, scale)
        .map_err(|e| format!("Failed to create Gamma distribution: {}", e))?;
    Ok(gamma_dist.cdf(x))
}

/// Beta CDF using statrs
pub fn beta_cdf(x: f64, alpha: f64, beta: f64) -> Result<f64, String> {
    // Use statrs Beta CDF
    let beta_dist = statrs::distribution::Beta::new(alpha, beta)
        .map_err(|e| format!("Failed to create Beta distribution: {}", e))?;
    Ok(beta_dist.cdf(x))
}

/// Student's t CDF using statrs
pub fn student_t_cdf(x: f64, location: f64, scale: f64, df: f64) -> Result<f64, String> {
    // Standardize to location 0, scale 1
    let standardized = (x - location) / scale;

    // Use statrs StudentsT CDF
    let t_dist = statrs::distribution::StudentsT::new(0.0, 1.0, df)
        .map_err(|e| format!("Failed to create StudentsT distribution: {}", e))?;
    Ok(t_dist.cdf(standardized))
}

/// Cauchy CDF
pub fn cauchy_cdf(x: f64, location: f64, scale: f64) -> Result<f64, String> {
    Ok(0.5 + (1.0 / std::f64::consts::PI) * ((x - location) / scale).atan())
}

/// Pareto CDF
pub fn pareto_cdf(x: f64, shape: f64, scale: f64) -> Result<f64, String> {
    if x < scale {
        Ok(0.0)
    } else {
        Ok(1.0 - (scale / x).powf(shape))
    }
}

/// Gumbel CDF
pub fn gumbel_cdf(x: f64, location: f64, scale: f64) -> Result<f64, String> {
    let z = (x - location) / scale;
    Ok((-(-z).exp()).exp())
}