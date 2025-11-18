use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::distribution::StatisticalDistributionEngine;
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::StatisticalDistributions;

#[derive(Debug, Clone)]
pub struct QualityControlAnalysis {
    pub control_limits: ControlLimits,
    pub process_capability: Option<ProcessCapability>,
    pub stability_assessment: String,
}

#[derive(Debug, Clone)]
pub struct ControlLimits {
    pub center_line: f64,
    pub upper_control_limit: f64,
    pub lower_control_limit: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessCapability {
    pub cp: f64,
    pub cpk: f64,
    pub ppm_defective: f64,
    pub capability_assessment: String,
}

/// Quality Control Coordinator
/// Coordinates process capability and control chart analysis
pub struct QualityControlCoordinator;

impl QualityControlCoordinator {
    /// Analyze process quality and capability
    pub fn analyze(data: &[f64], lsl: Option<f64>, usl: Option<f64>) -> Result<QualityControlAnalysis, String> {
        if data.is_empty() {
            return Err("Cannot analyze quality control for empty dataset".to_string());
        }

        // Control limits (assuming normal distribution)
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std_dev = StatisticalDistributionEngine::variance(data).sqrt();

        let control_limits = ControlLimits {
            center_line: mean,
            upper_control_limit: mean + 3.0 * std_dev,
            lower_control_limit: mean - 3.0 * std_dev,
        };

        // Process capability (if specifications provided)
        let process_capability = if let (Some(lsl), Some(usl)) = (lsl, usl) {
            Some(Self::compute_process_capability(data, lsl, usl)?)
        } else {
            None
        };

        // Stability assessment
        let stability_assessment = Self::assess_process_stability(data, &control_limits)?;

        Ok(QualityControlAnalysis {
            control_limits,
            process_capability,
            stability_assessment,
        })
    }

    /// Compute process capability indices
    fn compute_process_capability(data: &[f64], lsl: f64, usl: f64) -> Result<ProcessCapability, String> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std_dev = StatisticalDistributionEngine::variance(data).sqrt();

        let tolerance = usl - lsl;
        let cp = tolerance / (6.0 * std_dev);

        // Cpk considers centering
        let cpu = (usl - mean) / (3.0 * std_dev);
        let cpl = (mean - lsl) / (3.0 * std_dev);
        let cpk = cpu.min(cpl);

        // PPM defective (simplified)
        let ppm_defective = (1.0 - StatisticalDistributions::normal_cdf(usl, mean, std_dev) +
                           StatisticalDistributions::normal_cdf(lsl, mean, std_dev)) * 1_000_000.0;

        Ok(ProcessCapability {
            cp,
            cpk,
            ppm_defective,
            capability_assessment: Self::assess_capability(cpk),
        })
    }

    /// Assess capability based on Cpk
    fn assess_capability(cpk: f64) -> String {
        if cpk >= 1.67 {
            "Excellent (6σ)".to_string()
        } else if cpk >= 1.33 {
            "Good (5σ)".to_string()
        } else if cpk >= 1.0 {
            "Adequate (4σ)".to_string()
        } else if cpk >= 0.67 {
            "Poor (3σ)".to_string()
        } else {
            "Very Poor (<3σ)".to_string()
        }
    }

    /// Assess process stability
    fn assess_process_stability(data: &[f64], limits: &ControlLimits) -> Result<String, String> {
        let mut points_out_of_control = 0;
        let mut consecutive_above = 0;
        let mut consecutive_below = 0;
        let mut window_last3 = Vec::new();
        let mut window_last5 = Vec::new();
        let upper_2sigma = limits.center_line + 2.0 * (limits.upper_control_limit - limits.center_line) / 3.0; // approximate std location
        let lower_2sigma = limits.center_line - 2.0 * (limits.center_line - limits.lower_control_limit) / 3.0;
        let upper_1sigma = limits.center_line + 1.0 * (limits.upper_control_limit - limits.center_line) / 3.0;
        let lower_1sigma = limits.center_line - 1.0 * (limits.center_line - limits.lower_control_limit) / 3.0;

        for &value in data {
            if value > limits.upper_control_limit || value < limits.lower_control_limit {
                points_out_of_control += 1;
            }

            if value > limits.center_line {
                consecutive_above += 1;
                consecutive_below = 0;
            } else {
                consecutive_below += 1;
                consecutive_above = 0;
            }

            // Update last3 and last5 windows
            window_last3.push(value);
            if window_last3.len() > 3 { window_last3.remove(0); }
            window_last5.push(value);
            if window_last5.len() > 5 { window_last5.remove(0); }

            // Check for runs of 7 or more
            if consecutive_above >= 7 || consecutive_below >= 7 {
                return Ok("Unstable - Excessive runs".to_string());
            }

            // Western Electric Rule 2: Two out of three beyond 2σ on same side
            if window_last3.len() == 3 {
                let above_count = window_last3.iter().filter(|&&v| v > upper_2sigma).count();
                let below_count = window_last3.iter().filter(|&&v| v < lower_2sigma).count();
                if above_count >= 2 || below_count >= 2 {
                    return Ok("Unstable - Western Electric rule: 2 of 3 beyond 2σ".to_string());
                }
            }

            // Western Electric Rule 3: Four out of five beyond 1σ on same side
            if window_last5.len() == 5 {
                let above_count = window_last5.iter().filter(|&&v| v > upper_1sigma).count();
                let below_count = window_last5.iter().filter(|&&v| v < lower_1sigma).count();
                if above_count >= 4 || below_count >= 4 {
                    return Ok("Unstable - Western Electric rule: 4 of 5 beyond 1σ".to_string());
                }
            }
        }

        if points_out_of_control > data.len() / 20 { // More than 5% out of control
            Ok("Unstable - Too many points out of control".to_string())
        } else {
            Ok("Stable".to_string())
        }
    }
}
