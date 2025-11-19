//! Non-central distributions for precise statistical power analysis
//! 
//! This module implements non-central t and F distributions using series expansions
//! for accurate power calculations, especially important for small sample sizes.
//! 
//! ## Mathematical Background
//! - Non-central t-distribution: T'(δ, ν) where δ is non-centrality parameter
//! - Non-central F-distribution: F'(λ, d1, d2) where λ is non-centrality parameter
//! - Series expansions provide exact calculations rather than approximations

use statrs::function::gamma;

/// Non-central t-distribution implementation
#[derive(Debug, Clone, Copy)] // Added Copy trait for recursion
pub struct NonCentralT {
    pub non_centrality: f64,
    pub degrees_of_freedom: f64,
}
impl NonCentralT {
    /// Create a new non-central t-distribution
    pub fn new(non_centrality: f64, degrees_of_freedom: f64) -> Result<Self, String> {
        if degrees_of_freedom <= 0.0 {
            return Err("Degrees of freedom must be positive".to_string());
        }
        if !non_centrality.is_finite() {
            return Err("Non-centrality parameter must be finite".to_string());
        }

        Ok(Self {
            non_centrality,
            degrees_of_freedom,
        })
    }

    /// Cumulative distribution function using series expansion (adapted from Johnson, Kotz, & Balakrishnan)
    /// F(t; ν, δ) = Σ_{j=0}^∞ [ e^{-δ²/2} * (δ²/2)^j / j! ] * P(T_{ν+2j} ≤ t - δ)
    /// where P(T_{ν+2j} ≤ t - δ) is the central t cdf with ν+2j df
    pub fn cdf(&self, x: f64) -> f64 {
        if !x.is_finite() {
            return if x.is_nan() { f64::NAN } else if x.is_sign_positive() { 1.0 } else { 0.0 };
        }

        let delta = self.non_centrality;
        let nu = self.degrees_of_freedom;

        if nu <= 0.0 { return f64::NAN; } // Degrees of freedom must be positive

        let mut cdf_val: f64 = 0.0;
        let max_terms = 200; // Truncate infinite series
        let delta_sq_half = delta.powi(2) / 2.0;
        
        for j in 0..max_terms {
            let poisson_term_numerator = delta_sq_half.powi(j) * (-delta_sq_half).exp();
            let factorial_j = gamma::gamma(j as f64 + 1.0); // j!
            
            if factorial_j == 0.0 { continue; } // Avoid division by zero
            
            let term_coefficient = poisson_term_numerator / factorial_j;
            
            if term_coefficient.abs() < 1e-20 && j > 0 { // Check for convergence
                break;
            }

            let adjusted_nu = nu + 2.0 * j as f64;
            
            // Use statrs StudentsT for central t cdf
            use statrs::distribution::{ContinuousCDF, StudentsT};
            let central_cdf = if let Ok(central_dist) = StudentsT::new(0.0, 1.0, adjusted_nu) {
                central_dist.cdf(x - delta)
            } else {
                // Fallback for invalid degrees of freedom
                if x - delta >= 0.0 { 1.0 } else { 0.0 }
            };
            
            cdf_val += term_coefficient * central_cdf;
        }

        cdf_val.clamp(0.0, 1.0)
    }

    /// Survival function (1 - CDF)
    pub fn sf(&self, x: f64) -> f64 {
        1.0 - self.cdf(x)
    }

    /// Power calculation for two-sided test: P(|T'(δ, ν)| > t_crit)
    pub fn power_two_sided(&self, t_critical: f64) -> f64 {
        // Power = P(T' > t_crit) + P(T' < -t_crit)
        // This is sf(t_critical) + cdf(-t_critical)
        let power = self.sf(t_critical) + self.cdf(-t_critical);
        power.clamp(0.0, 1.0)
    }
}

/// Non-central F-distribution implementation
pub struct NonCentralF {
    pub non_centrality: f64,
    pub df1: f64,
    pub df2: f64,
}

impl NonCentralF {
    /// Create a new non-central F-distribution
    pub fn new(non_centrality: f64, df1: f64, df2: f64) -> Result<Self, String> {
        if df1 <= 0.0 || df2 <= 0.0 {
            return Err("Degrees of freedom must be positive".to_string());
        }
        if non_centrality < 0.0 {
            return Err("Non-centrality parameter must be non-negative".to_string());
        }
        if !non_centrality.is_finite() {
            return Err("Non-centrality parameter must be finite".to_string());
        }

        Ok(Self {
            non_centrality,
            df1,
            df2,
        })
    }

    /// Cumulative distribution function using series expansion
    /// P(F'(λ, d1, d2) ≤ x) = ∑_{k=0}^∞ [e^{-λ/2} * (λ/2)^k / k!] * P(F_{d1+2k, d2} ≤ x * d1/(d1+2k))
    pub fn cdf(&self, x: f64) -> f64 {
        if !x.is_finite() || x <= 0.0 {
            return if x.is_nan() { f64::NAN } else if x > 0.0 { 1.0 } else { 0.0 };
        }

        let lambda = self.non_centrality;
        let d1 = self.df1;
        let d2 = self.df2;

        // Series expansion
        let mut cdf = 0.0;
        let mut term = (-lambda / 2.0).exp();
        let max_terms = 50;

        for k in 0..max_terms {
            if term.abs() < 1e-12 {
                break;
            }

            // Central F-distribution CDF with adjusted degrees of freedom and scaled argument
            let adjusted_d1 = d1 + 2.0 * k as f64;
            let scaled_x = x * d1 / adjusted_d1;
            let central_cdf = self.central_f_cdf(scaled_x, adjusted_d1, d2);

            cdf += term * central_cdf;

            // Next term: multiply by (λ/2) / (k+1)
            term *= lambda / (2.0 * (k + 1) as f64);
        }

        cdf.clamp(0.0, 1.0)
    }

    /// Survival function (1 - CDF)
    pub fn sf(&self, x: f64) -> f64 {
        1.0 - self.cdf(x)
    }

    /// Power calculation: P(F'(λ, d1, d2) > f_crit)
    pub fn power(&self, f_critical: f64) -> f64 {
        // Power is simply the survival function at the critical value
        self.sf(f_critical).clamp(0.0, 1.0)
    }

    /// Central F-distribution CDF (helper function)
    fn central_f_cdf(&self, x: f64, df1: f64, df2: f64) -> f64 {
        // Use statrs for central F-distribution
        use statrs::distribution::{ContinuousCDF, FisherSnedecor};
        FisherSnedecor::new(df1, df2).map_or(f64::NAN, |dist| dist.cdf(x))
    }
}

/// Utility functions for power analysis
pub struct PowerAnalysisUtils;

impl PowerAnalysisUtils {
    /// Calculate non-centrality parameter for t-test
    pub fn t_test_ncp(delta: f64, sigma: f64, n: usize) -> f64 {
        delta / (sigma / (n as f64).sqrt())
    }

    /// Calculate non-centrality parameter for ANOVA
    pub fn anova_ncp(cohen_f: f64, total_n: usize) -> f64 {
        cohen_f.powi(2) * total_n as f64
    }

    /// Calculate non-centrality parameter for chi-square test
    pub fn chi_square_ncp(effect_size: f64, n: usize) -> f64 {
        effect_size.powi(2) * n as f64
    }
}