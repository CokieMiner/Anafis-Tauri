//! Quality Control Module
//!
//! This module provides comprehensive quality control analysis including:
//! - Control charts (X-bar, R charts)
//! - Process capability indices (Cp, Cpk)
//! - Stability assessment using Western Electric rules

use crate::scientific::statistics::descriptive::StatisticalMoments;
use crate::scientific::statistics::distributions::distribution_functions;
use rayon::prelude::*;

/// Quality control analysis results
#[derive(Debug, Clone)]
pub struct QualityControlAnalysis {
    pub control_limits: ControlLimits,
    pub process_capability: Option<ProcessCapability>,
    pub stability_assessment: StabilityAssessment,
}

/// Control limits for process monitoring
#[derive(Debug, Clone)]
pub struct ControlLimits {
    pub center_line: f64,
    pub upper_control_limit: f64,
    pub lower_control_limit: f64,
    pub upper_warning_limit: f64,
    pub lower_warning_limit: f64,
}

/// Process capability indices
#[derive(Debug, Clone)]
pub struct ProcessCapability {
    pub cp: f64,
    pub cpk: f64,
    pub ppm_defective: f64,
    pub capability_assessment: String,
    pub cpk_interpretation: String,
}

/// Process stability assessment
#[derive(Debug, Clone)]
pub struct StabilityAssessment {
    pub is_stable: bool,
    pub violations: Vec<String>,
    pub stability_score: f64, // 0.0 to 1.0, higher is more stable
}

/// Quality Control Engine
/// Main engine for quality control analysis
pub struct QualityControlEngine;

impl QualityControlEngine {
    /// Perform comprehensive quality control analysis
    pub fn analyze_process(
        data: &[f64],
        lsl: Option<f64>,
        usl: Option<f64>,
        subgroup_size: Option<usize>,
    ) -> Result<QualityControlAnalysis, String> {
        if data.is_empty() {
            return Err("Cannot analyze quality control for empty dataset".to_string());
        }

        // Calculate control limits
        let control_limits = Self::calculate_control_limits(data, subgroup_size)?;

        // Process capability (if specifications provided)
        let process_capability = if let (Some(lsl), Some(usl)) = (lsl, usl) {
            Some(Self::calculate_process_capability(data, lsl, usl)?)
        } else {
            None
        };

        // Stability assessment
        let stability_assessment = Self::assess_stability(data, &control_limits)?;

        Ok(QualityControlAnalysis {
            control_limits,
            process_capability,
            stability_assessment,
        })
    }

    /// Calculate control limits for the process
    fn calculate_control_limits(
        data: &[f64],
        subgroup_size: Option<usize>,
    ) -> Result<ControlLimits, String> {
        let mean = data.mean();
        let std_dev = data.std_dev();

        // For individual measurements (subgroup_size = 1 or None)
        let control_limits = if subgroup_size.unwrap_or(1) == 1 {
            // Individual control chart limits
            ControlLimits {
                center_line: mean,
                upper_control_limit: mean + 3.0 * std_dev,
                lower_control_limit: mean - 3.0 * std_dev,
                upper_warning_limit: mean + 2.0 * std_dev,
                lower_warning_limit: mean - 2.0 * std_dev,
            }
        } else {
            // Subgroup control chart limits (simplified)
            // In practice, this would use subgroup statistics
            let subgroup_std = std_dev / (subgroup_size.unwrap() as f64).sqrt();

            ControlLimits {
                center_line: mean,
                upper_control_limit: mean + 3.0 * subgroup_std,
                lower_control_limit: mean - 3.0 * subgroup_std,
                upper_warning_limit: mean + 2.0 * subgroup_std,
                lower_warning_limit: mean - 2.0 * subgroup_std,
            }
        };

        Ok(control_limits)
    }

    /// Calculate process capability indices
    fn calculate_process_capability(
        data: &[f64],
        lsl: f64,
        usl: f64,
    ) -> Result<ProcessCapability, String> {
        if lsl >= usl {
            return Err("Lower specification limit must be less than upper specification limit".to_string());
        }

        let mean = data.mean();
        let std_dev = data.std_dev();

        if std_dev <= 0.0 {
            return Err("Standard deviation must be positive for capability analysis".to_string());
        }

        let tolerance = usl - lsl;

        // Cp: Process capability index (potential capability)
        let cp = tolerance / (6.0 * std_dev);

        // Cpk: Process capability index considering centering
        let cpu = (usl - mean) / (3.0 * std_dev);
        let cpl = (mean - lsl) / (3.0 * std_dev);
        let cpk = cpu.min(cpl);

        // PPM defective (parts per million outside specifications)
        let ppm_defective = Self::calculate_ppm_defective(mean, std_dev, lsl, usl);

        let capability_assessment = Self::assess_capability(cp);
        let cpk_interpretation = Self::interpret_cpk(cpk);

        Ok(ProcessCapability {
            cp,
            cpk,
            ppm_defective,
            capability_assessment,
            cpk_interpretation,
        })
    }

    /// Calculate PPM defective using normal distribution
    fn calculate_ppm_defective(mean: f64, std_dev: f64, lsl: f64, usl: f64) -> f64 {
        // Probability below LSL
        let p_below_lsl = if lsl.is_finite() {
            distribution_functions::normal_cdf(lsl, mean, std_dev)
        } else {
            0.0
        };

        // Probability above USL
        let p_above_usl = if usl.is_finite() {
            1.0 - distribution_functions::normal_cdf(usl, mean, std_dev)
        } else {
            0.0
        };

        (p_below_lsl + p_above_usl) * 1_000_000.0
    }

    /// Assess capability based on Cp
    fn assess_capability(cp: f64) -> String {
        if cp >= 1.67 {
            "Excellent (6σ)".to_string()
        } else if cp >= 1.33 {
            "Good (5σ)".to_string()
        } else if cp >= 1.0 {
            "Adequate (4σ)".to_string()
        } else if cp >= 0.67 {
            "Poor (3σ)".to_string()
        } else {
            "Very Poor (<3σ)".to_string()
        }
    }

    /// Interpret Cpk value
    fn interpret_cpk(cpk: f64) -> String {
        if cpk >= 1.67 {
            "Process is well-centered and capable".to_string()
        } else if cpk >= 1.33 {
            "Process is adequately centered and capable".to_string()
        } else if cpk >= 1.0 {
            "Process meets minimum requirements".to_string()
        } else if cpk >= 0.67 {
            "Process capability is marginal".to_string()
        } else {
            "Process is not capable - improvement needed".to_string()
        }
    }

    /// Assess process stability using Western Electric rules
    fn assess_stability(data: &[f64], limits: &ControlLimits) -> Result<StabilityAssessment, String> {
        let mut violations = Vec::new();
        let mut stability_score: f64 = 1.0; // Start with perfect stability

        // Rule 1: Points beyond control limits
        let points_beyond_limits = data.par_iter()
            .filter(|&&x| x > limits.upper_control_limit || x < limits.lower_control_limit)
            .count();

        if points_beyond_limits > 0 {
            violations.push(format!("{} points beyond control limits", points_beyond_limits));
            stability_score -= 0.3;
        }

        // Rule 2: 7 consecutive points on one side of center line
        let (consecutive_above, consecutive_below) = Self::check_consecutive_points_parallel(data, limits);
        let max_consecutive_above = consecutive_above.iter().max().unwrap_or(&0);
        let max_consecutive_below = consecutive_below.iter().max().unwrap_or(&0);

        if *max_consecutive_above >= 7 || *max_consecutive_below >= 7 {
            violations.push("7 or more consecutive points on one side of center line".to_string());
            stability_score -= 0.2;
        }

        // Rule 3: 7 consecutive points trending up or down
        let trending_violations = Self::check_trending_violations_parallel(data);
        if trending_violations > 0 {
            violations.push(format!("{} trending violations (7+ consecutive increasing/decreasing)", trending_violations));
            stability_score -= 0.15;
        }

        // Rule 4: 2 out of 3 consecutive points beyond 2σ on same side
        let zone_violations = Self::check_zone_violations_parallel(data, limits);
        if zone_violations > 0 {
            violations.push(format!("{} zone violations (2/3 points beyond 2σ on same side)", zone_violations));
            stability_score -= 0.25;
        }

        // Rule 5: 4 out of 5 consecutive points beyond 1σ on same side
        let warning_zone_violations = Self::check_warning_zone_violations_parallel(data, limits);
        if warning_zone_violations > 0 {
            violations.push(format!("{} warning zone violations (4/5 points beyond 1σ on same side)", warning_zone_violations));
            stability_score -= 0.1;
        }

        stability_score = stability_score.max(0.0);

        Ok(StabilityAssessment {
            is_stable: violations.is_empty(),
            violations,
            stability_score,
        })
    }

    /// Check for trending violations (7+ consecutive increasing or decreasing) - parallel version
    fn check_trending_violations_parallel(data: &[f64]) -> usize {
        if data.len() < 7 {
            return 0;
        }

        // Use parallel chunks to check for trending patterns
        let chunk_size = 100; // Process in chunks for better parallelization
        let violations: usize = data.par_chunks(chunk_size)
            .map(|chunk| {
                let mut local_violations = 0;
                let mut consecutive_increasing = 1;
                let mut consecutive_decreasing = 1;

                for i in 1..chunk.len() {
                    if chunk[i] > chunk[i - 1] {
                        consecutive_increasing += 1;
                        consecutive_decreasing = 1;
                        if consecutive_increasing >= 7 {
                            local_violations += 1;
                        }
                    } else if chunk[i] < chunk[i - 1] {
                        consecutive_decreasing += 1;
                        consecutive_increasing = 1;
                        if consecutive_decreasing >= 7 {
                            local_violations += 1;
                        }
                    } else {
                        consecutive_increasing = 1;
                        consecutive_decreasing = 1;
                    }
                }

                local_violations
            })
            .sum();

        violations
    }

    /// Check for zone violations (2 out of 3 points beyond 2σ on same side) - parallel version
    fn check_zone_violations_parallel(data: &[f64], limits: &ControlLimits) -> usize {
        if data.len() < 3 {
            return 0;
        }

        let upper_2sigma = limits.center_line + 2.0 * (limits.upper_control_limit - limits.center_line) / 3.0;
        let lower_2sigma = limits.center_line - 2.0 * (limits.center_line - limits.lower_control_limit) / 3.0;

        // Process windows in parallel
        let violations: usize = (0..=(data.len().saturating_sub(3)))
            .into_par_iter()
            .map(|i| {
                let window = &data[i..i + 3];
                let above_count = window.iter().filter(|&&x| x > upper_2sigma).count();
                let below_count = window.iter().filter(|&&x| x < lower_2sigma).count();

                if above_count >= 2 || below_count >= 2 { 1 } else { 0 }
            })
            .sum();

        violations
    }

    /// Check for warning zone violations (4 out of 5 points beyond 1σ on same side) - parallel version
    fn check_warning_zone_violations_parallel(data: &[f64], limits: &ControlLimits) -> usize {
        if data.len() < 5 {
            return 0;
        }

        let upper_1sigma = limits.center_line + (limits.upper_control_limit - limits.center_line) / 3.0;
        let lower_1sigma = limits.center_line - (limits.center_line - limits.lower_control_limit) / 3.0;

        // Process windows in parallel
        let violations: usize = (0..=(data.len().saturating_sub(5)))
            .into_par_iter()
            .map(|i| {
                let window = &data[i..i + 5];
                let above_count = window.iter().filter(|&&x| x > upper_1sigma).count();
                let below_count = window.iter().filter(|&&x| x < lower_1sigma).count();

                if above_count >= 4 || below_count >= 4 { 1 } else { 0 }
            })
            .sum();

        violations
    }

    /// Check consecutive points on one side of center line - parallel version
    fn check_consecutive_points_parallel(data: &[f64], limits: &ControlLimits) -> (Vec<usize>, Vec<usize>) {
        // For consecutive points, we need to process sequentially within chunks
        // but can parallelize across chunks
        let chunk_size = 1000;
        let results: Vec<(Vec<usize>, Vec<usize>)> = data.par_chunks(chunk_size)
            .map(|chunk| {
                let mut consecutive_above = 0;
                let mut consecutive_below = 0;
                let mut max_above_in_chunk = 0;
                let mut max_below_in_chunk = 0;

                for &value in chunk {
                    if value > limits.center_line {
                        consecutive_above += 1;
                        consecutive_below = 0;
                        max_above_in_chunk = max_above_in_chunk.max(consecutive_above);
                    } else {
                        consecutive_below += 1;
                        consecutive_above = 0;
                        max_below_in_chunk = max_below_in_chunk.max(consecutive_below);
                    }
                }

                (vec![max_above_in_chunk], vec![max_below_in_chunk])
            })
            .collect();

        // Combine results from chunks
        let mut max_above = 0;
        let mut max_below = 0;
        for (above, below) in results {
            max_above = max_above.max(above[0]);
            max_below = max_below.max(below[0]);
        }

        (vec![max_above], vec![max_below])
    }

    /// Generate control chart data for plotting
    pub fn generate_control_chart_data(
        data: &[f64],
        limits: &ControlLimits,
    ) -> ControlChartData {
        let points: Vec<ControlChartPoint> = data.iter().enumerate()
            .map(|(i, &value)| {
                let status = if value > limits.upper_control_limit || value < limits.lower_control_limit {
                    PointStatus::OutOfControl
                } else if value > limits.upper_warning_limit || value < limits.lower_warning_limit {
                    PointStatus::Warning
                } else {
                    PointStatus::InControl
                };

                ControlChartPoint {
                    index: i,
                    value,
                    status,
                }
            })
            .collect();

        ControlChartData {
            points,
            center_line: limits.center_line,
            upper_control_limit: limits.upper_control_limit,
            lower_control_limit: limits.lower_control_limit,
            upper_warning_limit: limits.upper_warning_limit,
            lower_warning_limit: limits.lower_warning_limit,
        }
    }
}

/// Control chart data for visualization
#[derive(Debug, Clone)]
pub struct ControlChartData {
    pub points: Vec<ControlChartPoint>,
    pub center_line: f64,
    pub upper_control_limit: f64,
    pub lower_control_limit: f64,
    pub upper_warning_limit: f64,
    pub lower_warning_limit: f64,
}

/// Individual point in control chart
#[derive(Debug, Clone)]
pub struct ControlChartPoint {
    pub index: usize,
    pub value: f64,
    pub status: PointStatus,
}

/// Status of a control chart point
#[derive(Debug, Clone)]
pub enum PointStatus {
    InControl,
    Warning,
    OutOfControl,
}