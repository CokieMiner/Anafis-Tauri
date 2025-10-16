use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CString;

#[derive(Debug, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value_range: String,  // e.g., "A1:A10"
    pub uncertainty_range: String,  // e.g., "B1:B10"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UncertaintyFormulas {
    pub value_formulas: Vec<String>,  // Excel formulas for calculated values
    pub uncertainty_formulas: Vec<String>,  // Excel formulas for propagated uncertainties
    pub success: bool,
    pub error: Option<String>,
}

/// Parse range notation like "A1:A10" into start/end row numbers
fn parse_range(range: &str) -> Result<(String, usize, usize), String> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid range format: {}", range));
    }
    
    // Extract column letter and row numbers
    let start_col = parts[0].chars().take_while(|c| c.is_alphabetic()).collect::<String>();
    let start_row: usize = parts[0].chars()
        .skip_while(|c| c.is_alphabetic())
        .collect::<String>()
        .parse()
        .map_err(|_| format!("Invalid start row in range: {}", range))?;
    
    let end_row: usize = parts[1].chars()
        .skip_while(|c| c.is_alphabetic())
        .collect::<String>()
        .parse()
        .map_err(|_| format!("Invalid end row in range: {}", range))?;
    
    Ok((start_col, start_row, end_row))
}

/// Generate Excel cell reference from column and row
fn cell_ref(col: &str, row: usize) -> String {
    format!("{}{}", col, row)
}

/// Load Python module and calculate symbolic derivatives
fn calculate_derivatives_with_python(
    formula: &str,
    variables: &[String],
) -> Result<HashMap<String, String>, String> {
    Python::attach(|py| -> Result<HashMap<String, String>, String> {
        // Load sympy_calls.py module
        let resource_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .ok_or("Failed to get resource directory")?;

        let python_file = if cfg!(debug_assertions) {
            // Dev mode: look in src-tauri/python/
            let src_tauri_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {e}"))?;
            src_tauri_dir.join("python").join("sympy_calls.py")
        } else {
            // Release mode: look in resources
            resource_dir.join("python/sympy_calls.py")
        };

        if !python_file.exists() {
            return Err(format!("Python file not found: {:?}", python_file));
        }

        let code = std::fs::read_to_string(&python_file)
            .map_err(|e| format!("Failed to read sympy_calls.py: {e}"))?;

        let filename_cstr = CString::new("sympy_calls.py")
            .map_err(|e| format!("Failed to create CString: {e}"))?;
        let code_cstr = CString::new(code)
            .map_err(|e| format!("Failed to create code CString: {e}"))?;
        let module_name_cstr = CString::new("sympy_calls")
            .map_err(|e| format!("Failed to create module name CString: {e}"))?;

        let module = PyModule::from_code(py, &code_cstr, &filename_cstr, &module_name_cstr)
            .map_err(|e| format!("Failed to load Python module: {e}"))?;

        // Call calculate_symbolic_data with dummy values (we only need derivatives)
        let func = module.getattr("calculate_symbolic_data")
            .map_err(|e| format!("Failed to get function: {e}"))?;

        let vars_vec: Vec<String> = variables.to_vec();
        let vals_vec: Vec<f64> = vec![1.0; variables.len()]; // Dummy values

        let result = func.call1((formula, vars_vec, vals_vec))
            .map_err(|e| format!("Python function call failed: {e}"))?;

        let result_dict = result.downcast::<PyDict>()
            .map_err(|e| format!("Result is not a dict: {e}"))?;

        let success: bool = result_dict.get_item("success")
            .map_err(|e| e.to_string())?
            .ok_or("Missing 'success' key")?
            .extract()
            .map_err(|e| format!("Failed to extract success: {e}"))?;

        if !success {
            let error: String = result_dict.get_item("error")
                .map_err(|e| e.to_string())?
                .ok_or("Missing 'error' key")?
                .extract()
                .map_err(|e| format!("Failed to extract error: {e}"))?;
            return Err(error);
        }

        let derivatives_obj = result_dict.get_item("derivatives")
            .map_err(|e| e.to_string())?
            .ok_or("No derivatives in result")?;
        
        let derivatives_dict = derivatives_obj.downcast::<PyDict>()
            .map_err(|e| format!("Derivatives is not a dict: {e}"))?;

        let mut derivatives = HashMap::new();
        for (key, value) in derivatives_dict.iter() {
            let var_name: String = key.extract()
                .map_err(|e| format!("Failed to extract variable name: {e}"))?;
            let deriv_expr: String = value.extract()
                .map_err(|e| format!("Failed to extract derivative: {e}"))?;
            derivatives.insert(var_name, deriv_expr);
        }

        Ok(derivatives)
    })
}

/// Convert SymPy derivative expression to Excel formula
fn sympy_to_excel(
    sympy_expr: &str,
    var_map: &HashMap<String, String>,  // Maps variable names to Excel cell refs
) -> Result<String, String> {
    let mut excel_formula = sympy_expr.to_string();
    
    // Check for unsupported functions before conversion
    let unsupported_functions = [
        "Ynm", "assoc_legendre", "elliptic_e"
    ];
    
    for unsupported in &unsupported_functions {
        if excel_formula.contains(unsupported) {
            return Err(format!("Function '{}' is not supported in Excel formulas", unsupported));
        }
    }
    
    // Replace variable names with cell references
    for (var_name, cell_ref) in var_map {
        excel_formula = excel_formula.replace(var_name, cell_ref);
    }
    
    // Convert SymPy operators
    excel_formula = excel_formula.replace("**", "^");  // Power operator
    
    // Convert logarithmic functions (ORDER MATTERS! Specific before general)
    // Note: Input is already lowercase from normalize, so we only handle lowercase
    excel_formula = excel_formula.replace("log10", "LOG10");
    excel_formula = excel_formula.replace("log2", "LOG2");
    excel_formula = excel_formula.replace("ln", "LN");  // Natural log
    excel_formula = excel_formula.replace("log", "LOG");  // General log
    
    // Convert exponential and roots
    excel_formula = excel_formula.replace("sqrt", "SQRT");
    excel_formula = excel_formula.replace("exp", "EXP");
    excel_formula = excel_formula.replace("exp_polar", "EXP");  // Approximation
    excel_formula = excel_formula.replace("asin", "ASIN");
    excel_formula = excel_formula.replace("acos", "ACOS");
    excel_formula = excel_formula.replace("atan", "ATAN");
    excel_formula = excel_formula.replace("sin", "SIN");
    excel_formula = excel_formula.replace("sen", "SIN");  // Portuguese alias
    excel_formula = excel_formula.replace("cos", "COS");
    excel_formula = excel_formula.replace("tan", "TAN");
    
    // Convert less common trig (use workarounds)
    excel_formula = excel_formula.replace("cot(", "(1/TAN(");
    excel_formula = excel_formula.replace("sec(", "(1/COS(");
    excel_formula = excel_formula.replace("csc(", "(1/SIN(");
    
    // Convert hyperbolic functions
    excel_formula = excel_formula.replace("asinh", "ASINH");
    excel_formula = excel_formula.replace("acosh", "ACOSH");
    excel_formula = excel_formula.replace("atanh", "ATANH");
    excel_formula = excel_formula.replace("sinh", "SINH");
    excel_formula = excel_formula.replace("cosh", "COSH");
    excel_formula = excel_formula.replace("tanh", "TANH");
    
    // Convert less common hyperbolic (use workarounds)
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
    excel_formula = excel_formula.replace("LambertW", "LAMBERTW");
    excel_formula = excel_formula.replace("hermite", "HERMITE");
    excel_formula = excel_formula.replace("zeta", "ZETA");
    excel_formula = excel_formula.replace("elliptic_k", "ELLIPTIC_K");
    
    // Convert constants
    excel_formula = excel_formula.replace("pi", "PI()");
    excel_formula = excel_formula.replace("e", "EXP(1)");
    excel_formula = excel_formula.replace("E", "EXP(1)");
    
    // Convert other functions
    excel_formula = excel_formula.replace("abs", "ABS");
    excel_formula = excel_formula.replace("sinc", "SINC");
    excel_formula = excel_formula.replace("acot", "ACOT");
    excel_formula = excel_formula.replace("asec", "ASEC");
    excel_formula = excel_formula.replace("acsc", "ACSC");
    excel_formula = excel_formula.replace("acoth", "ACOTH");
    excel_formula = excel_formula.replace("asech", "ASECH");
    excel_formula = excel_formula.replace("acsch", "ACSCH");
    
    Ok(excel_formula)
}

#[tauri::command]
pub async fn generate_uncertainty_formulas(
    variables: Vec<Variable>,
    formula: String,
) -> Result<UncertaintyFormulas, String> {
    // Normalize formula to lowercase for SymPy (it only recognizes lowercase)
    // But we'll preserve the original for value formula generation
    let formula_for_sympy = formula.to_lowercase();
    
    // Extract variable names for Python
    let var_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();
    
    // Calculate derivatives using Python/SymPy with normalized formula
    let derivatives = match calculate_derivatives_with_python(&formula_for_sympy, &var_names) {
        Ok(d) => d,
        Err(e) => {
            return Ok(UncertaintyFormulas {
                value_formulas: vec![],
                uncertainty_formulas: vec![],
                success: false,
                error: Some(format!("Failed to calculate derivatives: {}", e)),
            });
        }
    };
    
    // Parse all ranges to get row counts
    let mut row_count = 0;
    let mut var_info = Vec::new();
    
    for var in &variables {
        let (val_col, val_start, val_end) = parse_range(&var.value_range)?;
        
        // Handle optional uncertainty range (empty string means no uncertainty)
        let uncertainty_info = if var.uncertainty_range.is_empty() {
            None
        } else {
            let (unc_col, unc_start, unc_end) = parse_range(&var.uncertainty_range)?;
            
            if val_end - val_start != unc_end - unc_start {
                return Err(format!(
                    "Value and uncertainty ranges must have the same length for variable '{}'",
                    var.name
                ));
            }
            
            Some((unc_col, unc_start))
        };
        
        let current_row_count = val_end - val_start + 1;
        if row_count == 0 {
            row_count = current_row_count;
        } else if row_count != current_row_count {
            return Err("All variable ranges must have the same length".to_string());
        }
        
        var_info.push((var.name.clone(), val_col, val_start, uncertainty_info));
    }
    
    let mut value_formulas = Vec::new();
    let mut uncertainty_formulas = Vec::new();
    
    // Generate formulas for each row
    for i in 0..row_count {
        // Create variable mapping for this row
        let mut var_map = HashMap::new();
        for (var_name, val_col, val_start, _uncertainty_info) in &var_info {
            let row = val_start + i;
            var_map.insert(var_name.clone(), cell_ref(val_col, row));
        }
        
        // Generate value formula (substitute variables in original formula and convert to Excel)
        let mut value_var_map = HashMap::new();
        for (var_name, val_col, val_start, _uncertainty_info) in &var_info {
            let row = val_start + i;
            value_var_map.insert(var_name.clone(), cell_ref(val_col, row));
        }
        // Use normalized formula and sympy_to_excel to properly convert all functions
        let value_formula_body = match sympy_to_excel(&formula_for_sympy, &value_var_map) {
            Ok(formula) => formula,
            Err(e) => return Ok(UncertaintyFormulas {
                value_formulas: vec![],
                uncertainty_formulas: vec![],
                success: false,
                error: Some(format!("Formula conversion error: {}", e)),
            }),
        };
        let value_formula = format!("={}", value_formula_body);
        value_formulas.push(value_formula);
        
        // Generate uncertainty formula: σ_f = sqrt(Σ(∂f/∂xi * σ_xi)²)
        let mut uncertainty_terms = Vec::new();
        
        for (var_name, _val_col, _val_start, uncertainty_info) in &var_info {
            // Skip variables with no uncertainty range (treated as zero uncertainty)
            if uncertainty_info.is_none() {
                continue;
            }
            
            if let Some(deriv_expr) = derivatives.get(var_name) {
                let (unc_col, unc_start) = uncertainty_info.as_ref().unwrap();
                let row = unc_start + i;
                
                // Create cell reference map for derivative
                let mut deriv_var_map = HashMap::new();
                for (vname, vcol, vstart, _uncertainty_info) in &var_info {
                    let vrow = vstart + i;
                    deriv_var_map.insert(vname.clone(), cell_ref(vcol, vrow));
                }
                
                // Convert derivative to Excel formula
                let deriv_excel = match sympy_to_excel(deriv_expr, &deriv_var_map) {
                    Ok(formula) => formula,
                    Err(e) => return Ok(UncertaintyFormulas {
                        value_formulas: vec![],
                        uncertainty_formulas: vec![],
                        success: false,
                        error: Some(format!("Derivative conversion error: {}", e)),
                    }),
                };
                let sigma_ref = cell_ref(unc_col, row);
                
                // Term: (∂f/∂xi * σ_xi)²
                let term = format!("(({}) * {})^2", deriv_excel, sigma_ref);
                uncertainty_terms.push(term);
            }
        }
        
        // Combine all terms: =SQRT(term1 + term2 + ...)
        let uncertainty_formula = format!("=SQRT({})", uncertainty_terms.join(" + "));
        uncertainty_formulas.push(uncertainty_formula);
    }
    
    Ok(UncertaintyFormulas {
        value_formulas,
        uncertainty_formulas,
        success: true,
        error: None,
    })
}
