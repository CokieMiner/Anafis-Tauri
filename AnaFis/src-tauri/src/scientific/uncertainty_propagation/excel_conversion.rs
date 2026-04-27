use super::types::ExcelRange;
use regex::{NoExpand, Regex, escape};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::LazyLock;
use thiserror::Error;

static EULER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[eE]\b").expect("Valid static regex for Euler constant"));

/// Error type for Excel conversion operations
#[derive(Debug, Error)]
pub enum ConversionError {
    /// Identifier in formula not present in mapping.
    #[error("Identifier not found in mapping: {0}")]
    IdentifierNotFound(String),
    /// Function not supported by the Excel converter.
    #[error("Unsupported function: {0}")]
    UnsupportedFunction(String),
}

/// Error type for Excel range parsing
#[derive(Debug, Error)]
pub enum RangeError {
    /// The range format (e.g., "A1:B10") is invalid.
    #[error("Invalid range format: {0}")]
    InvalidFormat(String),

    /// A specific cell identifier (e.g., "A1") is invalid.
    #[error("Invalid cell format: {0}")]
    InvalidCell(String),

    /// A row number is out of bounds or not a positive integer.
    #[error("Invalid row number in range: {0}")]
    InvalidRow(String),
}

/// Convert `symb_anafis` derivative expression to Excel formula
///
/// This function converts mathematical expressions from `symb_anafis`'s format to Excel's format,
/// handling function names, operators, and variable substitutions.
///
/// # Arguments
/// * `symb_anafis_expr` - The expression string from `symb_anafis`
/// * `var_map` - Maps variable names to Excel cell references (e.g., "x" -> "A1")
///
/// # Returns
/// An Excel formula string (without the leading "=")
///
/// # Examples
/// ```
/// # use std::collections::HashMap;
/// # use anafis_lib::scientific::uncertainty_propagation::excel_conversion::symb_anafis_to_excel;
/// let mut var_map = HashMap::new();
/// var_map.insert("x".to_string(), "A1".to_string());
/// let formula = symb_anafis_to_excel("2*x", &var_map).unwrap();
/// assert_eq!(formula, "2*A1");
/// ```
/// # Errors
/// Returns `ConversionError::UnsupportedExcelFunction` if the expression contains functions
/// not supported by Excel.
///
/// # Panics
/// Panics if internal regex compilation fails.
pub fn symb_anafis_to_excel<S: BuildHasher>(
    symb_anafis_expr: &str,
    var_map: &HashMap<String, String, S>,
) -> Result<String, ConversionError> {
    // All symb_anafis functions are now available as custom formulas in AnaFis
    // (see math_functions.rs), so no unsupported function check is needed.

    let mut excel_formula = symb_anafis_expr.to_owned();

    // Replace variable names with cell references using identifier boundaries.
    // This prevents replacing `a` inside `sigma_a` and similar partial matches.
    excel_formula = replace_identifiers(&excel_formula, var_map);

    // Convert operators
    excel_formula = excel_formula.replace("**", "^"); // Power operator

    // Convert logarithmic functions (ORDER MATTERS! Specific before general)
    excel_formula = excel_formula.replace("log10", "LOG10");
    excel_formula = excel_formula.replace("log2", "LOG2");
    excel_formula = excel_formula.replace("ln", "LN"); // Natural log
    excel_formula = excel_formula.replace("log", "LOG"); // General log

    // Convert exponential and roots
    excel_formula = excel_formula.replace("sqrt", "SQRT");
    // Convert cbrt directly (custom formula via symb_anafis, no workaround needed)
    excel_formula = excel_formula.replace("cbrt", "CBRT");
    excel_formula = excel_formula.replace("exp_polar", "EXP"); // Approximation - must come before exp
    excel_formula = excel_formula.replace("exp", "EXP");

    // Convert inverse trig (must come before regular trig to avoid double replacement)
    // ORDER: longer names first (asec before sec, acsc before csc, etc.)
    excel_formula = excel_formula.replace("asin", "ASIN");
    excel_formula = excel_formula.replace("acos", "ACOS");
    excel_formula = excel_formula.replace("atan2", "ATAN2"); // Must come before atan
    excel_formula = excel_formula.replace("atan", "ATAN");
    excel_formula = excel_formula.replace("acot", "ACOT");
    excel_formula = excel_formula.replace("asec", "ASEC");
    excel_formula = excel_formula.replace("acsc", "ACSC");

    // Convert regular trig (Univer supports COT, SEC, CSC natively)
    excel_formula = excel_formula.replace("sinc", "SINC"); // Must come before sin
    excel_formula = excel_formula.replace("sin", "SIN");
    excel_formula = excel_formula.replace("sen", "SIN"); // Portuguese/Spanish alias
    excel_formula = excel_formula.replace("cos", "COS");
    excel_formula = excel_formula.replace("cot", "COT");
    excel_formula = excel_formula.replace("csc", "CSC");
    excel_formula = excel_formula.replace("sec", "SEC");
    excel_formula = excel_formula.replace("tan", "TAN");

    // Convert inverse hyperbolic (must come before regular hyperbolic)
    excel_formula = excel_formula.replace("asinh", "ASINH");
    excel_formula = excel_formula.replace("acosh", "ACOSH");
    excel_formula = excel_formula.replace("atanh", "ATANH");
    excel_formula = excel_formula.replace("acoth", "ACOTH");
    excel_formula = excel_formula.replace("asech", "ASECH");
    excel_formula = excel_formula.replace("acsch", "ACSCH");

    // Convert hyperbolic functions (Univer supports COTH, SECH, CSCH natively)
    excel_formula = excel_formula.replace("sinh", "SINH");
    excel_formula = excel_formula.replace("cosh", "COSH");
    excel_formula = excel_formula.replace("coth", "COTH");
    excel_formula = excel_formula.replace("csch", "CSCH");
    excel_formula = excel_formula.replace("sech", "SECH");
    excel_formula = excel_formula.replace("tanh", "TANH");

    // Convert special functions (native Excel/Univer)
    excel_formula = excel_formula.replace("erf", "ERF");
    excel_formula = excel_formula.replace("erfc", "ERFC");
    excel_formula = excel_formula.replace("besselj", "BESSELJ");
    excel_formula = excel_formula.replace("bessely", "BESSELY");
    excel_formula = excel_formula.replace("besseli", "BESSELI");
    excel_formula = excel_formula.replace("besselk", "BESSELK");

    // Convert special functions (available as AnaFis custom formulas via symb_anafis)
    // ORDER MATTERS: longer names before shorter to avoid partial matches
    // (e.g. "digamma" must be replaced before "gamma", otherwise "di" + "GAMMA")
    excel_formula = excel_formula.replace("tetragamma", "TETRAGAMMA");
    excel_formula = excel_formula.replace("trigamma", "TRIGAMMA");
    excel_formula = excel_formula.replace("digamma", "DIGAMMA");
    excel_formula = excel_formula.replace("polygamma", "POLYGAMMA");
    excel_formula = excel_formula.replace("gamma", "GAMMA");
    excel_formula = excel_formula.replace("beta", "BETA");
    excel_formula = excel_formula.replace("zeta_deriv", "ZETA_DERIV"); // Must come before "zeta"
    excel_formula = excel_formula.replace("zeta", "ZETA");
    excel_formula = excel_formula.replace("elliptic_k", "ELLIPTIC_K");
    excel_formula = excel_formula.replace("elliptic_e", "ELLIPTIC_E");
    excel_formula = excel_formula.replace("hermite", "HERMITE");
    excel_formula = excel_formula.replace("lambertw", "LAMBERTW");
    // Spherical harmonics and associated Legendre (custom formulas via symb_anafis)
    excel_formula = excel_formula.replace("spherical_harmonic", "SPHERICAL_HARMONIC");
    excel_formula = excel_formula.replace("Ynm", "SPHERICAL_HARMONIC"); // Alias
    excel_formula = excel_formula.replace("assoc_legendre", "ASSOC_LEGENDRE");

    // Convert other functions
    excel_formula = excel_formula.replace("abs", "ABS");
    excel_formula = excel_formula.replace("signum", "SIGN"); // Sign function
    excel_formula = excel_formula.replace("floor", "FLOOR");
    excel_formula = excel_formula.replace("ceil", "CEILING");
    excel_formula = excel_formula.replace("round", "ROUND");

    // Convert constants (MUST come after function replacements to avoid conflicts)
    excel_formula = excel_formula.replace("pi", "PI()");
    // Use word boundaries to only match standalone 'e' or 'E' (Euler's constant),
    // not when they appear as part of function names like ERF or CEILING
    excel_formula = EULER_REGEX
        .replace_all(&excel_formula, "EXP(1)")
        .to_string();
    Ok(excel_formula)
}

fn replace_identifiers<S: BuildHasher>(
    formula: &str,
    var_map: &HashMap<String, String, S>,
) -> String {
    let mut output = formula.to_owned();

    // Replace longer identifiers first to keep behavior deterministic with overlapping names.
    let mut pairs: Vec<_> = var_map.iter().collect();
    pairs.sort_by(|(left, _), (right, _)| {
        right.len().cmp(&left.len()).then_with(|| left.cmp(right))
    });

    for (name, replacement) in pairs {
        let pattern = format!(r"\b{}\b", escape(name));
        let re = Regex::new(&pattern).expect("escaped identifier pattern must compile");
        output = re
            .replace_all(&output, NoExpand(replacement.as_str()))
            .to_string();
    }

    output
}

/// Parse Excel range notation into structured `ExcelRange`
///
/// Supports both single cell (e.g., "A1") and range (e.g., "A1:A10") formats.
///
/// # Arguments
/// * `range` - The range string to parse
///
/// # Returns
/// An `ExcelRange` struct containing column, `start_row`, and `end_row`
/// # Errors
/// Returns `RangeError` if the range format is invalid or rows are not numbers.
pub fn parse_excel_range(range: &str) -> Result<ExcelRange, RangeError> {
    let parts: Vec<&str> = range.split(':').collect();

    if parts.len() == 1 {
        // Single cell format like "A1"
        let cell = parts[0];
        let col = cell
            .chars()
            .take_while(|c| c.is_alphabetic())
            .collect::<String>();
        let row: usize = cell
            .chars()
            .skip_while(|c| c.is_alphabetic())
            .collect::<String>()
            .parse()
            .map_err(|_err| RangeError::InvalidRow(range.to_owned()))?;

        if col.is_empty() {
            return Err(RangeError::InvalidCell(range.to_owned()));
        }

        Ok(ExcelRange::new(col, row, row))
    } else if parts.len() == 2 {
        // Range format like "A1:A10"
        let start_col = parts[0]
            .chars()
            .take_while(|c| c.is_alphabetic())
            .collect::<String>();
        let start_row: usize = parts[0]
            .chars()
            .skip_while(|c| c.is_alphabetic())
            .collect::<String>()
            .parse()
            .map_err(|_err| RangeError::InvalidRow(format!("start: {range}")))?;

        let end_row: usize = parts[1]
            .chars()
            .skip_while(|c| c.is_alphabetic())
            .collect::<String>()
            .parse()
            .map_err(|_err| RangeError::InvalidRow(format!("end: {range}")))?;

        Ok(ExcelRange::new(start_col, start_row, end_row))
    } else {
        Err(RangeError::InvalidFormat(range.to_owned()))
    }
}

/// Generate Excel cell reference from column and row
///
/// # Examples
/// ```
/// # use anafis_lib::scientific::uncertainty_propagation::excel_conversion::create_cell_ref;
/// assert_eq!(create_cell_ref("A", 1), "A1");
/// assert_eq!(create_cell_ref("BC", 42), "BC42");
/// ```
#[must_use]
pub fn create_cell_ref(col: &str, row: usize) -> String {
    format!("{col}{row}")
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::shadow_unrelated,
    reason = "Tests use unwrap for brevity and sequential shadowing for state progression"
)]
mod tests {
    use super::*;

    #[test]
    fn test_symb_anafis_to_excel_power() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());

        let result = symb_anafis_to_excel("x**2", &var_map).unwrap();
        assert_eq!(result, "A1^2");
    }

    #[test]
    fn test_symb_anafis_to_excel_trig() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());

        let result = symb_anafis_to_excel("sin(x)", &var_map).unwrap();
        assert_eq!(result, "SIN(A1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_sigma_identifiers_do_not_get_partial_replacements() {
        let mut var_map = HashMap::new();
        var_map.insert("a".to_owned(), "A1".to_owned());
        var_map.insert("b".to_owned(), "C1".to_owned());
        var_map.insert("sigma_a".to_owned(), "B1".to_owned());
        var_map.insert("sigma_b".to_owned(), "D1".to_owned());

        let result =
            symb_anafis_to_excel("sqrt(sigma_b**2 + (sigma_a*b*cos(a))**2)", &var_map).unwrap();
        assert_eq!(result, "SQRT(D1^2 + (B1*C1*COS(A1))^2)");
    }

    #[test]
    fn test_symb_anafis_to_excel_atan2() {
        let mut var_map = HashMap::new();
        var_map.insert("y".to_owned(), "A1".to_owned());
        var_map.insert("x".to_owned(), "B1".to_owned());

        let result = symb_anafis_to_excel("atan2(y, x)", &var_map).unwrap();
        assert_eq!(result, "ATAN2(A1, B1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_log() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());

        let result = symb_anafis_to_excel("log10(x)", &var_map).unwrap();
        assert_eq!(result, "LOG10(A1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_special_functions() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());
        var_map.insert("y".to_owned(), "B1".to_owned());

        // Test error functions
        let result = symb_anafis_to_excel("erf(x)", &var_map).unwrap();
        assert_eq!(result, "ERF(A1)");

        // Test Bessel functions
        let result = symb_anafis_to_excel("besselj(0, x)", &var_map).unwrap();
        assert_eq!(result, "BESSELJ(0, A1)");

        // Test beta function
        let result = symb_anafis_to_excel("beta(x, y)", &var_map).unwrap();
        assert_eq!(result, "BETA(A1, B1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_rounding() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());

        let result = symb_anafis_to_excel("floor(x)", &var_map).unwrap();
        assert_eq!(result, "FLOOR(A1)");

        let result = symb_anafis_to_excel("ceil(x)", &var_map).unwrap();
        assert_eq!(result, "CEILING(A1)");

        let result = symb_anafis_to_excel("round(x)", &var_map).unwrap();
        assert_eq!(result, "ROUND(A1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_cbrt() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());

        let result = symb_anafis_to_excel("cbrt(x)", &var_map).unwrap();
        assert_eq!(result, "CBRT(A1)");

        // Test with expression
        let result = symb_anafis_to_excel("cbrt(x + 2)", &var_map).unwrap();
        assert_eq!(result, "CBRT(A1 + 2)");
    }

    #[test]
    fn test_symb_anafis_to_excel_signum() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());

        let result = symb_anafis_to_excel("signum(x)", &var_map).unwrap();
        assert_eq!(result, "SIGN(A1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_custom_functions() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());
        var_map.insert("y".to_owned(), "B1".to_owned());

        // Functions now available as custom formulas via symb_anafis
        let result = symb_anafis_to_excel("digamma(x)", &var_map).unwrap();
        assert_eq!(result, "DIGAMMA(A1)");

        let result = symb_anafis_to_excel("trigamma(x)", &var_map).unwrap();
        assert_eq!(result, "TRIGAMMA(A1)");

        let result = symb_anafis_to_excel("tetragamma(x)", &var_map).unwrap();
        assert_eq!(result, "TETRAGAMMA(A1)");

        let result = symb_anafis_to_excel("polygamma(2, x)", &var_map).unwrap();
        assert_eq!(result, "POLYGAMMA(2, A1)");

        let result = symb_anafis_to_excel("zeta_deriv(1, x)", &var_map).unwrap();
        assert_eq!(result, "ZETA_DERIV(1, A1)");

        let result = symb_anafis_to_excel("elliptic_e(x)", &var_map).unwrap();
        assert_eq!(result, "ELLIPTIC_E(A1)");

        let result = symb_anafis_to_excel("elliptic_k(x)", &var_map).unwrap();
        assert_eq!(result, "ELLIPTIC_K(A1)");

        let result = symb_anafis_to_excel("lambertw(x)", &var_map).unwrap();
        assert_eq!(result, "LAMBERTW(A1)");

        // Spherical harmonics and associated Legendre (now supported as custom formulas)
        let result = symb_anafis_to_excel("spherical_harmonic(1, 0, x, y)", &var_map).unwrap();
        assert_eq!(result, "SPHERICAL_HARMONIC(1, 0, A1, B1)");

        let result = symb_anafis_to_excel("Ynm(1, 0, x, y)", &var_map).unwrap();
        assert_eq!(result, "SPHERICAL_HARMONIC(1, 0, A1, B1)");

        let result = symb_anafis_to_excel("assoc_legendre(2, 1, x)", &var_map).unwrap();
        assert_eq!(result, "ASSOC_LEGENDRE(2, 1, A1)");
    }

    #[test]
    fn test_parse_single_cell() {
        let range = parse_excel_range("A1").unwrap();
        assert_eq!(range.column, "A");
        assert_eq!(range.start_row, 1);
        assert_eq!(range.end_row, 1);
    }

    #[test]
    fn test_parse_range() {
        let range = parse_excel_range("B5:B15").unwrap();
        assert_eq!(range.column, "B");
        assert_eq!(range.start_row, 5);
        assert_eq!(range.end_row, 15);
        assert_eq!(range.row_count(), 11);
    }

    #[test]
    fn test_parse_invalid_range() {
        assert!(parse_excel_range("invalid").is_err());
        assert!(parse_excel_range("A").is_err());
        assert!(parse_excel_range("1").is_err());
    }

    #[test]
    fn test_create_cell_ref() {
        assert_eq!(create_cell_ref("A", 1), "A1");
        assert_eq!(create_cell_ref("BC", 42), "BC42");
        assert_eq!(create_cell_ref("XYZ", 999), "XYZ999");
    }

    #[test]
    fn test_complex_formula() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_owned(), "A1".to_owned());
        var_map.insert("y".to_owned(), "B1".to_owned());

        let result = symb_anafis_to_excel("sqrt(x**2 + y**2)", &var_map).unwrap();
        assert_eq!(result, "SQRT(A1^2 + B1^2)");
    }
}
