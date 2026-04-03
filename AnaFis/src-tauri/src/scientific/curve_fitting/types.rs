use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Input data for a variable (independent or dependent) in a profiled ODR fit.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableInput {
    /// The name of the variable.
    pub name: String,
    /// The observed values for this variable.
    pub values: Vec<f64>,
    /// Optional absolute uncertainties for each measurement value.
    pub uncertainties: Option<Vec<f64>>, // Absolute uncertainties
}

/// A single equation layer in a multilayered profiled ODR fit.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLayer {
    /// The mathematical model formula (e.g., "a*x + b").
    pub formula: String,
    /// The name of the dependent variable for this layer.
    pub dependent_variable: String,
    /// Names of the specific independent variables used in this layer.
    pub independent_variables: Vec<String>,
}

/// Request structure for performing a custom multi-layer profiled ODR fit.
///
/// Note: This solver uses a nested/profiled strategy where per-point latent x-corrections
/// are solved in an inner loop and the outer LM uses an approximate profiled gradient.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OdrFitRequest {
    /// The layers forming the system to be fitted.
    pub layers: Vec<ModelLayer>,
    /// List of ALL independent variables and their data.
    pub independent_variables: Vec<VariableInput>,
    /// List of ALL dependent observations/targets and their data.
    pub dependent_variables: Vec<VariableInput>,
    /// Combined names of the parameters to be fitted globally.
    pub parameter_names: Vec<String>,
    /// Optional initial guess for global parameter values.
    pub initial_guess: Option<Vec<f64>>,
    /// Optional maximum number of iterations.
    pub max_iterations: Option<usize>,
    /// Convergence tolerance (default 1e-9).
    pub tolerance: Option<f64>,
    /// Initial damping factor for Levenberg-Marquardt (default 1e-3).
    pub initial_damping: Option<f64>,
    /// Optional full correlation matrices between measurements.
    pub point_correlations: Option<Vec<Vec<Vec<f64>>>>,
    /// If true, applies Poisson-based uncertainty σ = √max(y, ε) for dependent variables without explicit uncertainties.
    pub use_poisson_weighting: Option<bool>,
    /// Optional confidence level for expanded uncertainties (default 0.95).
    pub confidence_level: Option<f64>,
}

/// Response containing the results of a profiled ODR fit.
///
/// Note: Results use a profiled ODR objective with implicit correction sensitivity in a
/// Gauss-Newton outer linearization; full structural latent-state coupling across equations
/// is not yet implemented.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OdrFitResponse {
    /// Whether the fit was successful.
    pub success: bool,
    /// Why the optimizer stopped (e.g., converged or max iterations reached).
    pub termination_reason: String,
    /// Optional message (e.g., convergence status or warnings).
    pub message: Option<String>,
    /// Number of iterations performed.
    pub iterations: usize,
    /// The model formula used.
    pub formula: String,
    /// Name of the dependent variable.
    pub dependent_variable: String,
    /// Names of the independent variables.
    pub independent_variables: Vec<String>,
    /// Names of the parameters fitted.
    pub parameter_names: Vec<String>,
    /// Optimized parameter values.
    pub parameter_values: Vec<f64>,
    /// Estimated uncertainties for each parameter.
    pub parameter_uncertainties: Vec<f64>,
    /// Expanded uncertainties for each parameter at the selected confidence level.
    pub parameter_expanded_uncertainties: Vec<f64>,
    /// Coverage factor used to compute expanded uncertainties.
    pub coverage_factor: f64,
    /// Full parameter covariance matrix.
    pub parameter_covariance: Vec<Vec<f64>>, // Full covariance matrix
    /// Raw residuals at the final state.
    pub residuals: Vec<f64>,
    /// Model predictions at the final state.
    pub fitted_values: Vec<f64>,
    /// Observation-only weighted chi-squared value used for fit reporting.
    pub chi_squared: f64,
    /// Reduced chi-squared value (per degree of freedom).
    pub chi_squared_reduced: f64,
    /// Root Mean Square Error of residuals.
    pub rmse: f64,
    /// Coefficient of determination (R²).
    pub r_squared: f64,
    /// Per-layer R² values; each entry is the R² for the corresponding model layer.
    pub r_squared_per_layer: Vec<f64>,
    /// Effective numerical rank of the final normal matrix.
    pub effective_rank: usize,
    /// Condition number estimate of the final normal matrix.
    pub condition_number: f64,
    /// Assumptions used for uncertainty interpretation (NIST GUM context).
    pub assumptions: Vec<String>,
}

/// Request structure for evaluating a model on a 2D grid.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridEvaluationRequest {
    /// The model formula to evaluate.
    pub model_formula: String,
    /// Names of the independent variables.
    pub independent_names: Vec<String>,
    /// Names of the model parameters.
    pub parameter_names: Vec<String>,
    /// Constant values for the parameters.
    pub parameter_values: Vec<f64>,
    /// Range (min, max) for the first independent variable.
    pub x_range: (f64, f64),
    /// Range (min, max) for the second independent variable.
    pub y_range: (f64, f64),
    /// Number of points per dimension in the grid.
    pub resolution: usize,
}

/// Response containing the results of a grid evaluation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridEvaluationResponse {
    /// X coordinates of grid points.
    pub x: Vec<f64>,
    /// Y coordinates of grid points.
    pub y: Vec<f64>,
    /// Z (evaluated) values for the model at grid points.
    pub z: Vec<f64>,
}

/// Errors that can occur during ODR fitting.
#[derive(Debug, Error)]
pub enum OdrError {
    /// Input data validation failure.
    #[error("{0}")]
    Validation(String),
    /// Formula parsing failure.
    #[error("Failed to parse model formula: {0}")]
    Parse(String),
    /// Model compilation failure.
    #[error("Failed to compile model: {0}")]
    Compile(String),
    /// Numerical failure during ODR solver (e.g., non-invertible matrix).
    #[error("Numerical failure: {0}")]
    Numerical(String),
    /// Internal model cache lock poisoned.
    #[error("Internal model cache lock poisoned")]
    CachePoisoned,
}

/// Result type for ODR operations.
pub type OdrResult<T> = Result<T, OdrError>;
