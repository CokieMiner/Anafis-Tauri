pub mod batch_eval;
pub mod curvature;
pub mod data_prep;
pub mod diagnostics;
pub mod evaluation;
pub mod inference;
pub mod inner_solve;
pub mod linear_algebra;
pub mod solver;
pub mod state;
pub use batch_eval::{
    evaluate_hessian_exprs_batch, evaluate_model_and_gradients_batch, evaluate_model_expr_batch,
};
pub use curvature::{
    compute_second_derivative_corrections_numerical, dependent_curvature_coefficient,
    extract_joint_covariance,
};
pub use data_prep::{is_positive_semidefinite, prepare_data};
pub use diagnostics::{build_normal_equations, diagnose_matrix};
pub use evaluation::evaluate_model;
pub use inference::{ParameterInference, compute_parameter_inference};
pub use inner_solve::{ParameterSource, solve_inner_corrections_multi_point};
pub use linear_algebra::{
    invert_small_psd, solve_linear_system, solve_linear_system_matrix, sqrt_psd_matrix,
};
pub use solver::solve_odr;
pub use state::{
    BatchEvaluationResult, EvaluationState, OdrTerminationReason, PointCovariances, PreparedData,
};

pub use super::cache::{CompiledModel, get_or_compile_model};
pub use super::constants::*;
pub use super::sanitization::{normalize_identifiers, validate_identifier, validate_symbol_sets};
pub use super::{OdrError, OdrFitRequest, OdrResult, UncertaintyType, VariableInput};

pub use super::constants::CORRECTION_VARIANCE_THRESHOLD;
