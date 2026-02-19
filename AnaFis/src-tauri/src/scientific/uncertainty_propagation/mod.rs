//! Uncertainty Propagation Module
//!
//! Generates Excel formulas and calculates uncertainty propagation using symb_anafis.

pub mod calculator;
pub mod confidence;
pub mod excel_conversion;
pub mod types;

// Re-export calculator commands and types
pub use calculator::{CalculationResult, CalculatorVariable, LatexResult};
pub use confidence::{confidence_to_sigma, sigma_to_confidence, validate_confidence_level};
pub use excel_conversion::{create_cell_ref, parse_excel_range, symb_anafis_to_excel};
pub use types::{ExcelRange, UncertaintyFormulas, Variable};

// Note: generate_uncertainty_formulas is defined in this module (mod.rs)
// and is already a #[tauri::command] function exported directly

use std::collections::{HashMap, HashSet};
use symb_anafis::{parse, uncertainty_propagation};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UncertaintyError {
    #[error("Formula parsing failed: {0}")]
    ParseError(String),

    #[error("Excel conversion failed: {0}")]
    Conversion(#[from] excel_conversion::ConversionError),

    #[error("Confidence calculation failed: {0}")]
    Confidence(#[from] confidence::ConfidenceError),

    #[error("Range parsing failed: {0}")]
    Range(#[from] excel_conversion::RangeError),

    #[error("Uncertainty propagation failed: {0}")]
    UncertaintyPropagation(String),

    #[error("Variable '{0}' has mismatched value and uncertainty range lengths")]
    MismatchedVariableRanges(String),

    #[error("All variable ranges must have the same length")]
    MismatchedRangeLengths,
}

/// Generate Excel formulas for uncertainty propagation (synchronous)
#[tauri::command]
pub fn generate_uncertainty_formulas(
    variables: Vec<Variable>,
    formula: String,
    output_confidence: f64,
) -> Result<UncertaintyFormulas, String> {
    match generate_uncertainty_formulas_inner(variables, formula, output_confidence) {
        Ok(result) => Ok(result),
        Err(e) => Ok(UncertaintyFormulas {
            value_formulas: vec![],
            uncertainty_formulas: vec![],
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

fn generate_uncertainty_formulas_inner(
    variables: Vec<Variable>,
    formula: String,
    output_confidence: f64,
) -> Result<UncertaintyFormulas, UncertaintyError> {
    let formula_normalized = formula.to_lowercase();
    let var_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();
    let normalized_var_names: Vec<String> = var_names.iter().map(|v| v.to_lowercase()).collect();
    let mut seen = HashSet::new();
    for name in &normalized_var_names {
        if !seen.insert(name.clone()) {
            return Err(UncertaintyError::ParseError(format!(
                "Variable names must be unique ignoring case (collision on '{}')",
                name
            )));
        }
    }
    let known_symbols: HashSet<String> = normalized_var_names.iter().cloned().collect();

    // Parse formula once
    let expr = parse(&formula_normalized, &known_symbols, &HashSet::new(), None)
        .map_err(|e| UncertaintyError::ParseError(e.to_string()))?;

    // Parse ranges and validate
    let mut row_count = 0;
    let mut var_info = Vec::new();

    for var in &variables {
        let val_range = parse_excel_range(&var.value_range)?;
        let unc_range = if var.uncertainty_range.is_empty() {
            None
        } else {
            let r = parse_excel_range(&var.uncertainty_range)?;
            if val_range.row_count() != r.row_count() {
                return Err(UncertaintyError::MismatchedVariableRanges(var.name.clone()));
            }
            Some(r)
        };

        if row_count == 0 {
            row_count = val_range.row_count();
        } else if row_count != val_range.row_count() {
            return Err(UncertaintyError::MismatchedRangeLengths);
        }

        var_info.push((var.name.clone(), val_range, unc_range, var.confidence));
    }

    // Get output sigma for confidence conversion
    let output_sigma = confidence_to_sigma(output_confidence)?;

    // Get uncertainty expression from symb_anafis
    let all_vars: Vec<&str> = normalized_var_names.iter().map(|s| s.as_str()).collect();
    let sigma_expr = uncertainty_propagation(&expr, &all_vars, None)
        .map_err(|e| UncertaintyError::UncertaintyPropagation(e.to_string()))?;

    let mut value_formulas = Vec::new();
    let mut uncertainty_formulas = Vec::new();

    // Generate formulas for each row
    for i in 0..row_count {
        // Value formula: substitute variable names with cell references
        let mut var_map: HashMap<String, String> = HashMap::new();
        for (name, val_range, _, _) in &var_info {
            if let Some(cell) = val_range.cell_at(i) {
                var_map.insert(name.to_lowercase(), cell);
            }
        }

        let value_formula = format!("={}", symb_anafis_to_excel(&formula_normalized, &var_map)?);
        value_formulas.push(value_formula);

        // Uncertainty formula: substitute both variables and sigma variables
        let mut sigma_var_map: HashMap<String, String> = var_map.clone();
        for (name, _, unc_range, confidence) in &var_info {
            if let Some(unc_r) = unc_range
                && let Some(sigma_cell) = unc_r.cell_at(i)
            {
                // Apply confidence conversion factor
                let input_sigma = confidence_to_sigma(*confidence)?;
                let conversion_factor = output_sigma / input_sigma;
                let converted_sigma = if (conversion_factor - 1.0).abs() < 1e-10 {
                    sigma_cell.clone()
                } else {
                    format!("({}) * {}", sigma_cell, conversion_factor)
                };
                sigma_var_map.insert(format!("sigma_{}", name.to_lowercase()), converted_sigma);
            }
        }

        let sigma_formula_str = sigma_expr.to_string();
        let unc_formula = if sigma_var_map.iter().any(|(k, _)| k.starts_with("sigma_")) {
            format!(
                "={}",
                symb_anafis_to_excel(&sigma_formula_str, &sigma_var_map)?
            )
        } else {
            "=0".to_string()
        };
        uncertainty_formulas.push(unc_formula);
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
    fn test_generate_uncertainty_formulas_does_not_corrupt_sigma_identifiers() {
        let variables = vec![
            Variable {
                name: "a".to_string(),
                value_range: "A1:A2".to_string(),
                uncertainty_range: "B1:B2".to_string(),
                confidence: 95.0,
            },
            Variable {
                name: "b".to_string(),
                value_range: "C1:C2".to_string(),
                uncertainty_range: "D1:D2".to_string(),
                confidence: 95.0,
            },
        ];

        let result =
            generate_uncertainty_formulas_inner(variables, "sin(a) * b".to_string(), 95.0).unwrap();

        assert!(result.success);
        assert_eq!(result.uncertainty_formulas.len(), 2);
        assert!(!result.uncertainty_formulas[0].contains("sigm"));
        assert!(result.uncertainty_formulas[0].contains("B1"));
        assert!(result.uncertainty_formulas[0].contains("D1"));
    }

    #[test]
    fn test_generate_uncertainty_formulas_mixed_case_variable_name() {
        let variables = vec![Variable {
            name: "AlotA".to_string(),
            value_range: "A1:A1".to_string(),
            uncertainty_range: "B1:B1".to_string(),
            confidence: 95.0,
        }];

        let result =
            generate_uncertainty_formulas_inner(variables, "AlotA^2".to_string(), 95.0).unwrap();
        assert!(result.success);
        assert_eq!(result.value_formulas, vec!["=A1^2".to_string()]);
        assert!(!result.uncertainty_formulas[0].contains("sigma_alota"));
    }
}
