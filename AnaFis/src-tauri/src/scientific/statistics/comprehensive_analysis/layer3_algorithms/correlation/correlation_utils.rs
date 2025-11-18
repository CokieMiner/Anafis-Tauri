//! Correlation utility functions

use ndarray::Array2;

/// Correlation utility functions
pub fn rank_data(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    let mut indexed: Vec<(f64, usize)> = data.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    indexed.sort_by(|a, b| a.0.total_cmp(&b.0));

    let mut ranks = vec![0.0; n];
    let mut i = 0;

    while i < n {
        let mut j = i;
        while j < n && indexed[j].0 == indexed[i].0 {
            j += 1;
        }

        // Average rank for tied values
        let avg_rank = (i + j - 1) as f64 / 2.0 + 1.0;
        for k in i..j {
            ranks[indexed[k].1] = avg_rank;
        }

        i = j;
    }

    ranks
}

/// Ensure correlation matrix is positive semi-definite using Higham's algorithm for small matrices or eigenvalue clipping for large matrices
pub fn ensure_positive_definite(matrix: &mut Array2<f64>) {
    let n = matrix.nrows();

    // Check if already PD by examining eigenvalues
    let (eigenvalues, _) = crate::scientific::statistics::comprehensive_analysis::layer4_primitives::LinearAlgebra::eigenvalue_decomposition(matrix)
        .unwrap_or_else(|_| (ndarray::Array1::zeros(n), ndarray::Array2::eye(n)));
    let min_eigenvalue = eigenvalues.iter().cloned().fold(f64::INFINITY, f64::min);

    // If all eigenvalues are non-negative (within tolerance), matrix is PD
    if min_eigenvalue >= -1e-10 {
        return;
    }

    // For large matrices, use simple eigenvalue clipping for efficiency
    if n > 100 {
        clip_negative_eigenvalues(matrix);
    } else {
        // Apply Higham's algorithm to find nearest correlation matrix
        higham_nearest_correlation_matrix(matrix);
    }
}

/// Higham's algorithm for finding the nearest correlation matrix
fn higham_nearest_correlation_matrix(matrix: &mut Array2<f64>) {
    let n = matrix.nrows();
    let max_iter = 100;
    let tolerance = 1e-8;

    for _ in 0..max_iter {
        // Step 1: Eigenvalue decomposition
        let (eigenvalues, eigenvectors) = crate::scientific::statistics::comprehensive_analysis::layer4_primitives::LinearAlgebra::eigenvalue_decomposition(matrix)
            .unwrap_or_else(|_| (ndarray::Array1::zeros(n), ndarray::Array2::eye(n)));

        // Step 2: Clip negative eigenvalues to small positive value
        let mut eigenvalues_fixed = eigenvalues.clone();
        for ev in eigenvalues_fixed.iter_mut() {
            if *ev < tolerance {
                *ev = tolerance;
            }
        }

        // Step 3: Reconstruct matrix
        let eigenvals_diag = ndarray::Array2::from_diag(&eigenvalues_fixed);
        let temp = eigenvectors.dot(&eigenvals_diag);
        let new_matrix = temp.dot(&eigenvectors.t());

        // Step 4: Project back to correlation matrix (unit diagonal)
        for i in 0..n {
            matrix[[i, i]] = 1.0;
            for j in (i + 1)..n {
                let val = new_matrix[[i, j]].clamp(-1.0, 1.0); // Ensure valid correlation range
                matrix[[i, j]] = val;
                matrix[[j, i]] = val;
            }
        }

        // Check convergence
        let frobenius_norm = (0..n).map(|i| (new_matrix[[i, i]] - 1.0).powi(2)).sum::<f64>().sqrt();
        if frobenius_norm < tolerance {
            break;
        }
    }
}

/// Simple eigenvalue clipping for large matrices to ensure positive semi-definiteness
fn clip_negative_eigenvalues(matrix: &mut Array2<f64>) {
    let n = matrix.nrows();

    // Eigenvalue decomposition
    let (eigenvalues, eigenvectors) = crate::scientific::statistics::comprehensive_analysis::layer4_primitives::LinearAlgebra::eigenvalue_decomposition(matrix)
        .unwrap_or_else(|_| (ndarray::Array1::zeros(n), ndarray::Array2::eye(n)));

    // Clip negative eigenvalues to a small positive value
    let mut eigenvalues_fixed = eigenvalues.clone();
    for ev in eigenvalues_fixed.iter_mut() {
        if *ev < 1e-8 {
            *ev = 1e-8;
        }
    }

    // Reconstruct matrix
    let eigenvals_diag = ndarray::Array2::from_diag(&eigenvalues_fixed);
    let temp = eigenvectors.dot(&eigenvals_diag);
    let new_matrix = temp.dot(&eigenvectors.t());

    // Project back to correlation matrix (unit diagonal and valid range)
    for i in 0..n {
        matrix[[i, i]] = 1.0;
        for j in (i + 1)..n {
            let val = new_matrix[[i, j]].clamp(-1.0, 1.0);
            matrix[[i, j]] = val;
            matrix[[j, i]] = val;
        }
    }
}