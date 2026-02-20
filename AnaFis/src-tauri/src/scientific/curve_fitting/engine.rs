use nalgebra::{DMatrix, DVector};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use symb_anafis::{CompiledEvaluator, Symbol, gradient, parse, symb};

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

/// A model that has been compiled into executable bytecode.
#[derive(Debug)]
pub struct CompiledModel {
    /// The original mathematical formula.
    pub formula: String,
    /// Names of the parameters to be fitted.
    pub parameter_names: Vec<String>,
    /// Names of the independent variables.
    pub independent_names: Vec<String>,
    /// Compiled evaluator for the main formula.
    pub model_evaluator: CompiledEvaluator,
    /// Compiled evaluators for the partial derivatives with respect to each parameter.
    pub parameter_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d p_j
    /// Compiled evaluators for the partial derivatives with respect to each independent variable.
    pub independent_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d x_k
}

/// Data prepared and validated for the ODR solver.
pub struct PreparedData {
    /// Names of the independent variables.
    pub independent_names: Vec<String>,
    /// Matrix of independent values: [`variable_index`][`point_index`].
    pub independent_values: Vec<Vec<f64>>, // [var][point]
    /// Vector of dependent (observed) values.
    pub dependent_values: Vec<f64>,
    /// Full covariance matrices for each data point.
    /// Format: <code>[point_index][dimension_index][dimension_index]</code>, where dimension = `n_independent` + 1.
    pub point_covariances: Vec<Vec<Vec<f64>>>, // [point][dim][dim], dim = n_independent + 1
    /// Total number of data points.
    pub point_count: usize,
    /// Whether any near-zero uncertainties were clamped to a minimum value.
    pub had_uncertainty_clamp: bool,
}

/// The current state of an ODR evaluation, including residuals and Jacobians.
pub struct EvaluationState {
    /// Current weighted total chi-squared value.
    pub chi_squared: f64,
    /// Raw residuals (observed - predicted).
    pub residuals: Vec<f64>,
    /// Values predicted by the model at the current state.
    pub fitted_values: Vec<f64>,
    /// Residuals weighted by the inverse covariance matrix.
    pub weighted_residuals: DVector<f64>,
    /// Jacobian matrix weighted by the inverse covariance matrix.
    pub weighted_jacobian: DMatrix<f64>,
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
    independent_names: &[String],
    parameter_names: &[String],
) -> String {
    format!(
        "{}||x:{}||p:{}",
        formula.trim().to_lowercase(),
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
    independent_names: &[String],
    parameter_names: &[String],
) -> OdrResult<Arc<CompiledModel>> {
    let key = build_model_cache_key(model_formula, independent_names, parameter_names);

    {
        let mut cache = MODEL_CACHE.lock().map_err(|_| OdrError::CachePoisoned)?;
        if let Some(model) = cache.get(&key) {
            return Ok(model);
        }
    }

    let compiled = Arc::new(compile_model_inner(
        model_formula,
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
    for gradient_expr in parameter_gradients {
        parameter_gradient_evaluators.push(
            CompiledEvaluator::compile(&gradient_expr, &evaluator_order, None).map_err(
                |error| OdrError::Compile(format!("parameter derivative evaluator: {error:?}")),
            )?,
        );
    }

    let independent_symbols: Vec<Symbol> =
        independent_names.iter().map(|name| symb(name)).collect();
    let independent_symbol_refs: Vec<&Symbol> = independent_symbols.iter().collect();
    let independent_gradients = gradient(&expr, &independent_symbol_refs)
        .map_err(|error| OdrError::Compile(format!("independent gradients: {error:?}")))?;

    let mut independent_gradient_evaluators = Vec::with_capacity(independent_gradients.len());
    for gradient_expr in independent_gradients {
        independent_gradient_evaluators.push(
            CompiledEvaluator::compile(&gradient_expr, &evaluator_order, None).map_err(
                |error| OdrError::Compile(format!("independent derivative evaluator: {error:?}")),
            )?,
        );
    }

    Ok(CompiledModel {
        formula,
        parameter_names: parameter_names.to_vec(),
        independent_names: independent_names.to_vec(),
        model_evaluator,
        parameter_gradient_evaluators,
        independent_gradient_evaluators,
    })
}

/// Prepares data for ODR fitting.
///
/// # Errors
/// Returns `OdrError::Validation` if data length or values are invalid.
pub fn prepare_data(request: &OdrFitRequest) -> OdrResult<PreparedData> {
    if request.independent_variables.is_empty() {
        return Err(OdrError::Validation(
            "At least one independent variable is required".to_string(),
        ));
    }

    if request.observed_values.len() < 2 {
        return Err(OdrError::Validation(
            "At least two observations are required for fitting".to_string(),
        ));
    }

    let point_count = request.observed_values.len();
    let dependent_values = sanitize_values(&request.observed_values, "observed values")?;

    let (dependent_uncertainties, mut had_uncertainty_clamp) =
        if let Some(uncertainties) = &request.observed_uncertainties {
            if uncertainties.len() != point_count {
                return Err(OdrError::Validation(format!(
                    "Observed uncertainty length mismatch: expected {}, got {}",
                    point_count,
                    uncertainties.len()
                )));
            }
            sanitize_uncertainties(uncertainties, "observed uncertainties")?
        } else {
            (vec![1.0; point_count], false)
        };

    let raw_independent_names: Vec<String> = request
        .independent_variables
        .iter()
        .map(|variable| variable.name.clone())
        .collect();
    let independent_names = normalize_identifiers(&raw_independent_names, "independent variable")?;

    let mut independent_values = Vec::with_capacity(request.independent_variables.len());
    let mut independent_sigmas = Vec::with_capacity(request.independent_variables.len());

    for variable in &request.independent_variables {
        if variable.values.len() != point_count {
            return Err(OdrError::Validation(format!(
                "Independent variable '{}' length mismatch: expected {}, got {}",
                variable.name,
                point_count,
                variable.values.len()
            )));
        }

        independent_values.push(sanitize_values(&variable.values, &variable.name)?);

        if let Some(uncertainties) = &variable.uncertainties {
            if uncertainties.len() != point_count {
                return Err(OdrError::Validation(format!(
                    "Uncertainty length mismatch for '{}': expected {}, got {}",
                    variable.name,
                    point_count,
                    uncertainties.len()
                )));
            }
            let (sigma, clamped) = sanitize_uncertainties(uncertainties, &variable.name)?;
            had_uncertainty_clamp |= clamped;
            independent_sigmas.push(sigma);
        } else {
            independent_sigmas.push(vec![0.0; point_count]);
        }
    }

    let point_covariances = build_point_covariances(
        point_count,
        &independent_sigmas,
        &dependent_uncertainties,
        request.point_correlations.as_deref(),
    )?;

    Ok(PreparedData {
        independent_names,
        independent_values,
        dependent_values,
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

/// Constructs the full covariance matrix for each measurement point.
fn build_point_covariances(
    point_count: usize,
    independent_sigmas: &[Vec<f64>],
    dependent_sigmas: &[f64],
    point_correlations: Option<&[Vec<Vec<f64>>]>,
) -> OdrResult<Vec<Vec<Vec<f64>>>> {
    let dim = independent_sigmas.len() + 1;

    if dependent_sigmas.len() != point_count {
        return Err(OdrError::Validation(format!(
            "Dependent uncertainty length mismatch: expected {}, got {}",
            point_count,
            dependent_sigmas.len()
        )));
    }

    for (idx, sigma) in independent_sigmas.iter().enumerate() {
        if sigma.len() != point_count {
            return Err(OdrError::Validation(format!(
                "Independent uncertainty length mismatch for var {}: expected {}, got {}",
                idx,
                point_count,
                sigma.len()
            )));
        }
    }

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
        for var_idx in 0..independent_sigmas.len() {
            sigmas[var_idx] = independent_sigmas[var_idx][point];
        }
        sigmas[dim - 1] = dependent_sigmas[point];

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

/// Solves the Orthogonal Distance Regression (ODR) problem using Levenberg-Marquardt.
///
/// # Errors
/// Returns `OdrError` if numerical convergence fails or fitting error occurs.
pub fn solve_odr(
    model: &CompiledModel,
    data: &PreparedData,
    mut parameters: Vec<f64>,
    max_iterations: usize,
    tolerance: f64,
    initial_damping: f64,
) -> OdrResult<(Vec<f64>, EvaluationState, usize)> {
    let mut damping = initial_damping;
    let mut nu = 2.0;
    let mut current = evaluate_model(model, data, &parameters)?;
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

        let trial = evaluate_model(model, data, &trial_parameters)?;
        let actual_reduction = current.chi_squared - trial.chi_squared;

        let h_delta = &normal_matrix * &delta;
        let mut predicted_reduction = -2.0f64.mul_add(gradient_vector.dot(&delta), delta.dot(&h_delta));
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

/// Evaluates the model at current parameters.
///
/// # Errors
/// Returns `OdrError::Numerical` if model or gradients evaluate to non-finite values.
pub fn evaluate_model(
    model: &CompiledModel,
    data: &PreparedData,
    parameters: &[f64],
) -> OdrResult<EvaluationState> {
    let variable_count = data.independent_names.len();
    let parameter_count = model.parameter_names.len();
    let point_count = data.point_count;

    let mut columns: Vec<&[f64]> = Vec::with_capacity(variable_count + parameter_count);
    for values in &data.independent_values {
        columns.push(values);
    }

    let parameter_columns: Vec<Vec<f64>> = parameters
        .iter()
        .map(|parameter| vec![*parameter; point_count])
        .collect();
    for values in &parameter_columns {
        columns.push(values);
    }

    let fitted_values = evaluate_compiled_batch_or_scalar(
        &model.model_evaluator,
        &columns,
        point_count,
        "model evaluator",
    )?;

    let independent_derivatives: Vec<Vec<f64>> = model
        .independent_gradient_evaluators
        .iter()
        .enumerate()
        .map(|(idx, evaluator)| {
            evaluate_compiled_batch_or_scalar(
                evaluator,
                &columns,
                point_count,
                &format!("independent gradient evaluator {idx}"),
            )
        })
        .collect::<OdrResult<Vec<Vec<f64>>>>()?;

    let parameter_derivatives: Vec<Vec<f64>> = model
        .parameter_gradient_evaluators
        .iter()
        .enumerate()
        .map(|(idx, evaluator)| {
            evaluate_compiled_batch_or_scalar(
                evaluator,
                &columns,
                point_count,
                &format!("parameter gradient evaluator {idx}"),
            )
        })
        .collect::<OdrResult<Vec<Vec<f64>>>>()?;

    let mut residuals = Vec::with_capacity(point_count);
    let mut weighted_residuals = Vec::with_capacity(point_count);
    let mut weighted_jacobian = vec![0.0; point_count * parameter_count];
    let mut chi_squared = 0.0;

    let obs_dim = variable_count + 1; // [x1..xn, y]
    let mut a = vec![0.0; obs_dim];

    for point in 0..point_count {
        let fitted = fitted_values[point];
        if !fitted.is_finite() {
            return Err(OdrError::Numerical(format!(
                "Model evaluated to non-finite value at data point {point}"
            )));
        }

        let residual = data.dependent_values[point] - fitted;

        for var_idx in 0..variable_count {
            let dfdx = independent_derivatives[var_idx][point];
            if !dfdx.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Independent gradient evaluated to non-finite value at point {point}"
                )));
            }
            a[var_idx] = -dfdx;
        }
        a[obs_dim - 1] = 1.0;

        let sigma = &data.point_covariances[point];
        let sigma_a = mat_vec(sigma, &a);
        let s2 = dot(&a, &sigma_a).max(MIN_VARIANCE);

        let inv_sqrt_s2 = 1.0 / s2.sqrt();
        let weighted_residual = residual * inv_sqrt_s2;

        for parameter_idx in 0..parameter_count {
            let dfdp = parameter_derivatives[parameter_idx][point];
            if !dfdp.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Parameter gradient evaluated to non-finite value at point {point}"
                )));
            }
            weighted_jacobian[point * parameter_count + parameter_idx] = -dfdp * inv_sqrt_s2;
        }

        chi_squared += weighted_residual.powi(2);
        residuals.push(residual);
        weighted_residuals.push(weighted_residual);
    }

    Ok(EvaluationState {
        chi_squared,
        residuals,
        fitted_values,
        weighted_residuals: DVector::from_vec(weighted_residuals),
        weighted_jacobian: DMatrix::from_row_slice(
            point_count,
            parameter_count,
            &weighted_jacobian,
        ),
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
    let j_t = state.weighted_jacobian.transpose();
    let normal = &j_t * &state.weighted_jacobian;
    let gradient = &j_t * &state.weighted_residuals;
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
