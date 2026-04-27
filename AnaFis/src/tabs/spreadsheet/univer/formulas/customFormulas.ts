/**
 * Custom mathematical functions for the spreadsheet.
 *
 * Functions backed by symb_anafis (via Tauri invoke) are evaluated by
 * pre-compiled Rust bytecode for maximum accuracy and performance.
 *
 * These are registered as async functions since Tauri invoke is asynchronous.
 * Functions that Univer already supports natively are NOT registered here.
 */

import { invoke } from '@tauri-apps/api/core';
import type { IRegisterFunctionService } from '@univerjs/sheets-formula';

// ─── Argument parsing helpers ────────────────────────────────────────────────

/** Parse arg at index as number. */
function num(args: unknown[], i: number): number {
  const v = args[i];
  return typeof v === 'number' ? v : parseFloat(v as string) || 0;
}

// ─── Registration helpers ────────────────────────────────────────────────────

/** Register a 1-arg async function backed by a Tauri command. */
function reg1(
  svc: IRegisterFunctionService,
  name: string,
  cmd: string,
  desc: string
) {
  svc.registerAsyncFunction({
    name,
    func: async (...args: unknown[]) =>
      invoke<number>(cmd, { x: num(args, 0) }),
    description: desc,
  });
}

/** Register a 2-arg async function backed by a Tauri command. */
function reg2(
  svc: IRegisterFunctionService,
  name: string,
  cmd: string,
  p: [string, string],
  desc: string
) {
  svc.registerAsyncFunction({
    name,
    func: async (...args: unknown[]) =>
      invoke<number>(cmd, { [p[0]]: num(args, 0), [p[1]]: num(args, 1) }),
    description: desc,
  });
}

/**
 * Register all custom mathematical functions with the formula engine.
 *
 * These are functions NOT natively supported by Univer, backed by
 * pre-compiled symb_anafis evaluators in the Rust backend.
 */
export function registerCustomFunctions(svc: IRegisterFunctionService) {
  // ── Inverse Trigonometric ──────────────────────────────────────────────
  reg1(svc, 'ASEC', 'math_asec', 'Inverse secant: asec(x) = acos(1/x)');
  reg1(svc, 'ACSC', 'math_acsc', 'Inverse cosecant: acsc(x) = asin(1/x)');

  // ── Inverse Hyperbolic ─────────────────────────────────────────────────
  reg1(svc, 'ASECH', 'math_asech', 'Inverse hyperbolic secant: asech(x)');
  reg1(
    svc,
    'ACSCH',
    'math_acsch',
    'Inverse hyperbolic cosecant: acsch(x) = asinh(1/x)'
  );

  // ── Gamma Family ───────────────────────────────────────────────────────
  reg1(svc, 'GAMMA', 'math_gamma', 'Gamma function: Γ(x)');
  reg1(
    svc,
    'DIGAMMA',
    'math_digamma',
    'Digamma function: ψ(x) = d/dx ln(Γ(x))'
  );
  reg1(
    svc,
    'TRIGAMMA',
    'math_trigamma',
    'Trigamma function: ψ₁(x) = d²/dx² ln(Γ(x))'
  );
  reg1(
    svc,
    'TETRAGAMMA',
    'math_tetragamma',
    'Tetragamma function: ψ₂(x) = d³/dx³ ln(Γ(x))'
  );
  reg2(
    svc,
    'POLYGAMMA',
    'math_polygamma',
    ['n', 'x'],
    'Polygamma function: ψₙ(x)'
  );
  reg2(
    svc,
    'BETA',
    'math_beta',
    ['a', 'b'],
    'Beta function: B(a, b) = Γ(a)Γ(b) / Γ(a+b)'
  );

  // ── Zeta ───────────────────────────────────────────────────────────────
  reg1(svc, 'ZETA', 'math_zeta', 'Riemann zeta function: ζ(s)');
  reg2(
    svc,
    'ZETA_DERIV',
    'math_zeta_deriv',
    ['n', 's'],
    'Zeta derivative: ζ⁽ⁿ⁾(s)'
  );

  // ── Elliptic Integrals ─────────────────────────────────────────────────
  reg1(
    svc,
    'ELLIPTIC_K',
    'math_elliptic_k',
    'Complete elliptic integral of the first kind: K(k)'
  );
  reg1(
    svc,
    'ELLIPTIC_E',
    'math_elliptic_e',
    'Complete elliptic integral of the second kind: E(k)'
  );

  // ── Orthogonal Polynomials ─────────────────────────────────────────────
  reg2(svc, 'HERMITE', 'math_hermite', ['n', 'x'], 'Hermite polynomial: Hₙ(x)');

  // Associated Legendre — 3 args
  svc.registerAsyncFunction({
    name: 'ASSOC_LEGENDRE',
    func: async (...args: unknown[]) =>
      invoke<number>('math_assoc_legendre', {
        l: num(args, 0),
        m: num(args, 1),
        x: num(args, 2),
      }),
    description: 'Associated Legendre polynomial: Pₗᵐ(x)',
  });

  // ── Spherical Harmonics ────────────────────────────────────────────────
  svc.registerAsyncFunction({
    name: 'SPHERICAL_HARMONIC',
    func: async (...args: unknown[]) =>
      invoke<number>('math_spherical_harmonic', {
        l: num(args, 0),
        m: num(args, 1),
        theta: num(args, 2),
        phi: num(args, 3),
      }),
    description: 'Spherical harmonic: Yₗᵐ(θ, φ)',
  });

  // ── Other ──────────────────────────────────────────────────────────────
  reg1(
    svc,
    'SINC',
    'math_sinc',
    'Sinc function: sinc(x) = sin(x)/x, sinc(0) = 1'
  );
  reg1(
    svc,
    'LAMBERTW',
    'math_lambertw',
    'Lambert W function: W(x) * exp(W(x)) = x'
  );
  reg1(svc, 'CBRT', 'math_cbrt', 'Cube root: cbrt(x) = x^(1/3)');
}
