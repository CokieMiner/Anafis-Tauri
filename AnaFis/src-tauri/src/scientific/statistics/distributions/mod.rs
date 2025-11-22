//! Distribution fitting and analysis module
//!
//! This module provides distribution fitting, normality testing,
//! goodness-of-fit tests, and data transformations.

pub mod fitting;
pub mod normality_tests;
pub mod goodness_of_fit;
pub mod types;
pub mod transformations;
pub mod distribution_functions;

pub use fitting::*;
pub use normality_tests::*;
pub use goodness_of_fit::*;
pub use types::*;
pub use transformations::*;
pub use distribution_functions::*;