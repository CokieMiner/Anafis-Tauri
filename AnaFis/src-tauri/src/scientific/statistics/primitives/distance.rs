//! Distance and similarity metrics.
//!
//! This module provides various distance and similarity measures
//! commonly used in statistical analysis, machine learning, and data science.

/// Distance and similarity metrics
pub struct Distance;

/// Common distance metrics enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceMetric {
    /// Euclidean distance: sqrt(sum((x_i - y_i)^2))
    Euclidean,
    /// Manhattan distance: sum(|x_i - y_i|)
    Manhattan,
    /// Chebyshev distance: max(|x_i - y_i|)
    Chebyshev,
    /// Minkowski distance: (sum(|x_i - y_i|^p))^(1/p)
    Minkowski(f64),
    /// Cosine similarity: dot(x,y) / (||x|| * ||y||)
    Cosine,
    /// Standardized Euclidean distance with feature weights/scaling
    StandardizedEuclidean,
}

impl Distance {
    /// Calculate distance between two vectors using the specified metric
    ///
    /// # Arguments
    /// * `x` - First vector
    /// * `y` - Second vector
    /// * `metric` - Distance metric to use
    /// * `weights` - Optional weights for standardized distance (used for StandardizedEuclidean)
    ///
    /// # Returns
    /// Distance value
    pub fn distance(x: &[f64], y: &[f64], metric: DistanceMetric, weights: Option<&[f64]>) -> f64 {
        match metric {
            DistanceMetric::Euclidean => Self::euclidean(x, y),
            DistanceMetric::Manhattan => Self::manhattan(x, y),
            DistanceMetric::Chebyshev => Self::chebyshev(x, y),
            DistanceMetric::Minkowski(p) => Self::minkowski(x, y, p),
            DistanceMetric::Cosine => Self::cosine_distance(x, y),
            DistanceMetric::StandardizedEuclidean => Self::standardized_euclidean(x, y, weights),
        }
    }

    /// Euclidean distance between two vectors
    ///
    /// # Arguments
    /// * `x` - First vector
    /// * `y` - Second vector
    ///
    /// # Returns
    /// Euclidean distance: sqrt(sum((x_i - y_i)^2))
    pub fn euclidean(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() {
            return f64::INFINITY;
        }

        x.iter().zip(y.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Manhattan (L1) distance between two vectors
    ///
    /// # Arguments
    /// * `x` - First vector
    /// * `y` - Second vector
    ///
    /// # Returns
    /// Manhattan distance: sum(|x_i - y_i|)
    pub fn manhattan(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() {
            return f64::INFINITY;
        }

        x.iter().zip(y.iter())
            .map(|(a, b)| (a - b).abs())
            .sum()
    }

    /// Chebyshev (L∞) distance between two vectors
    ///
    /// # Arguments
    /// * `x` - First vector
    /// * `y` - Second vector
    ///
    /// # Returns
    /// Chebyshev distance: max(|x_i - y_i|)
    pub fn chebyshev(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() {
            return f64::INFINITY;
        }

        x.iter().zip(y.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max)
    }

    /// Minkowski distance between two vectors
    ///
    /// # Arguments
    /// * `x` - First vector
    /// * `y` - Second vector
    /// * `p` - Order of the Minkowski distance (p=1: Manhattan, p=2: Euclidean, p=∞: Chebyshev)
    ///
    /// # Returns
    /// Minkowski distance: (sum(|x_i - y_i|^p))^(1/p)
    pub fn minkowski(x: &[f64], y: &[f64], p: f64) -> f64 {
        if x.len() != y.len() {
            return f64::INFINITY;
        }

        if p == 1.0 {
            Self::manhattan(x, y)
        } else if p == 2.0 {
            Self::euclidean(x, y)
        } else if p.is_infinite() {
            Self::chebyshev(x, y)
        } else {
            x.iter().zip(y.iter())
                .map(|(a, b)| (a - b).abs().powf(p))
                .sum::<f64>()
                .powf(1.0 / p)
        }
    }

    /// Cosine distance between two vectors (1 - cosine similarity)
    ///
    /// # Arguments
    /// * `x` - First vector
    /// * `y` - Second vector
    ///
    /// # Returns
    /// Cosine distance: 1 - (dot(x,y) / (||x|| * ||y||))
    pub fn cosine_distance(x: &[f64], y: &[f64]) -> f64 {
        let similarity = Self::cosine_similarity(x, y);
        1.0 - similarity
    }

    /// Cosine similarity between two vectors
    ///
    /// # Arguments
    /// * `x` - First vector
    /// * `y` - Second vector
    ///
    /// # Returns
    /// Cosine similarity: dot(x,y) / (||x|| * ||y||)
    pub fn cosine_similarity(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() {
            return 0.0;
        }

        let dot_product: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
        let norm_x = x.iter().map(|a| a * a).sum::<f64>().sqrt();
        let norm_y = y.iter().map(|b| b * b).sum::<f64>().sqrt();

        if norm_x == 0.0 || norm_y == 0.0 {
            0.0
        } else {
            dot_product / (norm_x * norm_y)
        }
    }

    /// Standardized Euclidean distance with optional weights
    ///
    /// # Arguments
    /// * `x` - First vector
    /// * `y` - Second vector
    /// * `weights` - Optional weights (standard deviations for each dimension)
    ///
    /// # Returns
    /// Standardized Euclidean distance: sqrt(sum(((x_i - y_i) / w_i)^2))
    pub fn standardized_euclidean(x: &[f64], y: &[f64], weights: Option<&[f64]>) -> f64 {
        if x.len() != y.len() {
            return f64::INFINITY;
        }

        let default_weights = vec![1.0; x.len()];
        let weights = weights.unwrap_or(&default_weights);

        if weights.len() != x.len() {
            return f64::INFINITY;
        }

        x.iter().zip(y.iter()).zip(weights.iter())
            .map(|((a, b), w)| {
                if *w > 0.0 {
                    ((a - b) / w).powi(2)
                } else {
                    0.0 // Skip dimensions with zero weight
                }
            })
            .sum::<f64>()
            .sqrt()
    }

    /// Calculate pairwise distance matrix for a set of vectors
    ///
    /// # Arguments
    /// * `data` - Matrix where each row is a vector (n_samples x n_features)
    /// * `metric` - Distance metric to use
    ///
    /// # Returns
    /// Distance matrix (n_samples x n_samples)
    pub fn pairwise_distances(data: &[Vec<f64>], metric: DistanceMetric) -> Vec<Vec<f64>> {
        let n = data.len();
        let mut distances = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let dist = Self::distance(&data[i], &data[j], metric, None);
                distances[i][j] = dist;
                distances[j][i] = dist; // Symmetric matrix
            }
        }

        distances
    }

    /// Calculate distance matrix for a single vector (used in time series analysis)
    ///
    /// # Arguments
    /// * `data` - Input vector
    ///
    /// # Returns
    /// Distance matrix where element (i,j) = |data[i] - data[j]|
    pub fn distance_matrix(data: &[f64]) -> Vec<Vec<f64>> {
        let n = data.len();
        let mut dist = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                dist[i][j] = (data[i] - data[j]).abs();
            }
        }

        dist
    }

    /// Calculate centered distance matrix (used in distance correlation)
    ///
    /// # Arguments
    /// * `dist` - Input distance matrix
    ///
    /// # Returns
    /// Centered distance matrix
    pub fn center_distance_matrix(dist: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = dist.len();
        if n == 0 {
            return Vec::new();
        }

        let mut centered = vec![vec![0.0; n]; n];

        // Calculate row means
        let row_means: Vec<f64> = dist.iter()
            .map(|row| row.iter().sum::<f64>() / n as f64)
            .collect();

        // Calculate overall mean
        let overall_mean = row_means.iter().sum::<f64>() / n as f64;

        // Center the matrix
        for i in 0..n {
            for j in 0..n {
                centered[i][j] = dist[i][j] - row_means[i] - row_means[j] + overall_mean;
            }
        }

        centered
    }

    /// Calculate distance covariance between two distance matrices
    ///
    /// # Arguments
    /// * `dist_x` - Distance matrix for first variable
    /// * `dist_y` - Distance matrix for second variable
    ///
    /// # Returns
    /// Distance covariance
    pub fn distance_covariance(dist_x: &[Vec<f64>], dist_y: &[Vec<f64>]) -> f64 {
        let n = dist_x.len();
        if n != dist_y.len() || n == 0 {
            return 0.0;
        }

        let centered_x = Self::center_distance_matrix(dist_x);
        let centered_y = Self::center_distance_matrix(dist_y);

        let mut covariance = 0.0;
        for i in 0..n {
            for j in 0..n {
                covariance += centered_x[i][j] * centered_y[i][j];
            }
        }

        (covariance / (n as f64 * n as f64)).max(0.0).sqrt()
    }
}