use crate::spreadsheet::types::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use chrono::Utc;

// Global spreadsheet state
static SPREADSHEET_STATE: Lazy<Arc<Mutex<SpreadsheetState>>> = 
    Lazy::new(|| Arc::new(Mutex::new(SpreadsheetState::default())));

/// Core spreadsheet engine for managing cells and state
pub struct SpreadsheetEngine;

impl SpreadsheetEngine {
    /// Get reference to global spreadsheet state
    pub fn get_state() -> Arc<Mutex<SpreadsheetState>> {
        SPREADSHEET_STATE.clone()
    }
    
    /// Set cell value with automatic type detection and metadata creation
    pub fn set_cell_value(cell_ref: &str, content: String) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        // Parse cell type from content
        let cell_type = CellType::parse_from_input(&content);
        
        // Create metadata with current timestamp
        let mut metadata = CellMetadata::default();
        metadata.is_formula = matches!(cell_type, CellType::Formula(_));
        metadata.modified_at = Utc::now();
        
        // Update version if cell already exists
        if let Some(existing_cell) = state.cells.get(cell_ref) {
            metadata.version = existing_cell.metadata.version + 1;
            metadata.created_at = existing_cell.metadata.created_at;
        }
        
        // Create unified cell
        let cell = UnifiedCell {
            content: content.clone(),
            cell_type,
            metadata,
            computed_value: None, // Will be computed later if needed
        };
        
        // Insert cell and update dependency graph if it's a formula
        state.cells.insert(cell_ref.to_string(), cell);
        
        // TODO: Update dependency graph for formulas (will be implemented in later tasks)
        
        Ok(())
    }
    
    /// Get cell value by reference
    pub fn get_cell_value(cell_ref: &str) -> Result<Option<UnifiedCell>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.cells.get(cell_ref).cloned())
    }
    
    /// Set active cell
    pub fn set_active_cell(row: usize, col: usize) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        state.active_cell = Some(CellReference { row, col });
        Ok(())
    }
    
    /// Get active cell reference
    pub fn get_active_cell() -> Result<Option<CellReference>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.active_cell.clone())
    }
    
    /// Create a new cell with specified type and content
    pub fn create_cell(content: String, cell_type: Option<CellType>) -> UnifiedCell {
        let detected_type = cell_type.unwrap_or_else(|| CellType::parse_from_input(&content));
        
        let mut metadata = CellMetadata::default();
        metadata.is_formula = matches!(detected_type, CellType::Formula(_));
        
        UnifiedCell {
            content,
            cell_type: detected_type,
            metadata,
            computed_value: None,
        }
    }
    
    /// Validate cell content against its metadata rules
    pub fn validate_cell(cell: &UnifiedCell) -> Vec<String> {
        cell.cell_type.validate(&cell.metadata.validation_rules)
    }
    
    /// Convert cell to different type if possible
    pub fn convert_cell_type(cell: &mut UnifiedCell, target_type: &str) -> Result<(), String> {
        let new_type = cell.cell_type.convert_to(target_type)?;
        cell.cell_type = new_type;
        cell.metadata.modified_at = Utc::now();
        cell.metadata.version += 1;
        Ok(())
    }
    
    /// Add quality flag to cell
    pub fn add_quality_flag(cell_ref: &str, flag: QualityFlag) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get_mut(cell_ref) {
            cell.metadata.quality_flags.push(flag);
            cell.metadata.modified_at = Utc::now();
            cell.metadata.version += 1;
        }
        
        Ok(())
    }
    
    /// Set unit for a cell (only for numeric types)
    pub fn set_cell_unit(cell_ref: &str, unit: String) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get_mut(cell_ref) {
            if !cell.cell_type.supports_units() {
                return Err("Cell type does not support units".to_string());
            }
            
            // TODO: Validate unit using existing unit system (will be implemented in later tasks)
            
            cell.metadata.unit = Some(unit);
            cell.metadata.modified_at = Utc::now();
            cell.metadata.version += 1;
        }
        
        Ok(())
    }
    
    /// Set cell format
    pub fn set_cell_format(cell_ref: &str, format: CellFormat) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get_mut(cell_ref) {
            cell.metadata.format = format;
            cell.metadata.modified_at = Utc::now();
            cell.metadata.version += 1;
        }
        
        Ok(())
    }
    
    /// Add validation rule to cell
    pub fn add_validation_rule(cell_ref: &str, rule: ValidationRule) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get_mut(cell_ref) {
            cell.metadata.validation_rules.push(rule);
            cell.metadata.modified_at = Utc::now();
            cell.metadata.version += 1;
        }
        
        Ok(())
    }
    
    /// Set experimental context for cell
    pub fn set_experimental_context(cell_ref: &str, context: ExperimentalContext) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get_mut(cell_ref) {
            cell.metadata.experimental_context = Some(context);
            cell.metadata.modified_at = Utc::now();
            cell.metadata.version += 1;
        }
        
        Ok(())
    }
    
    /// Get all cells in a range
    pub fn get_cells_in_range(range: &CellRange) -> Result<Vec<(String, UnifiedCell)>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        let mut cells = Vec::new();
        
        let start_row = range.start.row.min(range.end.row);
        let end_row = range.start.row.max(range.end.row);
        let start_col = range.start.col.min(range.end.col);
        let end_col = range.start.col.max(range.end.col);
        
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let cell_ref = CellReference { row, col }.to_string();
                if let Some(cell) = state.cells.get(&cell_ref) {
                    cells.push((cell_ref, cell.clone()));
                }
            }
        }
        
        Ok(cells)
    }
    
    /// Clear cell content but preserve metadata structure
    pub fn clear_cell(cell_ref: &str) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get_mut(cell_ref) {
            cell.content.clear();
            cell.cell_type = CellType::Empty;
            cell.computed_value = None;
            cell.metadata.is_formula = false;
            cell.metadata.modified_at = Utc::now();
            cell.metadata.version += 1;
        }
        
        Ok(())
    }
    
    /// Delete cell completely
    pub fn delete_cell(cell_ref: &str) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        // Remove from dependency graph before deleting
        state.dependency_graph.remove_cell(cell_ref);
        
        state.cells.remove(cell_ref);
        
        Ok(())
    }
    
    // ===== CELL RANGE OPERATIONS =====
    
    /// Set selection range
    pub fn set_selection(start_row: usize, start_col: usize, end_row: usize, end_col: usize) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        let range = CellRange {
            start: CellReference { row: start_row, col: start_col },
            end: CellReference { row: end_row, col: end_col },
        };
        
        state.selection = Some(range);
        Ok(())
    }
    
    /// Get current selection
    pub fn get_selection() -> Result<Option<CellRange>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.selection.clone())
    }
    
    /// Clear selection
    pub fn clear_selection() -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        state.selection = None;
        Ok(())
    }
    
    /// Apply formula to range with placeholder substitution
    pub fn apply_formula_to_range(range: &CellRange, formula_template: &str) -> Result<Vec<String>, String> {
        let mut results = Vec::new();
        
        let start_row = range.start.row.min(range.end.row);
        let end_row = range.start.row.max(range.end.row);
        let start_col = range.start.col.min(range.end.col);
        let end_col = range.start.col.max(range.end.col);
        
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let cell_ref = CellReference { row, col };
                let formula = Self::substitute_placeholders(formula_template, &cell_ref)?;
                
                Self::set_cell_value(&cell_ref.to_string(), formula.clone())?;
                results.push(format!("Applied '{}' to {}", formula, cell_ref.to_string()));
            }
        }
        
        Ok(results)
    }
    
    /// Substitute placeholders in formula template
    fn substitute_placeholders(template: &str, cell_ref: &CellReference) -> Result<String, String> {
        let mut result = template.to_string();
        let col_letter = CellReference::col_to_string(cell_ref.col);
        
        // Replace A# pattern with column letter + row number (most specific first)
        let row_pattern = format!("{}#", col_letter);
        let row_replacement = format!("{}{}", col_letter, cell_ref.row + 1);
        result = result.replace(&row_pattern, &row_replacement);
        
        // Replace #12 pattern with column letter + fixed row
        let col_pattern_regex = regex::Regex::new(r"#(\d+)").map_err(|e| format!("Regex error: {}", e))?;
        result = col_pattern_regex.replace_all(&result, |caps: &regex::Captures| {
            let row_num = &caps[1];
            format!("{}{}", col_letter, row_num)
        }).to_string();
        
        // Replace remaining # with row number (1-based) - do this last
        result = result.replace("#", &(cell_ref.row + 1).to_string());
        
        Ok(result)
    }
    
    /// Get all cells in range (enhanced version)
    pub fn get_range_cells(range: &CellRange) -> Result<HashMap<String, UnifiedCell>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        let mut cells = HashMap::new();
        
        let start_row = range.start.row.min(range.end.row);
        let end_row = range.start.row.max(range.end.row);
        let start_col = range.start.col.min(range.end.col);
        let end_col = range.start.col.max(range.end.col);
        
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let cell_ref = CellReference { row, col }.to_string();
                if let Some(cell) = state.cells.get(&cell_ref) {
                    cells.insert(cell_ref, cell.clone());
                }
            }
        }
        
        Ok(cells)
    }
    
    /// Clear range of cells
    pub fn clear_range(range: &CellRange) -> Result<(), String> {
        let start_row = range.start.row.min(range.end.row);
        let end_row = range.start.row.max(range.end.row);
        let start_col = range.start.col.min(range.end.col);
        let end_col = range.start.col.max(range.end.col);
        
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let cell_ref = CellReference { row, col }.to_string();
                Self::clear_cell(&cell_ref)?;
            }
        }
        
        Ok(())
    }
    
    /// Delete range of cells
    pub fn delete_range(range: &CellRange) -> Result<(), String> {
        let start_row = range.start.row.min(range.end.row);
        let end_row = range.start.row.max(range.end.row);
        let start_col = range.start.col.min(range.end.col);
        let end_col = range.start.col.max(range.end.col);
        
        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let cell_ref = CellReference { row, col }.to_string();
                Self::delete_cell(&cell_ref)?;
            }
        }
        
        Ok(())
    }
    
    // ===== DEPENDENCY TRACKING =====
    
    /// Add dependency relationship
    pub fn add_dependency(dependent: &str, dependency: &str) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        state.dependency_graph.add_dependency(dependent, dependency);
        Ok(())
    }
    
    /// Remove dependency relationship
    pub fn remove_dependency(dependent: &str, dependency: &str) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        state.dependency_graph.remove_dependency(dependent, dependency);
        Ok(())
    }
    
    /// Get all dependencies for a cell
    pub fn get_dependencies(cell_ref: &str) -> Result<Vec<String>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.dependency_graph.get_dependencies(cell_ref))
    }
    
    /// Get all dependents for a cell
    pub fn get_dependents(cell_ref: &str) -> Result<Vec<String>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.dependency_graph.get_dependents(cell_ref))
    }
    
    /// Check for circular references
    pub fn check_circular_reference(cell_ref: &str) -> Result<Option<Vec<String>>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.dependency_graph.find_circular_reference(cell_ref))
    }
    
    /// Get calculation order for cells
    pub fn get_calculation_order() -> Result<Vec<String>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        state.dependency_graph.topological_sort()
    }
    
    // ===== UNDO/REDO FUNCTIONALITY =====
    
    /// Create snapshot of current state
    pub fn create_snapshot(description: String) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        let snapshot = SpreadsheetSnapshot {
            timestamp: Utc::now(),
            description,
            cells: state.cells.clone(),
            metadata: state.metadata_store.clone(),
        };
        
        state.version_history.add_snapshot(snapshot);
        Ok(())
    }
    
    /// Undo last operation
    pub fn undo() -> Result<bool, String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        // First, try to get the snapshot
        let snapshot_data = if state.version_history.can_undo() {
            let snapshot = state.version_history.undo().unwrap();
            Some((snapshot.cells.clone(), snapshot.metadata.clone()))
        } else {
            None
        };
        
        // Then apply the changes
        if let Some((cells, metadata)) = snapshot_data {
            state.cells = cells;
            state.metadata_store = metadata;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Redo last undone operation
    pub fn redo() -> Result<bool, String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        // First, try to get the snapshot
        let snapshot_data = if state.version_history.can_redo() {
            let snapshot = state.version_history.redo().unwrap();
            Some((snapshot.cells.clone(), snapshot.metadata.clone()))
        } else {
            None
        };
        
        // Then apply the changes
        if let Some((cells, metadata)) = snapshot_data {
            state.cells = cells;
            state.metadata_store = metadata;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Get version history
    pub fn get_version_history() -> Result<Vec<SpreadsheetSnapshot>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.version_history.versions.clone())
    }
    
    /// Get current version index
    pub fn get_current_version() -> Result<usize, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.version_history.current_version)
    }
    
    /// Can undo check
    pub fn can_undo() -> Result<bool, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.version_history.can_undo())
    }
    
    /// Can redo check
    pub fn can_redo() -> Result<bool, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(state.version_history.can_redo())
    }
    
    // ===== CELL REFERENCE VALIDATION =====
    
    /// Validate cell reference format
    pub fn validate_cell_reference(cell_ref: &str) -> Result<bool, String> {
        match CellReference::from_string(cell_ref) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
    
    /// Parse and validate cell range
    pub fn parse_cell_range(range_str: &str) -> Result<CellRange, String> {
        CellRange::from_string(range_str)
    }
    
    /// Check if cell reference is valid
    pub fn is_valid_cell_reference(cell_ref: &str) -> bool {
        CellReference::from_string(cell_ref).is_some()
    }
    
    // ===== UNCERTAINTY CELL OPERATIONS =====
    
    /// Toggle uncertainty mode for a cell
    pub fn toggle_uncertainty_mode(cell_ref: &str, enable: bool) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get_mut(cell_ref) {
            let (new_cell_type, new_content) = match (&cell.cell_type, enable) {
                (CellType::Number(value), true) => {
                    // Convert number to uncertainty with zero uncertainty
                    let value = *value;
                    (
                        CellType::NumberWithUncertainty {
                            value,
                            uncertainty: 0.0,
                            uncertainty_type: UncertaintyType::Absolute,
                        },
                        format!("{} ± 0", value)
                    )
                }
                (CellType::NumberWithUncertainty { value, .. }, false) => {
                    // Convert uncertainty back to number
                    let value = *value;
                    (CellType::Number(value), value.to_string())
                }
                _ => return Err("Cannot toggle uncertainty mode for this cell type".to_string()),
            };
            
            cell.cell_type = new_cell_type;
            cell.content = new_content;
            cell.metadata.modified_at = Utc::now();
            cell.metadata.version += 1;
        }
        
        Ok(())
    }
    
    /// Set uncertainty cell value with components
    pub fn set_uncertainty_cell_value(
        cell_ref: &str,
        value: f64,
        uncertainty: f64,
        uncertainty_type: UncertaintyType,
    ) -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        let display_string = Self::format_uncertainty_display(value, uncertainty, &uncertainty_type);
        
        let cell_type = CellType::NumberWithUncertainty {
            value,
            uncertainty,
            uncertainty_type,
        };
        
        // Create or update cell
        let mut metadata = CellMetadata::default();
        if let Some(existing_cell) = state.cells.get(cell_ref) {
            metadata = existing_cell.metadata.clone();
            metadata.version += 1;
        }
        metadata.modified_at = Utc::now();
        
        let cell = UnifiedCell {
            content: display_string,
            cell_type,
            metadata,
            computed_value: None,
        };
        
        state.cells.insert(cell_ref.to_string(), cell);
        Ok(())
    }
    
    /// Get uncertainty components for a cell
    pub fn get_uncertainty_components(cell_ref: &str) -> Result<Option<UncertaintyComponents>, String> {
        let state = Self::get_state();
        let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get(cell_ref) {
            match &cell.cell_type {
                CellType::NumberWithUncertainty { value, uncertainty, uncertainty_type } => {
                    let display_string = Self::format_uncertainty_display(*value, *uncertainty, uncertainty_type);
                    Ok(Some(UncertaintyComponents {
                        value: *value,
                        uncertainty: *uncertainty,
                        uncertainty_type: uncertainty_type.clone(),
                        display_string,
                    }))
                }
                CellType::Number(value) => {
                    // Return as uncertainty with zero uncertainty
                    let display_string = format!("{} ± 0", value);
                    Ok(Some(UncertaintyComponents {
                        value: *value,
                        uncertainty: 0.0,
                        uncertainty_type: UncertaintyType::Absolute,
                        display_string,
                    }))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
    
    /// Convert uncertainty type for a cell
    pub fn convert_uncertainty_type(
        cell_ref: &str,
        target_type: UncertaintyType,
    ) -> Result<UncertaintyComponents, String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(cell) = state.cells.get_mut(cell_ref) {
            match &cell.cell_type {
                CellType::NumberWithUncertainty { value, uncertainty, uncertainty_type } => {
                    let value = *value;
                    let uncertainty = *uncertainty;
                    let current_type = uncertainty_type.clone();
                    
                    let (new_uncertainty, new_type) = Self::convert_uncertainty_value(
                        value, uncertainty, &current_type, &target_type
                    )?;
                    
                    let display_string = Self::format_uncertainty_display(value, new_uncertainty, &new_type);
                    
                    // Update cell
                    cell.cell_type = CellType::NumberWithUncertainty {
                        value,
                        uncertainty: new_uncertainty,
                        uncertainty_type: new_type.clone(),
                    };
                    cell.content = display_string.clone();
                    cell.metadata.modified_at = Utc::now();
                    cell.metadata.version += 1;
                    
                    Ok(UncertaintyComponents {
                        value,
                        uncertainty: new_uncertainty,
                        uncertainty_type: new_type,
                        display_string,
                    })
                }
                _ => Err("Cell is not an uncertainty cell".to_string()),
            }
        } else {
            Err("Cell not found".to_string())
        }
    }
    
    /// Handle click on uncertainty cell to determine focus area
    pub fn handle_uncertainty_cell_click(
        cell_ref: &str,
        click_x: f64,
        cell_width: f64,
    ) -> Result<UncertaintyClickResult, String> {
        let components = Self::get_uncertainty_components(cell_ref)?
            .ok_or("Cell is not an uncertainty cell")?;
        
        // Simple heuristic: if click is in left half, focus on value; right half, focus on uncertainty
        // In a real implementation, this would be more sophisticated based on text layout
        let focus_area = if click_x < cell_width / 2.0 {
            UncertaintyFocusArea::Value
        } else {
            UncertaintyFocusArea::Uncertainty
        };
        
        Ok(UncertaintyClickResult {
            focus_area,
            components,
        })
    }
    
    /// Format uncertainty for display
    fn format_uncertainty_display(value: f64, uncertainty: f64, uncertainty_type: &UncertaintyType) -> String {
        match uncertainty_type {
            UncertaintyType::Absolute => format!("{} ± {}", value, uncertainty),
            UncertaintyType::Percentage => {
                let percent = if value != 0.0 { (uncertainty / value.abs()) * 100.0 } else { 0.0 };
                format!("{} ± {}%", value, percent)
            }
            UncertaintyType::StandardDeviation => format!("{} ± {} (σ)", value, uncertainty),
            UncertaintyType::StandardError => format!("{} ± {} (SE)", value, uncertainty),
        }
    }
    
    /// Convert uncertainty between different types
    fn convert_uncertainty_value(
        value: f64,
        uncertainty: f64,
        from_type: &UncertaintyType,
        to_type: &UncertaintyType,
    ) -> Result<(f64, UncertaintyType), String> {
        if from_type == to_type {
            return Ok((uncertainty, to_type.clone()));
        }
        
        // First convert to absolute uncertainty
        let absolute_uncertainty = match from_type {
            UncertaintyType::Absolute => uncertainty,
            UncertaintyType::Percentage => {
                if value == 0.0 {
                    return Err("Cannot convert percentage uncertainty for zero value".to_string());
                }
                value.abs() * uncertainty / 100.0
            }
            UncertaintyType::StandardDeviation => uncertainty, // Assume same as absolute for now
            UncertaintyType::StandardError => uncertainty, // Assume same as absolute for now
        };
        
        // Then convert to target type
        let target_uncertainty = match to_type {
            UncertaintyType::Absolute => absolute_uncertainty,
            UncertaintyType::Percentage => {
                if value == 0.0 {
                    return Err("Cannot convert to percentage uncertainty for zero value".to_string());
                }
                (absolute_uncertainty / value.abs()) * 100.0
            }
            UncertaintyType::StandardDeviation => absolute_uncertainty, // Assume same as absolute for now
            UncertaintyType::StandardError => absolute_uncertainty, // Assume same as absolute for now
        };
        
        Ok((target_uncertainty, to_type.clone()))
    }
    
    /// Clear all dependencies (for testing purposes)
    #[cfg(test)]
    pub fn clear_all_dependencies() -> Result<(), String> {
        let state = Self::get_state();
        let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        state.dependency_graph = DependencyGraph::default();
        Ok(())
    }
}