//! Uncertainty Calculator Module
//!
//! This module provides uncertainty propagation calculations using symbolic differentiation
//! with Symbolica and numerical evaluation with meval. Pure Rust implementation with no
//! Python dependencies.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use lru::LruCache;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::num::NonZeroUsize;
use tauri::AppHandle;

// Import our pure Rust uncertainty propagation modules
use crate::scientific::uncertainty_propagation::derivatives;

#[derive(Clone)]
struct SymbolicResult {
    numerical_value: f64,
    numerical_derivatives: HashMap<String, f64>,
}

#[derive(Deserialize, Clone)]
pub struct Variable {
    pub name: String,
    pub value: f64,
    pub uncertainty: f64,
}

impl From<&Variable> for crate::utils::VariableInput {
    fn from(var: &Variable) -> Self {
        crate::utils::VariableInput {
            name: var.name.clone(),
            value: var.value,
            uncertainty: var.uncertainty,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct CalculationResult {
    pub value: f64,
    pub uncertainty: f64,
    pub formula: String,
    pub derivatives: HashMap<String, f64>,  // Numerical derivatives
    pub confidence_level: f64,
}

#[derive(Serialize, Clone)]
pub struct LatexResult {
    pub string: String,
    pub latex: String,
}

// Simple LRU cache for calculation results
static FORMULA_CACHE: Lazy<RwLock<LruCache<String, CalculationResult>>> = Lazy::new(|| {
    RwLock::new(LruCache::new(NonZeroUsize::new(100).unwrap()))
});

// LRU cache for LaTeX generation
static LATEX_CACHE: Lazy<RwLock<LruCache<String, LatexResult>>> = Lazy::new(|| {
    RwLock::new(LruCache::new(NonZeroUsize::new(100).unwrap()))
});

/// Evaluate a mathematical expression using meval
///
/// # Arguments
/// * `expr_str` - The expression string
/// * `var_names` - List of variable names
/// * `var_values` - Corresponding values for each variable
///
/// # Returns
/// The numerical result of the evaluation
fn evaluate_expression(
    expr_str: &str,
    var_names: &[String],
    var_values: &[f64],
) -> Result<f64, String> {
    use meval::Context;
    
    // Create evaluation context
    let mut ctx = Context::new();
    
    // Add variables to context
    for (name, value) in var_names.iter().zip(var_values.iter()) {
        ctx.var(name, *value);
    }
    
    // Evaluate expression
    meval::eval_str_with_context(expr_str, &ctx)
        .map_err(|e| format!("Failed to evaluate expression '{}': {}", expr_str, e))
}

/// Compute symbolic derivatives and numerical evaluations using pure Rust
///
/// This replaces the Python/SymPy implementation with Symbolica + meval
fn compute_symbolic_derivatives_rust(
    formula: &str,
    variables: &[String],
    values: &[f64],
) -> Result<SymbolicResult, String> {
    // Normalize formula to lowercase for Symbolica
    let formula_normalized = formula.to_lowercase();
    
    // Calculate symbolic derivatives using Symbolica
    let derivatives_map = derivatives::calculate_derivatives(&formula_normalized, variables)
        .map_err(|e| format!("Derivative calculation failed: {}", e))?;
    
    // Evaluate the original formula numerically using meval
    let numerical_value = evaluate_expression(&formula_normalized, variables, values)?;
    
    // Check for finite result
    if !numerical_value.is_finite() {
        return Err("Expression evaluated to non-finite value (inf or NaN)".to_string());
    }
    
    // Evaluate each derivative numerically
    let mut numerical_derivatives = HashMap::new();
    for (var, deriv_expr) in derivatives_map {
        let deriv_value = evaluate_expression(&deriv_expr, variables, values)?;
        
        // Check for finite derivative
        if !deriv_value.is_finite() {
            return Err(format!(
                "Derivative with respect to '{}' is non-finite (inf or NaN)",
                var
            ));
        }
        
        numerical_derivatives.insert(var, deriv_value);
    }
    
    Ok(SymbolicResult {
        numerical_value,
        numerical_derivatives,
    })
}

/// Generate LaTeX representation of uncertainty propagation formula using pure Rust
///
/// Creates the formula: σ_f = √(Σ(∂f/∂xi · σ_xi)²)
fn generate_latex_rust(
    formula: &str,
    variables: &[String],
) -> Result<LatexResult, String> {
    // Normalize formula to lowercase for Symbolica
    let formula_normalized = formula.to_lowercase();
    
    // Calculate symbolic derivatives
    let derivatives_map = derivatives::calculate_derivatives(&formula_normalized, variables)
        .map_err(|e| format!("Derivative calculation failed: {}", e))?;
    
    // Build uncertainty formula components
    let mut latex_terms = Vec::new();
    let mut string_terms = Vec::new();
    
    for var in variables {
        if let Some(deriv_expr) = derivatives_map.get(var) {
            // Convert derivative expression to LaTeX-friendly format
            let deriv_latex = expression_to_latex(deriv_expr);
            let sigma_var_latex = format!("\\sigma_{{{}}}", var);
            let sigma_var_string = format!("σ_{}", var);
            
            // Create term: (∂f/∂xi · σ_xi)²
            let latex_term = format!("\\left({} \\cdot {}\\right)^2", deriv_latex, sigma_var_latex);
            let string_term = format!("({} * {})^2", deriv_expr, sigma_var_string);
            
            latex_terms.push(latex_term);
            string_terms.push(string_term);
        }
    }
    
    // Build final formula
    let latex = if latex_terms.is_empty() {
        "0".to_string()
    } else {
        format!("\\sqrt{{{}}}", latex_terms.join(" + "))
    };
    
    let string = if string_terms.is_empty() {
        "0".to_string()
    } else {
        format!("sqrt({})", string_terms.join(" + "))
    };
    
    Ok(LatexResult { string, latex })
}

/// Convert a mathematical expression to LaTeX format
///
/// This is a simple converter that handles common mathematical notation
fn expression_to_latex(expr: &str) -> String {
    let mut latex = expr.to_string();
    
    // Replace operators with LaTeX equivalents
    latex = latex.replace("**", "^");
    latex = latex.replace("*", " \\cdot ");
    latex = latex.replace("sqrt", "\\sqrt");
    
    // Wrap exponents in braces
    // Simple regex-free approach: find ^ and wrap following term
    let chars: Vec<char> = latex.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '^' && i + 1 < chars.len() {
            result.push('^');
            i += 1;
            
            // Check if next char is already a brace
            if chars[i] == '{' {
                // Already wrapped, just continue
                result.push(chars[i]);
                i += 1;
                continue;
            }
            
            // Wrap the exponent
            result.push('{');
            
            // Single character or number
            if chars[i].is_alphanumeric() {
                result.push(chars[i]);
                i += 1;
                
                // Continue if it's a multi-digit number
                while i < chars.len() && chars[i].is_numeric() {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            
            result.push('}');
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    
    result
}

#[tauri::command]
pub async fn calculate_uncertainty(
    _app: AppHandle,  // Kept for API compatibility, not needed anymore
    formula: String,
    variables: Vec<Variable>,
) -> Result<CalculationResult, String> {
    // Input validation using centralized utilities
    if let Err(e) = crate::utils::validate_formula(&formula) {
        return Err(format!("{e:?}"));
    }

    // Convert variables to VariableInput for validation
    let variable_inputs: Vec<crate::utils::VariableInput> = variables.iter().map(|v| v.into()).collect();
    if let Err(e) = crate::utils::validate_variables(&variable_inputs) {
        return Err(format!("{e:?}"));
    }

    // Generate cache key using centralized utility
    let cache_key = crate::utils::generate_cache_key(&formula, &variable_inputs);

    // Check cache first
    {
        let cache_result = FORMULA_CACHE.write();
        match cache_result {
            Ok(mut cache) => {
                if let Some(result) = cache.get(&cache_key) {
                    crate::utils::log_info(&format!("Cache hit for formula: {formula}"));
                    return Ok(result.clone());
                }
            }
            Err(_e) => {
                crate::utils::log_error("Cache access failed", &std::io::Error::other("Mutex poisoned"));
                return Err("Cache access failed".to_string());
            }
        }
    }

    // Move heavy computation to background thread
    let formula_clone = formula.clone();
    let variables_clone = variables.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        compute_uncertainty_sync(formula_clone, variables_clone)
    }).await.map_err(|e| format!("Async computation failed: {e:?}"))?;

    match result {
        Ok(calc_result) => {
            // Cache the result
            {
                let mut cache = FORMULA_CACHE.write().map_err(|_| "Cache access failed".to_string())?;
                cache.put(cache_key, calc_result.clone());
            }
            Ok(calc_result)
        }
        Err(e) => Err(e)
    }
}

fn compute_uncertainty_sync(formula: String, variables: Vec<Variable>) -> Result<CalculationResult, String> {
    // Extract variable names and values
    let variable_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();
    let variable_values: Vec<f64> = variables.iter().map(|v| v.value).collect();

    // Use pure Rust computation with Symbolica + meval
    let symbolic_result = compute_symbolic_derivatives_rust(&formula, &variable_names, &variable_values)?;

    // Calculate uncertainty using standard error propagation formula:
    // σ_f² = Σ((∂f/∂xi)² · σ_xi²)
    let total_uncertainty_squared = variables.iter()
        .filter(|var| var.uncertainty > 0.0)
        .try_fold(0.0, |acc, var| {
            symbolic_result.numerical_derivatives.get(&var.name)
                .map(|deriv_value| {
                    let term = deriv_value * deriv_value * var.uncertainty * var.uncertainty;
                    acc + term
                })
                .ok_or_else(|| format!("Missing derivative for variable {}", var.name))
        })?;

    let total_uncertainty = total_uncertainty_squared.max(0.0).sqrt();

    let result = CalculationResult {
        value: symbolic_result.numerical_value,
        uncertainty: total_uncertainty,
        formula: formula.to_string(),
        derivatives: symbolic_result.numerical_derivatives.clone(),
        confidence_level: 0.95, // Default 95% confidence level
    };

    Ok(result)
}

#[tauri::command]
pub async fn generate_latex(
    _app: AppHandle,  // Kept for API compatibility, not needed anymore
    formula: String,
    variables: Vec<String>,
) -> Result<LatexResult, String> {
    // Input validation
    if variables.is_empty() {
        return Err("Please provide at least one variable".into());
    }
    for var in &variables {
        if var.trim().is_empty() {
            return Err("Variable names cannot be empty".into());
        }
        if !var.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("Invalid variable name: {var}"));
        }
    }

    let mut cache_key = formula.clone();
    cache_key.push('|');
    let mut sorted_vars = variables.clone();
    sorted_vars.sort();
    cache_key.push_str(&sorted_vars.join(","));

    // Check LaTeX cache
    {
        let mut cache = LATEX_CACHE.write().map_err(|_| "Cache access failed".to_string())?;
        if let Some(cached_latex) = cache.get(&cache_key) {
            return Ok(cached_latex.clone());
        }
    }

    // Move computation to background thread
    let formula_clone = formula.clone();
    let variables_clone = variables.clone();
    let latex_result = tauri::async_runtime::spawn_blocking(move || {
        generate_latex_rust(&formula_clone, &variables_clone)
    }).await.map_err(|e| format!("Async LaTeX computation failed: {e:?}"))?;

    match latex_result {
        Ok(result) => {
            // Cache the LaTeX result
            {
                let mut cache = LATEX_CACHE.write().map_err(|_| "Cache access failed".to_string())?;
                cache.put(cache_key, result.clone());
            }
            Ok(result)
        }
        Err(e) => Err(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_expression_simple() {
        let result = evaluate_expression("2 + 3", &[], &[]).unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_evaluate_expression_with_variables() {
        let result = evaluate_expression(
            "x^2 + y",
            &["x".to_string(), "y".to_string()],
            &[3.0, 4.0],
        ).unwrap();
        assert_eq!(result, 13.0); // 3² + 4 = 13
    }

    #[test]
    fn test_compute_derivatives_simple() {
        let result = compute_symbolic_derivatives_rust(
            "x^2",
            &["x".to_string()],
            &[3.0],
        ).unwrap();
        
        assert_eq!(result.numerical_value, 9.0); // 3² = 9
        assert!(result.numerical_derivatives.contains_key("x"));
        let deriv = result.numerical_derivatives.get("x").unwrap();
        assert!((deriv - 6.0).abs() < 0.01); // d(x²)/dx = 2x = 2*3 = 6
    }

    #[test]
    fn test_expression_to_latex() {
        let latex = expression_to_latex("x^2 + y");
        assert!(latex.contains("x^{2}"));
    }

    #[test]
    fn test_generate_latex_simple() {
        let result = generate_latex_rust(
            "x + y",
            &["x".to_string(), "y".to_string()],
        ).unwrap();
        
        // Should contain sigma symbols and square root
        assert!(result.string.contains("sqrt"));
        assert!(result.latex.contains("\\sqrt"));
    }
}
