//! Mathematical function evaluation using pre-compiled `symb_anafis` expressions.
//!
//! Each function is compiled to bytecode on first use (via `LazyLock`) and then
//! evaluated directly without any string parsing overhead.
//!
//! Only functions NOT natively supported by Univer's formula engine are included.
//! Univer already handles: sin, cos, tan, asin, acos, atan, atan2, sinh, cosh, tanh,
//! asinh, acosh, atanh, acot, acoth, cot, coth, csc, csch, sec, sech, sign, exp, ln,
//! log, log10, sqrt, abs, floor, ceil, round, erf, erfc, besselj, bessely, besseli, besselk.

use std::sync::LazyLock;
use symb_anafis::{CompiledEvaluator, Expr, symb};

// ═══════════════════════════════════════════════════════════════════════════════
// Pre-compiled evaluators (LazyLock — compiled once on first use)
// ═══════════════════════════════════════════════════════════════════════════════

/// Macro to define a pre-compiled single-variable evaluator using `Expr::call`.
macro_rules! compiled_fn1 {
    ($static_name:ident, $fn_name:literal) => {
        static $static_name: LazyLock<CompiledEvaluator> = LazyLock::new(|| {
            let x = symb("x");
            let expr = Expr::call($fn_name, [x.into()]);
            CompiledEvaluator::compile(&expr, &["x"], None)
                .expect(concat!("failed to compile ", $fn_name))
        });
    };
}

/// Macro to define a pre-compiled two-variable evaluator using `Expr::call`.
macro_rules! compiled_fn2 {
    ($static_name:ident, $fn_name:literal, $v1:literal, $v2:literal) => {
        static $static_name: LazyLock<CompiledEvaluator> = LazyLock::new(|| {
            let a = symb($v1);
            let b = symb($v2);
            let expr = Expr::call($fn_name, [a.into(), b.into()]);
            CompiledEvaluator::compile(&expr, &[$v1, $v2], None)
                .expect(concat!("failed to compile ", $fn_name))
        });
    };
}

/// Macro to define a pre-compiled three-variable evaluator using `Expr::call`.
macro_rules! compiled_fn3 {
    ($static_name:ident, $fn_name:literal, $v1:literal, $v2:literal, $v3:literal) => {
        static $static_name: LazyLock<CompiledEvaluator> = LazyLock::new(|| {
            let a = symb($v1);
            let b = symb($v2);
            let c = symb($v3);
            let expr = Expr::call($fn_name, [a.into(), b.into(), c.into()]);
            CompiledEvaluator::compile(&expr, &[$v1, $v2, $v3], None)
                .expect(concat!("failed to compile ", $fn_name))
        });
    };
}

/// Macro to define a pre-compiled four-variable evaluator using `Expr::call`.
macro_rules! compiled_fn4 {
    ($static_name:ident, $fn_name:literal, $v1:literal, $v2:literal, $v3:literal, $v4:literal) => {
        static $static_name: LazyLock<CompiledEvaluator> = LazyLock::new(|| {
            let a = symb($v1);
            let b = symb($v2);
            let c = symb($v3);
            let d = symb($v4);
            let expr = Expr::call($fn_name, [a.into(), b.into(), c.into(), d.into()]);
            CompiledEvaluator::compile(&expr, &[$v1, $v2, $v3, $v4], None)
                .expect(concat!("failed to compile ", $fn_name))
        });
    };
}

// ── Inverse Trigonometric (ASEC and ACSC not in Univer) ────────────────────
compiled_fn1!(ASEC_EVAL, "asec");
compiled_fn1!(ACSC_EVAL, "acsc");

// ── Inverse Hyperbolic (ASECH and ACSCH not in Univer) ─────────────────────
compiled_fn1!(ASECH_EVAL, "asech");
compiled_fn1!(ACSCH_EVAL, "acsch");

// ── Gamma Family ───────────────────────────────────────────────────────────
compiled_fn1!(GAMMA_EVAL, "gamma");
compiled_fn1!(DIGAMMA_EVAL, "digamma");
compiled_fn1!(TRIGAMMA_EVAL, "trigamma");
compiled_fn1!(TETRAGAMMA_EVAL, "tetragamma");
compiled_fn2!(POLYGAMMA_EVAL, "polygamma", "n", "x");
compiled_fn2!(BETA_EVAL, "beta", "a", "b");

// ── Zeta ───────────────────────────────────────────────────────────────────
compiled_fn1!(ZETA_EVAL, "zeta");
compiled_fn2!(ZETA_DERIV_EVAL, "zeta_deriv", "n", "s");

// ── Elliptic Integrals ─────────────────────────────────────────────────────
compiled_fn1!(ELLIPTIC_K_EVAL, "elliptic_k");
compiled_fn1!(ELLIPTIC_E_EVAL, "elliptic_e");

// ── Orthogonal Polynomials ─────────────────────────────────────────────────
compiled_fn2!(HERMITE_EVAL, "hermite", "n", "x");
compiled_fn3!(ASSOC_LEGENDRE_EVAL, "assoc_legendre", "l", "m", "x");

// ── Spherical Harmonics ────────────────────────────────────────────────────
compiled_fn4!(
    SPHERICAL_HARMONIC_EVAL,
    "spherical_harmonic",
    "l",
    "m",
    "theta",
    "phi"
);

// ── Other ──────────────────────────────────────────────────────────────────
compiled_fn1!(SINC_EVAL, "sinc");
compiled_fn1!(LAMBERTW_EVAL, "lambertw");
compiled_fn1!(CBRT_EVAL, "cbrt");

// ═══════════════════════════════════════════════════════════════════════════════
// Tauri commands
// ═══════════════════════════════════════════════════════════════════════════════

// ── Inverse Trigonometric ──────────────────────────────────────────────────

/// Inverse secant: asec(x) = acos(1/x)
#[tauri::command]
pub fn math_asec(x: f64) -> f64 {
    ASEC_EVAL.evaluate(&[x])
}

/// Inverse cosecant: acsc(x) = asin(1/x)
#[tauri::command]
pub fn math_acsc(x: f64) -> f64 {
    ACSC_EVAL.evaluate(&[x])
}

// ── Inverse Hyperbolic ─────────────────────────────────────────────────────

/// Inverse hyperbolic secant: asech(x)
#[tauri::command]
pub fn math_asech(x: f64) -> f64 {
    ASECH_EVAL.evaluate(&[x])
}

/// Inverse hyperbolic cosecant: acsch(x) = asinh(1/x)
#[tauri::command]
pub fn math_acsch(x: f64) -> f64 {
    ACSCH_EVAL.evaluate(&[x])
}

// ── Gamma Family ───────────────────────────────────────────────────────────

/// Gamma function: Γ(x)
#[tauri::command]
pub fn math_gamma(x: f64) -> f64 {
    GAMMA_EVAL.evaluate(&[x])
}

/// Digamma function: ψ(x) = d/dx ln(Γ(x))
#[tauri::command]
pub fn math_digamma(x: f64) -> f64 {
    DIGAMMA_EVAL.evaluate(&[x])
}

/// Trigamma function: ψ₁(x) = d²/dx² ln(Γ(x))
#[tauri::command]
pub fn math_trigamma(x: f64) -> f64 {
    TRIGAMMA_EVAL.evaluate(&[x])
}

/// Tetragamma function: ψ₂(x) = d³/dx³ ln(Γ(x))
#[tauri::command]
pub fn math_tetragamma(x: f64) -> f64 {
    TETRAGAMMA_EVAL.evaluate(&[x])
}

/// Polygamma function: ψₙ(x) = dⁿ⁺¹/dxⁿ⁺¹ ln(Γ(x))
#[tauri::command]
pub fn math_polygamma(n: f64, x: f64) -> f64 {
    POLYGAMMA_EVAL.evaluate(&[n, x])
}

/// Beta function: B(a, b) = Γ(a)Γ(b) / Γ(a+b)
#[tauri::command]
pub fn math_beta(a: f64, b: f64) -> f64 {
    BETA_EVAL.evaluate(&[a, b])
}

// ── Zeta ───────────────────────────────────────────────────────────────────

/// Riemann zeta function: ζ(s)
#[tauri::command]
pub fn math_zeta(x: f64) -> f64 {
    ZETA_EVAL.evaluate(&[x])
}

/// Zeta derivative: ζ⁽ⁿ⁾(s)
#[tauri::command]
pub fn math_zeta_deriv(n: f64, s: f64) -> f64 {
    ZETA_DERIV_EVAL.evaluate(&[n, s])
}

// ── Elliptic Integrals ─────────────────────────────────────────────────────

/// Complete elliptic integral of the first kind: K(k)
#[tauri::command]
pub fn math_elliptic_k(x: f64) -> f64 {
    ELLIPTIC_K_EVAL.evaluate(&[x])
}

/// Complete elliptic integral of the second kind: E(k)
#[tauri::command]
pub fn math_elliptic_e(x: f64) -> f64 {
    ELLIPTIC_E_EVAL.evaluate(&[x])
}

// ── Orthogonal Polynomials ─────────────────────────────────────────────────

/// Hermite polynomial: Hₙ(x)
#[tauri::command]
pub fn math_hermite(n: f64, x: f64) -> f64 {
    HERMITE_EVAL.evaluate(&[n, x])
}

/// Associated Legendre polynomial: Pₗᵐ(x)
#[tauri::command]
pub fn math_assoc_legendre(l: f64, m: f64, x: f64) -> f64 {
    ASSOC_LEGENDRE_EVAL.evaluate(&[l, m, x])
}

// ── Spherical Harmonics ────────────────────────────────────────────────────

/// Spherical harmonic: Yₗᵐ(θ, φ)
#[tauri::command]
pub fn math_spherical_harmonic(l: f64, m: f64, theta: f64, phi: f64) -> f64 {
    SPHERICAL_HARMONIC_EVAL.evaluate(&[l, m, theta, phi])
}

// ── Other ──────────────────────────────────────────────────────────────────

/// Sinc function: sinc(x) = sin(x)/x, sinc(0) = 1
#[tauri::command]
pub fn math_sinc(x: f64) -> f64 {
    SINC_EVAL.evaluate(&[x])
}

/// Lambert W function: W(x) where W(x)·eᵂ⁽ˣ⁾ = x
#[tauri::command]
pub fn math_lambertw(x: f64) -> f64 {
    LAMBERTW_EVAL.evaluate(&[x])
}

/// Cube root: cbrt(x) = x^(1/3)
#[tauri::command]
pub fn math_cbrt(x: f64) -> f64 {
    CBRT_EVAL.evaluate(&[x])
}
