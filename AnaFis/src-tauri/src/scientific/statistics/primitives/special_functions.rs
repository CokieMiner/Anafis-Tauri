use statrs::function::{gamma, beta, erf}; // Added erf for Normal CDFs

pub struct SpecialFunctions;

impl SpecialFunctions {
    /// Gamma function: Γ(z)
    #[inline]
    pub fn gamma(z: f64) -> f64 {
        gamma::gamma(z)
    }

    /// Beta function: B(a,b)
    #[inline]
    pub fn beta(a: f64, b: f64) -> f64 {
        beta::beta(a, b)
    }

    /// Digamma function: ψ(x)
    #[inline]
    pub fn digamma(x: f64) -> f64 {
        gamma::digamma(x)
    }

    /// Error function: erf(x)
    /// Essential for Normal distribution CDFs
    #[inline]
    pub fn erf(x: f64) -> f64 {
        erf::erf(x)
    }

    /// Complementary error function: erfc(x)
    #[inline]
    pub fn erfc(x: f64) -> f64 {
        erf::erfc(x)
    }

    /// Trigamma function: ψ'(x)
    /// Uses Recurrence relation + Asymptotic expansion for high precision
    pub fn trigamma(mut x: f64) -> f64 {
        // Handle negative arguments using reflection formula:
        // ψ'(1-z) + ψ'(z) = π² / sin²(πz)
        if x < 0.0 {
            let pi = std::f64::consts::PI;
            let sin_pi_x = (pi * x).sin();
            return (pi / sin_pi_x).powi(2) - Self::trigamma(1.0 - x);
        }

        // Use recurrence ψ'(z) = ψ'(z+1) + 1/z^2 to shift x to a larger value
        // for better asymptotic accuracy.
        let mut res = 0.0;
        while x < 6.0 {
            res += 1.0 / (x * x);
            x += 1.0;
        }

        // Asymptotic expansion:
        // ψ'(x) ≈ 1/x + 1/(2x^2) + 1/(6x^3) - 1/(30x^5) + 1/(42x^7)
        let x2 = x * x;
        let x3 = x * x2;
        let x5 = x3 * x2;
        let x7 = x5 * x2;

        res + (1.0 / x) 
            + (0.5 / x2) 
            + (1.0 / (6.0 * x3)) 
            - (1.0 / (30.0 * x5)) 
            + (1.0 / (42.0 * x7))
    }
}