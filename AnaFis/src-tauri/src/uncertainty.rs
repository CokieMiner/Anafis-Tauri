use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use lru::LruCache;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::num::NonZeroUsize;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::fs;
use std::ffi::CString;

// (Symbolic parsing, differentiation and LaTeX are performed by the
// embedded Python helper `python_formulas.py` — SymPy is the authority.)

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

#[derive(Serialize, Clone)]
pub struct CalculationResult {
    pub value: f64,
    pub uncertainty: f64,
    pub formula: String,
}

#[derive(Serialize, Clone)]
pub struct LatexResult {
    pub string: String,
    pub latex: String,
}

// Simple LRU cache for calculation results
static FORMULA_CACHE: Lazy<Mutex<LruCache<String, CalculationResult>>> = Lazy::new(|| {
    Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))
});

// LRU cache for LaTeX generation
static LATEX_CACHE: Lazy<Mutex<LruCache<String, LatexResult>>> = Lazy::new(|| {
    Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))
});

static PYTHON_MODULE: Lazy<Mutex<Option<Py<PyModule>>>> = Lazy::new(|| Mutex::new(None));

pub fn initialize_python_module(py: Python) -> PyResult<()> {
    let mut module_cache = PYTHON_MODULE.lock().unwrap();
    if module_cache.is_none() {
        // Get the path to the Python file relative to the executable
        let exe_path = std::env::current_exe()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to get executable path: {}", e)))?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to get executable directory"))?;
        // Go up from target/debug to src-tauri, then into src/
        let python_file_path = exe_dir.parent()  // target/
            .and_then(|p| p.parent())  // src-tauri/
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to navigate to src-tauri"))?
            .join("src")
            .join("python_formulas.py");

        let python_code = fs::read_to_string(&python_file_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to read python_formulas.py from {}: {}", python_file_path.display(), e)))?;
        let code_cstr = CString::new(python_code)?;
        let filename_cstr = CString::new("python_formulas.py")?;
        let module_name_cstr = CString::new("python_formulas")?;
        let module = PyModule::from_code(
            py,
            &code_cstr,
            &filename_cstr,
            &module_name_cstr,
        )?;
        *module_cache = Some(module.into());
    }
    Ok(())
}

pub fn get_python_module(_py: Python) -> PyResult<Py<PyModule>> {
    let module_cache = PYTHON_MODULE.lock().unwrap();
    if let Some(module) = &*module_cache {
        Ok(module.clone())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Python module not initialized"))
    }
}

/// Convert a PyErr into a string, attempting to include a traceback
fn python_err_to_string(py: Python<'_>, err: PyErr) -> String {
    // Try to use Python's traceback.format_exception
    let res = (|| {
        let traceback = py.import("traceback").ok()?;
        let etype = err.get_type(py);
        let evalue = err.value(py);
        let tb = err.traceback(py);
        // format_exception may return a list of strings
        let formatted = traceback.call_method1("format_exception", (etype, evalue, tb)).ok()?;
        let joined: String = formatted.extract::<Vec<String>>().ok()?.join("");
        Some(joined)
    })();

    if let Some(s) = res {
        s
    } else {
        // fallback to the error Display
        err.to_string()
    }
}

/// Basic input validation for the public API
fn validate_input(formula: &str, variables: &[Variable]) -> Result<(), String> {
    if formula.trim().is_empty() {
        return Err("Formula cannot be empty".into());
    }
    if variables.is_empty() {
        return Err("Please provide at least one variable".into());
    }
    for v in variables {
        if v.name.trim().is_empty() {
            return Err("Variable names cannot be empty".into());
        }
        if !v.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("Invalid variable name: {}", v.name));
        }
    }
    let names: HashSet<&str> = variables.iter().map(|v| v.name.as_str()).collect();
    if names.len() != variables.len() {
        return Err("Duplicate variable names".into());
    }
    for v in variables {
        if v.uncertainty < 0.0 {
            return Err("Uncertainty cannot be negative".into());
        }
    }
    Ok(())
}

/// Generate a cache key from formula and variables (stable order)
fn generate_cache_key(formula: &str, variables: &[Variable]) -> String {
    let mut vars: Vec<(&str, f64, f64)> = variables.iter().map(|v| (v.name.as_str(), v.value, v.uncertainty)).collect();
    vars.sort_by_key(|&(name, _, _)| name);
    let mut key = formula.to_string();
    key.push('|');
    for (name, value, uncertainty) in vars {
        key.push_str(name);
        key.push('=');
        key.push_str(&value.to_string());
        key.push(':');
        key.push_str(&uncertainty.to_string());
        key.push(',');
    }
    key
}

/// Compute symbolic derivatives and numerical evaluations using the embedded Python helper
fn compute_symbolic_derivatives_py(formula: &str, variables: &[String], values: &[f64]) -> Result<SymbolicResult, String> {
    Python::attach(|py| -> Result<SymbolicResult, String> {
        initialize_python_module(py).map_err(|e| e.to_string())?;
        let module = get_python_module(py).map_err(|e| e.to_string())?;
        let func = module.getattr(py, "calculate_symbolic_data").map_err(|e| python_err_to_string(py, e))?;
        let vars_vec: Vec<String> = variables.iter().cloned().collect();
        let vals_vec: Vec<f64> = values.to_vec();

        let res_any = func.call1(py, (formula, vars_vec, vals_vec)).map_err(|e| python_err_to_string(py, e))?;
        let res_dict = res_any.downcast_bound::<PyDict>(py).map_err(|e| python_err_to_string(py, e.into()))?;

        // success flag
        let success = match res_dict.get_item("success").map_err(|e| e.to_string())? {
            Some(o) => o.extract::<bool>().map_err(|e| e.to_string())?,
            None => false,
        };
        if !success {
            let err = match res_dict.get_item("error").map_err(|e| e.to_string())? {
                Some(o) => o.extract::<String>().map_err(|e| e.to_string())?,
                None => "Unknown Python error".to_string(),
            };
            return Err(err);
        }

        // numerical_value may be string or float
        let numerical_value = match res_dict.get_item("numerical_value").map_err(|e| e.to_string())? {
            Some(o) => {
                if let Ok(n) = o.extract::<f64>() { n }
                else if let Ok(s) = o.extract::<String>() { s.parse::<f64>().map_err(|e| e.to_string())? }
                else { return Err("numerical_value has unknown type".to_string()) }
            }
            None => return Err("numerical_value missing".to_string()),
        };
        if !numerical_value.is_finite() { return Err("numerical_value is not finite".to_string()) }

        let mut numerical_derivatives = HashMap::new();
        if let Ok(Some(obj)) = res_dict.get_item("numerical_derivatives") {
            if let Ok(d) = obj.downcast::<PyDict>() {
                for (k, v) in d.iter() {
                    let key: String = k.extract().map_err(|e| e.to_string())?;
                    let val: f64 = v.extract().map_err(|e| e.to_string())?;
                    if !val.is_finite() { return Err(format!("numerical_derivative for {} not finite", key)); }
                    numerical_derivatives.insert(key, val);
                }
            }
        }

        Ok(SymbolicResult { numerical_value, numerical_derivatives })
    })
}

/// Compute LaTeX using Python helper via PyO3
fn compute_uncertainty_formula_py(formula: &str, variables: &[String]) -> Result<LatexResult, String> {
    Python::attach(|py| -> Result<LatexResult, String> {
        initialize_python_module(py).map_err(|e| python_err_to_string(py, e))?;
        let module = get_python_module(py).map_err(|e| python_err_to_string(py, e))?;
        let func = module.getattr(py, "generate_latex_data").map_err(|e| python_err_to_string(py, e))?;
        let vars_vec: Vec<String> = variables.iter().cloned().collect();
        let res_any = func.call1(py, (formula, vars_vec)).map_err(|e| python_err_to_string(py, e))?;
        let res_dict = res_any.downcast_bound::<PyDict>(py).map_err(|e| python_err_to_string(py, e.into()))?;

        let success = res_dict
            .get_item("success").map_err(|e| python_err_to_string(py, e))?
            .ok_or_else(|| "Missing 'success' key")?
            .extract::<bool>()
            .map_err(|e| python_err_to_string(py, e))?;
        if !success {
            let err = res_dict
                .get_item("error").map_err(|e| python_err_to_string(py, e))?
                .ok_or_else(|| "Missing 'error' key")?
                .extract::<String>()
                .unwrap_or_else(|_| "Unknown Python error".to_string());
            return Err(err);
        }

        let string = res_dict
            .get_item("string").map_err(|e| python_err_to_string(py, e))?
            .ok_or_else(|| "Missing 'string' key")?
            .extract::<String>()
            .unwrap_or_default();

        let latex = res_dict
            .get_item("latex").map_err(|e| python_err_to_string(py, e))?
            .ok_or_else(|| "Missing 'latex' key")?
            .extract::<String>()
            .unwrap_or_default();

        Ok(LatexResult { string, latex })
    })
}

#[tauri::command]
pub fn calculate_uncertainty(formula: String, variables: Vec<Variable>) -> Result<CalculationResult, String> {
    // Input validation
    validate_input(&formula, &variables)?;

    // Generate cache key
    let cache_key = generate_cache_key(&formula, &variables);

    // Check cache first
    {
        let mut cache = FORMULA_CACHE.lock().map_err(|_| "Cache access failed".to_string())?;
        if let Some(result) = cache.get(&cache_key) {
            return Ok(result.clone());
        }
    }

    // Extract variable names and values
    let variable_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();
    let variable_values: Vec<f64> = variables.iter().map(|v| v.value).collect();

    // Use Python-backed computation exclusively. If Python fails, return the error.
    let symbolic_result = compute_symbolic_derivatives_py(&formula, &variable_names, &variable_values)?;

    // Calculate uncertainty
    let mut total_uncertainty_squared = 0.0;
    for var in &variables {
        if var.uncertainty > 0.0 {
            if let Some(deriv_value) = symbolic_result.numerical_derivatives.get(&var.name) {
                let term = deriv_value * deriv_value * var.uncertainty * var.uncertainty;
                total_uncertainty_squared += term;
            } else {
                return Err(format!("Missing derivative for variable {}", var.name));
            }
        }
    }

    let total_uncertainty = total_uncertainty_squared.max(0.0).sqrt();

    let result = CalculationResult {
        value: symbolic_result.numerical_value,
        uncertainty: total_uncertainty,
        formula: formula.to_string(),
    };

    // Cache the result
    {
        let mut cache = FORMULA_CACHE.lock().map_err(|_| "Cache access failed".to_string())?;
        cache.put(cache_key.to_string(), result.clone());
    }

    Ok(result)
}

#[tauri::command]
pub fn generate_latex(formula: String, variables: Vec<String>) -> Result<LatexResult, String> {
    // Input validation
    if variables.is_empty() {
        return Err("Please provide at least one variable".into());
    }
    for var in &variables {
        if var.trim().is_empty() {
            return Err("Variable names cannot be empty".into());
        }
        if !var.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("Invalid variable name: {}", var));
        }
    }

    let mut cache_key = formula.clone();
    cache_key.push('|');
    let mut sorted_vars = variables.clone();
    sorted_vars.sort();
    cache_key.push_str(&sorted_vars.join(","));

    // Check LaTeX cache
    {
        let mut cache = LATEX_CACHE.lock().map_err(|_| "Cache access failed".to_string())?;
        if let Some(cached_latex) = cache.get(&cache_key) {
            return Ok(cached_latex.clone());
        }
    }

    // Compute using Python if not cached
    let latex_result = compute_uncertainty_formula_py(&formula, &variables)?;

    // Cache the LaTeX result
    {
        let mut cache = LATEX_CACHE.lock().map_err(|_| "Cache access failed".to_string())?;
        cache.put(cache_key, latex_result.clone());
    }

    Ok(latex_result)
}
