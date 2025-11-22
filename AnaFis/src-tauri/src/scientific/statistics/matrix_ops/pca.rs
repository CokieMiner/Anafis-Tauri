//! Principal Component Analysis (PCA)

use super::types::PcaResult;
use super::covariance::CovarianceOps;
use ndarray::{Array1, Array2, Axis};
use crate::scientific::statistics::primitives::LinearAlgebra;

/// PCA operations
pub struct PcaOps;

impl PcaOps {
    /// Principal Component Analysis
    pub fn pca(
        data: &[Vec<f64>],
        n_components: Option<usize>,
    ) -> Result<PcaResult, String> {
        if data.is_empty() {
            return Err("Empty data".to_string());
        }

        let n_samples = data.len();
        let n_features = data[0].len();

        if n_samples < 2 {
            return Err("Need at least 2 samples for PCA".to_string());
        }

        let n_components = n_components.unwrap_or(n_features.min(n_samples));

        // Compute covariance matrix
        let cov = CovarianceOps::covariance_matrix(data, 1)?; // Use sample covariance

        // Eigenvalue decomposition using BLAS/LAPACK
        let (eigenvalues_array, eigenvectors) = LinearAlgebra::eigenvalue_decomposition(&cov)?;

        // Convert eigenvalues to Vec<f64>
        let eigenvalues: Vec<f64> = eigenvalues_array.to_vec();

        // Sort eigenvalues and eigenvectors in descending order
        let mut eigen_pairs: Vec<(f64, Array1<f64>)> = eigenvalues
            .into_iter()
            .zip(eigenvectors.axis_iter(Axis(1)))
            .map(|(val, vec)| (val, vec.to_owned()))
            .collect();

        eigen_pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Extract top n_components
        let eigenvalues: Vec<f64> = eigen_pairs.iter().take(n_components).map(|(val, _)| *val).collect();
        let eigenvectors: Array2<f64> = Array2::from_shape_vec(
            (n_features, n_components),
            eigen_pairs
                .into_iter()
                .take(n_components)
                .flat_map(|(_, vec)| vec.to_vec())
                .collect(),
        )
        .map_err(|e| format!("Failed to create eigenvectors matrix: {}", e))?;

        // Compute explained variance ratio
        let total_variance: f64 = eigenvalues.iter().sum();
        let explained_variance_ratio: Vec<f64> = eigenvalues
            .iter()
            .map(|&ev| if total_variance > 0.0 { ev / total_variance } else { 0.0 })
            .collect();

        // Project data onto principal components
        let data_matrix = Array2::from_shape_vec(
            (n_samples, n_features),
            data.iter().flatten().cloned().collect(),
        )
        .map_err(|e| format!("Failed to create data matrix: {}", e))?;

        // Center the data
        let means = data_matrix.mean_axis(Axis(0)).ok_or("Failed to compute column means")?;
        let centered_data = &data_matrix - &means.insert_axis(Axis(0));

        let projected_data = centered_data.dot(&eigenvectors);

        Ok(PcaResult {
            eigenvalues,
            eigenvectors: eigenvectors
                .axis_iter(Axis(1))
                .map(|col| col.to_vec())
                .collect(),
            explained_variance_ratio,
            projected_data: projected_data
                .axis_iter(Axis(1))
                .map(|col| col.to_vec())
                .collect(),
            n_components,
        })
    }
}
