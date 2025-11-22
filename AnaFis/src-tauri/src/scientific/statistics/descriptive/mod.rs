//! # Descriptive Statistics
//!
//! This module provides a consolidated and cleaned-up API for calculating
//! descriptive statistics, following a domain-driven structure.
//!
//! It introduces the `StatisticalMoments` trait, which provides fundamental
//! calculations like mean, variance, and standard deviation directly on slices.
//! Other measures of central tendency, dispersion, and shape are organized
//! into their respective submodules.
//!
//! ## Key Components
//!
//! - `moments::StatisticalMoments`: A trait for fundamental statistical properties.
//! - `central_tendency::CentralTendency`: Functions for median and mode.
//! - `dispersion::Dispersion`: Functions for range, IQR, and MAD.
//! - `quantiles::Quantiles`: Robust quantile estimation functions.
//!

// Declare the submodules
pub mod moments;
pub mod quantiles;
pub mod uncertainty;
pub mod central_tendency;
pub mod dispersion;
pub mod types;

// Hide internal-only modules like kde
mod kde;

// Re-export the primary APIs for easier access
pub use self::moments::StatisticalMoments;
pub use self::central_tendency::CentralTendency;
pub use self::dispersion::Dispersion;
pub use self::quantiles::{Quantiles, QuantileMethod};
