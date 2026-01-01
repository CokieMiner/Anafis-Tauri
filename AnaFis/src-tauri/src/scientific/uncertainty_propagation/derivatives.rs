use std::collections::{HashMap, HashSet};
use symb_anafis::{gradient_str, parse, Context};
use thiserror::Error;

/// Error type for derivative calculations
#[derive(Debug, Error)]
pub enum DerivativeError {
    #[error("Failed to parse formula '{formula}': {error}")]
    ParseError { formula: String, error: String },

    #[error("Variable '{0}' not found in expression")]
    VariableNotFound(String),

    #[error("Empty formula provided")]
    EmptyFormula,

    #[error("No variables specified")]
    NoVariables,
}

/// Calculate symbolic derivatives using symb_anafis (pure Rust)
///
/// # Arguments
/// * `formula` - The mathematical formula as a string
/// * `variables` - List of variable names to differentiate with respect to
///
/// # Returns
/// A HashMap mapping variable names to their derivative expressions as strings
///
/// # Examples
/// ```
/// # use anafis_lib::scientific::uncertainty_propagation::derivatives::calculate_derivatives;
/// let formula = "x^2 + 2*x*y + y^2";
/// let variables = vec!["x".to_string(), "y".to_string()];
/// let derivs = calculate_derivatives(formula, &variables).unwrap();
/// assert!(derivs.contains_key("x"));
/// assert!(derivs.contains_key("y"));
/// ```
pub fn calculate_derivatives(
    formula: &str,
    variables: &[String],
) -> Result<HashMap<String, String>, DerivativeError> {
    // Validate inputs
    if formula.trim().is_empty() {
        return Err(DerivativeError::EmptyFormula);
    }

    if variables.is_empty() {
        return Err(DerivativeError::NoVariables);
    }

    // Load environment variables from .env file (for any config)
    dotenv::dotenv().ok();

    // First, extract variables that actually appear in the formula
    let formula_variables = extract_variables(formula)?;

    // Check that all requested variables appear in the formula
    for var in variables {
        if !formula_variables.contains(var) {
            return Err(DerivativeError::VariableNotFound(var.clone()));
        }
    }

    // Calculate derivatives using symb_anafis gradient_str function
    let variables_str: Vec<&str> = variables.iter().map(|s| s.as_str()).collect();
    let derivatives_vec =
        gradient_str(formula, &variables_str).map_err(|e| DerivativeError::ParseError {
            formula: formula.to_string(),
            error: format!("{}", e),
        })?;

    // Convert vector result to HashMap
    let mut derivatives = HashMap::new();
    for (i, var) in variables.iter().enumerate() {
        if i < derivatives_vec.len() {
            derivatives.insert(var.clone(), derivatives_vec[i].clone());
        } else {
            return Err(DerivativeError::VariableNotFound(var.clone()));
        }
    }

    Ok(derivatives)
}

pub fn validate_formula(formula: &str) -> Result<(), DerivativeError> {
    if formula.trim().is_empty() {
        return Err(DerivativeError::EmptyFormula);
    }

    // Try to parse by attempting differentiation with empty variables
    let _ = gradient_str(formula, &[]).map_err(|e| DerivativeError::ParseError {
        formula: formula.to_string(),
        error: format!("{}", e),
    })?;

    Ok(())
}

/// Extract all variable names from a formula
///
/// # Arguments
/// * `formula` - The mathematical formula
///
/// # Returns
/// A HashSet containing all variable names found in the formula
pub fn extract_variables(formula: &str) -> Result<HashSet<String>, DerivativeError> {
    if formula.trim().is_empty() {
        return Err(DerivativeError::EmptyFormula);
    }

    // Create a mutable context to track symbols
    // Note: Context symbols are cleared when it goes out of scope to prevent memory leaks
    let mut ctx = Context::new();
    let _ = parse(formula, &HashSet::new(), &HashSet::new(), Some(&ctx)).map_err(|e| {
        DerivativeError::ParseError {
            formula: formula.to_string(),
            error: format!("{}", e),
        }
    })?;

    // Get symbols from the context
    let symbols = ctx.symbol_names();
    let variables: HashSet<String> = symbols.into_iter().collect();

    // Clear context symbols to prevent memory leaks
    ctx.clear_symbols();

    Ok(variables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_derivative() {
        let formula = "x^2";
        let variables = vec!["x".to_string()];
        let derivs = calculate_derivatives(formula, &variables).unwrap();

        assert_eq!(derivs.len(), 1);
        assert!(derivs.contains_key("x"));
        // Derivative of x^2 is 2*x
        let deriv = derivs.get("x").unwrap();
        assert!(deriv.contains("2") && deriv.contains("x"));
    }

    #[test]
    fn test_multiple_variables() {
        let formula = "x*y + x^2";
        let variables = vec!["x".to_string(), "y".to_string()];
        let derivs = calculate_derivatives(formula, &variables).unwrap();

        assert_eq!(derivs.len(), 2);
        assert!(derivs.contains_key("x"));
        assert!(derivs.contains_key("y"));
    }

    #[test]
    fn test_trigonometric() {
        let formula = "sin(x)";
        let variables = vec!["x".to_string()];
        let derivs = calculate_derivatives(formula, &variables).unwrap();

        // Derivative of sin(x) is cos(x)
        let deriv = derivs.get("x").unwrap();
        assert!(deriv.contains("cos"));
    }

    #[test]
    fn test_exponential() {
        let formula = "exp(x)";
        let variables = vec!["x".to_string()];
        let derivs = calculate_derivatives(formula, &variables).unwrap();

        // Derivative of exp(x) is exp(x)
        let deriv = derivs.get("x").unwrap();
        assert!(deriv.contains("exp"));
    }

    #[test]
    fn test_logarithm() {
        let formula = "log(x)";
        let variables = vec!["x".to_string()];
        let derivs = calculate_derivatives(formula, &variables).unwrap();

        assert!(derivs.contains_key("x"));
    }

    #[test]
    fn test_empty_formula() {
        let formula = "";
        let variables = vec!["x".to_string()];
        let result = calculate_derivatives(formula, &variables);

        assert!(matches!(result, Err(DerivativeError::EmptyFormula)));
    }

    #[test]
    fn test_no_variables() {
        let formula = "x^2";
        let variables = vec![];
        let result = calculate_derivatives(formula, &variables);

        assert!(matches!(result, Err(DerivativeError::NoVariables)));
    }

    #[test]
    fn test_variable_not_found() {
        let formula = "x^2";
        let variables = vec!["y".to_string()];
        let result = calculate_derivatives(formula, &variables);

        assert!(matches!(result, Err(DerivativeError::VariableNotFound(_))));
    }

    #[test]
    fn test_validate_formula_valid() {
        assert!(validate_formula("x^2 + y^2").is_ok());
        assert!(validate_formula("sin(x) * cos(y)").is_ok());
        assert!(validate_formula("exp(x + y)").is_ok());
    }

    #[test]
    fn test_validate_formula_invalid() {
        assert!(validate_formula("").is_err());
        assert!(validate_formula("   ").is_err());
    }

    #[test]
    fn test_extract_variables() {
        let formula = "x^2 + y*z + sin(w)";
        let vars = extract_variables(formula).unwrap();

        assert_eq!(vars.len(), 4);
        assert!(vars.contains("x"));
        assert!(vars.contains("y"));
        assert!(vars.contains("z"));
        assert!(vars.contains("w"));
    }

    #[test]
    fn test_extract_variables_empty() {
        let result = extract_variables("");
        assert!(matches!(result, Err(DerivativeError::EmptyFormula)));
    }
}
