//! Layer 4: Fundamental Mathematical Primitives
//!
//! This layer contains atomic mathematical operations that only depend on external crates.
//! No internal function calls - pure input/output functions.
//!
//! This module serves as a facade that re-exports all primitive functions
//! from their respective focused modules for backward compatibility.

pub mod special_functions;
pub mod statistical_power;
pub mod numerical_integration;
pub mod linear_algebra;
pub mod ndarray_linear_algebra;
pub mod distributions;
pub mod random_sampling;
pub mod root_finding;
pub mod interpolation;

// Re-export all structs and functions for backward compatibility
pub use special_functions::SpecialFunctions;
pub use statistical_power::StatisticalPower;
pub use numerical_integration::NumericalIntegration;
pub use linear_algebra::{LinearAlgebra, SvdResult};
pub use ndarray_linear_algebra::{NdLinearAlgebra, NdSvdResult};
pub use distributions::StatisticalDistributions;
pub use random_sampling::RandomSampling;
pub use root_finding::RootFinding;
pub use interpolation::Interpolation;