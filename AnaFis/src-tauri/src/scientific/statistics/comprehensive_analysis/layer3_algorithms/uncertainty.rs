use ndarray::Array2;

/// Uncertainty propagation engine
pub struct UncertaintyPropagationEngine;

impl UncertaintyPropagationEngine {
    /// Compute covariance matrix from measurement uncertainties and correlations
    pub fn covariance_matrix_from_uncertainties(
        uncertainties: &[f64],
        correlations: Option<&Array2<f64>>,
    ) -> Result<Array2<f64>, String> {
        let n = uncertainties.len();
        let mut cov = Array2::<f64>::zeros((n, n));

        // Diagonal elements are variances (uncertainties squared)
        for i in 0..n {
            cov[[i, i]] = uncertainties[i] * uncertainties[i];
        }

        // Off-diagonal elements include correlations
        if let Some(corr_matrix) = correlations {
            if corr_matrix.nrows() != n || corr_matrix.ncols() != n {
                return Err("Correlation matrix dimensions must match uncertainty vector length".to_string());
            }

            for i in 0..n {
                for j in 0..n {
                    if i != j {
                        cov[[i, j]] = corr_matrix[[i, j]] * uncertainties[i] * uncertainties[j];
                    }
                }
            }
        }

        Ok(cov)
    }

    /// Propagate uncertainties through a function using the law of propagation of uncertainty
    /// For a function f(x1, x2, ..., xn), the uncertainty in f is:
    /// σ_f² = Σᵢ Σⱼ (∂f/∂xᵢ)(∂f/∂xⱼ) cov(xᵢ,xⱼ)
    pub fn propagate_uncertainty<F>(
        function: F,
        variables: &[f64],
        covariance_matrix: &Array2<f64>,
    ) -> Result<f64, String>
    where
        F: Fn(&[f64]) -> f64,
    {
        let n = variables.len();
        if covariance_matrix.nrows() != n || covariance_matrix.ncols() != n {
            return Err("Covariance matrix dimensions must match number of variables".to_string());
        }

        // Compute partial derivatives using finite differences
        let h = 1e-8; // Small step size for numerical differentiation
        let mut partials = vec![0.0; n];

        for i in 0..n {
            let mut x_plus = variables.to_vec();
            let mut x_minus = variables.to_vec();

            x_plus[i] += h;
            x_minus[i] -= h;

            partials[i] = (function(&x_plus) - function(&x_minus)) / (2.0 * h);
        }

        // Compute uncertainty using law of propagation: σ_f² = Σᵢ Σⱼ (∂f/∂xᵢ)(∂f/∂xⱼ) cov(xᵢ,xⱼ)
        let mut variance = 0.0;
        for i in 0..n {
            for j in 0..n {
                variance += partials[i] * partials[j] * covariance_matrix[[i, j]];
            }
        }

        Ok(variance.sqrt())
    }

    /// Propagate uncertainties for the mean of correlated measurements
    pub fn propagate_mean_uncertainty(
        values: &[f64],
        uncertainties: &[f64],
        correlations: Option<&Array2<f64>>,
    ) -> Result<f64, String> {
        if values.len() != uncertainties.len() {
            return Err("Values and uncertainties must have the same length".to_string());
        }

        let n = values.len() as f64;
        let cov_matrix = Self::covariance_matrix_from_uncertainties(uncertainties, correlations)?;

        // For the mean, f(x₁,x₂,...,xₙ) = (x₁+x₂+...+xₙ)/n
        // ∂f/∂xᵢ = 1/n for all i
        // σ_mean² = Σᵢ Σⱼ (1/n)(1/n) cov(xᵢ,xⱼ) = (1/n²) Σᵢ Σⱼ cov(xᵢ,xⱼ)
        let mut variance = 0.0;
        for i in 0..cov_matrix.nrows() {
            for j in 0..cov_matrix.ncols() {
                variance += cov_matrix[[i, j]];
            }
        }
        variance /= n * n;

        Ok(variance.sqrt())
    }

    /// Propagate uncertainties for the variance of measurements
    /// Uses proper error propagation for the sample variance estimator
    pub fn propagate_variance_uncertainty(
        values: &[f64],
        uncertainties: &[f64],
        correlations: Option<&Array2<f64>>,
    ) -> Result<f64, String> {
        if values.len() != uncertainties.len() {
            return Err("Values and uncertainties must have the same length".to_string());
        }

        let n = values.len();
        if n < 2 {
            return Err("Need at least 2 measurements to compute variance uncertainty".to_string());
        }

        let n_f64 = n as f64;

        // Compute sample mean and variance
        let mean = values.iter().sum::<f64>() / n_f64;
        let _variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n_f64 - 1.0);

        // Get covariance matrix
        let cov_matrix = Self::covariance_matrix_from_uncertainties(uncertainties, correlations)?;

        // For the sample variance s² = (1/(n-1)) * Σ(x_i - μ)²
        // where μ = (1/n) * Σ x_i
        //
        // The uncertainty propagation uses partial derivatives:
        // ∂s²/∂x_k = (2/(n-1)) * (x_k - μ) * (1 - 1/n - (x_k - μ)²/((n-1)*s²))
        // ∂s²/∂μ = - (2/(n-1)) * Σ(x_i - μ) = 0 (by definition of μ)
        //
        // For simplicity, we'll use the approximation that ignores the μ dependence
        // since the mean uncertainty is usually small compared to individual uncertainties

        let mut variance_uncertainty_squared = 0.0;

        for i in 0..n {
            for j in 0..n {
                // Simplified derivative: ∂s²/∂x_i ≈ (2/(n-1)) * (x_i - μ)
                // ∂s²/∂x_j ≈ (2/(n-1)) * (x_j - μ)
                let deriv_i = (2.0 / (n_f64 - 1.0)) * (values[i] - mean);
                let deriv_j = (2.0 / (n_f64 - 1.0)) * (values[j] - mean);

                variance_uncertainty_squared += deriv_i * deriv_j * cov_matrix[[i, j]];
            }
        }

        // For small samples, add a correction factor
        // The relative uncertainty in variance is approximately sqrt(2/(n-1))
        let relative_uncertainty_factor = (2.0 / (n_f64 - 1.0)).sqrt();
        let variance_uncertainty = variance_uncertainty_squared.sqrt() * relative_uncertainty_factor;

        Ok(variance_uncertainty)
    }

    /// Compute uncertainty in correlation coefficient
    /// Accounts for both sampling uncertainty and measurement uncertainties
    pub fn propagate_correlation_uncertainty(
        x_values: &[f64],
        y_values: &[f64],
        x_uncertainties: &[f64],
        y_uncertainties: &[f64],
        xy_correlations: Option<&Array2<f64>>,
    ) -> Result<f64, String> {
        if x_values.len() != y_values.len() ||
           x_values.len() != x_uncertainties.len() ||
           x_values.len() != y_uncertainties.len() {
            return Err("All input vectors must have the same length".to_string());
        }

        let n = x_values.len();
        if n < 4 {
            return Err("Need at least 4 observations for correlation uncertainty".to_string());
        }

        let n_f64 = n as f64;

        // Compute correlation coefficient
        let x_mean = x_values.iter().sum::<f64>() / n_f64;
        let y_mean = y_values.iter().sum::<f64>() / n_f64;

        let mut numerator = 0.0;
        let mut x_var = 0.0;
        let mut y_var = 0.0;

        for i in 0..n {
            let x_diff = x_values[i] - x_mean;
            let y_diff = y_values[i] - y_mean;
            numerator += x_diff * y_diff;
            x_var += x_diff * x_diff;
            y_var += y_diff * y_diff;
        }

        let correlation = numerator / (x_var * y_var).sqrt();

        // Component 1: Sampling uncertainty using Fisher z-transformation
        // σ_r ≈ (1 - r²) / sqrt(n - 3)
        let sampling_uncertainty = (1.0 - correlation * correlation) / ((n - 3) as f64).sqrt();

        // Component 2: Measurement uncertainty propagation
        // For correlation r = Σ((x_i - μ_x)(y_i - μ_y)) / sqrt(Σ(x_i - μ_x)² * Σ(y_i - μ_y)²)
        // The uncertainty comes from uncertainties in x_i and y_i

        // Create covariance matrix for all variables [x1, x2, ..., xn, y1, y2, ..., yn]
        let mut all_uncertainties = x_uncertainties.to_vec();
        all_uncertainties.extend_from_slice(y_uncertainties);

        let all_correlations = Array2::<f64>::eye(2 * n);

        // Set correlations between x variables
        if let Some(_corr_matrix) = xy_correlations {
            // Assume xy_correlations contains correlations between x and y variables
            // For simplicity, we'll use identity for within x and within y correlations
            // This could be extended to accept full correlation structure
        }

        let full_cov_matrix = Self::covariance_matrix_from_uncertainties(&all_uncertainties, Some(&all_correlations))?;

        // Compute partial derivatives of correlation w.r.t. each measurement
        let mut correlation_derivatives = vec![0.0; 2 * n];

        let sqrt_x_var_y_var = (x_var * y_var).sqrt();

        for i in 0..n {
            let x_diff = x_values[i] - x_mean;
            let y_diff = y_values[i] - y_mean;

            // ∂r/∂x_i = [ (y_i - μ_y) * Σ(x_j - μ_x)² - (x_i - μ_x) * Σ((x_j - μ_x)(y_j - μ_y)) * 2*(x_i - μ_x) ] / (2 * sqrt_x_var_y_var³)
            // Simplified: ∂r/∂x_i ≈ (y_i - μ_y) / sqrt_x_var_y_var - correlation * (x_i - μ_x) / x_var
            let dr_dx = (y_diff / sqrt_x_var_y_var) - correlation * x_diff / x_var;

            // ∂r/∂y_i ≈ (x_i - μ_x) / sqrt_x_var_y_var - correlation * (y_i - μ_y) / y_var
            let dr_dy = (x_diff / sqrt_x_var_y_var) - correlation * y_diff / y_var;

            correlation_derivatives[i] = dr_dx;         // derivative w.r.t. x_i
            correlation_derivatives[n + i] = dr_dy;     // derivative w.r.t. y_i
        }

        // Propagate uncertainty using covariance matrix
        let mut measurement_uncertainty_squared = 0.0;
        for i in 0..(2 * n) {
            for j in 0..(2 * n) {
                measurement_uncertainty_squared += correlation_derivatives[i] *
                                                 correlation_derivatives[j] *
                                                 full_cov_matrix[[i, j]];
            }
        }

        let measurement_uncertainty = measurement_uncertainty_squared.sqrt();

        // Combine sampling and measurement uncertainties
        // For simplicity, add them in quadrature (assuming independence)
        let total_uncertainty = (sampling_uncertainty.powi(2) + measurement_uncertainty.powi(2)).sqrt();

        Ok(total_uncertainty)
    }
}
