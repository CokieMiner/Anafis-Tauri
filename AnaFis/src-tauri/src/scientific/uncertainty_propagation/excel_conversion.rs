use super::types::ExcelRange;
use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;

/// Error type for Excel conversion operations
#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Function '{0}' is not supported in Excel")]
    UnsupportedExcelFunction(String),

    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
}

/// Error type for range parsing operations
#[derive(Debug, Error)]
pub enum RangeError {
    #[error("Invalid range format: {0}")]
    InvalidFormat(String),

    #[error("Invalid cell format: {0}")]
    InvalidCell(String),

    #[error("Invalid row number in range: {0}")]
    InvalidRow(String),
}

/// Convert symb_anafis derivative expression to Excel formula
///
/// This function converts mathematical expressions from symb_anafis's format to Excel's format,
/// handling function names, operators, and variable substitutions.
///
/// # Arguments
/// * `symb_anafis_expr` - The expression string from symb_anafis
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
pub fn symb_anafis_to_excel(
    symb_anafis_expr: &str,
    var_map: &HashMap<String, String>,
) -> Result<String, ConversionError> {
    let mut excel_formula = symb_anafis_expr.to_string();

    // Check for unsupported functions before conversion
    static UNSUPPORTED_FUNCTIONS: &[&str] = &[
        "Ynm",                // Spherical harmonics (not in Excel)
        "spherical_harmonic", // Spherical harmonics (not in Excel)
        "assoc_legendre",     // Associated Legendre polynomials (not in Excel)
        "elliptic_e",         // Complete elliptic integral of 2nd kind (not in Excel)
        "zeta_deriv",         // Zeta function derivative (not in Excel)
        "polygamma",          // Polygamma function (not in Excel)
        "trigamma",           // Trigamma function (not in Excel)
        "tetragamma",         // Tetragamma function (not in Excel)
    ];

    for unsupported in UNSUPPORTED_FUNCTIONS {
        if excel_formula.contains(unsupported) {
            return Err(ConversionError::UnsupportedExcelFunction(
                unsupported.to_string(),
            ));
        }
    }

    // Replace variable names with cell references
    for (var_name, cell_ref) in var_map {
        excel_formula = excel_formula.replace(var_name, cell_ref);
    }

    // Convert operators
    excel_formula = excel_formula.replace("**", "^"); // Power operator

    // Convert logarithmic functions (ORDER MATTERS! Specific before general)
    excel_formula = excel_formula.replace("log10", "LOG10");
    excel_formula = excel_formula.replace("log2", "LOG2");
    excel_formula = excel_formula.replace("ln", "LN"); // Natural log
    excel_formula = excel_formula.replace("log", "LOG"); // General log

    // Convert exponential and roots
    excel_formula = excel_formula.replace("sqrt", "SQRT");
    // Convert cbrt(x) to POWER(x, 1/3) - must match cbrt( and capture the argument
    let cbrt_regex = Regex::new(r"cbrt\(([^)]+)\)").unwrap();
    excel_formula = cbrt_regex
        .replace_all(&excel_formula, "POWER($1, 1/3)")
        .to_string();
    excel_formula = excel_formula.replace("exp_polar", "EXP"); // Approximation - must come before exp
    excel_formula = excel_formula.replace("exp", "EXP");

    // Convert inverse trig (must come before regular trig to avoid double replacement)
    excel_formula = excel_formula.replace("asin", "ASIN");
    excel_formula = excel_formula.replace("acos", "ACOS");
    excel_formula = excel_formula.replace("atan", "ATAN");
    excel_formula = excel_formula.replace("atan2", "ATAN2"); // Two-argument arctangent
    excel_formula = excel_formula.replace("acot", "ACOT");
    excel_formula = excel_formula.replace("asec", "ASEC");
    excel_formula = excel_formula.replace("acsc", "ACSC");

    // Convert regular trig
    excel_formula = excel_formula.replace("sin", "SIN");
    excel_formula = excel_formula.replace("sen", "SIN"); // Portuguese/Spanish alias
    excel_formula = excel_formula.replace("cos", "COS");
    excel_formula = excel_formula.replace("tan", "TAN");

    // Convert less common trig using workarounds
    excel_formula = excel_formula.replace("cot(", "(1/TAN(");
    excel_formula = excel_formula.replace("sec(", "(1/COS(");
    excel_formula = excel_formula.replace("csc(", "(1/SIN(");

    // Convert inverse hyperbolic (must come before regular hyperbolic)
    excel_formula = excel_formula.replace("asinh", "ASINH");
    excel_formula = excel_formula.replace("acosh", "ACOSH");
    excel_formula = excel_formula.replace("atanh", "ATANH");
    excel_formula = excel_formula.replace("acoth", "ACOTH");
    excel_formula = excel_formula.replace("asech", "ASECH");
    excel_formula = excel_formula.replace("acsch", "ACSCH");

    // Convert hyperbolic functions
    excel_formula = excel_formula.replace("sinh", "SINH");
    excel_formula = excel_formula.replace("cosh", "COSH");
    excel_formula = excel_formula.replace("tanh", "TANH");

    // Convert less common hyperbolic using workarounds
    excel_formula = excel_formula.replace("coth(", "(1/TANH(");
    excel_formula = excel_formula.replace("sech(", "(1/COSH(");
    excel_formula = excel_formula.replace("csch(", "(1/SINH(");

    // Convert special functions
    excel_formula = excel_formula.replace("erf", "ERF");
    excel_formula = excel_formula.replace("erfc", "ERFC");
    excel_formula = excel_formula.replace("gamma", "GAMMA");
    excel_formula = excel_formula.replace("besselj", "BESSELJ");
    excel_formula = excel_formula.replace("bessely", "BESSELY");
    excel_formula = excel_formula.replace("besseli", "BESSELI");
    excel_formula = excel_formula.replace("besselk", "BESSELK");
    excel_formula = excel_formula.replace("beta", "BETA");
    excel_formula = excel_formula.replace("digamma", "DIGAMMA");
    // Note: trigamma, tetragamma, polygamma not supported in Excel
    excel_formula = excel_formula.replace("lambertw", "LAMBERTW"); // May not be available in all Excel versions
    excel_formula = excel_formula.replace("hermite", "HERMITE");
    excel_formula = excel_formula.replace("zeta", "ZETA");
    // Note: zeta_deriv not supported in Excel
    excel_formula = excel_formula.replace("elliptic_k", "ELLIPTIC_K");
    // Note: elliptic_e, assoc_legendre, spherical_harmonic, ynm not supported in Excel

    // Convert other functions
    excel_formula = excel_formula.replace("abs", "ABS");
    excel_formula = excel_formula.replace("signum", "SIGN"); // Sign function
    excel_formula = excel_formula.replace("sinc", "SINC");
    excel_formula = excel_formula.replace("floor", "FLOOR");
    excel_formula = excel_formula.replace("ceil", "CEILING");
    excel_formula = excel_formula.replace("round", "ROUND");

    // Convert constants (MUST come after function replacements to avoid conflicts)
    excel_formula = excel_formula.replace("pi", "PI()");
    // Use word boundaries to only match standalone 'e' or 'E' (Euler's constant),
    // not when they appear as part of function names like ERF or CEILING
    let euler_regex = Regex::new(r"\b[eE]\b").unwrap();
    excel_formula = euler_regex
        .replace_all(&excel_formula, "EXP(1)")
        .to_string();
    Ok(excel_formula)
}

/// Parse Excel range notation into structured ExcelRange
///
/// Supports both single cell (e.g., "A1") and range (e.g., "A1:A10") formats.
///
/// # Arguments
/// * `range` - The range string to parse
///
/// # Returns
/// An ExcelRange struct containing column, start_row, and end_row
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
            .map_err(|_| RangeError::InvalidRow(range.to_string()))?;

        if col.is_empty() {
            return Err(RangeError::InvalidCell(range.to_string()));
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
            .map_err(|_| RangeError::InvalidRow(format!("start: {}", range)))?;

        let end_row: usize = parts[1]
            .chars()
            .skip_while(|c| c.is_alphabetic())
            .collect::<String>()
            .parse()
            .map_err(|_| RangeError::InvalidRow(format!("end: {}", range)))?;

        Ok(ExcelRange::new(start_col, start_row, end_row))
    } else {
        Err(RangeError::InvalidFormat(range.to_string()))
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
pub fn create_cell_ref(col: &str, row: usize) -> String {
    format!("{}{}", col, row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symb_anafis_to_excel_power() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_string(), "A1".to_string());

        let result = symb_anafis_to_excel("x**2", &var_map).unwrap();
        assert_eq!(result, "A1^2");
    }

    #[test]
    fn test_symb_anafis_to_excel_trig() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_string(), "A1".to_string());

        let result = symb_anafis_to_excel("sin(x)", &var_map).unwrap();
        assert_eq!(result, "SIN(A1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_atan2() {
        let mut var_map = HashMap::new();
        var_map.insert("y".to_string(), "A1".to_string());
        var_map.insert("x".to_string(), "B1".to_string());

        let result = symb_anafis_to_excel("atan2(y, x)", &var_map).unwrap();
        assert_eq!(result, "ATAN2(A1, B1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_log() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_string(), "A1".to_string());

        let result = symb_anafis_to_excel("log10(x)", &var_map).unwrap();
        assert_eq!(result, "LOG10(A1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_special_functions() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_string(), "A1".to_string());
        var_map.insert("y".to_string(), "B1".to_string());

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
        var_map.insert("x".to_string(), "A1".to_string());

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
        var_map.insert("x".to_string(), "A1".to_string());

        let result = symb_anafis_to_excel("cbrt(x)", &var_map).unwrap();
        assert_eq!(result, "POWER(A1, 1/3)");

        // Test with expression
        let result = symb_anafis_to_excel("cbrt(x + 2)", &var_map).unwrap();
        assert_eq!(result, "POWER(A1 + 2, 1/3)");
    }

    #[test]
    fn test_symb_anafis_to_excel_signum() {
        let mut var_map = HashMap::new();
        var_map.insert("x".to_string(), "A1".to_string());

        let result = symb_anafis_to_excel("signum(x)", &var_map).unwrap();
        assert_eq!(result, "SIGN(A1)");
    }

    #[test]
    fn test_symb_anafis_to_excel_unsupported() {
        let var_map = HashMap::new();

        // Test original unsupported functions
        let result = symb_anafis_to_excel("Ynm(x, y, z)", &var_map);
        assert!(matches!(
            result,
            Err(ConversionError::UnsupportedExcelFunction(_))
        ));

        // Test newly added unsupported functions
        let result = symb_anafis_to_excel("elliptic_e(x)", &var_map);
        assert!(matches!(
            result,
            Err(ConversionError::UnsupportedExcelFunction(_))
        ));

        let result = symb_anafis_to_excel("zeta_deriv(1, x)", &var_map);
        assert!(matches!(
            result,
            Err(ConversionError::UnsupportedExcelFunction(_))
        ));

        let result = symb_anafis_to_excel("polygamma(1, x)", &var_map);
        assert!(matches!(
            result,
            Err(ConversionError::UnsupportedExcelFunction(_))
        ));
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
        var_map.insert("x".to_string(), "A1".to_string());
        var_map.insert("y".to_string(), "B1".to_string());

        let result = symb_anafis_to_excel("sqrt(x**2 + y**2)", &var_map).unwrap();
        assert_eq!(result, "SQRT(A1^2 + B1^2)");
    }
}
