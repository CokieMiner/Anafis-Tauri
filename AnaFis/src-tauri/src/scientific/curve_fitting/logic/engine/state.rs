use nalgebra::{DMatrix, DVector};

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
    /// Number of times a joint covariance block required diagonal regularization to become PSD.
    pub covariance_regularization_count: usize,
    /// Maximum L2 norm of the inner first-order stationarity residual across point solves.
    pub inner_stationarity_norm_max: f64,
    /// Mean L2 norm of the inner first-order stationarity residual across point solves.
    pub inner_stationarity_norm_mean: f64,
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
