use nalgebra::{DMatrix, DVector};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use symb_anafis::{CompiledEvaluator, Expr, Symbol, eval_f64, gradient, parse, symb};

use super::types::{OdrError, OdrFitRequest, OdrResult};

/// Minimum variance allowed for a data point to avoid division by zero.
pub const MIN_VARIANCE: f64 = 1e-24;
/// Threshold for considering a matrix singular during inversion.
pub const MATRIX_SINGULAR_EPS: f64 = 1e-14;
/// Tolerance for identical independent values to be considered correlated.
pub const CORRELATION_TOLERANCE: f64 = 1e-9;
/// Default maximum number of iterations for ODR convergence.
pub const DEFAULT_MAX_ITERATIONS: usize = 200;
/// Default convergence tolerance for ODR.
pub const DEFAULT_TOLERANCE: f64 = 1e-9;
/// Default initial damping factor (lambda) for Levenberg-Marquardt.
pub const DEFAULT_DAMPING: f64 = 1e-3;
/// Maximum allowed damping factor before assuming divergence.
pub const MAX_DAMPING: f64 = 1e15;
/// Minimum allowed damping factor.
pub const MIN_DAMPING: f64 = 1e-15;
/// Maximum number of compiled models to keep in the cache.
pub const MODEL_CACHE_MAX_ENTRIES: usize = 64;
/// Tolerance for eigenvalue checks to ensure Positive Semi-Definiteness.
pub const PSD_EIGEN_TOLERANCE: f64 = 1e-10;

/// A model that has been compiled into executable bytecode for a specific layer.
#[derive(Debug)]
pub struct CompiledModel {
    /// The original mathematical formula.
    pub formula: String,
    /// The name of the dependent variable for this layer.
    pub dependent_name: String,
    /// Names of the parameters to be fitted.
    pub parameter_names: Vec<String>,
    /// Names of the independent variables.
    pub independent_names: Vec<String>,
    /// The parsed expression for the main formula (for `eval_f64`).
    pub model_expr: Expr,
    /// Parsed expressions for the partial derivatives with respect to each parameter.
    pub parameter_gradient_exprs: Vec<Expr>, // d f / d p_j
    /// Parsed expressions for the partial derivatives with respect to each independent variable.
    pub independent_gradient_exprs: Vec<Expr>, // d f / d x_k
    /// Compiled evaluator for the main formula (fallback).
    pub model_evaluator: CompiledEvaluator,
    /// Compiled evaluators for the partial derivatives with respect to each parameter (fallback).
    pub parameter_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d p_j
    /// Compiled evaluators for the partial derivatives with respect to each independent variable (fallback).
    pub independent_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d x_k
}

/// Data prepared and validated for the ODR solver.
pub struct PreparedData {
    /// Combined names of all variables (independent and dependent).
    pub variable_names: Vec<String>,
    /// Matrix of variable values: [`variable_index`][`point_index`].
    pub variable_values: Vec<Vec<f64>>,
    /// Full covariance matrices for each data point across the combined variable space.
    /// Format: `[point_index][dim][dim]`, where `dim = variable_names.len()`.
    pub point_covariances: Vec<Vec<Vec<f64>>>,
    /// Total number of data points.
    pub point_count: usize,
    /// Whether any near-zero uncertainties were clamped to a minimum value.
    pub had_uncertainty_clamp: bool,
}

/// The current state of an ODR evaluation across all layers.
pub struct EvaluationState {
    /// Current weighted total chi-squared value across all layers.
    pub chi_squared: f64,
    /// Raw residuals (observed - predicted) for each layer: [`layer_idx`][point_idx].
    pub layer_residuals: Vec<Vec<f64>>,
    /// Values predicted by the models at the current state: [`layer_idx`][point_idx].
    pub layer_fitted_values: Vec<Vec<f64>>,
    /// Flattened residuals weighted by the inverse covariance matrix.
    pub flat_weighted_residuals: DVector<f64>,
    /// Global Jacobian matrix weighted by the inverse covariance matrix.
    pub global_weighted_jacobian: DMatrix<f64>,
}

/// LRU cache for compiled models to avoid redundant recompilation.
#[derive(Debug, Default)]
pub struct ModelCache {
    /// Map of model formulas/keys to compiled models.
    pub entries: HashMap<String, Arc<CompiledModel>>,
    /// Order of access to implement LRU eviction.
    pub access_order: VecDeque<String>,
}

/// Global singleton for model caching.
pub static MODEL_CACHE: std::sync::LazyLock<Mutex<ModelCache>> = std::sync::LazyLock::new(|| Mutex::new(ModelCache::default()));

impl ModelCache {
    /// Returns a compiled model from the cache if it exists.
    pub fn get(&mut self, key: &str) -> Option<Arc<CompiledModel>> {
        if !self.entries.contains_key(key) {
            return None;
        }
        self.touch(key);
        self.entries.get(key).map(Arc::clone)
    }

    /// Inserts a compiled model into the cache, evicting the oldest entry if full.
    pub fn insert(&mut self, key: &str, model: Arc<CompiledModel>) {
        if self.entries.contains_key(key) {
            self.entries.insert(key.to_string(), model);
            self.touch(key);
            return;
        }

        if self.entries.len() >= MODEL_CACHE_MAX_ENTRIES {
            self.evict_one();
        }

        self.entries.insert(key.to_string(), model);
        self.touch(key);
    }

    fn evict_one(&mut self) {
        while let Some(oldest_key) = self.access_order.pop_front() {
            if self.entries.remove(&oldest_key).is_some() {
                return;
            }
        }
    }

    fn touch(&mut self, key: &str) {
        if let Some(position) = self.access_order.iter().position(|stored| stored == key) {
            self.access_order.remove(position);
        }
        self.access_order.push_back(key.to_string());
    }
}

/// Validates if an identifier is a valid symbol name.
///
/// # Errors
/// Returns `OdrError::Validation` if the identifier is empty or contains invalid characters.
pub fn validate_identifier(identifier: &str, label: &str) -> OdrResult<()> {
    if identifier.is_empty() {
        return Err(OdrError::Validation(format!(
            "{label} names cannot be empty"
        )));
    }

    let mut chars = identifier.chars();
    let first = chars
        .next()
        .ok_or_else(|| OdrError::Validation(format!("{label} names cannot be empty")))?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(OdrError::Validation(format!(
            "Invalid {label} '{identifier}': first character must be a letter or '_'"
        )));
    }

    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(OdrError::Validation(format!(
            "Invalid {label} '{identifier}': use only letters, digits, and '_'"
        )));
    }

    Ok(())
}

/// Normalizes a list of identifiers by trimming and converting to lowercase.
///
/// # Errors
/// Returns `OdrError::Validation` if any identifier is invalid or duplicate.
pub fn normalize_identifiers(raw: &[String], label: &str) -> OdrResult<Vec<String>> {
    if raw.is_empty() {
        return Err(OdrError::Validation(format!(
            "At least one {label} is required"
        )));
    }

    let mut normalized = Vec::with_capacity(raw.len());
    let mut seen = HashSet::new();

    for name in raw {
        let trimmed = name.trim();
        validate_identifier(trimmed, label)?;

        let lower = trimmed.to_lowercase();
        if !seen.insert(lower.clone()) {
            return Err(OdrError::Validation(format!(
                "Duplicate {label} names are not allowed (case-insensitive collision on '{name}')"
            )));
        }
        normalized.push(lower);
    }

    Ok(normalized)
}

/// Validates that the sets of independent variables and parameters are disjoint.
///
/// # Errors
/// Returns `OdrError::Validation` if a symbol is both an independent variable and a parameter.
pub fn validate_symbol_sets(independent: &[String], parameters: &[String]) -> OdrResult<()> {
    let independent_set: HashSet<&str> = independent.iter().map(String::as_str).collect();
    let parameter_set: HashSet<&str> = parameters.iter().map(String::as_str).collect();

    if let Some(symbol) = independent_set.intersection(&parameter_set).next() {
        return Err(OdrError::Validation(format!(
            "Symbol '{symbol}' is used both as independent variable and parameter"
        )));
    }

    Ok(())
}

/// Builds a cache key for a model based on its formula and variable names.
fn build_model_cache_key(
    formula: &str,
    dependent_name: &str,
    independent_names: &[String],
    parameter_names: &[String],
) -> String {
    format!(
        "{}|{}|x:{}|p:{}",
        formula.trim().to_lowercase(),
        dependent_name.trim().to_lowercase(),
        independent_names.join(","),
        parameter_names.join(",")
    )
}

/// Retrieves a compiled model from cache or compiles it if not found.
///
/// # Errors
/// Returns `OdrError` if model compilation fails or cache is poisoned.
pub fn get_or_compile_model(
    model_formula: &str,
    dependent_name: &str,
    independent_names: &[String],
    parameter_names: &[String],
) -> OdrResult<Arc<CompiledModel>> {
    let key = build_model_cache_key(model_formula, dependent_name, independent_names, parameter_names);

    {
        let mut cache = MODEL_CACHE.lock().map_err(|_| OdrError::CachePoisoned)?;
        if let Some(model) = cache.get(&key) {
            return Ok(model);
        }
    }

    let compiled = Arc::new(compile_model_inner(
        model_formula,
        dependent_name,
        independent_names,
        parameter_names,
    )?);

    let mut cache = MODEL_CACHE.lock().map_err(|_| OdrError::CachePoisoned)?;
    if let Some(model) = cache.get(&key) {
        return Ok(model);
    }

    cache.insert(&key, Arc::clone(&compiled));
    drop(cache);

    Ok(compiled)
}

/// Internal function to compile a model formula into evaluators.
fn compile_model_inner(
    model_formula: &str,
    dependent_name: &str,
    independent_names: &[String],
    parameter_names: &[String],
) -> OdrResult<CompiledModel> {
    let formula = model_formula.trim().to_lowercase();
    if formula.is_empty() {
        return Err(OdrError::Validation(
            "Model formula cannot be empty".to_string(),
        ));
    }

    let known_symbols: HashSet<String> = independent_names
        .iter()
        .chain(parameter_names.iter())
        .cloned()
        .collect();

    let expr = parse(&formula, &known_symbols, &HashSet::new(), None)
        .map_err(|error| OdrError::Parse(error.to_string()))?;

    let mut evaluator_order: Vec<&str> =
        Vec::with_capacity(independent_names.len() + parameter_names.len());
    evaluator_order.extend(independent_names.iter().map(String::as_str));
    evaluator_order.extend(parameter_names.iter().map(String::as_str));

    let model_evaluator = CompiledEvaluator::compile(&expr, &evaluator_order, None)
        .map_err(|error| OdrError::Compile(format!("model evaluator: {error:?}")))?;

    let parameter_symbols: Vec<Symbol> = parameter_names.iter().map(|name| symb(name)).collect();
    let parameter_symbol_refs: Vec<&Symbol> = parameter_symbols.iter().collect();
    let parameter_gradients = gradient(&expr, &parameter_symbol_refs)
        .map_err(|error| OdrError::Compile(format!("parameter gradients: {error:?}")))?;

    let mut parameter_gradient_evaluators = Vec::with_capacity(parameter_gradients.len());
    let mut parameter_gradient_exprs = Vec::with_capacity(parameter_gradients.len());
    for gradient_expr in parameter_gradients {
        parameter_gradient_evaluators.push(
            CompiledEvaluator::compile(&gradient_expr, &evaluator_order, None).map_err(
                |error| OdrError::Compile(format!("parameter derivative evaluator: {error:?}")),
            )?,
        );
        parameter_gradient_exprs.push(gradient_expr);
    }

    let independent_symbols: Vec<Symbol> =
        independent_names.iter().map(|name| symb(name)).collect();
    let independent_symbol_refs: Vec<&Symbol> = independent_symbols.iter().collect();
    let independent_gradients = gradient(&expr, &independent_symbol_refs)
        .map_err(|error| OdrError::Compile(format!("independent gradients: {error:?}")))?;

    let mut independent_gradient_evaluators = Vec::with_capacity(independent_gradients.len());
    let mut independent_gradient_exprs = Vec::with_capacity(independent_gradients.len());
    for gradient_expr in independent_gradients {
        independent_gradient_evaluators.push(
            CompiledEvaluator::compile(&gradient_expr, &evaluator_order, None).map_err(
                |error| OdrError::Compile(format!("independent derivative evaluator: {error:?}")),
            )?,
        );
        independent_gradient_exprs.push(gradient_expr);
    }

    Ok(CompiledModel {
        formula,
        dependent_name: dependent_name.to_lowercase(),
        parameter_names: parameter_names.to_vec(),
        independent_names: independent_names.to_vec(),
        model_expr: expr,
        parameter_gradient_exprs,
        independent_gradient_exprs,
        model_evaluator,
        parameter_gradient_evaluators,
        independent_gradient_evaluators,
    })
}

/// Prepares data for ODR fitting by combining all observed variables into a single unified space.
///
/// # Errors
/// Returns `OdrError::Validation` if data length or values are invalid.
pub fn prepare_data(request: &OdrFitRequest) -> OdrResult<PreparedData> {
    if request.layers.is_empty() {
        return Err(OdrError::Validation(
            "At least one model layer is required".to_string(),
        ));
    }

    if request.dependent_variables.is_empty() {
        return Err(OdrError::Validation(
            "At least one dependent variable observation is required".to_string(),
        ));
    }

    let point_count = request.dependent_variables[0].values.len();
    if point_count < 2 {
        return Err(OdrError::Validation(
            "At least two observations are required for fitting".to_string(),
        ));
    }

    let use_poisson = request.use_poisson_weighting.unwrap_or(false);

    let mut variable_names = Vec::new();
    let mut variable_values = Vec::new();
    let mut variable_sigmas = Vec::new();
    let mut had_uncertainty_clamp = false;

    let mut process_variable = |var: &super::types::VariableInput, is_dependent: bool| -> OdrResult<()> {
        if var.values.len() != point_count {
            return Err(OdrError::Validation(format!(
                "Variable '{}' length mismatch: expected {}, got {}",
                var.name, point_count, var.values.len()
            )));
        }

        let name = var.name.trim().to_lowercase();
        validate_identifier(&name, "variable")?;

        if variable_names.contains(&name) {
            return Err(OdrError::Validation(format!(
                "Duplicate expected variable name mapping: {name}"
            )));
        }

        variable_names.push(name);
        variable_values.push(sanitize_values(&var.values, &var.name)?);

        if let Some(uncertainties) = &var.uncertainties {
            if uncertainties.len() != point_count {
                return Err(OdrError::Validation(format!(
                    "Uncertainty length mismatch for '{}': expected {}, got {}",
                    var.name,
                    point_count,
                    uncertainties.len()
                )));
            }
            let (sigma, clamped) = sanitize_uncertainties(uncertainties, &var.name)?;
            had_uncertainty_clamp |= clamped;
            variable_sigmas.push(sigma);
        } else if is_dependent && use_poisson {
            let mut sigma = Vec::with_capacity(point_count);
            for val in &var.values {
                let err = if *val <= 1.0 { 1.0 } else { val.sqrt() };
                sigma.push(err);
            }
            variable_sigmas.push(sigma);
        } else {
            variable_sigmas.push(vec![0.0; point_count]);
        }

        Ok(())
    };

    for var in &request.independent_variables {
        process_variable(var, false)?;
    }
    for var in &request.dependent_variables {
        process_variable(var, true)?;
    }

    let point_covariances = build_point_covariances(
        point_count,
        &variable_sigmas,
        request.point_correlations.as_deref(),
    )?;

    Ok(PreparedData {
        variable_names,
        variable_values,
        point_covariances,
        point_count,
        had_uncertainty_clamp,
    })
}

/// Validates and ensures all values are finite.
fn sanitize_values(values: &[f64], label: &str) -> OdrResult<Vec<f64>> {
    let mut sanitized = Vec::with_capacity(values.len());
    for (idx, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Validation(format!(
                "Non-finite value in {label} at index {idx}"
            )));
        }
        sanitized.push(*value);
    }
    Ok(sanitized)
}

/// Validates uncertainties and clamps near-zero values.
fn sanitize_uncertainties(values: &[f64], label: &str) -> OdrResult<(Vec<f64>, bool)> {
    let mut sanitized = Vec::with_capacity(values.len());
    let mut had_clamp = false;
    let sigma_min = MIN_VARIANCE.sqrt();

    for (idx, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Validation(format!(
                "Non-finite uncertainty in {label} at index {idx}"
            )));
        }

        let positive = value.abs();
        let clamped = positive.max(sigma_min);
        if clamped > positive {
            had_clamp = true;
        }
        sanitized.push(clamped);
    }

    Ok((sanitized, had_clamp))
}

/// Constructs the full covariance matrix for each measurement point in the unified variable space.
fn build_point_covariances(
    point_count: usize,
    variable_sigmas: &[Vec<f64>],
    point_correlations: Option<&[Vec<Vec<f64>>]>,
) -> OdrResult<Vec<Vec<Vec<f64>>>> {
    let dim = variable_sigmas.len();

    if let Some(correlations) = point_correlations
        && correlations.len() != point_count
    {
        return Err(OdrError::Validation(format!(
            "point_correlations length mismatch: expected {}, got {}",
            point_count,
            correlations.len()
        )));
    }

    let mut covariances = Vec::with_capacity(point_count);

    for point in 0..point_count {
        let mut sigmas = vec![0.0; dim];
        for var_idx in 0..dim {
            sigmas[var_idx] = variable_sigmas[var_idx][point];
        }

        let covariance = if let Some(correlations) = point_correlations {
            let corr = &correlations[point];
            validate_point_correlation_matrix(corr, dim, point)?;

            let mut sigma = vec![vec![0.0; dim]; dim];
            for row in 0..dim {
                for col in 0..dim {
                    sigma[row][col] = corr[row][col] * sigmas[row] * sigmas[col];
                }
            }
            sigma
        } else {
            let mut sigma = vec![vec![0.0; dim]; dim];
            for idx in 0..dim {
                sigma[idx][idx] = sigmas[idx] * sigmas[idx];
            }
            sigma
        };

        covariances.push(covariance);
    }

    Ok(covariances)
}

/// Validates if a point correlation matrix is symmetric and has unit diagonal.
fn validate_point_correlation_matrix(
    matrix: &[Vec<f64>],
    dim: usize,
    point: usize,
) -> OdrResult<()> {
    if matrix.len() != dim {
        return Err(OdrError::Validation(format!(
            "point_correlations[{point}] has invalid shape: expected {dim} rows, got {}",
            matrix.len()
        )));
    }

    for row in matrix {
        if row.len() != dim {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}] has invalid shape: expected {dim} columns"
            )));
        }
    }

    for (row_idx, row_values) in matrix.iter().enumerate().take(dim) {
        let diagonal = row_values[row_idx];
        if !diagonal.is_finite() {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}][{row_idx}][{row_idx}] must be finite"
            )));
        }
        if (diagonal - 1.0).abs() > CORRELATION_TOLERANCE {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}][{row_idx}][{row_idx}] must be 1"
            )));
        }

        for (col_idx, value) in row_values.iter().copied().enumerate().take(dim) {
            if !value.is_finite() {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}][{row_idx}][{col_idx}] must be finite"
                )));
            }
            if !(-1.0 - CORRELATION_TOLERANCE..=1.0 + CORRELATION_TOLERANCE).contains(&value) {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}][{row_idx}][{col_idx}] must be in [-1, 1]"
                )));
            }

            let symmetric = matrix[col_idx][row_idx];
            if (value - symmetric).abs() > CORRELATION_TOLERANCE {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}] must be symmetric"
                )));
            }
        }
    }

    if !is_positive_semidefinite(matrix) {
        return Err(OdrError::Validation(format!(
            "point_correlations[{point}] must be positive semidefinite"
        )));
    }

    Ok(())
}

/// Checks if a matrix is Positive Semi-Definite using eigenvalue decomposition.
fn is_positive_semidefinite(matrix: &[Vec<f64>]) -> bool {
    let dim = matrix.len();
    if dim == 0 {
        return true;
    }

    let mut flat = Vec::with_capacity(dim * dim);
    for row in matrix {
        flat.extend(row.iter().copied());
    }

    let m = DMatrix::from_row_slice(dim, dim, &flat);
    let eigen = m.symmetric_eigen();
    eigen
        .eigenvalues
        .iter()
        .all(|value| value.is_finite() && *value >= -PSD_EIGEN_TOLERANCE)
}

/// Solves the Orthogonal Distance Regression (ODR) problem using Levenberg-Marquardt across all layers simultaneously.
///
/// # Errors
/// Returns `OdrError` if numerical convergence fails or fitting error occurs.
pub fn solve_odr(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    mut parameters: Vec<f64>,
    global_parameter_names: &[String],
    max_iterations: usize,
    tolerance: f64,
    initial_damping: f64,
) -> OdrResult<(Vec<f64>, EvaluationState, usize)> {
    let mut damping = initial_damping;
    let mut nu = 2.0;
    let mut current = evaluate_model(models, data, &parameters, global_parameter_names)?;
    let mut iterations = 0;

    for iteration in 0..max_iterations {
        iterations = iteration + 1;

        let (normal_matrix, gradient_vector) = build_normal_equations(&current);
        let gradient_norm = gradient_vector.norm();
        if gradient_norm <= tolerance {
            break;
        }

        let parameter_count = parameters.len();
        let mut damped_matrix = normal_matrix.clone();
        for diagonal in 0..parameter_count {
            damped_matrix[(diagonal, diagonal)] +=
                damping * (normal_matrix[(diagonal, diagonal)].abs() + 1.0);
        }

        let rhs = -&gradient_vector;
        let Ok(delta) = solve_linear_system(damped_matrix, &rhs) else {
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
            continue;
        };

        let delta_norm = delta.norm();
        let parameter_norm = parameters
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if delta_norm <= tolerance * (parameter_norm + tolerance) {
            break;
        }

        let trial_parameters: Vec<f64> = parameters
            .iter()
            .zip(delta.iter())
            .map(|(parameter, step)| parameter + step)
            .collect();

        if trial_parameters.iter().any(|value| !value.is_finite()) {
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
            continue;
        }

        let trial = evaluate_model(models, data, &trial_parameters, global_parameter_names)?;
        let actual_reduction = current.chi_squared - trial.chi_squared;

        let h_delta = &normal_matrix * &delta;
        let mut predicted_reduction = -2.0f64.mul_add(gradient_vector.dot(&delta), -delta.dot(&h_delta));
        if !predicted_reduction.is_finite() || predicted_reduction <= MIN_VARIANCE {
            predicted_reduction = MIN_VARIANCE;
        }

        let rho = actual_reduction / predicted_reduction;

        if actual_reduction > 0.0 && rho.is_finite() && rho > 0.0 {
            let improvement = actual_reduction.abs();
            parameters = trial_parameters;
            current = trial;

            let factor = (1.0 - 2.0f64.mul_add(rho, -1.0).powi(3)).clamp(1.0 / 3.0, 0.9);
            damping = (damping * factor).max(MIN_DAMPING);
            nu = 2.0;

            if improvement <= tolerance {
                break;
            }
        } else {
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
        }
    }

    Ok((parameters, current, iterations))
}

/// Evaluates the multi-layer model at the current global parameters.
///
/// # Errors
/// Returns `OdrError::Numerical` if models or gradients evaluate to non-finite values.
#[allow(clippy::too_many_lines, reason = "Multi-layer ODR evaluation requires comprehensive logic")]
pub fn evaluate_model(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    global_parameters: &[f64],
    global_parameter_names: &[String],
) -> OdrResult<EvaluationState> {
    let point_count = data.point_count;
    let global_parameter_count = global_parameters.len();
    let var_count = data.variable_names.len();

    let mut chi_squared = 0.0;
    
    let mut layer_residuals = Vec::with_capacity(models.len());
    let mut layer_fitted_values = Vec::with_capacity(models.len());
    
    let total_rows = point_count * models.len();
    let mut flat_weighted_residuals = vec![0.0; total_rows];
    let mut global_weighted_jacobian = vec![0.0; total_rows * global_parameter_count];

    for (layer_idx, model) in models.iter().enumerate() {
        let dep_var_idx = data.variable_names.iter().position(|name| name == &model.dependent_name).ok_or_else(|| {
            OdrError::Validation(format!("Dependent variable {} not found in data", model.dependent_name))
        })?;

        let mut indep_var_indices = Vec::with_capacity(model.independent_names.len());
        for name in &model.independent_names {
            let idx = data.variable_names.iter().position(|n| n == name).ok_or_else(|| {
                OdrError::Validation(format!("Independent variable {name} not found in data"))
            })?;
            indep_var_indices.push(idx);
        }

        let mut local_parameters = Vec::with_capacity(model.parameter_names.len());
        let mut param_global_to_local = Vec::with_capacity(model.parameter_names.len());

        for local_name in &model.parameter_names {
            let global_idx = global_parameter_names.iter().position(|name| name == local_name).ok_or_else(|| {
                OdrError::Validation(format!("Parameter {local_name} not found in global parameters"))
            })?;
            local_parameters.push(global_parameters[global_idx]);
            param_global_to_local.push(global_idx);
        }

        let mut columns: Vec<&[f64]> = Vec::with_capacity(indep_var_indices.len() + local_parameters.len());
        for &idx in &indep_var_indices {
            columns.push(&data.variable_values[idx]);
        }

        let parameter_columns: Vec<Vec<f64>> = local_parameters
            .iter()
            .map(|parameter| vec![*parameter; point_count])
            .collect();
        for values in &parameter_columns {
            columns.push(values);
        }

        // Use eval_f64 for SIMD+parallel batch evaluation of model + all gradients
        let batch_result = evaluate_model_and_gradients_batch(
            &model.model_expr,
            &model.independent_gradient_exprs,
            &model.parameter_gradient_exprs,
            &model.independent_names,
            &model.parameter_names,
            &columns,
            point_count,
            layer_idx,
        )?;
        let fitted_values = batch_result.fitted_values;
        let independent_derivatives = batch_result.independent_derivatives;
        let parameter_derivatives = batch_result.parameter_derivatives;

        let mut current_residuals = Vec::with_capacity(point_count);

        for point in 0..point_count {
            let fitted = fitted_values[point];
            if !fitted.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Model evaluated to non-finite value at data point {point} in layer {layer_idx}"
                )));
            }

            let residual = data.variable_values[dep_var_idx][point] - fitted;
            current_residuals.push(residual);

            let mut a = vec![0.0; var_count];
            for (local_idx, &global_var_idx) in indep_var_indices.iter().enumerate() {
                let dfdx = independent_derivatives[local_idx][point];
                if !dfdx.is_finite() {
                    return Err(OdrError::Numerical(format!(
                        "Independent gradient evaluated to non-finite value at point {point} layer {layer_idx}"
                    )));
                }
                a[global_var_idx] = -dfdx;
            }
            a[dep_var_idx] = 1.0;

            let sigma = &data.point_covariances[point];
            let sigma_a = mat_vec(sigma, &a);
            let s2 = dot(&a, &sigma_a).max(MIN_VARIANCE);

            let inv_sqrt_s2 = 1.0 / s2.sqrt();
            let weighted_residual = residual * inv_sqrt_s2;
            
            let row_idx = layer_idx * point_count + point;
            flat_weighted_residuals[row_idx] = weighted_residual;
            chi_squared += weighted_residual * weighted_residual;

            for (local_param_idx, &global_param_idx) in param_global_to_local.iter().enumerate() {
                let dfdp = parameter_derivatives[local_param_idx][point];
                if !dfdp.is_finite() {
                    return Err(OdrError::Numerical(format!(
                        "Parameter gradient evaluated to non-finite value at point {point} layer {layer_idx}"
                    )));
                }
                let j_val = -dfdp * inv_sqrt_s2;
                let j_idx = row_idx * global_parameter_count + global_param_idx;
                global_weighted_jacobian[j_idx] = j_val;
            }
        }
        
        layer_residuals.push(current_residuals);
        layer_fitted_values.push(fitted_values);
    }

    Ok(EvaluationState {
        chi_squared,
        layer_residuals,
        layer_fitted_values,
        flat_weighted_residuals: DVector::from_vec(flat_weighted_residuals),
        global_weighted_jacobian: DMatrix::from_row_slice(
            total_rows,
            global_parameter_count,
            &global_weighted_jacobian,
        ),
    })
}

/// Result of batch evaluation containing model values and derivatives.
pub struct BatchEvaluationResult {
    /// Model fitted values.
    pub fitted_values: Vec<f64>,
    /// Derivatives with respect to independent variables.
    pub independent_derivatives: Vec<Vec<f64>>,
    /// Derivatives with respect to parameters.
    pub parameter_derivatives: Vec<Vec<f64>>,
}

/// Evaluates the model and all its gradients in a single batched call using `eval_f64`.
///
/// This leverages SIMD vectorization and parallel evaluation for maximum performance.
///
/// # Errors
/// Returns `OdrError::Numerical` if evaluation fails or produces non-finite values.
#[allow(clippy::too_many_arguments, reason = "All parameters needed for batch evaluation")]
fn evaluate_model_and_gradients_batch(
    model_expr: &Expr,
    independent_gradient_exprs: &[Expr],
    parameter_gradient_exprs: &[Expr],
    independent_names: &[String],
    parameter_names: &[String],
    columns: &[&[f64]],
    _point_count: usize,
    layer_idx: usize,
) -> OdrResult<BatchEvaluationResult> {
    let total_exprs = 1 + independent_gradient_exprs.len() + parameter_gradient_exprs.len();
    
    // Build expression list: [model, df/dx1, df/dx2, ..., df/dp1, df/dp2, ...]
    let mut exprs: Vec<&Expr> = Vec::with_capacity(total_exprs);
    exprs.push(model_expr);
    for expr in independent_gradient_exprs {
        exprs.push(expr);
    }
    for expr in parameter_gradient_exprs {
        exprs.push(expr);
    }
    
    // Build variable names for each expression (all use the same variable order)
    let mut all_var_names: Vec<&str> = Vec::with_capacity(independent_names.len() + parameter_names.len());
    for name in independent_names {
        all_var_names.push(name.as_str());
    }
    for name in parameter_names {
        all_var_names.push(name.as_str());
    }
    
    // Each expression uses the same variable order
    let var_names: Vec<&[&str]> = exprs.iter().map(|_| &all_var_names[..]).collect();
    
    // Build data: each expression gets the same columnar data
    // data[expr_idx][var_idx][point_idx]
    let data: Vec<&[&[f64]]> = exprs.iter().map(|_| columns).collect();
    
    // Call eval_f64 for SIMD+parallel batch evaluation
    let results = eval_f64(&exprs, &var_names, &data)
        .map_err(|error| OdrError::Numerical(format!(
            "eval_f64 failed for layer {layer_idx}: {error:?}"
        )))?;
    
    // Validate and split results
    let mut offset = 0;
    
    // Model values
    let fitted_values = validate_evaluation_output(
        results[offset].clone(),
        &format!("model evaluator layer {layer_idx}"),
    )?;
    offset += 1;
    
    // Independent derivatives
    let mut independent_derivatives = Vec::with_capacity(independent_gradient_exprs.len());
    for (idx, _) in independent_gradient_exprs.iter().enumerate() {
        let deriv = validate_evaluation_output(
            results[offset].clone(),
            &format!("independent gradient evaluator {idx} layer {layer_idx}"),
        )?;
        independent_derivatives.push(deriv);
        offset += 1;
    }
    
    // Parameter derivatives
    let mut parameter_derivatives = Vec::with_capacity(parameter_gradient_exprs.len());
    for (idx, _) in parameter_gradient_exprs.iter().enumerate() {
        let deriv = validate_evaluation_output(
            results[offset].clone(),
            &format!("parameter gradient evaluator {idx} layer {layer_idx}"),
        )?;
        parameter_derivatives.push(deriv);
        offset += 1;
    }
    
    Ok(BatchEvaluationResult {
        fitted_values,
        independent_derivatives,
        parameter_derivatives,
    })
}

/// Evaluates the model in batch mode or scalar fallback.
///
/// # Errors
/// Returns `OdrError::Numerical` if evaluation fails or produces non-finite values.
pub fn evaluate_compiled_batch_or_scalar(
    evaluator: &CompiledEvaluator,
    columns: &[&[f64]],
    point_count: usize,
    evaluator_label: &str,
) -> OdrResult<Vec<f64>> {
    if let Ok(output) = evaluator.eval_batch_parallel(columns) {
        return validate_evaluation_output(output, evaluator_label);
    }

    let mut output = vec![0.0; point_count];

    let batch_result = evaluator.eval_batch(columns, &mut output, None);

    if batch_result.is_err() {
        let mut args = vec![0.0; columns.len()];
        for point in 0..point_count {
            for (arg_idx, column) in columns.iter().enumerate() {
                args[arg_idx] = column[point];
            }
            output[point] = evaluator.evaluate(&args);
        }
    }

    validate_evaluation_output(output, evaluator_label)
}

fn validate_evaluation_output(output: Vec<f64>, evaluator_label: &str) -> OdrResult<Vec<f64>> {
    for (idx, value) in output.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Numerical(format!(
                "{evaluator_label} produced non-finite output at point {idx}"
            )));
        }
    }

    Ok(output)
}

fn mat_vec(matrix: &[Vec<f64>], vector: &[f64]) -> Vec<f64> {
    let mut output = vec![0.0; matrix.len()];
    for row in 0..matrix.len() {
        let mut sum = 0.0;
        for col in 0..vector.len() {
            sum += matrix[row][col] * vector[col];
        }
        output[row] = sum;
    }
    output
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right.iter()).map(|(a, b)| a * b).sum()
}

#[must_use] 
/// Constructs the normal equations (`AtA` and `Atb`) from the Jacobian and residuals.
pub fn build_normal_equations(state: &EvaluationState) -> (DMatrix<f64>, DVector<f64>) {
    let j_t = state.global_weighted_jacobian.transpose();
    let normal = &j_t * &state.global_weighted_jacobian;
    let gradient = &j_t * &state.flat_weighted_residuals;
    (normal, gradient)
}

fn solve_linear_system(matrix: DMatrix<f64>, rhs: &DVector<f64>) -> OdrResult<DVector<f64>> {
    let svd = matrix.svd(true, true);
    svd.solve(rhs, MATRIX_SINGULAR_EPS)
        .map_err(|error| OdrError::Numerical(format!("SVD solve failed: {error}")))
}

/// Inverts the information matrix to compute covariance.
///
/// # Errors
/// Returns `OdrError::Numerical` if the matrix is singular or inversion fails.
pub fn invert_information_matrix(matrix: DMatrix<f64>) -> OdrResult<DMatrix<f64>> {
    let svd = matrix.svd(true, true);
    svd.pseudo_inverse(MATRIX_SINGULAR_EPS)
        .map_err(|error| OdrError::Numerical(format!("Pseudo-inverse failed: {error}")))
}
