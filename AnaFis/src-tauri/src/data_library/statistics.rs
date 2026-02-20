// Statistical calculations for data sequences
use super::models::{DataSequence, SequenceStatistics};

/// Calculate statistics for a data sequence
pub fn calculate_statistics(sequence: &DataSequence) -> SequenceStatistics {
    let data = &sequence.data;
    let count = data.len();

    if count == 0 {
        return SequenceStatistics {
            count: 0,
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
            has_uncertainties: sequence.uncertainties.is_some(),
        };
    }

    // Calculate mean
    let sum: f64 = data.iter().sum();
    #[allow(
        clippy::cast_precision_loss,
        reason = "Count to f64 for mean calculation"
    )]
    let mean = sum / count as f64;

    // Calculate standard deviation
    let sum_sq_diff: f64 = data
        .iter()
        .map(|x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<f64>();

    #[allow(
        clippy::cast_precision_loss,
        reason = "Count to f64 for variance calculation"
    )]
    let variance = sum_sq_diff / count as f64;
    let std_dev = variance.sqrt();

    // Find min and max
    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    // Calculate median
    let mut sorted_data = data.clone();
    sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if count.is_multiple_of(2) {
        f64::midpoint(sorted_data[count / 2 - 1], sorted_data[count / 2])
    } else {
        sorted_data[count / 2]
    };

    SequenceStatistics {
        count,
        mean,
        std_dev,
        min,
        max,
        median,
        has_uncertainties: sequence.uncertainties.is_some(),
    }
}
