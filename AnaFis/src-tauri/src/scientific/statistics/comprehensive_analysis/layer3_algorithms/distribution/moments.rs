//! Statistical moments computation

use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::UnifiedStats;

/// Compute statistical moments (mean, variance, skewness, kurtosis)
pub fn moments(data: &[f64]) -> Result<(f64, f64, f64, f64), String> {
    if data.is_empty() {
        return Err("Cannot compute moments of empty dataset".to_string());
    }

    let n = data.len() as f64;
    let mean = UnifiedStats::mean(data);
    let variance = UnifiedStats::variance(data);
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return Ok((mean, variance, 0.0, 0.0)); // No skewness/kurtosis for constant data
    }

    let skewness =
        data.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum::<f64>() / n;

    let kurtosis =
        data.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum::<f64>() / n - 3.0; // Excess kurtosis

    Ok((mean, variance, skewness, kurtosis))
}

/// Rank transformation for statistical tests
pub fn rank_transformation(data: &[f64]) -> Vec<f64> {
    let mut indexed_data: Vec<(f64, usize)> = data.iter().enumerate()
        .map(|(i, &x)| (x, i))
        .collect();

    indexed_data.sort_by(|a, b| match a.0.partial_cmp(&b.0) {
        Some(ord) => ord,
        None => std::cmp::Ordering::Equal,
    });

    let mut ranks = vec![0.0; data.len()];
    let mut _current_rank = 1.0;

    let mut i = 0;
    while i < indexed_data.len() {
        let start = i;
        let mut end = i;

        // Handle ties
        while end + 1 < indexed_data.len() &&
              (indexed_data[end + 1].0 - indexed_data[i].0).abs() < 1e-10 {
            end += 1;
        }

        let avg_rank = (start + 1 + end + 1) as f64 / 2.0;

        for j in start..=end {
            ranks[indexed_data[j].1] = avg_rank;
        }

        _current_rank = end as f64 + 2.0;
        i = end + 1;
    }

    ranks
}