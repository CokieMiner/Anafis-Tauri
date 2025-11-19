//! Special mathematical functions
//!
//! This module provides implementations of special mathematical functions
//! including gamma, beta, and error functions.

use statrs::function::{gamma, beta};

/// Special mathematical functions using the `statrs` crate
pub struct SpecialFunctions;

impl SpecialFunctions {
    /// Gamma function: Γ(z)
    pub fn gamma(z: f64) -> f64 {
        gamma::gamma(z)
    }

    /// Beta function: B(a,b) = ∫₀¹ t^(a-1) (1-t)^(b-1) dt
    pub fn beta(a: f64, b: f64) -> f64 {
        beta::beta(a, b)
    }

    /// Digamma function (derivative of log gamma): ψ(x) = d/dx ln(Γ(x))
    pub fn digamma(x: f64) -> f64 {
        gamma::digamma(x)
    }

    /// Trigamma function (second derivative of log gamma): ψ'(x) = d/dx ψ(x)
    pub fn trigamma(x: f64) -> f64 {
        // Approximation using derivative of digamma
        // trigamma(x) ≈ (digamma(x + h) - digamma(x - h)) / (2h) for small h
        let h = 1e-5;
        (gamma::digamma(x + h) - gamma::digamma(x - h)) / (2.0 * h)
    }
}