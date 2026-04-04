use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use symb_anafis::{CompiledEvaluator, Expr, Symbol, gradient, parse, symb};

use super::constants::MODEL_CACHE_MAX_ENTRIES;
use super::super::types::{OdrError, OdrResult};

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
    /// Parsed mixed second derivatives with respect to independent variables and parameters (row-major by x then p).
    pub independent_parameter_mixed_hessian_exprs: Vec<Expr>, // d2 f / d x_row d p_col
    /// Compiled evaluator for the main formula (fallback).
    pub model_evaluator: CompiledEvaluator,
    /// Compiled evaluators for the partial derivatives with respect to each parameter (fallback).
    pub parameter_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d p_j
    /// Compiled evaluators for the partial derivatives with respect to each independent variable (fallback).
    pub independent_gradient_evaluators: Vec<CompiledEvaluator>, // d f / d x_k
    /// Compiled evaluators for second derivatives with respect to independent variables (row-major Hessian).
    pub independent_hessian_evaluators: Vec<CompiledEvaluator>, // d2 f / d x_row d x_col
    /// Compiled evaluators for mixed second derivatives with respect to independent variables and parameters.
    pub independent_parameter_mixed_hessian_evaluators: Vec<CompiledEvaluator>, // d2 f / d x_row d p_col
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

/// Internal function to compile a model formula into evaluators.
#[allow(
    clippy::too_many_lines,
    reason = "Model compilation keeps all symbolic derivative generation in one place for cache consistency"
)]
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

    let mixed_hessian_capacity = independent_gradient_exprs.len() * parameter_names.len();
    let mut independent_parameter_mixed_hessian_evaluators =
        Vec::with_capacity(mixed_hessian_capacity);
    let mut independent_parameter_mixed_hessian_exprs = Vec::with_capacity(mixed_hessian_capacity);
    for row_gradient in &independent_gradient_exprs {
        let row_mixed_derivatives = gradient(row_gradient, &parameter_symbol_refs)
            .map_err(|error| {
                OdrError::Compile(format!(
                    "independent-parameter mixed Hessian gradients: {error:?}"
                ))
            })?;

        for mixed_derivative_expr in row_mixed_derivatives {
            independent_parameter_mixed_hessian_evaluators.push(
                CompiledEvaluator::compile(&mixed_derivative_expr, &evaluator_order, None)
                    .map_err(|error| {
                        OdrError::Compile(format!(
                            "independent-parameter mixed Hessian evaluator: {error:?}"
                        ))
                    })?,
            );
            independent_parameter_mixed_hessian_exprs.push(mixed_derivative_expr);
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
        independent_parameter_mixed_hessian_exprs,
        model_evaluator,
        parameter_gradient_evaluators,
        independent_gradient_evaluators,
        independent_hessian_evaluators,
        independent_parameter_mixed_hessian_evaluators,
    })
}
