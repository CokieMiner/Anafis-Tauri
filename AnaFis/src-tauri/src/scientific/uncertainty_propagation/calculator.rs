//! Uncertainty Calculator
//!
//! Provides numerical uncertainty propagation using symb_anafis.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use symb_anafis::{CovEntry, CovarianceMatrix, parse, uncertainty_propagation};

#[derive(Deserialize, Clone)]
pub struct CalculatorVariable {
    pub name: String,
    pub value: f64,
    pub uncertainty: f64,
}

#[derive(Serialize, Clone)]
pub struct CalculationResult {
    pub value: f64,
    pub uncertainty: f64,
    pub formula: String,
    pub derivatives: HashMap<String, f64>,
    pub confidence_level: f64,
}

#[derive(Serialize, Clone)]
pub struct LatexResult {
    pub string: String,
    pub latex: String,
}

fn normalize_variable_names(variable_names: &[String]) -> Result<Vec<String>, String> {
    let mut normalized = Vec::with_capacity(variable_names.len());
    let mut seen = HashSet::new();

    for name in variable_names {
        let lower = name.to_lowercase();
        if !seen.insert(lower.clone()) {
            return Err(format!(
                "Variable names must be unique ignoring case (collision on '{}')",
                name
            ));
        }
        normalized.push(lower);
    }

    Ok(normalized)
}

/// Calculate uncertainty propagation
#[tauri::command]
pub fn calculate_uncertainty(
    formula: String,
    variables: Vec<CalculatorVariable>,
) -> Result<CalculationResult, String> {
    let variable_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();
    let normalized_variable_names = normalize_variable_names(&variable_names)?;
    let known_symbols: HashSet<String> = normalized_variable_names.iter().cloned().collect();

    // Build values map
    let mut values_map = HashMap::new();
    for (idx, var) in variables.iter().enumerate() {
        values_map.insert(normalized_variable_names[idx].as_str(), var.value);
    }

    let formula_normalized = formula.to_lowercase();

    // Parse formula
    let expr = parse(&formula_normalized, &known_symbols, &HashSet::new(), None)
        .map_err(|e| format!("Failed to parse formula: {}", e))?;

    // Evaluate value
    let value_str = expr.evaluate(&values_map, &HashMap::new()).to_string();
    let value = value_str
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse result '{}': {}", value_str, e))?;

    if !value.is_finite() {
        return Err("Expression evaluated to non-finite value".to_string());
    }

    // Build covariance matrix
    let cov_entries: Vec<CovEntry> = variables
        .iter()
        .map(|v| CovEntry::Num(v.uncertainty * v.uncertainty))
        .collect();
    let cov = CovarianceMatrix::diagonal(cov_entries);

    // Calculate uncertainty
    let var_refs: Vec<&str> = normalized_variable_names
        .iter()
        .map(|s| s.as_str())
        .collect();
    let sigma_expr = uncertainty_propagation(&expr, &var_refs, Some(&cov))
        .map_err(|e| format!("Uncertainty propagation failed: {:?}", e))?;

    let sigma_str = sigma_expr
        .evaluate(&values_map, &HashMap::new())
        .to_string();
    let uncertainty = sigma_str
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse uncertainty '{}': {}", sigma_str, e))?;

    // Calculate derivatives for display
    let symbols: Vec<symb_anafis::Symbol> = normalized_variable_names
        .iter()
        .map(|s| symb_anafis::symb(s))
        .collect();
    let sym_refs: Vec<&symb_anafis::Symbol> = symbols.iter().collect();
    let gradient =
        symb_anafis::gradient(&expr, &sym_refs).map_err(|e| format!("Gradient failed: {:?}", e))?;

    let mut derivatives = HashMap::new();
    for (i, name) in variable_names.iter().enumerate() {
        if i < gradient.len() {
            let deriv_str = gradient[i]
                .evaluate(&values_map, &HashMap::new())
                .to_string();
            if let Ok(d) = deriv_str.parse::<f64>()
                && d.is_finite()
            {
                derivatives.insert(name.clone(), d);
            }
        }
    }

    Ok(CalculationResult {
        value,
        uncertainty,
        formula,
        derivatives,
        confidence_level: 0.95,
    })
}

/// Generate LaTeX representation
#[tauri::command]
pub fn generate_latex(formula: String, variables: Vec<String>) -> Result<LatexResult, String> {
    if variables.is_empty() {
        return Err("Please provide at least one variable".into());
    }

    let formula_normalized = formula.to_lowercase();
    let normalized_variables = normalize_variable_names(&variables)?;
    let known_symbols: HashSet<String> = normalized_variables.iter().cloned().collect();

    let expr = parse(&formula_normalized, &known_symbols, &HashSet::new(), None)
        .map_err(|e| format!("Failed to parse formula: {}", e))?;

    let var_refs: Vec<&str> = normalized_variables.iter().map(|s| s.as_str()).collect();
    let sigma_expr = uncertainty_propagation(&expr, &var_refs, None)
        .map_err(|e| format!("Uncertainty propagation failed: {:?}", e))?;

    Ok(LatexResult {
        string: format!("sigma_f = {}", expr),
        latex: format!("\\sigma_f = {}", sigma_expr.to_latex()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_uncertainty_simple() {
        let result = calculate_uncertainty(
            "x^2".to_string(),
            vec![CalculatorVariable {
                name: "x".to_string(),
                value: 3.0,
                uncertainty: 0.1,
            }],
        )
        .unwrap();

        assert!((result.value - 9.0).abs() < 1e-9);
        assert!((result.uncertainty - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_generate_latex_simple() {
        let result =
            generate_latex("x + y".to_string(), vec!["x".to_string(), "y".to_string()]).unwrap();
        assert!(result.latex.contains("\\sigma"));
    }

    #[test]
    fn test_calculate_uncertainty_mixed_case_variable_name() {
        let result = calculate_uncertainty(
            "AlotA^2".to_string(),
            vec![CalculatorVariable {
                name: "AlotA".to_string(),
                value: 3.0,
                uncertainty: 0.1,
            }],
        )
        .unwrap();

        assert!((result.value - 9.0).abs() < 1e-9);
        assert!((result.uncertainty - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_calculate_uncertainty_rejects_case_collisions() {
        let result = calculate_uncertainty(
            "a + A".to_string(),
            vec![
                CalculatorVariable {
                    name: "a".to_string(),
                    value: 1.0,
                    uncertainty: 0.1,
                },
                CalculatorVariable {
                    name: "A".to_string(),
                    value: 2.0,
                    uncertainty: 0.1,
                },
            ],
        );

        match result {
            Ok(_) => panic!("expected case-collision error"),
            Err(err) => assert!(err.contains("unique ignoring case")),
        }
    }
}
