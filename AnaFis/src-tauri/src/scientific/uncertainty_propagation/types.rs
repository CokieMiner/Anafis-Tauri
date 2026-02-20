use serde::{Deserialize, Serialize};

// ============================================================
// SPREADSHEET TYPES (for Excel formula generation)
// ============================================================

/// Represents a variable in the uncertainty calculation (spreadsheet mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// The name of the variable (e.g., "x").
    pub name: String,
    /// The Excel range containing the values (e.g., "A1:A10").
    pub value_range: String, // e.g., "A1:A10"
    /// The Excel range containing the uncertainties (e.g., "B1:B10").
    pub uncertainty_range: String, // e.g., "B1:B10"
    /// The confidence level of the input uncertainties in percent (e.g., 95.0).
    pub confidence: f64, // confidence level in percent (e.g., 95.0)
}

/// Result of uncertainty formula generation
#[derive(Debug, Serialize, Deserialize)]
pub struct UncertaintyFormulas {
    /// Excel formulas for calculating the values.
    pub value_formulas: Vec<String>, // Excel formulas for calculated values
    /// Excel formulas for calculating the propagated uncertainties.
    pub uncertainty_formulas: Vec<String>, // Excel formulas for propagated uncertainties
    /// Whether the generation was successful.
    pub success: bool,
    /// Optional error message if generation failed.
    pub error: Option<String>,
}

/// Represents a parsed Excel range (e.g., "A1:A10")
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelRange {
    /// The column identifier (e.g., "A").
    pub column: String,
    /// The starting row number (1-indexed).
    pub start_row: usize,
    /// The ending row number (1-indexed).
    pub end_row: usize,
}

impl ExcelRange {
    /// Creates a new `ExcelRange`.
    #[must_use]
    pub const fn new(column: String, start_row: usize, end_row: usize) -> Self {
        Self {
            column,
            start_row,
            end_row,
        }
    }

    /// Returns the number of rows in the range.
    #[must_use]
    pub const fn row_count(&self) -> usize {
        self.end_row - self.start_row + 1
    }

    /// Returns the cell reference at the given zero-based offset from the start of the range.
    #[must_use]
    pub fn cell_at(&self, offset: usize) -> Option<String> {
        let row = self.start_row + offset;
        if row <= self.end_row {
            Some(format!("{}{}", self.column, row))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excel_range_row_count() {
        let range = ExcelRange::new("A".to_string(), 1, 10);
        assert_eq!(range.row_count(), 10);
    }

    #[test]
    fn test_excel_range_cell_at() {
        let range = ExcelRange::new("B".to_string(), 5, 10);
        assert_eq!(range.cell_at(0), Some("B5".to_string()));
        assert_eq!(range.cell_at(5), Some("B10".to_string()));
        assert_eq!(range.cell_at(6), None);
    }
}
