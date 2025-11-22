//! Design Matrix Builder Module
//!
//! This module provides a centralized way to construct design matrices for regression
//! analysis, eliminating the duplication found across Prophet, Time Series, and
//! Imputation modules.

use ndarray::Array2;
use std::f64::consts::PI;

/// Options for design matrix construction
#[derive(Debug, Clone)]
pub struct DesignMatrixOptions {
    /// Whether to add an intercept column (column of 1s)
    pub add_intercept: bool,
    /// Whether to add polynomial features
    pub polynomial_degree: Option<usize>,
    /// Whether to add Fourier series terms for seasonality
    pub fourier_terms: Option<FourierOptions>,
    /// Whether to add time trend terms
    pub time_trend: Option<TimeTrendOptions>,
}

impl Default for DesignMatrixOptions {
    fn default() -> Self {
        Self {
            add_intercept: true,
            polynomial_degree: None,
            fourier_terms: None,
            time_trend: None,
        }
    }
}

/// Options for Fourier series terms
#[derive(Debug, Clone)]
pub struct FourierOptions {
    /// Number of harmonics to include
    pub n_harmonics: usize,
    /// Period of the seasonality
    pub period: f64,
}

/// Options for time trend terms
#[derive(Debug, Clone)]
pub struct TimeTrendOptions {
    /// Changepoints for piecewise linear trends
    pub changepoints: Vec<usize>,
    /// Base timestamps corresponding to the data indices
    pub base_timestamps: Vec<f64>,
    /// Index of the column containing time values (if None, uses row index)
    pub time_column_index: Option<usize>,
}

/// Encoding strategy for categorical variables
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CategoricalEncoding {
    /// Reference cell coding (0/1). First level is reference.
    /// Good for interpreting coefficients relative to a baseline.
    Reference,
    /// Effect coding (1/0/-1). Sum of effects is zero.
    /// REQUIRED for Type III Sums of Squares in ANOVA.
    Effect,
}

/// Design Matrix Builder
/// Centralizes the construction of regression design matrices
pub struct DesignMatrixBuilder;

impl DesignMatrixBuilder {
    /// Build a design matrix from raw data with specified options
    ///
    /// # Arguments
    /// * `data` - Raw input data (n_samples x n_features)
    /// * `options` - Configuration for matrix construction
    ///
    /// # Returns
    /// Design matrix ready for regression
    pub fn build(data: &[Vec<f64>], options: &DesignMatrixOptions) -> Result<Array2<f64>, String> {
        if data.is_empty() || data[0].is_empty() {
            return Err("Empty data provided".to_string());
        }

        let n_samples = data.len();
        let n_features = data[0].len();

        // Validate data dimensions
        for row in data {
            if row.len() != n_features {
                return Err("Inconsistent row lengths in data".to_string());
            }
        }

        let mut matrices = Vec::new();

        // Add intercept if requested
        if options.add_intercept {
            let intercept = Array2::ones((n_samples, 1));
            matrices.push(intercept);
        }

        // Add original features
        let original_matrix = Self::vec_to_array2(data);
        matrices.push(original_matrix);

        // Add polynomial features if requested
        if let Some(degree) = options.polynomial_degree {
            if degree > 1 {
                let polynomial_matrix = Self::add_polynomial_features(data, degree)?;
                matrices.push(polynomial_matrix);
            }
        }

        // Add Fourier terms if requested
        if let Some(fourier_opts) = &options.fourier_terms {
            let fourier_matrix = Self::add_fourier_features(data, fourier_opts)?;
            matrices.push(fourier_matrix);
        }

        // Add time trend terms if requested
        if let Some(trend_opts) = &options.time_trend {
            let trend_matrix = Self::add_time_trend_features(data, trend_opts)?;
            matrices.push(trend_matrix);
        }

        // Concatenate all matrices horizontally
        Self::concatenate_matrices_horizontally(&matrices)
    }

    /// Build design matrix for simple linear regression (intercept + linear terms)
    pub fn build_linear(data: &[Vec<f64>]) -> Result<Array2<f64>, String> {
        Self::build(data, &DesignMatrixOptions {
            add_intercept: true,
            polynomial_degree: None,
            fourier_terms: None,
            time_trend: None,
        })
    }

    /// Build design matrix for polynomial regression
    pub fn build_polynomial(data: &[Vec<f64>], degree: usize) -> Result<Array2<f64>, String> {
        Self::build(data, &DesignMatrixOptions {
            add_intercept: true,
            polynomial_degree: Some(degree),
            fourier_terms: None,
            time_trend: None,
        })
    }

    /// Build design matrix for seasonal regression with Fourier terms
    pub fn build_seasonal(data: &[Vec<f64>], n_harmonics: usize, period: f64) -> Result<Array2<f64>, String> {
        let options = DesignMatrixOptions {
            add_intercept: true,
            polynomial_degree: None,
            fourier_terms: Some(FourierOptions { n_harmonics, period }),
            time_trend: None,
        };

        if data.is_empty() || data[0].is_empty() {
            return Err("Empty data provided".to_string());
        }

        let n_samples = data.len();

        let mut matrices = Vec::new();

        // Add intercept if requested
        if options.add_intercept {
            let intercept = Array2::ones((n_samples, 1));
            matrices.push(intercept);
        }

        // For seasonal, we don't add original features, just Fourier terms
        if let Some(fourier_opts) = &options.fourier_terms {
            let fourier_matrix = Self::add_fourier_features(data, fourier_opts)?;
            matrices.push(fourier_matrix);
        }

        // Concatenate all matrices horizontally
        Self::concatenate_matrices_horizontally(&matrices)
    }

    /// Build design matrix for trend analysis with changepoints
    pub fn build_trend(data: &[Vec<f64>], changepoints: &[usize], base_timestamps: &[f64]) -> Result<Array2<f64>, String> {
        Self::build(data, &DesignMatrixOptions {
            add_intercept: true,
            polynomial_degree: None,
            fourier_terms: None,
            time_trend: Some(TimeTrendOptions {
                changepoints: changepoints.to_vec(),
                base_timestamps: base_timestamps.to_vec(),
                time_column_index: None,
            }),
        })
    }

    /// Convert Vec<Vec<f64>> to Array2<f64>
    fn vec_to_array2(data: &[Vec<f64>]) -> Array2<f64> {
        let n_rows = data.len();
        let n_cols = data[0].len();
        let flat: Vec<f64> = data.iter().flatten().cloned().collect();
        Array2::from_shape_vec((n_rows, n_cols), flat).unwrap()
    }

    /// Add polynomial features to the design matrix
    fn add_polynomial_features(data: &[Vec<f64>], degree: usize) -> Result<Array2<f64>, String> {
        if data.is_empty() || data[0].is_empty() {
            return Err("Empty data for polynomial features".to_string());
        }

        let n_samples = data.len();
        let n_features = data[0].len();
        let mut polynomial_features = Vec::new();

        // For each feature, add polynomial terms up to the specified degree
        for feature_idx in 0..n_features {
            for deg in 2..=degree {
                let mut column = Vec::with_capacity(n_samples);
                for row in data {
                    let value = row[feature_idx];
                    column.push(value.powi(deg as i32));
                }
                polynomial_features.push(column);
            }
        }

        if polynomial_features.is_empty() {
            return Ok(Array2::zeros((n_samples, 0)));
        }

        let n_poly_features = polynomial_features.len();
        let flat: Vec<f64> = polynomial_features.into_iter().flatten().collect();
        Array2::from_shape_vec((n_samples, n_poly_features), flat)
            .map_err(|e| format!("Failed to create polynomial features matrix: {:?}", e))
    }

    /// Add Fourier series features for seasonality
    fn add_fourier_features(data: &[Vec<f64>], options: &FourierOptions) -> Result<Array2<f64>, String> {
        if data.is_empty() {
            return Err("Empty data for Fourier features".to_string());
        }

        // Assume the first column contains time/timestamps
        // If no time column, use row indices as time
        let n_samples = data.len();
        let mut fourier_features = Vec::new();

        for (i, row) in data.iter().enumerate() {
            // Get time value (use first column if available, otherwise use index)
            let t = if !row.is_empty() && row[0].is_finite() {
                row[0]
            } else {
                i as f64
            };

            // Normalize time by period
            let t_norm = 2.0 * PI * t / options.period;

            // Add sine and cosine terms for each harmonic
            for k in 1..=options.n_harmonics {
                let k = k as f64;
                fourier_features.push((k * t_norm).sin());
                fourier_features.push((k * t_norm).cos());
            }
        }

        let n_fourier_features = options.n_harmonics * 2;
        Array2::from_shape_vec((n_samples, n_fourier_features), fourier_features)
            .map_err(|e| format!("Failed to create Fourier features matrix: {:?}", e))
    }

    /// Add piecewise linear trend features with changepoints
    fn add_time_trend_features(data: &[Vec<f64>], options: &TimeTrendOptions) -> Result<Array2<f64>, String> {
        if data.is_empty() {
            return Err("Empty data for trend features".to_string());
        }

        let n_samples = data.len();
        let n_changepoints = options.changepoints.len();
        let mut trend_features = Vec::new();
        let time_col = options.time_column_index.unwrap_or(0);

        // Base trend (linear in time)
        for (i, row) in data.iter().enumerate() {
            let t = if !row.is_empty() && row.len() > time_col && row[time_col].is_finite() {
                row[time_col]
            } else {
                i as f64
            };
            trend_features.push(t);
        }

        // Changepoint indicators
        for &cp_idx in &options.changepoints {
            if cp_idx >= options.base_timestamps.len() {
                return Err(format!("Changepoint index {} out of bounds", cp_idx));
            }

            let cp_time = options.base_timestamps[cp_idx];

            for (i, row) in data.iter().enumerate() {
                let t = if !row.is_empty() && row.len() > time_col && row[time_col].is_finite() {
                    row[time_col]
                } else {
                    i as f64
                };

                if t >= cp_time {
                    trend_features.push(t - cp_time);
                } else {
                    trend_features.push(0.0);
                }
            }
        }

        let n_trend_features = 1 + n_changepoints; // base trend + changepoint terms
        Array2::from_shape_vec((n_samples, n_trend_features), trend_features)
            .map_err(|e| format!("Failed to create trend features matrix: {:?}", e))
    }

    /// Concatenate matrices horizontally
    fn concatenate_matrices_horizontally(matrices: &[Array2<f64>]) -> Result<Array2<f64>, String> {
        if matrices.is_empty() {
            return Err("No matrices to concatenate".to_string());
        }

        let n_rows = matrices[0].nrows();
        for matrix in matrices {
            if matrix.nrows() != n_rows {
                return Err("All matrices must have the same number of rows".to_string());
            }
        }

        let total_cols: usize = matrices.iter().map(|m| m.ncols()).sum();
        let mut result = Array2::zeros((n_rows, total_cols));

        let mut col_offset = 0;
        for matrix in matrices {
            let cols = matrix.ncols();
            result.slice_mut(ndarray::s![.., col_offset..col_offset + cols])
                .assign(matrix);
            col_offset += cols;
        }

        Ok(result)
    }

    /// Build design matrix for factorial ANOVA with proper categorical encoding
    ///
    /// This creates a design matrix suitable for ANOVA with Type III Sums of Squares.
    /// Uses Effect Coding by default for proper ANOVA calculations.
    ///
    /// # Arguments
    /// * `factor_data` - Vector of factor vectors, each containing categorical values
    /// * `encoding` - Categorical encoding strategy (Reference or Effect)
    ///
    /// # Returns
    /// Design matrix with intercept and factor columns
    pub fn build_factorial_anova(
        factor_data: &[Vec<String>],
        encoding: CategoricalEncoding,
    ) -> Result<Array2<f64>, String> {
        if factor_data.is_empty() {
            return Err("No factor data provided".to_string());
        }

        let n_samples = factor_data[0].len();
        for factor in factor_data {
            if factor.len() != n_samples {
                return Err("All factors must have the same number of observations".to_string());
            }
        }

        // Start with intercept column (all ones)
        let mut design_matrix = Array2::ones((n_samples, 1));

        // Add each factor's columns
        for factor in factor_data {
            let factor_matrix = Self::encode_categorical_factor(factor, encoding)?;
            design_matrix = Self::concatenate_matrices_horizontally(&[design_matrix, factor_matrix])?;
        }

        Ok(design_matrix)
    }

    /// Encode a single categorical factor using the specified encoding strategy
    fn encode_categorical_factor(
        factor: &[String],
        encoding: CategoricalEncoding,
    ) -> Result<Array2<f64>, String> {
        if factor.is_empty() {
            return Err("Empty factor data".to_string());
        }

        // Get unique levels and sort them for consistent ordering
        let mut levels: Vec<String> = factor.iter().cloned().collect::<std::collections::HashSet<_>>()
            .into_iter().collect();
        levels.sort();

        let n_samples = factor.len();
        let n_levels = levels.len();

        if n_levels < 2 {
            return Err("Factor must have at least 2 levels".to_string());
        }

        // Create level-to-index mapping
        let level_to_idx: std::collections::HashMap<_, _> = levels.iter()
            .enumerate()
            .map(|(i, level)| (level.clone(), i))
            .collect();

        match encoding {
            CategoricalEncoding::Reference => {
                // Reference coding: n_levels - 1 columns
                // First level is reference (all zeros in its columns)
                let n_cols = n_levels - 1;
                let mut matrix = Array2::zeros((n_samples, n_cols));

                for (row, level) in factor.iter().enumerate() {
                    if let Some(&level_idx) = level_to_idx.get(level) {
                        // Skip reference level (index 0)
                        if level_idx > 0 {
                            matrix[[row, level_idx - 1]] = 1.0;
                        }
                    } else {
                        return Err(format!("Unknown factor level: {}", level));
                    }
                }

                Ok(matrix)
            }

            CategoricalEncoding::Effect => {
                // Effect coding: n_levels - 1 columns
                // Each level gets +1, reference level gets -1
                // This ensures sum of effects is zero (required for Type III SS)
                let n_cols = n_levels - 1;
                let mut matrix = Array2::zeros((n_samples, n_cols));

                for (row, level) in factor.iter().enumerate() {
                    if let Some(&level_idx) = level_to_idx.get(level) {
                        if level_idx == 0 {
                            // Reference level: -1 in all columns
                            for col in 0..n_cols {
                                matrix[[row, col]] = -1.0;
                            }
                        } else {
                            // Non-reference level: +1 in its column, 0 elsewhere
                            matrix[[row, level_idx - 1]] = 1.0;
                        }
                    } else {
                        return Err(format!("Unknown factor level: {}", level));
                    }
                }

                Ok(matrix)
            }
        }
    }
}