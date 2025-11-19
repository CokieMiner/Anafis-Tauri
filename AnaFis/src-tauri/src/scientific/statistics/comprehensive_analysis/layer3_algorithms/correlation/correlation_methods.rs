//! Basic correlation computation methods

use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::correlation_utils::rank_data;

/// Correlation computation methods
pub struct CorrelationMethods;

impl CorrelationMethods {
    /// Compute Pearson correlation coefficient between two vectors
    pub fn pearson_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Vectors must have equal length and at least 2 elements".to_string());
        }

        let n = x.len() as f64;
        let sum_x = x.iter().sum::<f64>();
        let sum_y = y.iter().sum::<f64>();
        let sum_xy = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum::<f64>();
        let sum_x2 = x.iter().map(|a| a * a).sum::<f64>();
        let sum_y2 = y.iter().map(|b| b * b).sum::<f64>();

        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

        if denominator == 0.0 {
            return Err("Cannot compute correlation: zero variance in data".to_string());
        }

        Ok(numerator / denominator)
    }

    /// Compute Spearman rank correlation coefficient
    pub fn spearman_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Vectors must have equal length and at least 2 elements".to_string());
        }

        // Convert to ranks
        let x_ranks = rank_data(x);
        let y_ranks = rank_data(y);

        Self::pearson_correlation(&x_ranks, &y_ranks)
    }

    /// Compute Kendall tau correlation coefficient
    pub fn kendall_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Vectors must have equal length and at least 2 elements".to_string());
        }

        let n = x.len();
        let mut concordant = 0;
        let mut discordant = 0;

        // Count concordant and discordant pairs
        for i in 0..n {
            for j in (i + 1)..n {
                let x_diff = x[i] - x[j];
                let y_diff = y[i] - y[j];

                if x_diff * y_diff > 0.0 {
                    concordant += 1;
                } else if x_diff * y_diff < 0.0 {
                    discordant += 1;
                }
                // Ties are ignored (neither concordant nor discordant)
            }
        }

        let total_pairs = concordant + discordant;
        if total_pairs == 0 {
            return Ok(0.0); // All pairs are tied
        }

        Ok((concordant as f64 - discordant as f64) / total_pairs as f64)
    }

    /// Compute biweight midcorrelation (robust correlation) with configurable tuning
    pub fn biweight_midcorrelation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        Self::biweight_midcorrelation_tuned(x, y, 9.0) // Default tuning constant
    }

    /// Compute biweight midcorrelation with specified tuning constant
    pub fn biweight_midcorrelation_tuned(x: &[f64], y: &[f64], tuning_constant: f64) -> Result<f64, String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Datasets must have equal length and at least 2 observations".to_string());
        }

        // Compute medians
        let mut x_sorted = x.to_vec();
        let mut y_sorted = y.to_vec();
        x_sorted.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });
        y_sorted.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        let median_x = if x_sorted.len().is_multiple_of(2) {
            (x_sorted[x_sorted.len() / 2 - 1] + x_sorted[x_sorted.len() / 2]) / 2.0
        } else {
            x_sorted[x_sorted.len() / 2]
        };

        let median_y = if y_sorted.len().is_multiple_of(2) {
            (y_sorted[y_sorted.len() / 2 - 1] + y_sorted[y_sorted.len() / 2]) / 2.0
        } else {
            y_sorted[y_sorted.len() / 2]
        };

        // Compute MAD (Median Absolute Deviation)
        let mad_x = Self::median_absolute_deviation(x, median_x);
        let mad_y = Self::median_absolute_deviation(y, median_y);

        if mad_x == 0.0 || mad_y == 0.0 {
            return Ok(0.0);
        }

        // Compute weights and weighted correlation
        let mut weighted_xy = 0.0;
        let mut weighted_x2 = 0.0;
        let mut weighted_y2 = 0.0;

        for (&xi, &yi) in x.iter().zip(y.iter()) {
            let u_x = (xi - median_x) / (tuning_constant * mad_x);
            let u_y = (yi - median_y) / (tuning_constant * mad_y);

            if u_x.abs() < 1.0 && u_y.abs() < 1.0 {
                let w_x = (1.0 - u_x * u_x).powi(2);
                let w_y = (1.0 - u_y * u_y).powi(2);
                let w = w_x * w_y;

                weighted_xy += w * (xi - median_x) * (yi - median_y);
                weighted_x2 += w * (xi - median_x).powi(2);
                weighted_y2 += w * (yi - median_y).powi(2);
            }
        }

        if weighted_x2 == 0.0 || weighted_y2 == 0.0 {
            return Ok(0.0);
        }

        Ok(weighted_xy / (weighted_x2 * weighted_y2).sqrt())
    }

    /// Helper function to compute median absolute deviation
    fn median_absolute_deviation(data: &[f64], median: f64) -> f64 {
        let mut deviations: Vec<f64> = data.iter()
            .map(|x| (x - median).abs())
            .collect();

        deviations.sort_by(|a, b| match a.partial_cmp(b) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        if deviations.len().is_multiple_of(2) {
            (deviations[deviations.len() / 2 - 1] + deviations[deviations.len() / 2]) / 2.0
        } else {
            deviations[deviations.len() / 2]
        }
    }
}