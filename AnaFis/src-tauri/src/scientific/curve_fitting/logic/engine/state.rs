use nalgebra::{DMatrix, DVector};

/// SVD-based numerical diagnostics for a matrix.
#[derive(Debug, Clone, Copy)]
pub struct MatrixDiagnostics {
    /// Effective numerical rank based on `MATRIX_SINGULAR_EPS * sigma_max`.
    pub(crate) effective_rank: usize,
    /// Matrix condition number estimate (`sigma_max / sigma_min_nonzero`).
    pub(crate) condition_number: f64,
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
    pub(crate) variable_names: Vec<String>,
    /// Matrix of variable values: [`variable_index`][`point_index`].
    pub(crate) variable_values: Vec<Vec<f64>>,
    /// Full covariance matrices for each data point across the combined variable space.
    /// Format: `[point_index][dim][dim]`, where `dim = variable_names.len()`.
    pub(crate) point_covariances: Vec<Vec<Vec<f64>>>,
    /// Total number of data points.
    pub(crate) point_count: usize,
    /// Whether any near-zero uncertainties were clamped to a minimum value.
    pub(crate) had_uncertainty_clamp: bool,
    /// Whether Poisson weighting was applied to at least one dependent variable with low counts (<20).
    pub(crate) had_low_count_poisson: bool,
    /// Number of variables where Type A uncertainty DOF was auto-inferred as n-1.
    pub(crate) inferred_type_a_dof_count: usize,
    /// Optional per-variable uncertainty DOF metadata in `variable_names` order.
    pub(crate) variable_uncertainty_dofs: Vec<Option<f64>>,
    /// Approximate effective input DOF via Welch-Satterthwaite when finite DOF metadata is available.
    pub(crate) welch_satterthwaite_dof: Option<f64>,
}

/// The current state of an ODR evaluation across all layers.
pub struct EvaluationState {
    /// Current profiled objective value used by the optimizer.
    pub(crate) chi_squared: f64,
    /// Observation-only weighted chi-squared used for reduced-chi-squared reporting.
    pub(crate) chi_squared_observation: f64,
    /// Raw residuals (observed - predicted) for each layer: [`layer_idx`][point_idx].
    pub(crate) layer_residuals: Vec<Vec<f64>>,
    /// Values predicted by the models at the current state: [`layer_idx`][point_idx].
    pub(crate) layer_fitted_values: Vec<Vec<f64>>,
    /// Flattened residuals weighted by the inverse covariance matrix.
    pub(crate) flat_weighted_residuals: DVector<f64>,
    /// Global Jacobian matrix weighted by the inverse covariance matrix.
    pub(crate) global_weighted_jacobian: DMatrix<f64>,
    /// Additive second-order curvature correction to the outer normal matrix.
    pub(crate) outer_second_order_normal: DMatrix<f64>,
    /// Number of per-point inner correction solves that did not meet convergence tolerance.
    pub(crate) inner_correction_nonconverged_points: usize,
    /// Number of times a joint covariance block required diagonal regularization to become PSD.
    pub(crate) covariance_regularization_count: usize,
    /// Maximum L2 norm of the inner first-order stationarity residual across point solves.
    pub(crate) inner_stationarity_norm_max: f64,
    /// Mean L2 norm of the inner first-order stationarity residual across point solves.
    pub(crate) inner_stationarity_norm_mean: f64,
    /// Sensitivity-weighted Welch-Satterthwaite effective DOF from current model sensitivities.
    pub(crate) welch_satterthwaite_dof: Option<f64>,
}

/// Result of batch evaluation containing model values and derivatives.
pub struct BatchEvaluationResult {
    /// Model fitted values.
    pub(crate) fitted_values: Vec<f64>,
    /// Derivatives with respect to independent variables.
    pub(crate) independent_derivatives: Vec<Vec<f64>>,
    /// Derivatives with respect to parameters.
    pub(crate) parameter_derivatives: Vec<Vec<f64>>,
}
