//! Data quality assessment and scoring

use crate::scientific::statistics::types::{AnalysisResults, DataQualityOutput};
use super::super::layer4_primitives::StatisticalPower;

/// Data quality assessment utilities
pub struct QualityAssessor;

impl QualityAssessor {
    /// Compute overall data quality score
    pub fn compute_data_quality(results: &AnalysisResults) -> DataQualityOutput {
        let mut score = 100.0; // Start with perfect score
        let mut issues = Vec::new();

        // Sample size penalty: compare to required n for mean detection
        if let Some(desc_stats) = &results.descriptive_stats {
            if desc_stats.count < 30 {
                let penalty: f64 = (30.0 - desc_stats.count as f64) * 2.0;
                score -= penalty.min(40.0);
                issues.push("Small sample size".to_string());
            }

            // Compute required sample size based on standard deviation and a default effect size of 0.5*std
            let std_dev = desc_stats.std_dev;
            if std_dev.is_finite() && std_dev > 0.0 {
                if let Ok(required_n) = StatisticalPower::required_sample_size_for_mean(std_dev, std_dev * 0.5, 0.05, 0.8) {
                    if desc_stats.count < required_n {
                        let penalty: f64 = 40.0 * (1.0 - desc_stats.count as f64 / required_n as f64);
                        score -= penalty.min(40.0);
                        issues.push(format!("Sample size {} below required {} for power", desc_stats.count, required_n));
                    }
                }
            }
        }

        // Normality penalty
        if let Some(normality_tests) = &results.normality_test {
            if let Some(test) = normality_tests.first() {
                if !test.is_normal {
                    score -= 15.0;
                    issues.push("Non-normal distribution".to_string());
                }
            }
        }

        // Outlier penalty
        if let Some(outlier_analysis) = &results.outlier_analysis {
            let outlier_penalty = outlier_analysis.outlier_analysis.contamination_rate * 2.0;
            score -= outlier_penalty.min(30.0);
            if outlier_analysis.outlier_analysis.contamination_rate > 5.0 {
                issues.push("High outlier percentage".to_string());
            }
        }

        // Process stability penalty
        if let Some(qc) = &results.quality_control {
            if !qc.stability_assessment.is_stable {
                score -= 20.0;
                issues.push("Process instability".to_string());
            }
        }

        score = score.max(0.0); // Ensure non-negative

        DataQualityOutput {
            sample_size_adequate: results.descriptive_stats
                .as_ref()
                .map(|stats| stats.count >= 30)
                .unwrap_or(false),
            is_normal: results.normality_test
                .as_ref()
                .and_then(|tests| tests.first())
                .map(|test| test.is_normal)
                .unwrap_or(false),
            outlier_summary: if issues.contains(&"High outlier percentage".to_string()) {
                "Significant outliers detected".to_string()
            } else {
                "Outliers within acceptable range".to_string()
            },
            missing_data: "No missing data detected".to_string(), // We sanitized this
            quality_score: score as u32,
        }
    }
}