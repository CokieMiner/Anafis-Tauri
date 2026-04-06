//! Curve fitting module using profiled ODR with per-point latent x-corrections.
// Note(odr-option-2): Profiled latent-variable ODR is implemented; future work is
// optional and focused on stronger trust-region/Schur-complement step control.
pub(crate) mod commands;
mod logic;
mod tests;
mod types;

pub use commands::{evaluate_model_curve, evaluate_model_grid, fit_custom_odr};
pub use types::{
    CurveEvaluationRequest, CurveEvaluationResponse, GridEvaluationRequest,
    GridEvaluationResponse, ModelLayer, OdrError, OdrFitRequest, OdrFitResponse, OdrResult,
    VariableInput,
};
