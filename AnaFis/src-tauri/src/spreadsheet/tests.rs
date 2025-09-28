#[cfg(test)]
mod tests {
    use crate::spreadsheet::types::*;
    use crate::spreadsheet::core::*;
    use chrono::{NaiveDate, NaiveTime, Utc};
    use std::collections::HashMap;

    #[test]
    fn test_cell_type_parsing_empty() {
        let cell_type = CellType::parse_from_input("");
        assert_eq!(cell_type, CellType::Empty);
    }

    #[test]
    fn test_cell_type_parsing_number() {
        let cell_type = CellType::parse_from_input("42.5");
        assert_eq!(cell_type, CellType::Number(42.5));
    }

    #[test]
    fn test_cell_type_parsing_text() {
        let cell_type = CellType::parse_from_input("Hello World");
        assert_eq!(cell_type, CellType::Text("Hello World".to_string()));
    }

    #[test]
    fn test_uncertainty_parsing() {
        let cell_type = CellType::parse_from_input("5.2 ± 0.1");
        match cell_type {
            CellType::NumberWithUncertainty { value, uncertainty, uncertainty_type } => {
                assert_eq!(value, 5.2);
                assert_eq!(uncertainty, 0.1);
                assert_eq!(uncertainty_type, UncertaintyType::Absolute);
            }
            _ => panic!("Expected NumberWithUncertainty"),
        }
    }

    #[test]
    fn test_uncertainty_percentage_parsing() {
        let cell_type = CellType::parse_from_input("100 ± 5%");
        match cell_type {
            CellType::NumberWithUncertainty { value, uncertainty, uncertainty_type } => {
                assert_eq!(value, 100.0);
                assert_eq!(uncertainty, 5.0); // Should be converted to absolute
                assert_eq!(uncertainty_type, UncertaintyType::Percentage);
            }
            _ => panic!("Expected NumberWithUncertainty"),
        }
    }

    #[test]
    fn test_cell_reference_parsing() {
        let cell_ref = CellReference::from_string("A1").unwrap();
        assert_eq!(cell_ref.row, 0);
        assert_eq!(cell_ref.col, 0);

        let cell_ref = CellReference::from_string("B2").unwrap();
        assert_eq!(cell_ref.row, 1);
        assert_eq!(cell_ref.col, 1);

        let cell_ref = CellReference::from_string("Z26").unwrap();
        assert_eq!(cell_ref.row, 25);
        assert_eq!(cell_ref.col, 25);
    }

    #[test]
    fn test_cell_reference_to_string() {
        let cell_ref = CellReference { row: 0, col: 0 };
        assert_eq!(cell_ref.to_string(), "A1");

        let cell_ref = CellReference { row: 1, col: 1 };
        assert_eq!(cell_ref.to_string(), "B2");

        let cell_ref = CellReference { row: 25, col: 25 };
        assert_eq!(cell_ref.to_string(), "Z26");
    }

    #[test]
    fn test_spreadsheet_engine_basic_operations() {
        // Clear any existing state
        #[cfg(test)]
        SpreadsheetEngine::clear_all_dependencies().unwrap();

        // Test setting and getting cell values
        SpreadsheetEngine::set_cell_value("A1", "42".to_string()).unwrap();
        let cell = SpreadsheetEngine::get_cell_value("A1").unwrap().unwrap();
        assert_eq!(cell.content, "42");
        match cell.cell_type {
            CellType::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number cell type"),
        }

        // Test uncertainty cell
        SpreadsheetEngine::set_cell_value("B1", "5.2 ± 0.1".to_string()).unwrap();
        let cell = SpreadsheetEngine::get_cell_value("B1").unwrap().unwrap();
        match cell.cell_type {
            CellType::NumberWithUncertainty { value, uncertainty, .. } => {
                assert_eq!(value, 5.2);
                assert_eq!(uncertainty, 0.1);
            }
            _ => panic!("Expected NumberWithUncertainty cell type"),
        }
    }

    #[test]
    fn test_uncertainty_mode_toggle() {
        // Clear any existing state
        #[cfg(test)]
        SpreadsheetEngine::clear_all_dependencies().unwrap();

        // Set a regular number
        SpreadsheetEngine::set_cell_value("C1", "10.5".to_string()).unwrap();
        
        // Toggle to uncertainty mode
        SpreadsheetEngine::toggle_uncertainty_mode("C1", true).unwrap();
        
        let cell = SpreadsheetEngine::get_cell_value("C1").unwrap().unwrap();
        match cell.cell_type {
            CellType::NumberWithUncertainty { value, uncertainty, .. } => {
                assert_eq!(value, 10.5);
                assert_eq!(uncertainty, 0.0);
            }
            _ => panic!("Expected NumberWithUncertainty after toggle"),
        }

        // Toggle back to number mode
        SpreadsheetEngine::toggle_uncertainty_mode("C1", false).unwrap();
        
        let cell = SpreadsheetEngine::get_cell_value("C1").unwrap().unwrap();
        match cell.cell_type {
            CellType::Number(value) => assert_eq!(value, 10.5),
            _ => panic!("Expected Number after toggle back"),
        }
    }

    #[test]
    fn test_uncertainty_type_conversion() {
        // Clear any existing state
        #[cfg(test)]
        SpreadsheetEngine::clear_all_dependencies().unwrap();

        // Set uncertainty cell with absolute uncertainty
        SpreadsheetEngine::set_uncertainty_cell_value(
            "D1", 
            100.0, 
            5.0, 
            UncertaintyType::Absolute
        ).unwrap();

        // Convert to percentage
        let components = SpreadsheetEngine::convert_uncertainty_type(
            "D1", 
            UncertaintyType::Percentage
        ).unwrap();

        assert_eq!(components.value, 100.0);
        assert_eq!(components.uncertainty, 5.0); // 5/100 * 100 = 5%
        assert_eq!(components.uncertainty_type, UncertaintyType::Percentage);

        // Convert back to absolute
        let components = SpreadsheetEngine::convert_uncertainty_type(
            "D1", 
            UncertaintyType::Absolute
        ).unwrap();

        assert_eq!(components.value, 100.0);
        assert_eq!(components.uncertainty, 5.0);
        assert_eq!(components.uncertainty_type, UncertaintyType::Absolute);
    }

    #[test]
    fn test_version_history() {
        let mut history = VersionHistory::default();
        
        // Test initial state
        assert_eq!(history.versions.len(), 0);
        assert_eq!(history.current_version, 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        
        // Add snapshots
        let snapshot1 = SpreadsheetSnapshot {
            timestamp: Utc::now(),
            description: "Snapshot 1".to_string(),
            cells: HashMap::new(),
            metadata: MetadataStore::default(),
        };
        
        let snapshot2 = SpreadsheetSnapshot {
            timestamp: Utc::now(),
            description: "Snapshot 2".to_string(),
            cells: HashMap::new(),
            metadata: MetadataStore::default(),
        };
        
        history.add_snapshot(snapshot1);
        history.add_snapshot(snapshot2);
        
        // Test state after adding snapshots
        assert_eq!(history.versions.len(), 2);
        assert_eq!(history.current_version, 2);
        assert!(history.can_undo());
        assert!(!history.can_redo());
        
        // Test undo
        let undone = history.undo();
        assert!(undone.is_some());
        assert_eq!(history.current_version, 1);
        assert!(!history.can_undo()); // Can't undo further with only 2 snapshots
        assert!(history.can_redo());
        
        // Test redo
        let redone = history.redo();
        assert!(redone.is_some());
        assert_eq!(history.current_version, 2);
        assert!(history.can_undo());
        assert!(!history.can_redo());
        
        // Test clear
        history.clear();
        assert_eq!(history.versions.len(), 0);
        assert_eq!(history.current_version, 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}