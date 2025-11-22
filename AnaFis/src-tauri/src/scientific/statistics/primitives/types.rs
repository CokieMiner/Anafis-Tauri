//! Mathematical Primitives Types
//!
//! Type definitions for low-level mathematical operations and utilities.

use serde::{Deserialize, Serialize};

/// Result of linear algebra operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearAlgebraResult<T> {
    pub result: T,
    pub condition_number: Option<f64>,
    pub rank: Option<usize>,
}

/// Special functions result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialFunctionResult {
    pub value: f64,
    pub error_estimate: Option<f64>,
}

/// Numerical integration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    pub integral: f64,
    pub error_estimate: f64,
    pub n_evaluations: usize,
}

/// Root finding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFindingResult {
    pub root: f64,
    pub residual: f64,
    pub n_iterations: usize,
    pub converged: bool,
}

/// Interpolation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolationResult {
    pub interpolated_value: f64,
    pub method: InterpolationMethod,
}

/// Interpolation methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InterpolationMethod {
    Linear,
    CubicSpline,
    Lagrange,
}

/// Random sampling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub method: SamplingMethod,
    pub n_samples: usize,
    pub seed: Option<u64>,
}

/// Random sampling methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SamplingMethod {
    Uniform,
    Normal,
    Exponential,
    Custom,
}

/// Non-central distribution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonCentralParameters {
    pub non_centrality_parameter: f64,
    pub degrees_of_freedom: Option<f64>,
}
