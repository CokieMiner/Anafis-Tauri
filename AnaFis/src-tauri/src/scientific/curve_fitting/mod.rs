//! Generic ODR (Orthogonal Distance Regression) engine powered by SymbAnaFis.
//!
//! Notes:
//! - User symbols are treated case-insensitively (`A` and `a` are equivalent).
//! - Inputs use absolute uncertainties (not percentages).
//! - Optional per-point correlations are defined over `[x1..xn, y]`.

use nalgebra::{DMatrix, DVector};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use symb_anafis::{CompiledEvaluator, Symbol, gradient, parse, symb};
use thiserror::Error;

const MIN_VARIANCE: f64 = 1e-24;
const MATRIX_SINGULAR_EPS: f64 = 1e-14;
const CORRELATION_TOLERANCE: f64 = 1e-9;
const DEFAULT_MAX_ITERATIONS: usize = 200;
const DEFAULT_TOLERANCE: f64 = 1e-9;
const DEFAULT_DAMPING: f64 = 1e-3;
const MAX_DAMPING: f64 = 1e15;
const MIN_DAMPING: f64 = 1e-15;
const MODEL_CACHE_MAX_ENTRIES: usize = 64;

#[derive(Debug, Clone, Deserialize)]
pub struct IndependentVariableInput {
    pub name: String,
    pub values: Vec<f64>,
    pub uncertainties: Option<Vec<f64>>, // Absolute uncertainties
}

#[derive(Debug, Clone, Deserialize)]
pub struct OdrFitRequest {
    pub model_formula: String,
    pub dependent_variable: String,
    pub independent_variables: Vec<IndependentVariableInput>,
    pub observed_values: Vec<f64>,
    pub observed_uncertainties: Option<Vec<f64>>, // Absolute uncertainties
    pub parameter_names: Vec<String>,
    pub initial_guess: Option<Vec<f64>>,
    pub max_iterations: Option<usize>,
    // Optional per-point correlation matrices over [x1..xn, y]
    // Shape: [n_points][d][d], d = n_independent + 1
    pub point_correlations: Option<Vec<Vec<Vec<f64>>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OdrFitResponse {
    pub success: bool,
    pub message: Option<String>,
    pub iterations: usize,
    pub formula: String,
    pub dependent_variable: String,
    pub independent_variables: Vec<String>,
    pub parameter_names: Vec<String>,
    pub parameter_values: Vec<f64>,
    pub parameter_uncertainties: Vec<f64>,
    pub residuals: Vec<f64>,
    pub fitted_values: Vec<f64>,
    pub chi_squared: f64,
    pub chi_squared_reduced: f64,
    pub rmse: f64,
    pub r_squared: f64,
}

#[derive(Debug)]
struct PreparedData {
    independent_names: Vec<String>,
    independent_values: Vec<Vec<f64>>, // [var][point]
    dependent_values: Vec<f64>,
    point_covariances: Vec<Vec<Vec<f64>>>, // [point][dim][dim], dim = n_independent + 1
    point_count: usize,
    had_uncertainty_clamp: bool,
}

#[derive(Debug)]
struct CompiledModel {
    formula: String,
    parameter_names: Vec<String>,
    independent_names: Vec<String>,
    model_evaluator: CompiledEvaluator,
    parameter_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d p_j
    independent_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d x_k
}

#[derive(Debug)]
struct EvaluationState {
    chi_squared: f64,
    residuals: Vec<f64>,
    fitted_values: Vec<f64>,
    weighted_residuals: DVector<f64>,
    weighted_jacobian: DMatrix<f64>,
}

#[derive(Debug, Error)]
enum OdrError {
    #[error("{0}")]
    Validation(String),
    #[error("Failed to parse model formula: {0}")]
    Parse(String),
    #[error("Failed to compile model: {0}")]
    Compile(String),
    #[error("Numerical failure: {0}")]
    Numerical(String),
    #[error("Internal model cache lock poisoned")]
    CachePoisoned,
}

type OdrResult<T> = Result<T, OdrError>;

#[derive(Debug, Default)]
struct ModelCache {
    entries: HashMap<String, Arc<CompiledModel>>,
    access_order: VecDeque<String>,
}

impl ModelCache {
    fn get(&mut self, key: &str) -> Option<Arc<CompiledModel>> {
        if !self.entries.contains_key(key) {
            return None;
        }
        self.touch(key);
        self.entries.get(key).map(Arc::clone)
    }

    fn insert(&mut self, key: String, model: Arc<CompiledModel>) {
        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), model);
            self.touch(&key);
            return;
        }

        if self.entries.len() >= MODEL_CACHE_MAX_ENTRIES {
            self.evict_one();
        }

        self.entries.insert(key.clone(), model);
        self.touch(&key);
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

static MODEL_CACHE: Lazy<Mutex<ModelCache>> = Lazy::new(|| Mutex::new(ModelCache::default()));

#[tauri::command]
pub fn fit_custom_odr(request: OdrFitRequest) -> Result<OdrFitResponse, String> {
    fit_custom_odr_inner(request).map_err(|error| error.to_string())
}

fn fit_custom_odr_inner(request: OdrFitRequest) -> OdrResult<OdrFitResponse> {
    let prepared = prepare_data(&request)?;
    let normalized_parameter_names = normalize_identifiers(&request.parameter_names, "parameter")?;

    validate_symbol_sets(&prepared.independent_names, &normalized_parameter_names)?;

    let compiled_model = get_or_compile_model(
        &request.model_formula,
        &prepared.independent_names,
        &normalized_parameter_names,
    )?;

    let initial_guess = build_initial_guess(&request, normalized_parameter_names.len())?;

    let max_iterations = request
        .max_iterations
        .unwrap_or(DEFAULT_MAX_ITERATIONS)
        .clamp(5, 5000);

    let (params, final_state, iterations) = solve_odr(
        &compiled_model,
        &prepared,
        initial_guess,
        max_iterations,
        DEFAULT_TOLERANCE,
        DEFAULT_DAMPING,
    )?;

    build_response(
        &compiled_model,
        prepared,
        params,
        final_state,
        iterations,
        request.dependent_variable,
    )
}

fn prepare_data(request: &OdrFitRequest) -> OdrResult<PreparedData> {
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

fn sanitize_values(values: &[f64], label: &str) -> OdrResult<Vec<f64>> {
    let mut sanitized = Vec::with_capacity(values.len());
    for (idx, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Validation(format!(
                "Non-finite value in {} at index {}",
                label, idx
            )));
        }
        sanitized.push(*value);
    }
    Ok(sanitized)
}

fn sanitize_uncertainties(values: &[f64], label: &str) -> OdrResult<(Vec<f64>, bool)> {
    let mut sanitized = Vec::with_capacity(values.len());
    let mut had_clamp = false;
    let sigma_min = MIN_VARIANCE.sqrt();

    for (idx, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Validation(format!(
                "Non-finite uncertainty in {} at index {}",
                label, idx
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

fn validate_identifier(identifier: &str, label: &str) -> OdrResult<()> {
    if identifier.is_empty() {
        return Err(OdrError::Validation(format!(
            "{} names cannot be empty",
            label
        )));
    }

    let mut chars = identifier.chars();
    let first = chars
        .next()
        .ok_or_else(|| OdrError::Validation(format!("{} names cannot be empty", label)))?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(OdrError::Validation(format!(
            "Invalid {} '{}': first character must be a letter or '_'",
            label, identifier
        )));
    }

    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(OdrError::Validation(format!(
            "Invalid {} '{}': use only letters, digits, and '_'",
            label, identifier
        )));
    }

    Ok(())
}

fn normalize_identifiers(raw: &[String], label: &str) -> OdrResult<Vec<String>> {
    if raw.is_empty() {
        return Err(OdrError::Validation(format!(
            "At least one {} is required",
            label
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
                "Duplicate {} names are not allowed (case-insensitive collision on '{}')",
                label, name
            )));
        }
        normalized.push(lower);
    }

    Ok(normalized)
}

fn validate_symbol_sets(independent: &[String], parameters: &[String]) -> OdrResult<()> {
    let independent_set: HashSet<&str> = independent.iter().map(String::as_str).collect();
    let parameter_set: HashSet<&str> = parameters.iter().map(String::as_str).collect();

    if let Some(symbol) = independent_set.intersection(&parameter_set).next() {
        return Err(OdrError::Validation(format!(
            "Symbol '{}' is used both as independent variable and parameter",
            symbol
        )));
    }

    Ok(())
}

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

fn get_or_compile_model(
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

    cache.insert(key, Arc::clone(&compiled));

    Ok(compiled)
}

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

    Ok(())
}

fn build_initial_guess(request: &OdrFitRequest, parameter_count: usize) -> OdrResult<Vec<f64>> {
    if let Some(initial) = &request.initial_guess {
        if initial.len() != parameter_count {
            return Err(OdrError::Validation(format!(
                "Initial guess length mismatch: expected {}, got {}",
                parameter_count,
                initial.len()
            )));
        }

        let mut guess = Vec::with_capacity(initial.len());
        for (idx, value) in initial.iter().enumerate() {
            if !value.is_finite() {
                return Err(OdrError::Validation(format!(
                    "Initial guess contains non-finite value at {}",
                    idx
                )));
            }
            guess.push(*value);
        }
        return Ok(guess);
    }

    Ok(vec![1.0; parameter_count])
}

fn solve_odr(
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
        let delta = match solve_linear_system(damped_matrix, &rhs) {
            Ok(solution) => solution,
            Err(_) => {
                damping = (damping * nu).min(MAX_DAMPING);
                nu = (nu * 2.0).min(1e12);
                continue;
            }
        };

        let delta_norm = delta.norm();
        let parameter_norm = DVector::from_vec(parameters.clone()).norm();
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

        // Predicted chi^2 reduction from quadratic model:
        // pred = -(2 g^T delta + delta^T H delta), for chi^2 = ||r||^2
        let h_delta = &normal_matrix * &delta;
        let mut predicted_reduction = -(2.0 * gradient_vector.dot(&delta) + delta.dot(&h_delta));
        if !predicted_reduction.is_finite() || predicted_reduction <= MIN_VARIANCE {
            predicted_reduction = MIN_VARIANCE;
        }

        let rho = actual_reduction / predicted_reduction;

        if actual_reduction > 0.0 && rho.is_finite() && rho > 0.0 {
            let improvement = actual_reduction.abs();
            parameters = trial_parameters;
            current = trial;

            // Marquardt/Nielsen gain-ratio update
            let factor = (1.0 - (2.0 * rho - 1.0).powi(3)).clamp(1.0 / 3.0, 0.9);
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

fn evaluate_model(
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
                &format!("independent gradient evaluator {}", idx),
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
                &format!("parameter gradient evaluator {}", idx),
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
                "Model evaluated to non-finite value at data point {}",
                point
            )));
        }

        let residual = data.dependent_values[point] - fitted; // g_i = y - f(x,p)

        // a_i = grad_z g = [-df/dx1, ..., -df/dxn, 1]
        for var_idx in 0..variable_count {
            let dfdx = independent_derivatives[var_idx][point];
            if !dfdx.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Independent gradient evaluated to non-finite value at point {}",
                    point
                )));
            }
            a[var_idx] = -dfdx;
        }
        a[obs_dim - 1] = 1.0;

        let sigma = &data.point_covariances[point];
        let sigma_a = mat_vec(sigma, &a);
        let s2 = dot(&a, &sigma_a).max(MIN_VARIANCE); // s_i^2 = a^T Sigma_i a_i

        let inv_sqrt_s2 = 1.0 / s2.sqrt();
        let weighted_residual = residual * inv_sqrt_s2;

        for parameter_idx in 0..parameter_count {
            let dfdp = parameter_derivatives[parameter_idx][point];
            if !dfdp.is_finite() {
                return Err(OdrError::Numerical(format!(
                    "Parameter gradient evaluated to non-finite value at point {}",
                    point
                )));
            }
            // dr/dp = (-df/dp)/sqrt(s2)
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

fn evaluate_compiled_batch_or_scalar(
    evaluator: &CompiledEvaluator,
    columns: &[&[f64]],
    point_count: usize,
    evaluator_label: &str,
) -> OdrResult<Vec<f64>> {
    if let Ok(output) = evaluator.eval_batch_parallel(columns) {
        return validate_evaluation_output(output, evaluator_label);
    }

    let mut output = vec![0.0; point_count];

    // Fast SIMD batch path.
    let batch_result = evaluator.eval_batch(columns, &mut output, None);

    // Fallback scalar path if batch evaluation fails.
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
                "{} produced non-finite output at point {}",
                evaluator_label, idx
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

fn build_normal_equations(state: &EvaluationState) -> (DMatrix<f64>, DVector<f64>) {
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

fn invert_information_matrix(matrix: DMatrix<f64>) -> OdrResult<DMatrix<f64>> {
    let svd = matrix.svd(true, true);
    svd.pseudo_inverse(MATRIX_SINGULAR_EPS)
        .map_err(|error| OdrError::Numerical(format!("Pseudo-inverse failed: {error}")))
}

fn build_response(
    model: &CompiledModel,
    prepared: PreparedData,
    parameter_values: Vec<f64>,
    final_state: EvaluationState,
    iterations: usize,
    dependent_variable: String,
) -> OdrResult<OdrFitResponse> {
    let parameter_count = parameter_values.len();
    let point_count = prepared.point_count;

    let (normal_matrix, _) = build_normal_equations(&final_state);

    let degrees_of_freedom = point_count as isize - parameter_count as isize;
    let chi_squared_reduced = if degrees_of_freedom > 0 {
        final_state.chi_squared / degrees_of_freedom as f64
    } else {
        f64::NAN
    };

    let covariance_scale = if degrees_of_freedom > 0 && chi_squared_reduced.is_finite() {
        chi_squared_reduced.max(0.0)
    } else {
        1.0
    };

    let mut warnings: Vec<String> = Vec::new();
    if prepared.had_uncertainty_clamp {
        warnings.push(
            "Some zero/near-zero uncertainties were clamped to a minimum positive value"
                .to_string(),
        );
    }
    if degrees_of_freedom <= 0 {
        warnings.push(
            "Degrees of freedom <= 0: reduced chi-squared and parameter uncertainty scaling may be unreliable".to_string(),
        );
    }

    let parameter_uncertainties = match invert_information_matrix(normal_matrix) {
        Ok(covariance) => (0..parameter_count)
            .map(|idx| (covariance[(idx, idx)].max(0.0) * covariance_scale).sqrt())
            .collect(),
        Err(error) => {
            warnings.push(format!(
                "Fit converged, but parameter covariance could not be estimated: {}",
                error
            ));
            vec![f64::NAN; parameter_count]
        }
    };

    let residual_sum_of_squares: f64 = final_state
        .residuals
        .iter()
        .map(|value| value * value)
        .sum();
    let rmse = (residual_sum_of_squares / point_count as f64).sqrt();

    let mean_y = prepared.dependent_values.iter().sum::<f64>() / point_count as f64;
    let total_sum_of_squares: f64 = prepared
        .dependent_values
        .iter()
        .map(|value| (value - mean_y).powi(2))
        .sum();
    let r_squared = if total_sum_of_squares > 0.0 {
        1.0 - residual_sum_of_squares / total_sum_of_squares
    } else {
        1.0
    };

    let message = if warnings.is_empty() {
        None
    } else {
        Some(warnings.join(" | "))
    };

    Ok(OdrFitResponse {
        success: true,
        message,
        iterations,
        formula: model.formula.clone(),
        dependent_variable: dependent_variable.trim().to_lowercase(),
        independent_variables: model.independent_names.clone(),
        parameter_names: model.parameter_names.clone(),
        parameter_values,
        parameter_uncertainties,
        residuals: final_state.residuals,
        fitted_values: final_state.fitted_values,
        chi_squared: final_state.chi_squared,
        chi_squared_reduced,
        rmse,
        r_squared,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repeat_corr(point_count: usize, matrix: Vec<Vec<f64>>) -> Vec<Vec<Vec<f64>>> {
        (0..point_count).map(|_| matrix.clone()).collect()
    }

    #[test]
    fn test_fit_custom_odr_linear_model_no_correlation() {
        let x: Vec<f64> = (0..50).map(|v| v as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.5 * xi - 4.0).collect();

        let request = OdrFitRequest {
            model_formula: "a*x + b".to_string(),
            dependent_variable: "y".to_string(),
            independent_variables: vec![IndependentVariableInput {
                name: "x".to_string(),
                values: x,
                uncertainties: Some(vec![0.1; 50]),
            }],
            observed_values: y,
            observed_uncertainties: Some(vec![0.2; 50]),
            parameter_names: vec!["a".to_string(), "b".to_string()],
            initial_guess: Some(vec![1.0, 0.0]),
            max_iterations: Some(120),
            point_correlations: None,
        };

        let result = fit_custom_odr(request).unwrap();
        assert!(result.success);
        assert!((result.parameter_values[0] - 2.5).abs() < 1e-8);
        assert!((result.parameter_values[1] + 4.0).abs() < 1e-8);
        assert!(result.r_squared > 0.999_999_999);
    }

    #[test]
    fn test_fit_custom_odr_with_independent_correlations() {
        let mut x1 = Vec::new();
        let mut x2 = Vec::new();
        let mut y = Vec::new();

        for i in 0..40 {
            let a = i as f64 * 0.25;
            let b = (i as f64 * 0.2).sin();
            x1.push(a);
            x2.push(b);
            y.push(1.2 * a - 0.8 * b + 3.0);
        }

        let corr = vec![
            vec![1.0, 0.35, 0.0],
            vec![0.35, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        let request = OdrFitRequest {
            model_formula: "p*x1 + q*x2 + r".to_string(),
            dependent_variable: "y".to_string(),
            independent_variables: vec![
                IndependentVariableInput {
                    name: "x1".to_string(),
                    values: x1,
                    uncertainties: Some(vec![0.05; 40]),
                },
                IndependentVariableInput {
                    name: "x2".to_string(),
                    values: x2,
                    uncertainties: Some(vec![0.04; 40]),
                },
            ],
            observed_values: y,
            observed_uncertainties: Some(vec![0.08; 40]),
            parameter_names: vec!["p".to_string(), "q".to_string(), "r".to_string()],
            initial_guess: Some(vec![0.0, 0.0, 0.0]),
            max_iterations: Some(200),
            point_correlations: Some(repeat_corr(40, corr)),
        };

        let result = fit_custom_odr(request).unwrap();
        assert!(result.success);
        assert!((result.parameter_values[0] - 1.2).abs() < 1e-6);
        assert!((result.parameter_values[1] + 0.8).abs() < 1e-6);
        assert!((result.parameter_values[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_fit_custom_odr_with_cross_xy_correlation() {
        let x: Vec<f64> = (0..30).map(|i| i as f64 * 0.1).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();

        let corr = vec![vec![1.0, 0.7], vec![0.7, 1.0]];

        let request = OdrFitRequest {
            model_formula: "a*x + b".to_string(),
            dependent_variable: "y".to_string(),
            independent_variables: vec![IndependentVariableInput {
                name: "x".to_string(),
                values: x,
                uncertainties: Some(vec![0.03; 30]),
            }],
            observed_values: y,
            observed_uncertainties: Some(vec![0.05; 30]),
            parameter_names: vec!["a".to_string(), "b".to_string()],
            initial_guess: Some(vec![1.0, 0.0]),
            max_iterations: Some(160),
            point_correlations: Some(repeat_corr(30, corr)),
        };

        let result = fit_custom_odr(request).unwrap();
        assert!(result.success);
        assert!(result.chi_squared.is_finite());
        assert!((result.parameter_values[0] - 2.0).abs() < 1e-6);
        assert!((result.parameter_values[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fit_custom_odr_zero_uncertainty_clamp() {
        let x: Vec<f64> = (0..25).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| -1.5 * xi + 6.0).collect();

        let request = OdrFitRequest {
            model_formula: "m*x + c".to_string(),
            dependent_variable: "y".to_string(),
            independent_variables: vec![IndependentVariableInput {
                name: "x".to_string(),
                values: x,
                uncertainties: Some(vec![0.0; 25]),
            }],
            observed_values: y,
            observed_uncertainties: Some(vec![0.0; 25]),
            parameter_names: vec!["m".to_string(), "c".to_string()],
            initial_guess: Some(vec![0.0, 0.0]),
            max_iterations: Some(200),
            point_correlations: None,
        };

        let result = fit_custom_odr(request).unwrap();
        assert!(result.success);
        assert!(
            result
                .message
                .unwrap_or_default()
                .to_lowercase()
                .contains("clamped")
        );
    }

    #[test]
    fn test_fit_custom_odr_invalid_correlation_shape() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 3.0 * xi + 2.0).collect();

        // dim should be 2 (x,y), but here it's 1x1
        let bad_corr = vec![vec![vec![1.0]]; 10];

        let request = OdrFitRequest {
            model_formula: "a*x + b".to_string(),
            dependent_variable: "y".to_string(),
            independent_variables: vec![IndependentVariableInput {
                name: "x".to_string(),
                values: x,
                uncertainties: Some(vec![0.1; 10]),
            }],
            observed_values: y,
            observed_uncertainties: Some(vec![0.1; 10]),
            parameter_names: vec!["a".to_string(), "b".to_string()],
            initial_guess: Some(vec![1.0, 0.0]),
            max_iterations: Some(100),
            point_correlations: Some(bad_corr),
        };

        let err = fit_custom_odr(request).unwrap_err();
        assert!(err.contains("invalid shape"));
    }

    #[test]
    fn test_fit_custom_odr_nonlinear_gaussian_like() {
        let x: Vec<f64> = (-40..=40).map(|i| i as f64 * 0.05).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|xi| 2.0 * (-0.7 * xi * xi).exp() + 0.5)
            .collect();

        let request = OdrFitRequest {
            model_formula: "a*exp(-b*x^2)+c".to_string(),
            dependent_variable: "y".to_string(),
            independent_variables: vec![IndependentVariableInput {
                name: "x".to_string(),
                values: x,
                uncertainties: Some(vec![0.02; 81]),
            }],
            observed_values: y,
            observed_uncertainties: Some(vec![0.03; 81]),
            parameter_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            initial_guess: Some(vec![1.0, 0.2, 0.0]),
            max_iterations: Some(600),
            point_correlations: None,
        };

        let result = fit_custom_odr(request).unwrap();
        assert!(result.success);
        assert!((result.parameter_values[0] - 2.0).abs() < 1e-3);
        assert!((result.parameter_values[1] - 0.7).abs() < 1e-3);
        assert!((result.parameter_values[2] - 0.5).abs() < 1e-3);
    }

    #[test]
    fn test_fit_custom_odr_multivariable_full_covariance() {
        let mut x1 = Vec::new();
        let mut x2 = Vec::new();
        let mut x3 = Vec::new();
        let mut y = Vec::new();

        for i in 0..35 {
            let a = i as f64 * 0.3;
            let b = (i as f64 * 0.17).cos();
            let c = (i as f64 * 0.11).sin();
            x1.push(a);
            x2.push(b);
            x3.push(c);
            y.push(0.9 * a - 1.1 * b + 0.7 * c + 4.0);
        }

        // Order: [x1, x2, x3, y]
        let corr = vec![
            vec![1.0, 0.2, -0.1, 0.15],
            vec![0.2, 1.0, 0.3, -0.2],
            vec![-0.1, 0.3, 1.0, 0.1],
            vec![0.15, -0.2, 0.1, 1.0],
        ];

        let request = OdrFitRequest {
            model_formula: "p*x1 + q*x2 + r*x3 + s".to_string(),
            dependent_variable: "y".to_string(),
            independent_variables: vec![
                IndependentVariableInput {
                    name: "x1".to_string(),
                    values: x1,
                    uncertainties: Some(vec![0.05; 35]),
                },
                IndependentVariableInput {
                    name: "x2".to_string(),
                    values: x2,
                    uncertainties: Some(vec![0.04; 35]),
                },
                IndependentVariableInput {
                    name: "x3".to_string(),
                    values: x3,
                    uncertainties: Some(vec![0.03; 35]),
                },
            ],
            observed_values: y,
            observed_uncertainties: Some(vec![0.06; 35]),
            parameter_names: vec![
                "p".to_string(),
                "q".to_string(),
                "r".to_string(),
                "s".to_string(),
            ],
            initial_guess: Some(vec![0.0, 0.0, 0.0, 0.0]),
            max_iterations: Some(300),
            point_correlations: Some(repeat_corr(35, corr)),
        };

        let result = fit_custom_odr(request).unwrap();
        assert!(result.success);
        assert!((result.parameter_values[0] - 0.9).abs() < 1e-6);
        assert!((result.parameter_values[1] + 1.1).abs() < 1e-6);
        assert!((result.parameter_values[2] - 0.7).abs() < 1e-6);
        assert!((result.parameter_values[3] - 4.0).abs() < 1e-6);
    }
}
