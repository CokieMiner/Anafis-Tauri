
//! Distribution functions (PDF, CDF, Quantiles)
//!
//! This module provides static functions for common statistical distributions.

use statrs::distribution::{Continuous, ContinuousCDF, Normal, StudentsT, FisherSnedecor, ChiSquared, Weibull, Gamma, Beta, Cauchy};

/// Normal distribution PDF
pub fn normal_pdf(x: f64, mean: f64, std: f64) -> f64 {
    if !std.is_finite() || std <= 0.0 {
        return 0.0; // Return 0 for invalid parameters
    }
    let normal = Normal::new(mean, std)
        .expect("Failed to create normal distribution with valid parameters");
    normal.pdf(x)
}

/// Normal distribution CDF
pub fn normal_cdf(x: f64, mean: f64, std: f64) -> f64 {
    if !std.is_finite() || std <= 0.0 {
        return if x < mean { 0.0 } else { 1.0 }; // Return step function for invalid std
    }
    let normal = Normal::new(mean, std)
        .expect("Failed to create normal distribution with valid parameters");
    normal.cdf(x)
}

/// Student's t-distribution PDF
pub fn t_pdf(x: f64, df: f64) -> f64 {
    if !df.is_finite() || df <= 0.0 {
        return 0.0; // Return 0 for invalid degrees of freedom
    }
    let t = StudentsT::new(0.0, 1.0, df)
        .expect("Failed to create t-distribution with valid degrees of freedom");
    t.pdf(x)
}

/// Student's t-distribution CDF
pub fn t_cdf(x: f64, df: f64) -> f64 {
    if !df.is_finite() || df <= 0.0 {
        return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid df
    }
    let t = StudentsT::new(0.0, 1.0, df)
        .expect("Failed to create t-distribution with valid degrees of freedom");
    t.cdf(x)
}

/// Chi-squared distribution PDF
pub fn chi_squared_pdf(x: f64, df: f64) -> f64 {
    if !df.is_finite() || df <= 0.0 || x < 0.0 {
        return 0.0; // Return 0 for invalid parameters
    }
    let chi2 = ChiSquared::new(df)
        .expect("Failed to create chi-squared distribution with valid degrees of freedom");
    chi2.pdf(x)
}

/// Chi-squared distribution CDF
/// Uses the statrs ChiSquared implementation directly
pub fn chi_squared_cdf(x: f64, df: f64) -> f64 {
    if !df.is_finite() || df <= 0.0 {
        return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid df
    }
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        return 0.0;
    }

    let chi2 = ChiSquared::new(df)
        .expect("Failed to create chi-squared distribution with valid degrees of freedom");
    chi2.cdf(x)
}

/// F-distribution CDF
pub fn f_cdf(x: f64, df1: f64, df2: f64) -> f64 {
    if !df1.is_finite() || df1 <= 0.0 || !df2.is_finite() || df2 <= 0.0 {
        return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid df
    }
    let f_dist = FisherSnedecor::new(df1, df2)
        .expect("Failed to create F-distribution with valid degrees of freedom");
    f_dist.cdf(x)
}

/// Fisher-Snedecor distribution CDF (alias for f_cdf)
pub fn fisher_snedecor_cdf(x: f64, df1: f64, df2: f64) -> f64 {
    f_cdf(x, df1, df2)
}

/// Weibull distribution CDF
pub fn weibull_cdf(x: f64, shape: f64, scale: f64) -> f64 {
    if !shape.is_finite() || shape <= 0.0 || !scale.is_finite() || scale <= 0.0 {
        return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid parameters
    }
    if x < 0.0 {
        return 0.0;
    }
    let weibull = Weibull::new(shape, scale)
        .expect("Failed to create Weibull distribution with valid parameters");
    weibull.cdf(x)
}

/// Gamma distribution CDF
pub fn gamma_cdf(x: f64, shape: f64, scale: f64) -> f64 {
    if !shape.is_finite() || shape <= 0.0 || !scale.is_finite() || scale <= 0.0 {
        return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid parameters
    }
    let gamma = Gamma::new(shape, scale)
        .expect("Failed to create Gamma distribution with valid parameters");
    gamma.cdf(x)
}

/// Beta distribution CDF
pub fn beta_cdf(x: f64, alpha: f64, beta: f64) -> f64 {
    if !alpha.is_finite() || alpha <= 0.0 || !beta.is_finite() || beta <= 0.0 {
        return if x < 0.0 { 0.0 } else if x >= 1.0 { 1.0 } else { 0.5 }; // Return step function for invalid parameters
    }
    let beta_dist = Beta::new(alpha, beta)
        .expect("Failed to create Beta distribution with valid parameters");
    beta_dist.cdf(x)
}

/// Pareto distribution CDF
pub fn pareto_cdf(x: f64, shape: f64, scale: f64) -> f64 {
    if !shape.is_finite() || shape <= 0.0 || !scale.is_finite() || scale <= 0.0 {
        return if x < scale { 0.0 } else { 1.0 }; // Return step function for invalid parameters
    }
    if x < scale {
        return 0.0;
    }
    1.0 - (scale / x).powf(shape)
}

/// Cauchy distribution CDF
pub fn cauchy_cdf(x: f64, location: f64, scale: f64) -> f64 {
    if !scale.is_finite() || scale <= 0.0 {
        return if x < location { 0.0 } else { 1.0 }; // Return step function for invalid scale
    }
    let cauchy = Cauchy::new(location, scale)
        .expect("Failed to create Cauchy distribution with valid parameters");
    cauchy.cdf(x)
}

/// Student's t-distribution CDF with location and scale parameters
pub fn student_t_cdf(x: f64, location: f64, scale: f64, df: f64) -> f64 {
    if !df.is_finite() || df <= 0.0 || !scale.is_finite() || scale <= 0.0 {
        return if x < location { 0.0 } else { 1.0 }; // Return step function for invalid parameters
    }
    let standardized = (x - location) / scale;
    let t_dist = StudentsT::new(0.0, 1.0, df)
        .expect("Failed to create t-distribution with valid degrees of freedom");
    t_dist.cdf(standardized)
}

/// Lognormal distribution CDF
pub fn lognormal_cdf(x: f64, mu: f64, sigma: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if !sigma.is_finite() || sigma <= 0.0 {
        return if x.ln() < mu { 0.0 } else { 1.0 }; // Return step function for invalid sigma
    }
    let normal = Normal::new(mu, sigma)
        .expect("Failed to create normal distribution for lognormal CDF");
    normal.cdf(x.ln())
}

/// Exponential distribution CDF
pub fn exponential_cdf(x: f64, rate: f64) -> f64 {
    if !rate.is_finite() || rate <= 0.0 {
        return if x < 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid rate
    }
    if x < 0.0 {
        return 0.0;
    }
    1.0 - (-rate * x).exp()
}

/// Gumbel distribution CDF (Extreme Value Type I)
pub fn gumbel_cdf(x: f64, location: f64, scale: f64) -> f64 {
    if !scale.is_finite() || scale <= 0.0 {
        return if x < location { 0.0 } else { 1.0 }; // Return step function for invalid scale
    }
    let z = (x - location) / scale;
    (-(-z).exp()).exp()
}

/// Johnson SU distribution CDF
pub fn johnson_su_cdf(x: f64, gamma: f64, delta: f64, lambda: f64, xi: f64) -> f64 {
    if !delta.is_finite() || delta <= 0.0 || !lambda.is_finite() || lambda <= 0.0 {
        return if x <= xi { 0.0 } else { 1.0 }; // Return step function for invalid parameters
    }
    if x <= xi {
        return 0.0;
    }
    if x >= xi + lambda {
        return 1.0;
    }

    let z = gamma + delta * ((x - xi) / lambda).ln();
    let normal = Normal::new(0.0, 1.0)
        .expect("Failed to create standard normal distribution");
    normal.cdf(z)
}

/// Burr Type XII distribution CDF
pub fn burr_type_xii_cdf(x: f64, c: f64, k: f64, lambda: f64) -> f64 {
    if !c.is_finite() || c <= 0.0 || !k.is_finite() || k <= 0.0 || !lambda.is_finite() || lambda <= 0.0 {
        return if x <= 0.0 { 0.0 } else { 1.0 }; // Return step function for invalid parameters
    }
    if x <= 0.0 {
        return 0.0;
    }

    let u = (x / lambda).powf(c);
    1.0 - (1.0 + u).powf(-k)
}

/// Calculate quantile from t-distribution
pub fn t_quantile(p: f64, df: f64) -> Result<f64, String> {
    let t_dist = StudentsT::new(0.0, 1.0, df)
        .map_err(|e| format!("Failed to create t-distribution: {}", e))?;
    Ok(t_dist.inverse_cdf(p))
}

/// Calculate quantile from F-distribution
pub fn f_quantile(p: f64, df1: f64, df2: f64) -> Result<f64, String> {
    let f_dist = FisherSnedecor::new(df1, df2)
        .map_err(|e| format!("Failed to create F-distribution: {}", e))?;
    Ok(f_dist.inverse_cdf(p))
}

/// Calculate quantile from chi-square distribution
pub fn chi_square_quantile(p: f64, df: f64) -> Result<f64, String> {
    let chi_dist = ChiSquared::new(df)
        .map_err(|e| format!("Failed to create chi-squared distribution: {}", e))?;
    Ok(chi_dist.inverse_cdf(p))
}

/// Calculate quantile from normal distribution
pub fn normal_quantile(p: f64) -> f64 {
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return 0.0; // Return 0 for invalid probability
    }
    let normal = Normal::new(0.0, 1.0)
        .expect("Failed to create standard normal distribution");
    normal.inverse_cdf(p)
}
