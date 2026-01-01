//! Uncertainty Propagation Module
//! 
//! This module provides pure Rust implementation for uncertainty propagation calculations,
//! including symbolic derivative computation using symb_anafis and Excel formula generation.
//! 
//! # Architecture
//! 
//! - `types`: Data structures (Variable, UncertaintyFormulas, ExcelRange)
//! - `confidence`: Confidence level ↔ sigma conversions
//! - `derivatives`: Symbolic differentiation using symb_anafis
//! - `excel_conversion`: Convert symb_anafis expressions to Excel formulas
//! 
//! # Example
//! 
//! ```
//! use anafis_lib::scientific::uncertainty_propagation::types::Variable;
//! 
//! let variables = vec![
//!     Variable {
//!         name: "x".to_string(),
//!         value_range: "A1:A10".to_string(),
//!         uncertainty_range: "B1:B10".to_string(),
//!         confidence: 95.0,
//!     },
//! ];
//! 
//! // Note: generate_uncertainty_formulas is async and requires Tauri runtime
//! // This example shows the Variable structure usage
//! assert_eq!(variables[0].name, "x");
//! ```

pub mod types;
pub mod confidence;
pub mod derivatives;
pub mod excel_conversion;

// Re-export public API
pub use types::{Variable, UncertaintyFormulas, ExcelRange};
pub use confidence::{confidence_to_sigma, sigma_to_confidence, validate_confidence_level};
pub use derivatives::{calculate_derivatives, validate_formula, extract_variables};
pub use excel_conversion::{symb_anafis_to_excel, parse_excel_range, create_cell_ref};

use std::collections::HashMap;
use thiserror::Error;

/// Unified error type for uncertainty propagation operations
#[derive(Debug, Error)]
pub enum UncertaintyError {
    #[error("Derivative calculation failed: {0}")]
    Derivative(#[from] derivatives::DerivativeError),
    
    #[error("Excel conversion failed: {0}")]
    Conversion(#[from] excel_conversion::ConversionError),
    
    #[error("Confidence calculation failed: {0}")]
    Confidence(#[from] confidence::ConfidenceError),
    
    #[error("Range parsing failed: {0}")]
    Range(#[from] excel_conversion::RangeError),
    
    #[error("Variable ranges have inconsistent lengths")]
    InconsistentRanges,
    
    #[error("All variable ranges must have the same length")]
    MismatchedRangeLengths,
    
    #[error("Variable '{0}' has mismatched value and uncertainty range lengths")]
    MismatchedVariableRanges(String),
}

/// Generate Excel formulas for uncertainty propagation
/// 
/// This is the main Tauri command that generates both value and uncertainty formulas
/// for a given mathematical expression with multiple variables.
/// 
/// # Arguments
/// * `variables` - Vector of variables with their ranges and confidence levels
/// * `formula` - The mathematical formula as a string
/// * `output_confidence` - Desired output confidence level (percentage)
/// 
/// # Returns
/// `UncertaintyFormulas` containing value and uncertainty formulas for each row
#[tauri::command]
pub async fn generate_uncertainty_formulas(
    variables: Vec<Variable>,
    formula: String,
    output_confidence: f64,
) -> Result<UncertaintyFormulas, String> {
    // Wrap the sync computation to make it async
    match generate_uncertainty_formulas_sync(variables, formula, output_confidence) {
        Ok(result) => Ok(result),
        Err(e) => Ok(UncertaintyFormulas {
            value_formulas: vec![],
            uncertainty_formulas: vec![],
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

/// Synchronous implementation of uncertainty formula generation
fn generate_uncertainty_formulas_sync(
    variables: Vec<Variable>,
    formula: String,
    output_confidence: f64,
) -> Result<UncertaintyFormulas, UncertaintyError> {
    // Normalize formula to lowercase (symb_anafis is case-insensitive)
    let formula_normalized = formula.to_lowercase();
    
    // Extract variable names
    let var_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();
    
    // Calculate derivatives using symb_anafis
    let derivatives = derivatives::calculate_derivatives(&formula_normalized, &var_names)?;
    
    // Parse all ranges and validate consistency
    let mut row_count = 0;
    let mut var_info = Vec::new();
    
    for var in &variables {
        let val_range = excel_conversion::parse_excel_range(&var.value_range)?;
        
        // Handle optional uncertainty range (empty string means no uncertainty)
        let uncertainty_info = if var.uncertainty_range.is_empty() {
            None
        } else {
            let unc_range = excel_conversion::parse_excel_range(&var.uncertainty_range)?;
            
            if val_range.row_count() != unc_range.row_count() {
                return Err(UncertaintyError::MismatchedVariableRanges(var.name.clone()));
            }
            
            Some(unc_range)
        };
        
        let current_row_count = val_range.row_count();
        if row_count == 0 {
            row_count = current_row_count;
        } else if row_count != current_row_count {
            return Err(UncertaintyError::MismatchedRangeLengths);
        }
        
        var_info.push((var.name.clone(), val_range, uncertainty_info));
    }
    
    let mut value_formulas = Vec::new();
    let mut uncertainty_formulas = Vec::new();
    
    // Generate formulas for each row
    for i in 0..row_count {
        // Create variable mapping for this row (for value formula)
        let mut var_map = HashMap::new();
        for (var_name, val_range, _) in &var_info {
            if let Some(cell) = val_range.cell_at(i) {
                var_map.insert(var_name.clone(), cell);
            }
        }
        
        // Generate value formula
        let value_formula_body = excel_conversion::symb_anafis_to_excel(&formula_normalized, &var_map)?;
        let value_formula = format!("={}", value_formula_body);
        value_formulas.push(value_formula);
        
        // Generate uncertainty formula: σ_f = sqrt(Σ(∂f/∂xi * σ_xi)²)
        let output_sigma = confidence::confidence_to_sigma(output_confidence)?;
        let mut uncertainty_terms = Vec::new();
        
        for (var_name, _val_range, uncertainty_info) in &var_info {
            // Skip variables with no uncertainty range
            if uncertainty_info.is_none() {
                continue;
            }
            
            // Find the variable's confidence level
            let var_confidence = variables
                .iter()
                .find(|v| v.name == *var_name)
                .map(|v| v.confidence)
                .unwrap_or(68.0); // Default to 68% (1 sigma)
            
            let input_sigma = confidence::confidence_to_sigma(var_confidence)?;
            
            if let Some(deriv_expr) = derivatives.get(var_name) {
                let unc_range = uncertainty_info.as_ref().unwrap();
                
                // Create cell reference map for derivative
                let mut deriv_var_map = HashMap::new();
                for (vname, vrange, _) in &var_info {
                    if let Some(cell) = vrange.cell_at(i) {
                        deriv_var_map.insert(vname.clone(), cell);
                    }
                }
                
                // Convert derivative to Excel formula
                let deriv_excel = excel_conversion::symb_anafis_to_excel(deriv_expr, &deriv_var_map)?;
                let sigma_cell = unc_range.cell_at(i).unwrap();
                
                // Apply confidence level conversion
                let conversion_factor = output_sigma / input_sigma;
                let converted_sigma = if (conversion_factor - 1.0).abs() < 1e-10 {
                    sigma_cell // No conversion needed
                } else {
                    format!("({}) * {}", sigma_cell, conversion_factor)
                };
                
                // Term: (∂f/∂xi * σ_xi_converted)²
                let term = format!("(({}) * {})^2", deriv_excel, converted_sigma);
                uncertainty_terms.push(term);
            }
        }
        
        // Combine all terms: =SQRT(term1 + term2 + ...)
        let uncertainty_formula = if uncertainty_terms.is_empty() {
            "=0".to_string() // No uncertainty if no terms
        } else {
            format!("=SQRT({})", uncertainty_terms.join(" + "))
        };
        
        uncertainty_formulas.push(uncertainty_formula);
    }
    
    Ok(UncertaintyFormulas {
        value_formulas,
        uncertainty_formulas,
        success: true,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_uncertainty_formula() {
        let variables = vec![
            Variable {
                name: "x".to_string(),
                value_range: "A1".to_string(),
                uncertainty_range: "B1".to_string(),
                confidence: 95.0,
            },
        ];
        
        let result = generate_uncertainty_formulas_sync(
            variables,
            "x^2".to_string(),
            95.0,
        ).unwrap();
        
        assert_eq!(result.value_formulas.len(), 1);
        assert_eq!(result.uncertainty_formulas.len(), 1);
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_multiple_variables() {
        let variables = vec![
            Variable {
                name: "x".to_string(),
                value_range: "A1:A5".to_string(),
                uncertainty_range: "B1:B5".to_string(),
                confidence: 95.0,
            },
            Variable {
                name: "y".to_string(),
                value_range: "C1:C5".to_string(),
                uncertainty_range: "D1:D5".to_string(),
                confidence: 95.0,
            },
        ];
        
        let result = generate_uncertainty_formulas_sync(
            variables,
            "x + y".to_string(),
            95.0,
        ).unwrap();
        
        assert_eq!(result.value_formulas.len(), 5);
        assert_eq!(result.uncertainty_formulas.len(), 5);
    }

    #[test]
    fn test_mismatched_ranges() {
        let variables = vec![
            Variable {
                name: "x".to_string(),
                value_range: "A1:A5".to_string(),
                uncertainty_range: "B1:B5".to_string(),
                confidence: 95.0,
            },
            Variable {
                name: "y".to_string(),
                value_range: "C1:C10".to_string(), // Different length
                uncertainty_range: "D1:D10".to_string(),
                confidence: 95.0,
            },
        ];
        
        let result = generate_uncertainty_formulas_sync(
            variables,
            "x + y".to_string(),
            95.0,
        );
        
        assert!(result.is_err());
    }
}
