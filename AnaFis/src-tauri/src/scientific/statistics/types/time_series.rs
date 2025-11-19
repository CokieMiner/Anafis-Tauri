#[derive(Debug, Clone)]
pub struct TimeSeriesComponents {
    pub trend: Vec<f64>,
    pub seasonal: Vec<f64>,
    pub residuals: Vec<f64>,
    pub period: usize,
}

#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub trend_present: bool,
    pub slope: f64,
    pub intercept: f64,
    pub r_squared: f64,
    pub significance: f64,
}

#[derive(Debug, Clone)]
pub struct StationarityResult {
    pub is_stationary: bool,
    pub p_value: f64,
}

#[derive(Debug, Clone)]
pub struct ForecastResult {
    pub forecasts: Vec<f64>,
    pub model_type: String,
    pub metrics: Option<ForecastMetrics>,
}

#[derive(Debug, Clone)]
pub struct ForecastMetrics {
    pub mse: f64,   // Mean Squared Error
    pub rmse: f64,  // Root Mean Squared Error
    pub mae: f64,   // Mean Absolute Error
    pub mape: f64,  // Mean Absolute Percentage Error
}

/// Configuration for Prophet forecasting
#[derive(Debug, Clone)]
pub struct ProphetConfig {
    pub seasonality_periods: Option<Vec<usize>>,
    pub changepoint_prior_scale: Option<f64>,
    pub seasonality_prior_scale: Option<f64>,
    pub holidays: Option<Vec<Holiday>>,
    pub growth_model: Option<GrowthModel>,
    pub auto_tune: Option<bool>,
}

impl Default for ProphetConfig {
    fn default() -> Self {
        Self {
            seasonality_periods: Some(vec![7, 365]),
            changepoint_prior_scale: Some(0.05),
            seasonality_prior_scale: Some(10.0),
            holidays: None,
            growth_model: Some(GrowthModel::Linear),
            auto_tune: Some(false),
        }
    }
}

/// Prophet-style forecasting for time series with seasonality and trend
#[derive(Debug)]
pub struct ProphetForecast {
    pub forecasts: Vec<f64>,
    pub trend_component: Vec<f64>,
    pub seasonal_component: Vec<f64>,
    pub holiday_component: Vec<f64>,
    pub model_info: String,
}

/// Holiday specification for Prophet
#[derive(Debug, Clone)]
pub struct Holiday {
    pub name: String,
    pub dates: Vec<usize>, // Indices in the time series where holidays occur
    pub prior_scale: f64,
}

/// Growth model types for Prophet
#[derive(Debug, Clone, Copy)]
pub enum GrowthModel {
    Linear,
    Logistic { capacity: f64 },
}