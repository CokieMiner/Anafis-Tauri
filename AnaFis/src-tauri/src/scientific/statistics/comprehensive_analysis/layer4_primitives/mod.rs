//! Layer 4: Fundamental Mathematical Primitives
//!
//! This layer contains atomic mathematical operations that only depend on external crates.
//! No internal function calls - pure input/output functions.
//!
//! This module serves as a facade that re-exports all primitive functions
//! from their respective focused modules for backward compatibility.

pub mod special_functions;
pub mod numerical_integration;
pub mod linear_algebra;
pub mod random_sampling;
pub mod root_finding;
pub mod interpolation;
pub mod non_central_distributions;
pub mod unified_stats;

// Re-export all structs and functions for backward compatibility
pub use special_functions::SpecialFunctions;
pub use numerical_integration::NumericalIntegration;
pub use linear_algebra::{LinearAlgebra, SvdResult};
pub use random_sampling::RandomSampling;
pub use root_finding::RootFinding;
pub use interpolation::Interpolation;
pub use non_central_distributions::{NonCentralT, NonCentralF, PowerAnalysisUtils};
pub use unified_stats::UnifiedStats;

// Backward compatibility aliases
pub use linear_algebra::LinearAlgebra as NdLinearAlgebra;
pub use linear_algebra::SvdResult as NdSvdResult;