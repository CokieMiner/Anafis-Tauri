//! Stationarity Testing Types
//!
//! Type definitions for time series stationarity testing methods and results.

use serde::{Deserialize, Serialize};

/// Types of regression for ADF test
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AdfRegressionType {
    Constant,
    ConstantTrend,
    NoConstant,
}

/// Critical values for ADF test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdfCriticalValues {
    pub one_percent: f64,
    pub five_percent: f64,
    pub ten_percent: f64,
}

/// Result of ADF test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdfTestResult {
    pub test_statistic: f64,
    pub p_value: f64,
    pub critical_values: AdfCriticalValues,
    pub lags_used: usize,
    pub n_obs: usize,
    pub regression_type: AdfRegressionType,
}

/// Types of regression for KPSS test
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KpssRegressionType {
    Constant,
    ConstantTrend,
}

/// Critical values for KPSS test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpssCriticalValues {
    pub one_percent: f64,
    pub two_point_five_percent: f64,
    pub five_percent: f64,
    pub ten_percent: f64,
}

/// Result of KPSS test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpssTestResult {
    pub test_statistic: f64,
    pub p_value: f64,
    pub critical_values: KpssCriticalValues,
    pub lags_used: usize,
    pub regression_type: KpssRegressionType,
    pub long_run_variance: f64,
}

/// Result of Phillips-Perron test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhillipsPerronResult {
    pub test_statistic: f64,
    pub p_value: f64,
    pub critical_values: AdfCriticalValues,
    pub lags_used: usize,
    pub regression_type: AdfRegressionType,
    pub long_run_variance: f64,
}

/// Result of rolling statistics analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingStatsResult {
    pub means: Vec<f64>,
    pub variances: Vec<f64>,
    pub autocorrelations: Vec<f64>,
    pub positions: Vec<usize>,
    pub window_size: usize,
}