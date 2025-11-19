//! Test modules for comprehensive statistical analysis
//!
//! This directory contains all tests organized by purpose:
//! - unit_tests: Basic unit tests for individual functions
//! - integration_tests: End-to-end pipeline tests
//! - benchmark_tests: Performance and comprehensive validation
//! - property_tests: Statistical property validation (Anscombe, etc.)

pub mod unit_tests;
pub mod integration_tests;
pub mod benchmark_tests;
pub mod property_tests;
pub mod anova_tests;
pub mod matrix_tests;
pub mod distribution_tests;
pub mod reliability_tests;
pub mod validation_tests;
pub mod non_central_distributions_tests;
pub mod time_series_test;