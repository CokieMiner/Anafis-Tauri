//! Prophet Forecasting Types
//!
//! Type definitions for Prophet-style time series forecasting.

use serde::{Deserialize, Serialize};

/// Configuration for Prophet model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProphetConfig {
    pub n_changepoints: usize,
    pub changepoint_prior_scale: f64,
    pub seasonality_prior_scale: f64,
    pub holidays: Option<std::collections::HashMap<String, Vec<f64>>>,
    pub growth_model: String,
    pub auto_tune: bool,
    pub seasonality_period: Option<f64>,
    pub seasonality_harmonics: usize,
    pub uncertainty_samples: usize,
    pub uncertainty_scale: f64,
}

impl Default for ProphetConfig {
    fn default() -> Self {
        Self {
            n_changepoints: 25,
            changepoint_prior_scale: 0.05,
            seasonality_prior_scale: 10.0,
            holidays: None,
            growth_model: "linear".to_string(),
            auto_tune: true,
            seasonality_period: Some(365.25),
            seasonality_harmonics: 10,
            uncertainty_samples: 1000,
            uncertainty_scale: 0.1,
        }
    }
}

/// Trend model for Prophet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendModel {
    pub coefficients: Vec<f64>,
    pub changepoints: Vec<usize>,
    pub base_timestamps: Vec<f64>,
}

/// Seasonal model for Prophet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalModel {
    pub coefficients: Vec<f64>,
    pub period: f64,
    pub n_harmonics: usize,
}

/// Holiday model for Prophet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayModel {
    pub coefficients: Vec<f64>,
    pub holiday_names: Vec<String>,
}

/// Fitted Prophet model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProphetModel {
    pub trend_model: TrendModel,
    pub seasonal_model: SeasonalModel,
    pub holiday_model: Option<HolidayModel>,
    pub changepoints: Vec<usize>,
    pub config: ProphetConfig,
    pub training_data: Vec<f64>,
    pub timestamps: Vec<f64>,
}

/// Prediction intervals for Prophet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionIntervals {
    pub lower_80: Vec<f64>,
    pub upper_80: Vec<f64>,
    pub lower_95: Vec<f64>,
    pub upper_95: Vec<f64>,
}

/// Prophet prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProphetPrediction {
    pub predictions: Vec<f64>,
    pub trend_components: Vec<f64>,
    pub seasonal_components: Vec<f64>,
    pub holiday_components: Vec<f64>,
    pub prediction_intervals: Option<PredictionIntervals>,
}