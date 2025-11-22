//! Multiple testing correction methods
//!
//! This module provides methods for adjusting p-values for multiple comparisons.

/// Apply Bonferroni multiple testing correction
pub fn bonferroni_correction(p_values: &[f64]) -> Vec<f64> {
    let m = p_values.len() as f64;
    p_values.iter().map(|&p| (p * m).min(1.0)).collect()
}

/// Apply Benjamini-Hochberg multiple testing correction (FDR control)
pub fn benjamini_hochberg_correction(p_values: &[f64]) -> Vec<f64> {
    let m = p_values.len();
    if m == 0 {
        return Vec::new();
    }

    let mut indexed_p: Vec<(f64, usize)> = p_values.iter().cloned().enumerate().map(|(i, p)| (p, i)).collect();
    indexed_p.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut adjusted_p = vec![0.0; m];
    let mut prev_adjusted = 1.0;

    for (rank, &(p, original_idx)) in indexed_p.iter().enumerate().rev() {
        let rank_f = (rank + 1) as f64;
        let adjusted = (p * m as f64 / rank_f).min(prev_adjusted);
        adjusted_p[original_idx] = adjusted;
        prev_adjusted = adjusted;
    }

    adjusted_p
}
