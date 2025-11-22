//! Correlation utility functions

/// Rank data for Spearman correlation (handles ties properly)
/// This is the centralized ranking function that should be used throughout the correlation module.
/// Uses Schwartzian transform for optimal cache locality and performance.
pub fn rank_data(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    let mut indices: Vec<usize> = (0..n).collect();

    // Use Schwartzian transform for better cache locality
    indices.sort_by(|&i, &j| data[i].partial_cmp(&data[j]).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0; n];
    let mut i = 0;

    while i < n {
        let mut j = i;
        let current_val = data[indices[i]];

        // Find all ties
        while j < n && (data[indices[j]] - current_val).abs() < 1e-10 {
            j += 1;
        }

        // Calculate average rank (using 1-based indexing then convert)
        let avg_rank = ((i + j - 1) as f64 / 2.0) + 1.0;

        for k in i..j {
            ranks[indices[k]] = avg_rank;
        }
        i = j;
    }
    ranks
}

/// Ensure correlation matrix is positive semi-definite by clipping negative eigenvalues
pub fn ensure_positive_definite(matrix: &mut ndarray::Array2<f64>) {
    use crate::scientific::statistics::primitives::LinearAlgebra;

    // Get eigenvalues and eigenvectors
    let (eigenvalues, eigenvectors) = match LinearAlgebra::eigenvalue_decomposition(matrix) {
        Ok(result) => result,
        Err(_) => return, // If decomposition fails, leave matrix as-is
    };

    // Clip negative eigenvalues to a small positive value
    let epsilon = 1e-8;
    let mut clipped_eigenvalues = eigenvalues.clone();
    for eig in clipped_eigenvalues.iter_mut() {
        if *eig < epsilon {
            *eig = epsilon;
        }
    }

    // Reconstruct matrix: V * diag(λ+) * V^T
    let diag_eigen = ndarray::Array2::from_diag(&clipped_eigenvalues);
    let temp = eigenvectors.dot(&diag_eigen);
    *matrix = temp.dot(&eigenvectors.t().to_owned());
}