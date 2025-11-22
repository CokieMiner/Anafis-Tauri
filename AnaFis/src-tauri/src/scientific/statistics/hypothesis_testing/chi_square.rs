//! Chi-square tests
//!
//! Statistical tests for categorical data.

use super::types::{StatsError, ChiSquareResult};
use super::helpers::chi_square_p_value;

/// Chi-square testing engine
pub struct ChiSquareTesting;

impl ChiSquareTesting {
    /// Chi-square goodness of fit test
    pub fn chi_square_goodness_of_fit(observed: &[f64], expected: &[f64]) -> Result<ChiSquareResult, StatsError> {
        if observed.len() != expected.len() {
            return Err(StatsError::DimensionMismatch);
        }

        if observed.iter().any(|&x| x < 0.0) || expected.iter().any(|&x| x <= 0.0) {
            return Err(StatsError::InvalidParameter("Frequencies must be non-negative, expected frequencies must be positive".to_string()));
        }

        let total_observed: f64 = observed.iter().sum();
        let total_expected: f64 = expected.iter().sum();

        if (total_observed - total_expected).abs() > 1e-10 {
            return Err(StatsError::ChiSquareError("Total observed and expected frequencies must be equal".to_string()));
        }

        let chi_square = observed.iter().zip(expected.iter())
            .map(|(&o, &e)| (o - e).powi(2) / e).sum::<f64>();
        let df = (observed.len() - 1) as f64;
        let p_value = chi_square_p_value(chi_square, df)?;

        // Cohen's w effect size for goodness of fit (not Cramér's V which is for contingency tables)
        let cohens_w = if total_observed > 0.0 {
            (chi_square / total_observed).sqrt()
        } else {
            0.0
        };

        Ok(ChiSquareResult {
            test_type: "Chi-square goodness of fit".to_string(),
            chi_square_statistic: chi_square,
            p_value,
            degrees_of_freedom: df,
            expected_frequencies: vec![expected.to_vec()],
            residuals: vec![observed.iter().zip(expected.iter())
                .map(|(&o, &e)| (o - e) / e.sqrt()).collect()],
            significant: p_value < 0.05,
            effect_size: Some(cohens_w),
        })
    }

    /// Chi-square test of independence
    pub fn chi_square_independence(table: &[&[f64]]) -> Result<ChiSquareResult, StatsError> {
        if table.is_empty() || table[0].is_empty() {
            return Err(StatsError::EmptyData);
        }

        let rows = table.len();
        let cols = table[0].len();

        if table.iter().any(|row| row.len() != cols) {
            return Err(StatsError::DimensionMismatch);
        }

        let mut row_totals = vec![0.0; rows];
        let mut col_totals = vec![0.0; cols];
        let mut grand_total = 0.0;

        for (i, row) in table.iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                if cell < 0.0 {
                    return Err(StatsError::InvalidParameter("Cell frequencies cannot be negative".to_string()));
                }
                row_totals[i] += cell;
                col_totals[j] += cell;
                grand_total += cell;
            }
        }

        if grand_total == 0.0 {
            return Err(StatsError::ChiSquareError("Grand total cannot be zero".to_string()));
        }

        let mut expected = vec![vec![0.0; cols]; rows];
        let mut chi_square = 0.0;

        for (i, row_total) in row_totals.iter().enumerate().take(rows) {
            for (j, col_total) in col_totals.iter().enumerate().take(cols) {
                expected[i][j] = (row_total * col_total) / grand_total;
                if expected[i][j] > 0.0 {
                    chi_square += (table[i][j] - expected[i][j]).powi(2) / expected[i][j];
                }
            }
        }

        let df = ((rows - 1) * (cols - 1)) as f64;
        let p_value = chi_square_p_value(chi_square, df)?;

        let mut residuals = vec![vec![0.0; cols]; rows];
        for (i, row_total) in row_totals.iter().enumerate().take(rows) {
            for (j, col_total) in col_totals.iter().enumerate().take(cols) {
                residuals[i][j] = (table[i][j] - expected[i][j]) /
                    (expected[i][j] * (1.0 - row_total/grand_total) * (1.0 - col_total/grand_total)).sqrt();
            }
        }

        let min_dim = rows.min(cols) as f64 - 1.0;
        let effect_size = ((chi_square / grand_total) / min_dim).sqrt();

        Ok(ChiSquareResult {
            test_type: "Chi-square test of independence".to_string(),
            chi_square_statistic: chi_square,
            p_value,
            degrees_of_freedom: df,
            expected_frequencies: expected,
            residuals,
            significant: p_value < 0.05,
            effect_size: Some(effect_size),
        })
    }
}
