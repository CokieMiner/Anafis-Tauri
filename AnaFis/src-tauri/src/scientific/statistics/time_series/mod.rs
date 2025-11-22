//! Time series analysis module
//!
//! This module provides time series analysis including ARIMA, seasonality,
//! trend analysis, decomposition, forecasting, and spectral analysis.

pub mod types;
pub mod arima;
pub mod seasonality;
pub mod trend;
pub mod decomposition;
pub mod forecasting;
pub mod spectral;

pub use types::*;
pub use arima::*;
pub use seasonality::*;
pub use trend::*;
pub use decomposition::*;
pub use forecasting::*;
pub use spectral::*;