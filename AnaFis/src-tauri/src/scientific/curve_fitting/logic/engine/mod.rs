pub mod batch_eval;
pub mod diagnostics;
pub mod linear_algebra;
pub mod solver;
pub mod state;

pub use batch_eval::evaluate_model_expr_batch;
pub use diagnostics::{build_normal_equations, diagnose_matrix};
pub use linear_algebra::invert_information_matrix;
pub use solver::{
    CompiledModel, DEFAULT_DAMPING, DEFAULT_MAX_ITERATIONS, DEFAULT_TOLERANCE,
    get_or_compile_model, normalize_identifiers, prepare_data, solve_odr, validate_identifier,
    validate_symbol_sets,
};
pub use state::{
    EvaluationState, OdrTerminationReason, PreparedData,
};
