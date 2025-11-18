use ndarray::{Array2, Array1, Axis};
use crate::scientific::statistics::comprehensive_analysis::layer4_primitives::*;
use crate::scientific::statistics::comprehensive_analysis::utils;

#[derive(Debug, Clone)]
pub struct Pca {
    pub components: Vec<Array1<f64>>,
    pub explained_variance: Vec<f64>,
    pub explained_variance_ratio: Vec<f64>,
    pub singular_values: Vec<f64>,
}

/// Matrix operations engine
pub struct MatrixOperationsEngine;

impl MatrixOperationsEngine {
    /// Compute covariance matrix
    pub fn covariance_matrix(data: &[Vec<f64>]) -> Result<Array2<f64>, String> {
        let n_obs = utils::validate_variable_lengths(data)?;

        // Convert data to ndarray format (n_observations x n_variables)
        let mut data_array = Array2::<f64>::zeros((n_obs, data.len()));
        for (j, var) in data.iter().enumerate() {
            for (i, &val) in var.iter().enumerate() {
                data_array[[i, j]] = val;
            }
        }

        // Use NdLinearAlgebra for efficient covariance computation
        NdLinearAlgebra::covariance_matrix(&data_array)
    }

    /// Principal component analysis
    pub fn pca(data: &[Vec<f64>], n_components: usize) -> Result<Pca, String> {
        let cov_matrix = Self::covariance_matrix(data)?;

        // Eigenvalue decomposition
        let (eigenvalues, eigenvectors) = LinearAlgebra::eigenvalue_decomposition(&cov_matrix)?;

        // Sort eigenvalues and eigenvectors in descending order
        let mut eigen_pairs: Vec<(f64, Array1<f64>)> = eigenvalues.iter()
            .zip(eigenvectors.axis_iter(Axis(1)))
            .map(|(&val, vec)| (val, vec.to_owned()))
            .collect();

        eigen_pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let n_comps = n_components.min(eigen_pairs.len());
        let explained_variance: Vec<f64> = eigen_pairs.iter()
            .take(n_comps)
            .map(|(val, _)| *val)
            .collect();

        let explained_variance_ratio: Vec<f64> = explained_variance.iter()
            .map(|&var| var / eigenvalues.sum())
            .collect();

        let singular_values: Vec<f64> = explained_variance.iter().map(|x| x.sqrt()).collect();

        let components: Vec<Array1<f64>> = eigen_pairs.into_iter()
            .take(n_comps)
            .map(|(_, vec)| vec)
            .collect();

        Ok(Pca {
            components,
            explained_variance,
            explained_variance_ratio,
            singular_values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_covariance_matrix_basic() {
        let data = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![2.0, 3.0, 4.0, 5.0],
            vec![3.0, 4.0, 5.0, 6.0],
        ];

        let result = MatrixOperationsEngine::covariance_matrix(&data);
        assert!(result.is_ok());

        let cov = result.unwrap();
        assert_eq!(cov.nrows(), 3);
        assert_eq!(cov.ncols(), 3);

        // Check that diagonal elements are positive (variances)
        for i in 0..3 {
            assert!(cov[[i, i]] > 0.0);
        }
    }

    #[test]
    fn test_pca_basic() {
        let data = vec![
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![2.0, 3.0, 4.0, 5.0, 6.0],
            vec![3.0, 4.0, 5.0, 6.0, 7.0],
        ];

        let result = MatrixOperationsEngine::pca(&data, 2);
        assert!(result.is_ok());

        let pca = result.unwrap();
        assert_eq!(pca.components.len(), 2);
        assert_eq!(pca.explained_variance.len(), 2);
        assert_eq!(pca.explained_variance_ratio.len(), 2);
        assert_eq!(pca.singular_values.len(), 2);

        // Check that explained variance ratios sum to less than or equal to 1
        let total_ratio: f64 = pca.explained_variance_ratio.iter().sum();
        assert!(total_ratio <= 1.0);
    }
}
