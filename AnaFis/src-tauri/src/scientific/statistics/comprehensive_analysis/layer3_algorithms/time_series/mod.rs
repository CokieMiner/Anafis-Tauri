// Time series analysis module
// Re-exports all time series functionality from submodules

pub mod decomposition;
pub mod stationarity;
pub mod arima;
pub mod prophet;

// Re-export types for convenience
pub use crate::scientific::statistics::types::*;

// Re-export decomposition functionality
pub use decomposition::TimeSeriesDecompositionEngine;

// Re-export stationarity testing
pub use stationarity::StationarityEngine;

// Re-export forecasting engines
pub use arima::TimeSeriesForecastingEngine;
pub use prophet::ProphetEngine;