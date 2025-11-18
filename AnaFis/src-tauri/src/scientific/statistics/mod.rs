//! Comprehensive Statistical Analysis Module
//!
//! This module provides state-of-the-art statistical analysis capabilities
//! organized in a layered architecture for maximum accuracy and usability.

pub mod types;
pub mod comprehensive_analysis;

/// Convenience function for performing comprehensive statistical analysis
pub fn perform_comprehensive_analysis(
    datasets: Vec<Vec<f64>>,
    options: Option<types::AnalysisOptions>,
) -> Result<types::ComprehensiveResult, String> {
    let opts = options.unwrap_or_default();
    comprehensive_analysis::layer1_command::ComprehensiveAnalysisCommand::execute(datasets, opts)
}