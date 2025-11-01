use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use lru::LruCache;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::num::NonZeroUsize;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::fs;
use std::ffi::CString;
use tauri::AppHandle;
use tauri::Manager;

// Numerical uncertainty propagation using finite differences and Monte Carlo methods

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

static PYTHON_MODULE: Lazy<RwLock<Option<Py<PyModule>>>> = Lazy::new(|| RwLock::new(None));

pub fn initialize_python_module(py: Python, app: &AppHandle) -> PyResult<()> {
    let mut module_cache = PYTHON_MODULE.write().unwrap();
    if module_cache.is_none() {
        // Use Tauri's resource resolver
        let resource_dir = app.path().resource_dir().map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to get resource dir: {e}")))?;

        let python_file_path = if cfg!(debug_assertions) {
            // Dev mode: sympy_calls.py is in src-tauri/src directory
            let exe_path = std::env::current_exe().unwrap();
            let src_tauri_dir = exe_path.parent().unwrap().parent().unwrap().parent().unwrap();
            src_tauri_dir.join("python").join("sympy_calls.py")
        } else {
            // Release mode: sympy_calls.py is in resource directory
            resource_dir.join("python/sympy_calls.py")
        };

        // Clean the path by removing the \\?\ prefix if present
        let python_file_path_str = python_file_path.display().to_string();
        let clean_python_file_path_str = python_file_path_str.strip_prefix(r"\\?\").unwrap_or(&python_file_path_str).to_string();
        let clean_python_file_path = std::path::PathBuf::from(clean_python_file_path_str);

        let python_code = fs::read_to_string(&clean_python_file_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to read sympy_calls.py: {e}")))?;

        let code_cstr = CString::new(python_code)?;
        let filename_cstr = CString::new("sympy_calls.py")?;
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
    let module_cache = PYTHON_MODULE.read().unwrap();
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

/// Compute symbolic derivatives and numerical evaluations using the embedded Python helper
fn compute_symbolic_derivatives_py(app: &AppHandle, formula: &str, variables: &[String], values: &[f64]) -> Result<SymbolicResult, String> {
    Python::attach(|py| -> Result<SymbolicResult, String> {
        initialize_python_module(py, app).map_err(|e| e.to_string())?;
        let module = get_python_module(py).map_err(|e| e.to_string())?;
        let func = module.getattr(py, "calculate_symbolic_data").map_err(|e| python_err_to_string(py, e))?;
        let vars_vec: Vec<String> = variables.to_vec();
        let vals_vec: Vec<f64> = values.to_vec();

        let res_any = func.call1(py, (formula, vars_vec, vals_vec)).map_err(|e| python_err_to_string(py, e))?;
        let res_dict = res_any.cast_bound::<PyDict>(py).map_err(|e| python_err_to_string(py, e.into()))?;

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
            crate::utils::log_error("Python calculation failed", &std::io::Error::other(err.clone()));
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
            if let Ok(d) = obj.cast::<PyDict>() {
                for (k, v) in d.iter() {
                    let key: String = k.extract().map_err(|e: PyErr| e.to_string())?;
                    let val: f64 = v.extract().map_err(|e: PyErr| e.to_string())?;
                    if !val.is_finite() { return Err(format!("numerical_derivative for {key} not finite")); }
                    numerical_derivatives.insert(key, val);
                }
            }
        }

        Ok(SymbolicResult { numerical_value, numerical_derivatives })
    })
}

/// Compute LaTeX using Python helper via PyO3
fn compute_uncertainty_formula_py(app: &AppHandle, formula: &str, variables: &[String]) -> Result<LatexResult, String> {
    Python::attach(|py| -> Result<LatexResult, String> {
        initialize_python_module(py, app).map_err(|e| python_err_to_string(py, e))?;
        let module = get_python_module(py).map_err(|e| python_err_to_string(py, e))?;
        let func = module.getattr(py, "generate_latex_data").map_err(|e| python_err_to_string(py, e))?;
        let vars_vec: Vec<String> = variables.to_vec();
        let res_any = func.call1(py, (formula, vars_vec)).map_err(|e| python_err_to_string(py, e))?;
        let res_dict = res_any.cast_bound::<PyDict>(py).map_err(|e| python_err_to_string(py, e.into()))?;

        let success = res_dict
            .get_item("success").map_err(|e| python_err_to_string(py, e))?
            .ok_or("Missing 'success' key")?
            .extract::<bool>()
            .map_err(|e| python_err_to_string(py, e))?;
        if !success {
            let err = res_dict
                .get_item("error").map_err(|e| python_err_to_string(py, e))?
                .ok_or("Missing 'error' key")?
                .extract::<String>()
                .unwrap_or_else(|_| "Unknown Python error".to_string());
            return Err(err);
        }

        let string = res_dict
            .get_item("string").map_err(|e| python_err_to_string(py, e))?
            .ok_or("Missing 'string' key")?
            .extract::<String>()
            .unwrap_or_default();

        let latex = res_dict
            .get_item("latex").map_err(|e| python_err_to_string(py, e))?
            .ok_or("Missing 'latex' key")?
            .extract::<String>()
            .unwrap_or_default();

        Ok(LatexResult { string, latex })
    })
}

#[tauri::command]
pub async fn calculate_uncertainty(app: AppHandle, formula: String, variables: Vec<Variable>) -> Result<CalculationResult, String> {
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
    let app_clone = app.clone();
    let formula_clone = formula.clone();
    let variables_clone = variables.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        compute_uncertainty_sync(app_clone, formula_clone, variables_clone)
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

fn compute_uncertainty_sync(app: AppHandle, formula: String, variables: Vec<Variable>) -> Result<CalculationResult, String> {
    // Extract variable names and values
    let variable_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();
    let variable_values: Vec<f64> = variables.iter().map(|v| v.value).collect();

    // Use Python-backed computation exclusively. If Python fails, return the error.
    let symbolic_result = compute_symbolic_derivatives_py(&app, &formula, &variable_names, &variable_values)?;

    // Calculate uncertainty
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
pub async fn generate_latex(app: AppHandle, formula: String, variables: Vec<String>) -> Result<LatexResult, String> {
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
    let app_clone = app.clone();
    let formula_clone = formula.clone();
    let variables_clone = variables.clone();
    let latex_result = tauri::async_runtime::spawn_blocking(move || {
        compute_uncertainty_formula_py(&app_clone, &formula_clone, &variables_clone)
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
