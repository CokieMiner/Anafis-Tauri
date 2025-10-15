import sympy as sp
from sympy.parsing.sympy_parser import (
    parse_expr,
    standard_transformations,
    implicit_multiplication_application,
)
from sympy.printing.latex import latex


def _get_math_functions():
    """Get mathematical functions dictionary for sympy evaluation."""
    return {
        # Trigonometric functions
        "sin": sp.sin,
        "sen": sp.sin,  # Portuguese/Spanish alias for sin
        "cos": sp.cos,
        "tan": sp.tan,
        "cot": sp.cot,
        "sec": sp.sec,
        "csc": sp.csc,
        "asin": sp.asin,
        "acos": sp.acos,
        "atan": sp.atan,
        "acot": sp.acot,
        "asec": sp.asec,
        "acsc": sp.acsc,
        "sinh": sp.sinh,
        "cosh": sp.cosh,
        "tanh": sp.tanh,
        "coth": sp.coth,
        "sech": sp.sech,
        "csch": sp.csch,
        "asinh": sp.asinh,
        "acosh": sp.acosh,
        "atanh": sp.atanh,
        "acoth": sp.acoth,
        "asech": sp.asech,
        "acsch": sp.acsch,
        # Logarithmic functions
        "log": sp.log,  # log(x) = log10(x), log(x, base) = log_base(x)
        "log10": lambda x: sp.log(x, 10),  # explicit base-10 log
        "log2": lambda x: sp.log(x, 2),  # base-2 log
        "ln": sp.ln,  # natural log
        # Exponential functions
        "exp": sp.exp,
        "exp_polar": sp.exp_polar,
        # Powers and roots
        "sqrt": sp.sqrt,
        "cbrt": lambda x: x ** sp.Rational(1, 3),
        "root": lambda x, n: x ** sp.Rational(1, n),
        # Special functions
        "erf": sp.erf,
        "erfc": sp.erfc,
        "gamma": sp.gamma,
        "beta": sp.beta,
        "zeta": sp.zeta,
        # Bessel functions
        "besselj": sp.besselj,  # Bessel function of the first kind
        "bessely": sp.bessely,  # Bessel function of the second kind
        "besseli": sp.besseli,  # Modified Bessel function of the first kind
        "besselk": sp.besselk,  # Modified Bessel function of the second kind
        # Additional special functions
        "sinc": sp.sinc,  # Sinc function (sin(x)/x)
        "digamma": sp.digamma,  # Digamma function (derivative of gamma)
        "LambertW": sp.LambertW,  # Lambert W function
        # Advanced functions for quantum mechanics and electromagnetism
        "Ynm": sp.Ynm,  # Spherical harmonics
        "assoc_legendre": sp.assoc_legendre,  # Associated Legendre polynomials
        "hermite": sp.hermite,  # Hermite polynomials
        "elliptic_e": sp.elliptic_e,  # Complete elliptic integral of the second kind
        "elliptic_k": sp.elliptic_k,  # Complete elliptic integral of the first kind

        # Constants
        "pi": sp.pi,
        "e": sp.E,
        "E": sp.E,
        "I": sp.I,
        "j": sp.I,  # Imaginary unit
    }


def _preprocess_formula(formula, variables, math_functions):
    """
    Preprocess formula to handle implicit
    multiplication while preserving function names.
    """
    symbols = {var: sp.Symbol(var) for var in variables}
    combined_locals = {**symbols, **math_functions}
    transformations = (
        standard_transformations + (implicit_multiplication_application,)
    )
    expr = parse_expr(
        formula, local_dict=combined_locals, transformations=transformations
    )
    return expr


def calculate_symbolic_data(formula, variables, values):
    """
    Return expression, derivatives, and numerical values for Rust processing.
    """
    try:
        math_functions = _get_math_functions()
        expr = _preprocess_formula(formula, variables, math_functions)

        derivatives = {}
        for var in variables:
            deriv = sp.diff(expr, sp.Symbol(var))
            derivatives[var] = str(deriv)

        # Compute numerical values
        value_subs = {var: values[i] for i, var in enumerate(variables)}
        expr_value = expr.subs(value_subs)
        if expr_value.free_symbols:
            raise ValueError(
                "Expression could not be fully evaluated: "
                "missing variables?"
            )
        expr_value = expr_value.evalf()
        numerical_value = float(expr_value)

        numerical_derivatives = {}
        for var in variables:
            deriv_expr = sp.sympify(derivatives[var])
            deriv_value = deriv_expr.subs(value_subs).evalf()
            if abs(sp.im(deriv_value)) > 1e-10:
                raise ValueError(
                    f"Derivative with respect to {var} has a "
                    "significant imaginary part."
                )
            deriv_value = sp.re(deriv_value)
            numerical_derivatives[var] = float(deriv_value)

        return {
            "expression": str(expr),
            "derivatives": derivatives,
            "numerical_value": numerical_value,
            "numerical_derivatives": numerical_derivatives,
            "success": True,
        }
    except sp.SympifyError as e:
        return {
            "expression": formula,
            "derivatives": {},
            "numerical_value": 0.0,
            "numerical_derivatives": {},
            "success": False,
            "error": f"Invalid formula syntax: {str(e)}",
        }
    except ValueError as e:
        return {
            "expression": formula,
            "derivatives": {},
            "numerical_value": 0.0,
            "numerical_derivatives": {},
            "success": False,
            "error": f"Evaluation error: {str(e)}",
        }
    except Exception as e:
        return {
            "expression": formula,
            "derivatives": {},
            "numerical_value": 0.0,
            "numerical_derivatives": {},
            "success": False,
            "error": f"Unexpected error: {str(e)}",
        }


def generate_latex_data(formula, variables):
    """
    Return uncertainty propagation formula with LaTeX for Rust formatting.
    """
    try:
        math_functions = _get_math_functions()
        expr = _preprocess_formula(formula, variables, math_functions)

        # Calculate derivatives for each variable
        derivatives = {}
        uncertainty_terms = []

        for var in variables:
            deriv = sp.diff(expr, sp.Symbol(var))
            derivatives[var] = deriv

            # Create uncertainty term: (∂f/∂var * σ_var)²
            sigma_var = sp.Symbol(f'σ_{var}')
            term = (deriv * sigma_var)**2
            uncertainty_terms.append(term)

        # Sum all uncertainty terms
        uncertainty_sum = sum(uncertainty_terms)

        # Take square root for final uncertainty
        uncertainty_expr = sp.sqrt(uncertainty_sum)

        # Generate LaTeX and string representations
        latex_formula = latex(uncertainty_expr, mul_symbol="dot")
        string_formula = str(uncertainty_expr)

        return {
            "string": string_formula,
            "latex": latex_formula,
            "success": True
        }

    except Exception as e:
        return {
            "string": "",
            "latex": "",
            "success": False,
            "error": str(e)
        }
