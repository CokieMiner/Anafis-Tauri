use crate::scientific::statistics::types::*;

/// ARIMA/SARIMA Time Series Forecasting Engine
pub struct TimeSeriesForecastingEngine;

impl TimeSeriesForecastingEngine {
    /// Fit ARIMA model and generate forecasts using oxidiviner
    pub fn fit_arima(data: &[f64], forecast_steps: usize) -> Result<ForecastResult, String> {
        if data.len() < 5 {
            return Err("Insufficient data for ARIMA modeling (need at least 5 observations)".to_string());
        }

        match oxidiviner::quick::values_only_forecast(data.to_vec(), forecast_steps) {
            Ok((forecasts, model_type)) => Ok(ForecastResult {
                forecasts,
                model_type,
                metrics: None,
            }),
            Err(e) => Err(format!("ARIMA forecasting failed: {:?}", e)),
        }
    }

    /// Automatic model selection and forecasting using oxidiviner
    pub fn auto_forecast(data: &[f64], forecast_steps: usize) -> Result<ForecastResult, String> {
        if data.len() < 5 {
            return Err("Insufficient data for forecasting (need at least 5 observations)".to_string());
        }

        match oxidiviner::quick::values_only_forecast(data.to_vec(), forecast_steps) {
            Ok((forecasts, model_type)) => Ok(ForecastResult {
                forecasts,
                model_type,
                metrics: None,
            }),
            Err(e) => Err(format!("Auto forecasting failed: {:?}", e)),
        }
    }

    /// Evaluate forecast accuracy using common metrics
    pub fn evaluate_forecast(actual: &[f64], predicted: &[f64]) -> Result<ForecastMetrics, String> {
        if actual.len() != predicted.len() {
            return Err("Actual and predicted values must have the same length".to_string());
        }

        let n = actual.len() as f64;
        let mut mse = 0.0;
        let mut mae = 0.0;
        let mut mape = 0.0;

        for (&a, &p) in actual.iter().zip(predicted.iter()) {
            let error = a - p;
            mse += error * error;
            mae += error.abs();

            // MAPE: avoid division by zero
            if a.abs() > 1e-10 {
                mape += (error / a).abs();
            }
        }

        mse /= n;
        mae /= n;
        mape = (mape / n) * 100.0; // Convert to percentage

        let rmse = mse.sqrt();

        Ok(ForecastMetrics {
            mse,
            rmse,
            mae,
            mape,
        })
    }
}