use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Input data for an independent variable in an ODR fit.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndependentVariableInput {
    /// The name of the independent variable.
    pub name: String,
    /// The observed values for this variable.
    pub values: Vec<f64>,
    /// Optional absolute uncertainties for each measurement value.
    pub uncertainties: Option<Vec<f64>>, // Absolute uncertainties
}

/// Request structure for performing a custom ODR fit.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OdrFitRequest {
    /// The mathematical model formula (e.g., "a*x + b").
    pub model_formula: String,
    /// The name of the dependent variable.
    pub dependent_variable: String,
    /// List of independent variables and their data.
    pub independent_variables: Vec<IndependentVariableInput>,
    /// Observed values for the dependent variable.
    pub observed_values: Vec<f64>,
    /// Optional absolute uncertainties for the dependent variable.
    pub observed_uncertainties: Option<Vec<f64>>, // Absolute uncertainties
    /// Names of the parameters to be fitted.
    pub parameter_names: Vec<String>,
    /// Optional initial guess for parameter values.
    pub initial_guess: Option<Vec<f64>>,
    /// Optional maximum number of iterations.
    pub max_iterations: Option<usize>,
    /// Optional full correlation matrices between measurements.
    pub point_correlations: Option<Vec<Vec<Vec<f64>>>>,
}

/// Response containing the results of an ODR fit.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OdrFitResponse {
    /// Whether the fit was successful.
    pub success: bool,
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
    /// Full parameter covariance matrix.
    pub parameter_covariance: Vec<Vec<f64>>, // Full covariance matrix
    /// Raw residuals at the final state.
    pub residuals: Vec<f64>,
    /// Model predictions at the final state.
    pub fitted_values: Vec<f64>,
    /// Total weighted chi-squared value.
    pub chi_squared: f64,
    /// Reduced chi-squared value (per degree of freedom).
    pub chi_squared_reduced: f64,
    /// Root Mean Square Error of residuals.
    pub rmse: f64,
    /// Coefficient of determination (R²).
    pub r_squared: f64,
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
