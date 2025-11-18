//! Special mathematical functions
//!
//! This module provides implementations of special mathematical functions
//! including gamma, beta, and error functions.

use statrs::function::{gamma, beta, erf};

/// Special mathematical functions using the `statrs` crate
pub struct SpecialFunctions;

impl SpecialFunctions {
    /// Gamma function: Γ(z)
    pub fn gamma(z: f64) -> f64 {
        gamma::gamma(z)
    }

    /// Log gamma function: ln(Γ(z))
    pub fn ln_gamma(z: f64) -> f64 {
        gamma::ln_gamma(z)
    }

    /// Incomplete gamma function: γ(s,x) = ∫₀ˣ t^(s-1) e^(-t) dt
    pub fn incomplete_gamma_lower(s: f64, x: f64) -> f64 {
        gamma::gamma_lr(s, x)
    }

    /// Complementary incomplete gamma function: Γ(s,x) = ∫ₓ^∞ t^(s-1) e^(-t) dt
    pub fn incomplete_gamma_upper(s: f64, x: f64) -> f64 {
        gamma::gamma_ur(s, x)
    }

    /// Beta function: B(a,b) = ∫₀¹ t^(a-1) (1-t)^(b-1) dt
    pub fn beta(a: f64, b: f64) -> f64 {
        beta::beta(a, b)
    }

    /// Incomplete beta function: B(x;a,b) = ∫₀ˣ t^(a-1) (1-t)^(b-1) dt
    pub fn incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
        beta::beta_reg(x, a, b)
    }

    /// Error function: erf(x) = (2/√π) ∫₀ˣ e^(-t²) dt
    pub fn erf(x: f64) -> f64 {
        erf::erf(x)
    }

    /// Complementary error function: erfc(x) = 1 - erf(x)
    pub fn erfc(x: f64) -> f64 {
        erf::erfc(x)
    }
}