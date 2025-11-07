/**
 * Custom mathematical functions for the spreadsheet
 * These functions are registered with Univer's formula engine
 */

// Import the service type
import type { IRegisterFunctionService } from '@univerjs/sheets-formula';
import type { FormulaFunctionValueType, FormulaFunctionResultValueType } from '@univerjs/engine-formula';
import { ERROR_MESSAGES } from '@/tabs/spreadsheet/univer/utils/constants';

/**
 * Gamma function approximation using Lanczos approximation
 * Good accuracy for most practical purposes
 */
function gamma(x: number): number {
    if (x < 0.5) {
        // Use gamma(x) = gamma(x+1)/x for x < 0.5
        return gamma(x + 1) / x;
    }

    // Lanczos coefficients
    const g = 7;
    const p = [0.99999999999980993, 676.5203681218851, -1259.1392167224028,
               771.32342877765313, -176.61502916214059, 12.507343278686905,
               -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7];

    x -= 1;
    let a = p[0]!; // Safe access - we know p[0] exists
    const t = x + g + 0.5;
    for (let i = 1; i < p.length; i++) {
        const pi = p[i]!;
        a += pi / (x + i);
    }

    return Math.sqrt(2 * Math.PI) * Math.pow(t, x + 0.5) * Math.exp(-t) * a;
}

/**
 * Log Gamma function for numerical stability
 */
function logGamma(x: number): number {
    if (x < 0.5) {
        // Use reflection formula
        return Math.log(Math.PI) - Math.log(Math.sin(Math.PI * x)) - logGamma(1 - x);
    }

    // Lanczos approximation in log space
    const g = 7;
    const p = [0.99999999999980993, 676.5203681218851, -1259.1392167224028,
               771.32342877765313, -176.61502916214059, 12.507343278686905,
               -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7];

    x -= 1;
    let a = p[0]!; // Safe access - we know p[0] exists
    const t = x + g + 0.5;
    for (let i = 1; i < p.length; i++) {
        const pi = p[i]!;
        a += pi / (x + i);
    }

    return 0.5 * Math.log(2 * Math.PI) + (x + 0.5) * Math.log(t) - t + Math.log(a);
}

/**
 * Digamma function (derivative of gamma function) using series approximation
 */
function digamma(x: number): number {
    // Handle poles at non-positive integers
    if (x <= 0 && x === Math.floor(x)) {
        return Infinity; // Positive infinity for poles
    }

    if (x < 0) {
        // Use reflection formula: ψ(1-x) = ψ(x) + π*cot(π*x)
        // But handle near-pole cases carefully
        return digamma(1 - x) - Math.PI / Math.tan(Math.PI * x);
    }

    // Series approximation for x > 0
    let result = 0;
    let xx = x;

    // Use recurrence: ψ(x+1) = ψ(x) + 1/x
    while (xx < 8) {
        result -= 1 / xx;
        xx += 1;
    }

    // Series approximation for large x
    const y = 1 / (xx * xx);
    result += Math.log(xx) - 0.5 / xx - y * (1/12 - y * (1/120 - y * (1/252 - y/240)));

    return result;
}

/**
 * Lambert W function using Newton's method (principal branch)
 * W(x) is the inverse function of x*e^x
 */
function lambertW(x: number): number {
    // CORRECTED: Function is defined for x ≥ -1/e
    if (x < -1/Math.E) {
        throw new Error(ERROR_MESSAGES.LAMBERT_W_UNDEFINED);
    }

    // Special case for the boundary
    if (Math.abs(x + 1/Math.E) < 1e-15) {
        return -1;
    }

    // Initial approximation
    let w: number;
    if (x === 0) {return 0;}
    if (x < 0) {
        // For negative values near -1/e, start close to -1
        w = -1 + Math.sqrt(2 * (x + 1/Math.E));
    } else if (x < 1) {
        w = x; // For small positive x
    } else {
        w = Math.log(x); // For larger x, start with ln(x)
    }

    // Newton's method: w_{n+1} = w_n - f(w_n)/f'(w_n)
    // where f(w) = w*e^w - x, f'(w) = e^w * (w + 1)
    for (let i = 0; i < 50; i++) {
        const ew: number = Math.exp(w);
        const f: number = w * ew - x;
        const fp: number = ew * (w + 1);

        // Avoid division by zero
        if (Math.abs(fp) < 1e-15) {break;}

        const delta: number = f / fp;
        w = w - delta;
        if (Math.abs(delta) < 1e-15) {break;}
    }

    return w;
}

/**
 * Hermite polynomial H_n(x) using recurrence relation
 * H_0(x) = 1
 * H_1(x) = 2x
 * H_{n+1}(x) = 2x*H_n(x) - 2n*H_{n-1}(x)
 */
function hermite(n: number, x: number): number {
    if (n < 0 || n !== Math.floor(n)) {
        throw new Error(ERROR_MESSAGES.HERMITE_NEGATIVE_DEGREE);
    }

    if (n === 0) {return 1;}
    if (n === 1) {return 2 * x;}

    let h_prev2 = 1; // H_0
    let h_prev1 = 2 * x; // H_1

    for (let i = 2; i <= n; i++) {
        const h_current = 2 * x * h_prev1 - 2 * (i - 1) * h_prev2;
        h_prev2 = h_prev1;
        h_prev1 = h_current;
    }

    return h_prev1;
}

/**
 * Complete elliptic integral of the first kind: K(k) = ∫₀^¹ dt / √((1-t²)(1-k²t²))
 * Using arithmetic-geometric mean with correct implementation
 */
function elliptic_k(k: number): number {
    if (Math.abs(k) >= 1) {
        if (k === 1 || k === -1) {return Infinity;}
        throw new Error(ERROR_MESSAGES.MODULUS_OUT_OF_RANGE);
    }
    
    if (k === 0) {return Math.PI / 2;}
    
    // Arithmetic-geometric mean method
    let a = 1.0;
    let b = Math.sqrt(1.0 - k * k);
    
    const tolerance = 1e-15;
    let iterations = 0;
    const maxIterations = 100;
    
    while (Math.abs(a - b) > tolerance && iterations < maxIterations) {
        const a_next = (a + b) / 2.0;
        const b_next = Math.sqrt(a * b);
        
        a = a_next;
        b = b_next;
        iterations++;
    }
    
    return Math.PI / (2.0 * a);
}

/**
 * Riemann zeta function: ζ(s) = Σ(1/n^s) for n=1 to ∞
 * For s > 1, use direct summation
 * For s = 2, use π²/6
 * For other values, use approximations
 */
function zeta(s: number): number {
    if (s === 1) {
        throw new Error(ERROR_MESSAGES.ZETA_FUNCTION_POLE);
    }
    
    if (s === 2) {
        return Math.PI * Math.PI / 6;
    }
    
    if (s > 1) {
        // Direct summation for s > 1
        let sum = 0;
        const terms = 50000; // More terms for better precision
        for (let n = 1; n <= terms; n++) {
            sum += 1 / Math.pow(n, s);
        }
        return sum;
    } else {
        // For s <= 1, use functional equation approximation
        // ζ(s) = 2^s π^{s-1} sin(π s / 2) Γ(1-s) ζ(1-s)
        // This is complex, so for now throw error for s <= 1 except s=2
        throw new Error(ERROR_MESSAGES.ZETA_FUNCTION_LIMITATION);
    }
}

/**
 * Register all custom mathematical functions with the formula engine
 */
export function registerCustomFunctions(formulaEngine: IRegisterFunctionService) {
    // Register SINC function with high precision using JavaScript
    formulaEngine.registerFunction({
        name: 'SINC',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            if (x === 0) {return 1;}
            return Math.sin(x) / x;
        },
        description: 'Sinc function: sinc(x) = sin(x)/x, with sinc(0) = 1.'
    });

    // Register ACOT function
    formulaEngine.registerFunction({
        name: 'ACOT',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            return Math.atan(1 / x);
        },
        description: 'Inverse cotangent function: acot(x) = atan(1/x)'
    });

    // Register ASEC function
    formulaEngine.registerFunction({
        name: 'ASEC',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            return Math.acos(1 / x);
        },
        description: 'Inverse secant function: asec(x) = acos(1/x)'
    });

    // Register ACSC function
    formulaEngine.registerFunction({
        name: 'ACSC',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            return Math.asin(1 / x);
        },
        description: 'Inverse cosecant function: acsc(x) = asin(1/x)'
    });

    // Register ACOTH function
    formulaEngine.registerFunction({
        name: 'ACOTH',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            if (Math.abs(x) <= 1) {return NaN;} // Domain error
            return 0.5 * Math.log((x + 1) / (x - 1));
        },
        description: 'Inverse hyperbolic cotangent function: acoth(x) = 0.5 * ln((x+1)/(x-1))'
    });

    // Register ASECH function
    formulaEngine.registerFunction({
        name: 'ASECH',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            if (x <= 0 || x > 1) {return NaN;} // Domain: (0, 1]
            return Math.log((1 + Math.sqrt(1 - x * x)) / x);
        },
        description: 'Inverse hyperbolic secant function: asech(x) = ln((1 + √(1-x²))/x)'
    });

    // Register ACSCH function
    formulaEngine.registerFunction({
        name: 'ACSCH',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            return Math.asinh(1 / x);
        },
        description: 'Inverse hyperbolic cosecant function: acsch(x) = asinh(1/x)'
    });

    // Register BETA function (Beta function)
    formulaEngine.registerFunction({
        name: 'BETA',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            const y = typeof args[1] === 'number' ? args[1] : parseFloat(args[1] as string) || 0;
            try {
                // Use logarithms for large values to avoid overflow
                if (x > 50 || y > 50 || x + y > 50) {
                    return Math.exp(logGamma(x) + logGamma(y) - logGamma(x + y));
                }
                return gamma(x) * gamma(y) / gamma(x + y);
            } catch {
                return NaN;
            }
        },
        description: 'Beta function: B(x,y) = Γ(x)Γ(y)/Γ(x+y)'
    });

    // Register DIGAMMA function
    formulaEngine.registerFunction({
        name: 'DIGAMMA',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            try {
                return digamma(x);
            } catch {
                return NaN;
            }
        },
        description: 'Digamma function: ψ(x) = d/dx ln(Γ(x))'
    });

    // Register LAMBERTW function
    formulaEngine.registerFunction({
        name: 'LAMBERTW',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            try {
                return lambertW(x);
            } catch {
                return NaN;
            }
        },
        description: 'Lambert W function: W(x) where W(x) * exp(W(x)) = x'
    });

    // Register HERMITE function
    formulaEngine.registerFunction({
        name: 'HERMITE',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const n = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            const x = typeof args[1] === 'number' ? args[1] : parseFloat(args[1] as string) || 0;
            try {
                return hermite(n, x);
            } catch {
                return NaN;
            }
        },
        description: 'Hermite polynomial: H_n(x) - orthogonal polynomials'
    });

    // Register GAMMA function
    formulaEngine.registerFunction({
        name: 'GAMMA',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const x = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            try {
                return gamma(x);
            } catch {
                return NaN;
            }
        },
        description: 'Gamma function: Γ(x) = ∫₀^∞ t^(x-1) e^(-t) dt'
    });

    // Register ZETA function
    formulaEngine.registerFunction({
        name: 'ZETA',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const s = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            try {
                return zeta(s);
            } catch {
                return NaN;
            }
        },
        description: 'Riemann zeta function: ζ(s) = Σ(1/n^s) for n=1 to ∞'
    });

    // Register ELLIPTIC_K function
    formulaEngine.registerFunction({
        name: 'ELLIPTIC_K',
        func: (...args: FormulaFunctionValueType[]): FormulaFunctionResultValueType => {
            const k = typeof args[0] === 'number' ? args[0] : parseFloat(args[0] as string) || 0;
            try {
                return elliptic_k(k);
            } catch {
                return NaN;
            }
        },
        description: 'Complete elliptic integral of the first kind: K(k) = ∫₀^¹ dt / √((1-t²)(1-k²t²))'
    });
}

/**
 * List of custom function names for formula interception logic
 */
export const CUSTOM_FUNCTION_NAMES = [
    'GAMMA',
    'SINC',
    'ACOT',
    'ASEC',
    'ACSC',
    'ACOTH',
    'ASECH',
    'ACSCH',
    'BETA',
    'DIGAMMA',
    'LAMBERTW',
    'HERMITE',
    'ZETA',
    'ELLIPTIC_K'
];