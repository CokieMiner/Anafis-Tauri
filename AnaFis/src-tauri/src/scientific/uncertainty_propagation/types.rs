use serde::{Deserialize, Serialize};

// ============================================================
// SPREADSHEET TYPES (for Excel formula generation)
// ============================================================

/// Represents a variable in the uncertainty calculation (spreadsheet mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value_range: String,       // e.g., "A1:A10"
    pub uncertainty_range: String, // e.g., "B1:B10"
    pub confidence: f64,           // confidence level in percent (e.g., 95.0)
}

/// Result of uncertainty formula generation
#[derive(Debug, Serialize, Deserialize)]
pub struct UncertaintyFormulas {
    pub value_formulas: Vec<String>, // Excel formulas for calculated values
    pub uncertainty_formulas: Vec<String>, // Excel formulas for propagated uncertainties
    pub success: bool,
    pub error: Option<String>,
}

/// Represents a parsed Excel range
#[derive(Debug, Clone, PartialEq)]
pub struct ExcelRange {
    pub column: String,
    pub start_row: usize,
    pub end_row: usize,
}

impl ExcelRange {
    pub fn new(column: String, start_row: usize, end_row: usize) -> Self {
        Self {
            column,
            start_row,
            end_row,
        }
    }

    pub fn row_count(&self) -> usize {
        self.end_row - self.start_row + 1
    }

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
