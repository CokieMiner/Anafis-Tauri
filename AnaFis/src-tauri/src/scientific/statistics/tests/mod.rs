//! Comprehensive test suite for AnaFis statistics module
//!
//! This module provides gold-standard testing against R/Python reference implementations
//! to ensure precision and correctness of all statistical functions.

pub mod core_statistics;
pub mod distributions;
pub mod time_series;
pub mod regression;
pub mod uncertainty;
pub mod preprocessing;
pub mod pipeline;
pub mod nist_validation;

#[cfg(test)]
mod test_utils {
    /// Compare floating point values with relative tolerance
    pub fn approx_eq(a: f64, b: f64, rtol: f64) -> bool {
        if a == b {
            return true;
        }
        let diff = (a - b).abs();
        let tol = rtol * (a.abs() + b.abs()) / 2.0;
        diff <= tol
    }
}


#[cfg(test)]
mod tests {
    use crate::scientific::statistics::primitives::DesignMatrixBuilder;
    use std::f64::consts::PI;

    #[test]
    fn test_build_linear() {
        let data = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
        ];

        let matrix = DesignMatrixBuilder::build_linear(&data).unwrap();

        // Should have 3 rows, 3 columns (intercept + 2 features)
        assert_eq!(matrix.nrows(), 3);
        assert_eq!(matrix.ncols(), 3);

        // Check intercept column (all 1s)
        for i in 0..3 {
            assert_eq!(matrix[[i, 0]], 1.0);
        }

        // Check feature columns
        assert_eq!(matrix[[0, 1]], 1.0);
        assert_eq!(matrix[[0, 2]], 2.0);
        assert_eq!(matrix[[1, 1]], 3.0);
        assert_eq!(matrix[[1, 2]], 4.0);
    }

    #[test]
    fn test_build_polynomial() {
        let data = vec![
            vec![1.0],
            vec![2.0],
            vec![3.0],
        ];

        let matrix = DesignMatrixBuilder::build_polynomial(&data, 2).unwrap();

        // Should have 3 rows, 3 columns (intercept + x + x^2)
        assert_eq!(matrix.nrows(), 3);
        assert_eq!(matrix.ncols(), 3);

        // Check values
        assert_eq!(matrix[[0, 0]], 1.0); // intercept
        assert_eq!(matrix[[0, 1]], 1.0); // x
        assert_eq!(matrix[[0, 2]], 1.0); // x^2

        assert_eq!(matrix[[1, 0]], 1.0); // intercept
        assert_eq!(matrix[[1, 1]], 2.0); // x
        assert_eq!(matrix[[1, 2]], 4.0); // x^2
    }

    #[test]
    fn test_build_seasonal() {
        let data = vec![
            vec![0.0], // time = 0
            vec![1.0], // time = 1
            vec![2.0], // time = 2
        ];

        let matrix = DesignMatrixBuilder::build_seasonal(&data, 1, 4.0).unwrap();

        // Should have 3 rows, 3 columns (intercept + sin + cos)
        assert_eq!(matrix.nrows(), 3);
        assert_eq!(matrix.ncols(), 3);

        // Check intercept
        for i in 0..3 {
            assert_eq!(matrix[[i, 0]], 1.0);
        }

        // Check Fourier terms (period = 4, so 2π/4 = π/2 radians per unit time)
        let expected_sin_0 = (1.0_f64 * 2.0 * PI * 0.0 / 4.0).sin(); // sin(0) = 0
        let expected_cos_0 = (1.0_f64 * 2.0 * PI * 0.0 / 4.0).cos(); // cos(0) = 1

        assert!((matrix[[0, 1]] - expected_sin_0).abs() < 1e-10);
        assert!((matrix[[0, 2]] - expected_cos_0).abs() < 1e-10);
    }
}