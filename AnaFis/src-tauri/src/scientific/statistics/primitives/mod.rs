//! Low-level mathematical primitives and utilities
//!
//! This module provides fundamental mathematical operations and utilities
//! that are used across the statistics library.

pub mod types;
pub mod linear_algebra;
pub mod sampling;
pub mod special_functions;
pub mod numerical_integration;
pub mod root_finding;
pub mod interpolation;
pub mod non_central_distributions;

pub mod regression;
pub mod design_matrix;
pub mod robust_regression_utils;
pub mod distance;

pub use types::*;
pub use linear_algebra::*;
pub use sampling::*;
pub use special_functions::*;
pub use numerical_integration::*;
pub use root_finding::*;
pub use interpolation::*;
pub use non_central_distributions::*;

pub use regression::*;
pub use design_matrix::*;
pub use robust_regression_utils::*;
pub use distance::*;