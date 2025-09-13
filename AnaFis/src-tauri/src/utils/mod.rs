// Utils module - contains utility functions, logging, and error handling

pub mod validation;
pub mod logging;
pub mod error;

// Re-export commonly used types and functions
pub use validation::{VariableInput, validate_formula, validate_variables, generate_cache_key};
pub use logging::{init_logging, log_info, log_error};
