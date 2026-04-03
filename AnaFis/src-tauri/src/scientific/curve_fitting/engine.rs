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
/// Maximum iterations for per-point independent-variable correction.
pub const INNER_CORRECTION_MAX_ITERS: usize = 30;
/// Convergence tolerance for per-point correction step norm.
pub const INNER_CORRECTION_TOLERANCE: f64 = 1e-12;
/// Damping used in the per-point inner correction solve.
pub const INNER_CORRECTION_DAMPING: f64 = 1e-6;

/// SVD-based numerical diagnostics for a matrix.
#[derive(Debug, Clone, Copy)]
pub struct MatrixDiagnostics {
    /// Effective numerical rank based on `MATRIX_SINGULAR_EPS * sigma_max`.
    pub effective_rank: usize,
    /// Matrix condition number estimate (`sigma_max / sigma_min_nonzero`).
    pub condition_number: f64,
}

/// Why the ODR loop stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OdrTerminationReason {
    /// Converged because scaled gradient norm is below tolerance.
    ScaledGradient,
    /// Converged because scaled step norm is below tolerance.
    ScaledStep,
    /// Converged because objective improvement is below tolerance.
    Improvement,
    /// Stopped because iterations stagnated without improving objective.
    Stagnated,
    /// Stopped because the normal system is numerically singular.
    Singular,
    /// Stopped because damping saturated without finding productive steps.
    DampingSaturated,
    /// Stopped after exhausting `max_iterations`.
    MaxIterations,
}

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
    /// Parsed expressions for second derivatives with respect to independent variables (row-major Hessian).
    pub independent_hessian_exprs: Vec<Expr>, // d2 f / d x_row d x_col
    /// Compiled evaluator for the main formula (fallback).
    pub model_evaluator: CompiledEvaluator,
    /// Compiled evaluators for the partial derivatives with respect to each parameter (fallback).
    pub parameter_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d p_j
    /// Compiled evaluators for the partial derivatives with respect to each independent variable (fallback).
    pub independent_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d x_k
    /// Compiled evaluators for second derivatives with respect to independent variables (row-major Hessian).
    pub independent_hessian_evaluators: Vec<CompiledEvaluator>, // d2 f / d x_row d x_col
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
    /// Current profiled objective value used by the optimizer.
    pub chi_squared: f64,
    /// Observation-only weighted chi-squared used for reduced-chi-squared reporting.
    pub chi_squared_observation: f64,
    /// Raw residuals (observed - predicted) for each layer: [`layer_idx`][point_idx].
    pub layer_residuals: Vec<Vec<f64>>,
    /// Values predicted by the models at the current state: [`layer_idx`][point_idx].
    pub layer_fitted_values: Vec<Vec<f64>>,
    /// Flattened residuals weighted by the inverse covariance matrix.
    pub flat_weighted_residuals: DVector<f64>,
    /// Global Jacobian matrix weighted by the inverse covariance matrix.
    pub global_weighted_jacobian: DMatrix<f64>,
    /// Number of unique independent-variable correction dimensions (K in N*K).
    pub correction_variable_count: usize,
    /// Number of per-point inner correction solves that did not meet convergence tolerance.
    pub inner_correction_nonconverged_points: usize,
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
pub static MODEL_CACHE: std::sync::LazyLock<Mutex<ModelCache>> =
    std::sync::LazyLock::new(|| Mutex::new(ModelCache::default()));

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
    if identifier.trim().is_empty() {
        return Err(OdrError::Validation(format!(
            "{label} names cannot be empty or only whitespace"
        )));
    }

    let mut chars = identifier.chars();
    let first = chars
        .next()
        .ok_or_else(|| OdrError::Validation(format!("{label} names cannot be empty")))?;
    if !(first.is_alphabetic() || first == '_') {
        return Err(OdrError::Validation(format!(
            "Invalid {label} '{identifier}': first character must be a letter or '_'"
        )));
    }

    if !chars.all(|c| c.is_alphanumeric() || c == '_') {
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
    let normalized_independent = independent_names
        .iter()
        .map(|name| name.trim().to_lowercase())
        .collect::<Vec<_>>()
        .join(",");
    let normalized_parameters = parameter_names
        .iter()
        .map(|name| name.trim().to_lowercase())
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{}|{}|x:{}|p:{}",
        formula.trim().to_lowercase(),
        dependent_name.trim().to_lowercase(),
        normalized_independent,
        normalized_parameters
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
    let normalized_dependent = dependent_name.trim().to_lowercase();
    let normalized_independent: Vec<String> = independent_names
        .iter()
        .map(|name| name.trim().to_lowercase())
        .collect();
    let normalized_parameters: Vec<String> = parameter_names
        .iter()
        .map(|name| name.trim().to_lowercase())
        .collect();

    let key = build_model_cache_key(
        model_formula,
        &normalized_dependent,
        &normalized_independent,
        &normalized_parameters,
    );

    {
        let mut cache = MODEL_CACHE.lock().map_err(|_| OdrError::CachePoisoned)?;
        if let Some(model) = cache.get(&key) {
            return Ok(model);
        }
    }

    let compiled = Arc::new(compile_model_inner(
        model_formula,
        &normalized_dependent,
        &normalized_independent,
        &normalized_parameters,
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

    let hessian_capacity = independent_gradient_exprs.len() * independent_gradient_exprs.len();
    let mut independent_hessian_evaluators = Vec::with_capacity(hessian_capacity);
    let mut independent_hessian_exprs = Vec::with_capacity(hessian_capacity);
    for row_gradient in &independent_gradient_exprs {
        let row_second_derivatives =
            gradient(row_gradient, &independent_symbol_refs).map_err(|error| {
                OdrError::Compile(format!("independent Hessian gradients: {error:?}"))
            })?;

        for second_derivative_expr in row_second_derivatives {
            independent_hessian_evaluators.push(
                CompiledEvaluator::compile(&second_derivative_expr, &evaluator_order, None)
                    .map_err(|error| {
                        OdrError::Compile(format!("independent Hessian evaluator: {error:?}"))
                    })?,
            );
            independent_hessian_exprs.push(second_derivative_expr);
        }
    }

    Ok(CompiledModel {
        formula,
        dependent_name: dependent_name.to_lowercase(),
        parameter_names: parameter_names.to_vec(),
        independent_names: independent_names.to_vec(),
        model_expr: expr,
        parameter_gradient_exprs,
        independent_gradient_exprs,
        independent_hessian_exprs,
        model_evaluator,
        parameter_gradient_evaluators,
        independent_gradient_evaluators,
        independent_hessian_evaluators,
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

    let mut process_variable = |var: &super::types::VariableInput,
                                is_dependent: bool|
     -> OdrResult<()> {
        if var.values.len() != point_count {
            return Err(OdrError::Validation(format!(
                "Variable '{}' length mismatch: expected {}, got {}",
                var.name,
                point_count,
                var.values.len()
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
            for (idx, val) in var.values.iter().enumerate() {
                if *val < 0.0 {
                    return Err(OdrError::Validation(format!(
                        "Poisson weighting requires non-negative counts for '{}' at index {idx}",
                        var.name
                    )));
                }

                let variance = (*val).max(MIN_VARIANCE);
                if *val <= 0.0 {
                    had_uncertainty_clamp = true;
                }
                sigma.push(variance.sqrt());
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
#[allow(
    clippy::too_many_lines,
    reason = "ODR loop keeps all acceptance/rejection and termination logic explicit for numerical safety"
)]
pub fn solve_odr(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    mut parameters: Vec<f64>,
    global_parameter_names: &[String],
    max_iterations: usize,
    tolerance: f64,
    initial_damping: f64,
) -> OdrResult<(Vec<f64>, EvaluationState, usize, OdrTerminationReason)> {
    let mut damping = initial_damping;
    let mut nu = 2.0;
    let mut current = evaluate_model(models, data, &parameters, global_parameter_names)?;
    let mut iterations = 0;
    let mut termination_reason = OdrTerminationReason::MaxIterations;
    let mut consecutive_rejections = 0usize;
    let parameter_count = parameters.len();
    let mut parameter_scales = vec![1.0f64; parameter_count];

    for iteration in 0..max_iterations {
        iterations = iteration + 1;

        let (normal_matrix, gradient_vector) = build_normal_equations(&current);
        let diagnostics = diagnose_matrix(&normal_matrix);
        if diagnostics.effective_rank == 0 {
            termination_reason = OdrTerminationReason::Singular;
            break;
        }
        for diagonal in 0..parameter_count {
            parameter_scales[diagonal] =
                parameter_scales[diagonal].max(normal_matrix[(diagonal, diagonal)].abs().sqrt());
        }

        let effective_scales: Vec<f64> = parameter_scales
            .iter()
            .map(|&scale| {
                if scale.is_finite() && scale >= 1e-30 {
                    scale
                } else {
                    1.0
                }
            })
            .collect();

        let scaled_gradient_norm = (0..parameter_count)
            .map(|i| (gradient_vector[i] / effective_scales[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        if scaled_gradient_norm <= tolerance {
            termination_reason = OdrTerminationReason::ScaledGradient;
            break;
        }

        let rhs = -&gradient_vector;

        let mut scaled_matrix = normal_matrix.clone();
        for row in 0..parameter_count {
            let row_scale = effective_scales[row];
            for column in 0..parameter_count {
                scaled_matrix[(row, column)] /= row_scale * effective_scales[column];
            }
        }

        for diagonal in 0..parameter_count {
            scaled_matrix[(diagonal, diagonal)] += damping;
        }

        let scaled_rhs = DVector::from_fn(parameter_count, |i, _| rhs[i] / effective_scales[i]);
        let Ok(scaled_delta) = solve_linear_system(scaled_matrix, &scaled_rhs) else {
            consecutive_rejections += 1;
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
            if damping >= MAX_DAMPING {
                termination_reason = OdrTerminationReason::DampingSaturated;
                break;
            }
            if consecutive_rejections >= 25 {
                termination_reason = OdrTerminationReason::Stagnated;
                break;
            }
            continue;
        };

        let delta = DVector::from_fn(parameter_count, |i, _| {
            scaled_delta[i] / effective_scales[i]
        });

        let scaled_delta_norm = (0..parameter_count)
            .map(|i| scaled_delta[i].powi(2))
            .sum::<f64>()
            .sqrt();
        let scaled_parameter_norm = (0..parameter_count)
            .map(|i| (parameters[i] * effective_scales[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        if scaled_delta_norm <= tolerance * (scaled_parameter_norm + tolerance) {
            termination_reason = OdrTerminationReason::ScaledStep;
            break;
        }

        let trial_parameters: Vec<f64> = parameters
            .iter()
            .zip(delta.iter())
            .map(|(parameter, step)| parameter + step)
            .collect();

        if trial_parameters.iter().any(|value| !value.is_finite()) {
            consecutive_rejections += 1;
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
            if damping >= MAX_DAMPING {
                termination_reason = OdrTerminationReason::DampingSaturated;
                break;
            }
            if consecutive_rejections >= 25 {
                termination_reason = OdrTerminationReason::Stagnated;
                break;
            }
            continue;
        }

        let trial = evaluate_model(models, data, &trial_parameters, global_parameter_names)?;
        let actual_reduction = current.chi_squared - trial.chi_squared;

        // Canonical LM/GN model reduction for chi_squared = r^T W r:
        // m(0)-m(delta) = -(2 * g^T delta + delta^T H delta)
        // where g = J^T W r and H = J^T W J.
        let h_delta = &normal_matrix * &delta;
        let mut predicted_reduction =
            -2.0f64.mul_add(gradient_vector.dot(&delta), -delta.dot(&h_delta));
        if !predicted_reduction.is_finite() || predicted_reduction <= MIN_VARIANCE {
            predicted_reduction = MIN_VARIANCE;
        }

        let rho = actual_reduction / predicted_reduction;

        if actual_reduction > 0.0 && rho.is_finite() && rho > 0.25 {
            let improvement = actual_reduction.abs();
            parameters = trial_parameters;
            current = trial;
            consecutive_rejections = 0;

            if rho > 0.75 {
                damping = (damping * (1.0 / 3.0)).max(MIN_DAMPING);
            } else if rho < 0.25 {
                damping = (damping * 2.0).min(MAX_DAMPING);
            }
            nu = 2.0;

            if improvement <= tolerance {
                termination_reason = OdrTerminationReason::Improvement;
                break;
            }
        } else {
            consecutive_rejections += 1;
            damping = (damping * nu).min(MAX_DAMPING);
            nu = (nu * 2.0).min(1e12);
            if damping >= MAX_DAMPING {
                termination_reason = OdrTerminationReason::DampingSaturated;
                break;
            }
            if consecutive_rejections >= 25 {
                termination_reason = OdrTerminationReason::Stagnated;
                break;
            }
        }
    }

    Ok((parameters, current, iterations, termination_reason))
}

/// Evaluates the multi-layer model at the current global parameters.
///
/// # Errors
/// Returns `OdrError::Numerical` if models or gradients evaluate to non-finite values.
#[allow(
    clippy::too_many_lines,
    reason = "Multi-layer ODR evaluation requires comprehensive logic"
)]
pub fn evaluate_model(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    global_parameters: &[f64],
    global_parameter_names: &[String],
) -> OdrResult<EvaluationState> {
    let point_count = data.point_count;
    let global_parameter_count = global_parameters.len();

    let mut chi_squared = 0.0;
    let mut chi_squared_observation = 0.0;

    let mut layer_residuals: Vec<Vec<f64>> = (0..models.len())
        .map(|_| Vec::with_capacity(point_count))
        .collect();
    let mut layer_fitted_values: Vec<Vec<f64>> = (0..models.len())
        .map(|_| Vec::with_capacity(point_count))
        .collect();
    let mut inner_correction_nonconverged_points = 0usize;

    let mut flat_weighted_residuals: Vec<f64> = Vec::new();
    let mut global_weighted_jacobian: Vec<f64> = Vec::new();

    let mut dep_var_indices = Vec::with_capacity(models.len());
    let mut indep_var_indices = Vec::with_capacity(models.len());
    let mut local_parameters_per_layer = Vec::with_capacity(models.len());
    let mut global_parameter_indices_per_layer = Vec::with_capacity(models.len());
    let mut variable_to_correction_index: Vec<Option<usize>> =
        vec![None; data.variable_names.len()];
    let mut correction_variable_indices: Vec<usize> = Vec::new();
    let mut layer_has_correctable_independent = Vec::with_capacity(models.len());

    for model in models {
        let dep_var_idx = data
            .variable_names
            .iter()
            .position(|name| name == &model.dependent_name)
            .ok_or_else(|| {
                OdrError::Validation(format!(
                    "Dependent variable {} not found in data",
                    model.dependent_name
                ))
            })?;
        dep_var_indices.push(dep_var_idx);

        let mut indep_indices = Vec::with_capacity(model.independent_names.len());
        let mut has_correctable_independent = false;
        for name in &model.independent_names {
            let idx = data
                .variable_names
                .iter()
                .position(|n| n == name)
                .ok_or_else(|| {
                    OdrError::Validation(format!("Independent variable {name} not found in data"))
                })?;
            indep_indices.push(idx);
            let has_uncertainty = (0..point_count)
                .any(|point| data.point_covariances[point][idx][idx] > MIN_VARIANCE * 10.0);
            if has_uncertainty {
                has_correctable_independent = true;
            }
            if has_uncertainty && variable_to_correction_index[idx].is_none() {
                let next = correction_variable_indices.len();
                variable_to_correction_index[idx] = Some(next);
                correction_variable_indices.push(idx);
            }
        }
        indep_var_indices.push(indep_indices);
        layer_has_correctable_independent.push(has_correctable_independent);

        let mut local_parameters = Vec::with_capacity(model.parameter_names.len());
        let mut param_global_indices = Vec::with_capacity(model.parameter_names.len());

        for local_name in &model.parameter_names {
            let global_idx = global_parameter_names
                .iter()
                .position(|name| name == local_name)
                .ok_or_else(|| {
                    OdrError::Validation(format!(
                        "Parameter {local_name} not found in global parameters"
                    ))
                })?;
            local_parameters.push(global_parameters[global_idx]);
            param_global_indices.push(global_idx);
        }
        local_parameters_per_layer.push(local_parameters);
        global_parameter_indices_per_layer.push(param_global_indices);
    }

    for point in 0..point_count {
        let point_has_active_corrections = correction_variable_indices
            .iter()
            .any(|&var_idx| data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0);

        let point_correction = if point_has_active_corrections {
            solve_coupled_point_correction(
                models,
                data,
                point,
                &dep_var_indices,
                &indep_var_indices,
                &local_parameters_per_layer,
                &variable_to_correction_index,
                &layer_has_correctable_independent,
                correction_variable_indices.len(),
            )?
        } else {
            PointCorrectionResult {
                corrections: vec![0.0; correction_variable_indices.len()],
                converged: true,
            }
        };
        if !point_correction.converged {
            inner_correction_nonconverged_points += 1;
        }

        let mut point_fitted_values = Vec::with_capacity(models.len());
        let mut point_residuals = Vec::with_capacity(models.len());
        let mut point_args_per_layer: Vec<Vec<f64>> = Vec::with_capacity(models.len());
        let mut point_parameter_gradients: Vec<Vec<f64>> = Vec::with_capacity(models.len());
        let mut point_independent_gradients: Vec<Vec<f64>> = Vec::with_capacity(models.len());

        for (layer_idx, model) in models.iter().enumerate() {
            let layer_indep_indices = &indep_var_indices[layer_idx];
            let local_parameters = &local_parameters_per_layer[layer_idx];

            let mut args = Vec::with_capacity(layer_indep_indices.len() + local_parameters.len());
            for &var_idx in layer_indep_indices {
                let corrected = if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    data.variable_values[var_idx][point] + point_correction.corrections[corr_idx]
                } else {
                    data.variable_values[var_idx][point]
                };
                args.push(corrected);
            }
            args.extend(local_parameters.iter().copied());

            let fitted = model.model_evaluator.evaluate(&args);
            if !fitted.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Model evaluated to non-finite value at data point {point} in layer {layer_idx}"
                )));
            }

            let dep_var_idx = dep_var_indices[layer_idx];
            let residual = data.variable_values[dep_var_idx][point] - fitted;

            let mut parameter_gradients = Vec::with_capacity(model.parameter_names.len());
            for evaluator in &model.parameter_gradient_evaluators {
                let value = evaluator.evaluate(&args);
                if !value.is_finite() {
                    return Err(OdrError::Numerical(format!(
                        "Parameter gradient evaluated to non-finite value at point {point} layer {layer_idx}"
                    )));
                }
                parameter_gradients.push(value);
            }

            let mut independent_gradients = Vec::with_capacity(model.independent_names.len());
            for evaluator in &model.independent_gradient_evaluators {
                let value = evaluator.evaluate(&args);
                if !value.is_finite() {
                    return Err(OdrError::Numerical(format!(
                        "Independent gradient evaluated to non-finite value at point {point} layer {layer_idx}"
                    )));
                }
                independent_gradients.push(value);
            }

            point_fitted_values.push(fitted);
            point_residuals.push(residual);
            point_args_per_layer.push(args);
            point_parameter_gradients.push(parameter_gradients);
            point_independent_gradients.push(independent_gradients);
        }

        let mut d_corrections_d_beta =
            DMatrix::<f64>::zeros(correction_variable_indices.len(), global_parameter_count);
        if !correction_variable_indices.is_empty() {
            let mut h_cc = DMatrix::<f64>::zeros(
                correction_variable_indices.len(),
                correction_variable_indices.len(),
            );
            let mut h_cbeta =
                DMatrix::<f64>::zeros(correction_variable_indices.len(), global_parameter_count);

            for layer_idx in 0..models.len() {
                if !layer_has_correctable_independent[layer_idx] {
                    continue;
                }

                let dep_var_idx = dep_var_indices[layer_idx];
                let layer_indep_indices = &indep_var_indices[layer_idx];
                let layer_has_point_correction = layer_indep_indices.iter().any(|&var_idx| {
                    variable_to_correction_index[var_idx].is_some()
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                });
                if !layer_has_point_correction {
                    continue;
                }
                let local_independent_count = layer_indep_indices.len();
                let block_dim = local_independent_count + 1;

                let sigma_joint = extract_joint_covariance(
                    &data.point_covariances[point],
                    layer_indep_indices,
                    dep_var_idx,
                )?;
                let w_joint = invert_small_psd(&sigma_joint)?;

                let mut j_corrections =
                    DMatrix::<f64>::zeros(block_dim, correction_variable_indices.len());
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                    {
                        j_corrections[(local_idx, corr_idx)] = -1.0;
                        j_corrections[(local_independent_count, corr_idx)] =
                            -point_independent_gradients[layer_idx][local_idx];
                    }
                }

                let mut parameter_block = DMatrix::<f64>::zeros(block_dim, global_parameter_count);
                for (local_param_idx, &global_param_idx) in global_parameter_indices_per_layer
                    [layer_idx]
                    .iter()
                    .enumerate()
                {
                    parameter_block[(local_independent_count, global_param_idx)] =
                        -point_parameter_gradients[layer_idx][local_param_idx];
                }

                h_cc += j_corrections.transpose() * &w_joint * &j_corrections;
                h_cbeta += j_corrections.transpose() * &w_joint * parameter_block;

                let dependent_weight = w_joint[(local_independent_count, local_independent_count)];
                let second_order_coefficient = dependent_weight * point_residuals[layer_idx];
                if second_order_coefficient.abs() > 0.0 {
                    let local_hessian = evaluate_model_hessian_wrt_independents(
                        &models[layer_idx],
                        &point_args_per_layer[layer_idx],
                        local_independent_count,
                    )?;
                    for local_row in 0..local_independent_count {
                        let Some(global_row) =
                            variable_to_correction_index[layer_indep_indices[local_row]]
                        else {
                            continue;
                        };
                        if data.point_covariances[point][layer_indep_indices[local_row]]
                            [layer_indep_indices[local_row]]
                            <= MIN_VARIANCE * 10.0
                        {
                            continue;
                        }
                        for local_col in 0..local_independent_count {
                            let Some(global_col) =
                                variable_to_correction_index[layer_indep_indices[local_col]]
                            else {
                                continue;
                            };
                            if data.point_covariances[point][layer_indep_indices[local_col]]
                                [layer_indep_indices[local_col]]
                                <= MIN_VARIANCE * 10.0
                            {
                                continue;
                            }
                            h_cc[(global_row, global_col)] -=
                                second_order_coefficient * local_hessian[(local_row, local_col)];
                        }
                    }
                }
            }

            for col in 0..global_parameter_count {
                let rhs = -h_cbeta.column(col).into_owned();
                let solved_column = if let Ok(solution) = solve_linear_system(h_cc.clone(), &rhs) {
                    solution
                } else {
                    let mut regularized_h_cc = h_cc.clone();
                    let max_diag = (0..correction_variable_indices.len())
                        .map(|i| regularized_h_cc[(i, i)].abs())
                        .fold(0.0, f64::max)
                        .max(1.0);
                    let sensitivity_damping = INNER_CORRECTION_DAMPING * max_diag;
                    for diagonal in 0..correction_variable_indices.len() {
                        regularized_h_cc[(diagonal, diagonal)] += sensitivity_damping;
                    }
                    solve_linear_system(regularized_h_cc, &rhs)?
                };
                for row in 0..correction_variable_indices.len() {
                    d_corrections_d_beta[(row, col)] = solved_column[row];
                }
            }
        }

        for layer_idx in 0..models.len() {
            let dep_var_idx = dep_var_indices[layer_idx];
            let layer_indep_indices = &indep_var_indices[layer_idx];
            let param_global_indices = &global_parameter_indices_per_layer[layer_idx];
            let local_independent_count = layer_indep_indices.len();
            let block_dim = local_independent_count + 1;

            let fitted = point_fitted_values[layer_idx];
            let residual = point_residuals[layer_idx];
            layer_residuals[layer_idx].push(residual);
            layer_fitted_values[layer_idx].push(fitted);

            let parameter_gradients = &point_parameter_gradients[layer_idx];
            let independent_gradients = &point_independent_gradients[layer_idx];
            let sigma_y2 =
                data.point_covariances[point][dep_var_idx][dep_var_idx].max(MIN_VARIANCE);
            chi_squared_observation += residual * residual / sigma_y2;

            let layer_has_point_correction = layer_has_correctable_independent[layer_idx]
                && layer_indep_indices.iter().any(|&var_idx| {
                    variable_to_correction_index[var_idx].is_some()
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                });

            if layer_has_point_correction {
                let sigma_joint = extract_joint_covariance(
                    &data.point_covariances[point],
                    layer_indep_indices,
                    dep_var_idx,
                )?;
                let w_joint = invert_small_psd(&sigma_joint)?;

                let mut joint_residual = DVector::<f64>::zeros(block_dim);
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                    {
                        joint_residual[local_idx] = -point_correction.corrections[corr_idx];
                    }
                }
                joint_residual[local_independent_count] = residual;

                let mut parameter_block = DMatrix::<f64>::zeros(block_dim, global_parameter_count);
                for (local_param_idx, &global_param_idx) in param_global_indices.iter().enumerate()
                {
                    parameter_block[(local_independent_count, global_param_idx)] =
                        -parameter_gradients[local_param_idx];
                }

                let mut j_corrections =
                    DMatrix::<f64>::zeros(block_dim, correction_variable_indices.len());
                for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                    if let Some(corr_idx) = variable_to_correction_index[var_idx]
                        && data.point_covariances[point][var_idx][var_idx] > MIN_VARIANCE * 10.0
                    {
                        j_corrections[(local_idx, corr_idx)] = -1.0;
                        j_corrections[(local_independent_count, corr_idx)] =
                            -independent_gradients[local_idx];
                    }
                }

                parameter_block += &j_corrections * &d_corrections_d_beta;

                let whitening = sqrt_psd_matrix(&w_joint)?;
                let weighted_residual = &whitening * joint_residual;
                let weighted_parameter_jacobian = &whitening * parameter_block;

                chi_squared += weighted_residual.dot(&weighted_residual);
                for row in 0..block_dim {
                    flat_weighted_residuals.push(weighted_residual[row]);
                    for col in 0..global_parameter_count {
                        global_weighted_jacobian.push(weighted_parameter_jacobian[(row, col)]);
                    }
                }
            } else {
                let weight = 1.0 / sigma_y2.sqrt();
                let weighted_residual = residual * weight;
                chi_squared += weighted_residual * weighted_residual;
                flat_weighted_residuals.push(weighted_residual);
                let mut jacobian_row = vec![0.0; global_parameter_count];
                for (local_pos, &global_idx) in param_global_indices.iter().enumerate() {
                    jacobian_row[global_idx] = -parameter_gradients[local_pos] * weight;
                }
                global_weighted_jacobian.extend_from_slice(&jacobian_row);
            }
        }
    }

    let total_rows = flat_weighted_residuals.len();

    if flat_weighted_residuals.is_empty() || global_weighted_jacobian.is_empty() {
        return Err(OdrError::Numerical(
            "Internal weighted system is empty after model evaluation".to_string(),
        ));
    }

    Ok(EvaluationState {
        chi_squared,
        chi_squared_observation,
        layer_residuals,
        layer_fitted_values,
        flat_weighted_residuals: DVector::from_vec(flat_weighted_residuals),
        global_weighted_jacobian: DMatrix::from_row_slice(
            total_rows,
            global_parameter_count,
            &global_weighted_jacobian,
        ),
        correction_variable_count: correction_variable_indices.len(),
        inner_correction_nonconverged_points,
    })
}

fn extract_joint_covariance(
    covariance: &[Vec<f64>],
    independent_indices: &[usize],
    dependent_index: usize,
) -> OdrResult<Vec<Vec<f64>>> {
    let dim = independent_indices.len() + 1;
    let mut block = vec![vec![0.0; dim]; dim];
    for (row_local, &row_global) in independent_indices.iter().enumerate() {
        for (col_local, &col_global) in independent_indices.iter().enumerate() {
            block[row_local][col_local] = covariance[row_global][col_global];
        }
        block[row_local][row_local] = block[row_local][row_local].max(MIN_VARIANCE);
        block[row_local][dim - 1] = covariance[row_global][dependent_index];
        block[dim - 1][row_local] = covariance[dependent_index][row_global];
    }
    block[dim - 1][dim - 1] = covariance[dependent_index][dependent_index].max(MIN_VARIANCE);

    if is_positive_semidefinite(&block) {
        return Ok(block);
    }

    let mut regularized = block;
    let mut jitter = MIN_VARIANCE;
    for _ in 0..8 {
        #[allow(
            clippy::needless_range_loop,
            reason = "Diagonal indexing requires row == col"
        )]
        for diagonal in 0..dim {
            regularized[diagonal][diagonal] += jitter;
        }
        if is_positive_semidefinite(&regularized) {
            return Ok(regularized);
        }
        jitter *= 10.0;
    }

    Err(OdrError::Numerical(
        "Extracted joint covariance block is not PSD after regularization".to_string(),
    ))
}

struct PointCorrectionResult {
    corrections: Vec<f64>,
    converged: bool,
}

#[allow(
    clippy::too_many_lines,
    reason = "Coupled point correction keeps numerical update and diagnostics in one place"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "All parameters needed for per-point coupled correction solve"
)]
fn solve_coupled_point_correction(
    models: &[Arc<CompiledModel>],
    data: &PreparedData,
    point_idx: usize,
    dep_var_indices: &[usize],
    indep_var_indices: &[Vec<usize>],
    local_parameters_per_layer: &[Vec<f64>],
    variable_to_correction_index: &[Option<usize>],
    layer_has_correctable_independent: &[bool],
    correction_count: usize,
) -> OdrResult<PointCorrectionResult> {
    if correction_count == 0 {
        return Ok(PointCorrectionResult {
            corrections: Vec::new(),
            converged: true,
        });
    }

    let mut corrections = vec![0.0; correction_count];
    let mut converged = false;

    for _ in 0..INNER_CORRECTION_MAX_ITERS {
        let mut gradient = DVector::<f64>::zeros(correction_count);
        let mut hessian = DMatrix::<f64>::zeros(correction_count, correction_count);

        for (layer_idx, model) in models.iter().enumerate() {
            if !layer_has_correctable_independent[layer_idx] {
                continue;
            }
            let dep_var_idx = dep_var_indices[layer_idx];
            let layer_indep_indices = &indep_var_indices[layer_idx];
            let local_parameters = &local_parameters_per_layer[layer_idx];
            let local_independent_count = layer_indep_indices.len();
            let block_dim = local_independent_count + 1;

            let mut args = Vec::with_capacity(local_independent_count + local_parameters.len());
            for &var_idx in layer_indep_indices {
                let corrected_value = if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    data.variable_values[var_idx][point_idx] + corrections[corr_idx]
                } else {
                    data.variable_values[var_idx][point_idx]
                };
                args.push(corrected_value);
            }
            args.extend(local_parameters.iter().copied());

            let fitted = model.model_evaluator.evaluate(&args);
            if !fitted.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Model evaluated to non-finite value during coupled point correction at point {point_idx} layer {layer_idx}"
                )));
            }

            let mut gradient_x = Vec::with_capacity(local_independent_count);
            for evaluator in &model.independent_gradient_evaluators {
                let value = evaluator.evaluate(&args);
                if !value.is_finite() {
                    return Err(OdrError::Numerical(format!(
                        "Independent gradient evaluated to non-finite value during coupled point correction at point {point_idx} layer {layer_idx}"
                    )));
                }
                gradient_x.push(value);
            }

            let residual = data.variable_values[dep_var_idx][point_idx] - fitted;

            let mut joint_residual = DVector::<f64>::zeros(block_dim);
            for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    joint_residual[local_idx] = -corrections[corr_idx];
                }
            }
            joint_residual[local_independent_count] = residual;

            let sigma_joint = extract_joint_covariance(
                &data.point_covariances[point_idx],
                layer_indep_indices,
                dep_var_idx,
            )?;
            let weight_joint = invert_small_psd(&sigma_joint)?;

            let mut j_corrections = DMatrix::<f64>::zeros(block_dim, correction_count);
            for (local_idx, &var_idx) in layer_indep_indices.iter().enumerate() {
                if let Some(corr_idx) = variable_to_correction_index[var_idx]
                    && data.point_covariances[point_idx][var_idx][var_idx] > MIN_VARIANCE * 10.0
                {
                    j_corrections[(local_idx, corr_idx)] = -1.0;
                    j_corrections[(local_independent_count, corr_idx)] = -gradient_x[local_idx];
                }
            }

            let weighted_residual = &weight_joint * &joint_residual;
            gradient += j_corrections.transpose() * &weighted_residual;
            hessian += j_corrections.transpose() * &weight_joint * &j_corrections;

            // Add the second-order term for the dependent residual row to improve inner Newton fidelity.
            let dependent_weight = weight_joint[(local_independent_count, local_independent_count)];
            let second_order_coefficient = dependent_weight * residual;
            if second_order_coefficient.abs() > 0.0 {
                let local_hessian =
                    evaluate_model_hessian_wrt_independents(model, &args, local_independent_count)?;
                for local_row in 0..local_independent_count {
                    let Some(global_row) =
                        variable_to_correction_index[layer_indep_indices[local_row]]
                    else {
                        continue;
                    };
                    for local_col in 0..local_independent_count {
                        let Some(global_col) =
                            variable_to_correction_index[layer_indep_indices[local_col]]
                        else {
                            continue;
                        };
                        // r_dep = y - f, so d2(r_dep)/dc2 = -d2f/dx2.
                        hessian[(global_row, global_col)] -=
                            second_order_coefficient * local_hessian[(local_row, local_col)];
                    }
                }
            }
        }

        let max_diag = (0..correction_count)
            .map(|diagonal| hessian[(diagonal, diagonal)].abs())
            .fold(0.0, f64::max)
            .max(1.0);
        let base_damping = INNER_CORRECTION_DAMPING * max_diag;
        let min_eigenvalue = hessian
            .clone()
            .symmetric_eigen()
            .eigenvalues
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let adaptive_damping = if min_eigenvalue.is_finite() && min_eigenvalue <= 0.0 {
            base_damping.max(-min_eigenvalue + base_damping)
        } else {
            base_damping
        };
        for diagonal in 0..correction_count {
            hessian[(diagonal, diagonal)] += adaptive_damping;
        }

        let rhs = -gradient;
        let solved = solve_linear_system(hessian, &rhs)?;
        let next_corrections: Vec<f64> = corrections
            .iter()
            .zip(solved.iter())
            .map(|(base, step)| base + step)
            .collect();
        if next_corrections.iter().any(|value| !value.is_finite()) {
            return Err(OdrError::Numerical(format!(
                "Coupled point correction produced non-finite values at point {point_idx}"
            )));
        }

        let delta_step_norm = next_corrections
            .iter()
            .zip(corrections.iter())
            .map(|(next, prev)| (next - prev).powi(2))
            .sum::<f64>()
            .sqrt();
        let correction_norm = next_corrections
            .iter()
            .map(|value| value.powi(2))
            .sum::<f64>()
            .sqrt();
        corrections = next_corrections;

        if delta_step_norm
            <= INNER_CORRECTION_TOLERANCE.mul_add(correction_norm, INNER_CORRECTION_TOLERANCE)
        {
            converged = true;
            break;
        }
    }

    Ok(PointCorrectionResult {
        corrections,
        converged,
    })
}

fn invert_small_psd(covariance: &[Vec<f64>]) -> OdrResult<DMatrix<f64>> {
    let dim = covariance.len();
    let mut flat = Vec::with_capacity(dim * dim);
    for row in covariance {
        flat.extend(row.iter().copied());
    }
    let matrix = DMatrix::from_row_slice(dim, dim, &flat);
    let svd = matrix.svd(true, true);
    svd.pseudo_inverse(MATRIX_SINGULAR_EPS)
        .map_err(|error| OdrError::Numerical(format!("Point covariance inversion failed: {error}")))
}

fn sqrt_psd_matrix(matrix: &DMatrix<f64>) -> OdrResult<DMatrix<f64>> {
    let eigen = matrix.clone().symmetric_eigen();
    let dim = matrix.nrows();
    let mut sqrt_diag = DMatrix::<f64>::zeros(dim, dim);
    for idx in 0..dim {
        let lambda = eigen.eigenvalues[idx];
        if !lambda.is_finite() {
            return Err(OdrError::Numerical(
                "Non-finite eigenvalue found while building weighted residual blocks".to_string(),
            ));
        }
        sqrt_diag[(idx, idx)] = lambda.max(0.0).sqrt();
    }
    Ok(&eigen.eigenvectors * sqrt_diag * eigen.eigenvectors.transpose())
}

fn evaluate_model_hessian_wrt_independents(
    model: &CompiledModel,
    args: &[f64],
    independent_count: usize,
) -> OdrResult<DMatrix<f64>> {
    let mut hessian = DMatrix::<f64>::zeros(independent_count, independent_count);
    if independent_count == 0 {
        return Ok(hessian);
    }

    for row in 0..independent_count {
        for col in 0..independent_count {
            let idx = row * independent_count + col;
            let value = model.independent_hessian_evaluators[idx].evaluate(args);
            if !value.is_finite() {
                return Err(OdrError::Numerical(
                    "Non-finite value while evaluating symbolic independent-variable Hessian"
                        .to_string(),
                ));
            }
            hessian[(row, col)] = value;
        }
    }

    // Enforce symmetry of mixed partials for numerical robustness.
    for row in 0..independent_count {
        for col in (row + 1)..independent_count {
            let sym = 0.5 * (hessian[(row, col)] + hessian[(col, row)]);
            hessian[(row, col)] = sym;
            hessian[(col, row)] = sym;
        }
    }

    Ok(hessian)
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
#[allow(
    clippy::too_many_arguments,
    reason = "All parameters needed for batch evaluation"
)]
#[allow(
    dead_code,
    reason = "Retained for potential high-throughput evaluator path"
)]
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
    let mut all_var_names: Vec<&str> =
        Vec::with_capacity(independent_names.len() + parameter_names.len());
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
    let results = eval_f64(&exprs, &var_names, &data).map_err(|error| {
        OdrError::Numerical(format!("eval_f64 failed for layer {layer_idx}: {error:?}"))
    })?;

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

#[must_use]
/// Constructs the normal equations (`AtA` and `Atb`) from the Jacobian and residuals.
pub fn build_normal_equations(state: &EvaluationState) -> (DMatrix<f64>, DVector<f64>) {
    let j_t = state.global_weighted_jacobian.transpose();
    let normal = &j_t * &state.global_weighted_jacobian;
    let gradient = &j_t * &state.flat_weighted_residuals;
    (normal, gradient)
}

#[must_use]
/// Estimates effective rank and condition number using singular values.
pub fn diagnose_matrix(matrix: &DMatrix<f64>) -> MatrixDiagnostics {
    let svd = matrix.clone().svd(false, false);
    let singular_values = svd.singular_values;

    if singular_values.is_empty() {
        return MatrixDiagnostics {
            effective_rank: 0,
            condition_number: f64::INFINITY,
        };
    }

    let sigma_max = singular_values.iter().copied().fold(0.0, f64::max);
    let threshold = MATRIX_SINGULAR_EPS * sigma_max;

    let mut effective_rank = 0usize;
    let mut sigma_min_nonzero = f64::INFINITY;
    for sigma in singular_values.iter().copied() {
        if sigma > threshold {
            effective_rank += 1;
            sigma_min_nonzero = sigma_min_nonzero.min(sigma);
        }
    }

    let condition_number = if effective_rank == 0 || !sigma_min_nonzero.is_finite() {
        f64::INFINITY
    } else {
        sigma_max / sigma_min_nonzero
    };

    MatrixDiagnostics {
        effective_rank,
        condition_number,
    }
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
