//! Time Series Analysis Types
//!
//! Type definitions for time series analysis, decomposition, and forecasting.

use serde::{Deserialize, Serialize};

/// Time series decomposition components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesComponents {
    pub trend: Vec<f64>,
    pub seasonal: Vec<f64>,
    pub residuals: Vec<f64>,
    pub period: usize,
}

/// Stationarity test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationarityResult {
    pub is_stationary: bool,
    pub p_value: f64,
}

/// Forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    pub forecasts: Vec<f64>,
    pub model_type: String,
    pub metrics: Option<ForecastMetrics>,
}

/// Forecast evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastMetrics {
    pub mse: f64,   // Mean Squared Error
    pub rmse: f64,  // Root Mean Squared Error
    pub mae: f64,   // Mean Absolute Error
    pub mape: f64,  // Mean Absolute Percentage Error
}