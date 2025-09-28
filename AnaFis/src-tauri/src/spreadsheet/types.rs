use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, NaiveDate, NaiveTime, Duration, Utc, Timelike};
use uuid::Uuid;
use regex::Regex;

// ===== CORE CELL STRUCTURES =====

/// Unified cell structure that can hold any type of data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCell {
    pub content: String,
    pub metadata: CellMetadata,
    pub computed_value: Option<ComputedValue>,
    pub cell_type: CellType,
}

/// All supported cell types with their specific data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellType {
    Empty,
    Text(String),
    Number(f64),
    NumberWithUncertainty { 
        value: f64, 
        uncertainty: f64, 
        uncertainty_type: UncertaintyType 
    },
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Date(NaiveDate),
    Time(NaiveTime),
    Duration(Duration),
    Formula(String),
    Error(String),
}

/// Types of uncertainty representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UncertaintyType {
    Absolute,
    Percentage,
    StandardDeviation,
    StandardError,
}

/// Components of an uncertainty cell for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyComponents {
    pub value: f64,
    pub uncertainty: f64,
    pub uncertainty_type: UncertaintyType,
    pub display_string: String,
}

/// Result of clicking on an uncertainty cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyClickResult {
    pub focus_area: UncertaintyFocusArea,
    pub components: UncertaintyComponents,
}

/// Which area of the uncertainty cell should be focused
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UncertaintyFocusArea {
    Value,
    Uncertainty,
}

/// Computed values after formula evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComputedValue {
    Number(f64),
    NumberWithUncertainty { value: f64, uncertainty: f64 },
    Boolean(bool),
    Text(String),
    DateTime(DateTime<Utc>),
    Date(NaiveDate),
    Time(NaiveTime),
    Duration(Duration),
    Array(Vec<ComputedValue>),
    Error(String),
}

/// Cell metadata including units, formatting, and quality information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    pub unit: Option<String>,
    pub format: CellFormat,
    pub is_formula: bool,
    pub quality_flags: Vec<QualityFlag>,
    pub experimental_context: Option<ExperimentalContext>,
    pub validation_rules: Vec<ValidationRule>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub version: u32,
}

/// Cell formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellFormat {
    Auto,
    Number { precision: usize, notation: NumberNotation },
    Scientific { precision: usize },
    Percentage { precision: usize },
    Boolean,
    Text,
    DateTime { format: String },
    Date { format: String },
    Time { format: String },
    Duration { format: String },
    Custom(String),
}

/// Number notation styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NumberNotation {
    Standard,
    Scientific,
    Engineering,
}

/// Quality flags for data validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityFlag {
    Valid,
    Outlier { method: String, confidence: f64 },
    OutOfRange { min: Option<f64>, max: Option<f64> },
    UnitMismatch { expected: String, actual: String },
    ValidationFailed { rule: String, reason: String },
    ManualFlag { reason: String, flagged_by: String },
    Interpolated { method: String },
    Estimated { method: String, confidence: f64 },
}

/// Experimental context for scientific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalContext {
    pub experiment_id: Uuid,
    pub operator: String,
    pub instrument: Option<String>,
    pub calibration_date: Option<DateTime<Utc>>,
    pub environmental_conditions: HashMap<String, f64>,
    pub measurement_protocol: Option<String>,
    pub notes: Option<String>,
}

/// Validation rules for data quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Range { min: Option<f64>, max: Option<f64> },
    UnitRequired { unit_type: String },
    DataType { expected_type: String },
    Pattern { regex: String },
    Custom { name: String, expression: String },
}

// ===== SPREADSHEET STATE STRUCTURES =====

/// Cell reference (A1, B2, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CellReference {
    pub row: usize,
    pub col: usize,
}

/// Range of cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRange {
    pub start: CellReference,
    pub end: CellReference,
}

/// Main spreadsheet state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetState {
    pub cells: HashMap<String, UnifiedCell>,
    pub active_cell: Option<CellReference>,
    pub selection: Option<CellRange>,
    pub dependency_graph: DependencyGraph,
    pub calculation_cache: CalculationCache,
    pub metadata_store: MetadataStore,
    pub version_history: VersionHistory,
}

/// Dependency tracking for formulas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub dependencies: HashMap<String, Vec<String>>,
    pub dependents: HashMap<String, Vec<String>>,
}

/// Calculation result caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationCache {
    pub formula_results: HashMap<String, ComputedValue>,
    pub uncertainty_derivatives: HashMap<String, f64>,
    pub unit_conversions: HashMap<String, f64>,
    pub last_updated: HashMap<String, DateTime<Utc>>,
}

/// Metadata storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataStore {
    pub global_settings: HashMap<String, String>,
    pub column_metadata: HashMap<usize, ColumnMetadata>,
    pub row_metadata: HashMap<usize, RowMetadata>,
}

/// Column-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: Option<String>,
    pub default_unit: Option<String>,
    pub data_type: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
}

/// Row-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowMetadata {
    pub name: Option<String>,
    pub group: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Version history for undo/redo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub versions: Vec<SpreadsheetSnapshot>,
    pub current_version: usize,
    pub max_versions: usize,
}

/// Snapshot of spreadsheet state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetSnapshot {
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub cells: HashMap<String, UnifiedCell>,
    pub metadata: MetadataStore,
}

// ===== IMPLEMENTATION BLOCKS =====

impl Default for UnifiedCell {
    fn default() -> Self {
        Self {
            content: String::new(),
            metadata: CellMetadata::default(),
            computed_value: None,
            cell_type: CellType::Empty,
        }
    }
}

impl Default for CellMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            unit: None,
            format: CellFormat::Auto,
            is_formula: false,
            quality_flags: vec![QualityFlag::Valid],
            experimental_context: None,
            validation_rules: Vec::new(),
            created_at: now,
            modified_at: now,
            version: 1,
        }
    }
}

impl Default for SpreadsheetState {
    fn default() -> Self {
        Self {
            cells: HashMap::new(),
            active_cell: None,
            selection: None,
            dependency_graph: DependencyGraph::default(),
            calculation_cache: CalculationCache::default(),
            metadata_store: MetadataStore::default(),
            version_history: VersionHistory::default(),
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }
}

impl Default for CalculationCache {
    fn default() -> Self {
        Self {
            formula_results: HashMap::new(),
            uncertainty_derivatives: HashMap::new(),
            unit_conversions: HashMap::new(),
            last_updated: HashMap::new(),
        }
    }
}

impl Default for MetadataStore {
    fn default() -> Self {
        Self {
            global_settings: HashMap::new(),
            column_metadata: HashMap::new(),
            row_metadata: HashMap::new(),
        }
    }
}

impl Default for VersionHistory {
    fn default() -> Self {
        Self {
            versions: Vec::new(),
            current_version: 0,
            max_versions: 100,
        }
    }
}

// ===== CELL REFERENCE IMPLEMENTATION =====

impl CellReference {
    /// Convert cell reference to string format (A1, B2, etc.)
    pub fn to_string(&self) -> String {
        let col_str = Self::col_to_string(self.col);
        format!("{}{}", col_str, self.row + 1)
    }
    
    /// Parse cell reference from string format
    pub fn from_string(s: &str) -> Option<Self> {
        let re = Regex::new(r"^([A-Z]+)(\d+)$").ok()?;
        let caps = re.captures(s)?;
        
        let col_str = caps.get(1)?.as_str();
        let row_str = caps.get(2)?.as_str();
        
        let col = Self::string_to_col(col_str)?;
        let row = row_str.parse::<usize>().ok()?.saturating_sub(1);
        
        Some(CellReference { row, col })
    }
    
    /// Convert column number to string (0 -> A, 25 -> Z, 26 -> AA)
    pub fn col_to_string(mut col: usize) -> String {
        let mut result = String::new();
        loop {
            result.insert(0, ((col % 26) as u8 + b'A') as char);
            if col < 26 {
                break;
            }
            col = col / 26 - 1;
        }
        result
    }
    
    /// Convert column string to number (A -> 0, Z -> 25, AA -> 26)
    fn string_to_col(s: &str) -> Option<usize> {
        let mut result = 0;
        for c in s.chars() {
            if !c.is_ascii_alphabetic() {
                return None;
            }
            result = result * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
        }
        Some(result - 1)
    }
}

// ===== CELL RANGE IMPLEMENTATION =====

impl CellRange {
    /// Parse cell range from string format (A1:B2, A1:A10, etc.)
    pub fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid range format: '{}'. Expected format: 'A1:B2'", s));
        }
        
        let start = CellReference::from_string(parts[0])
            .ok_or_else(|| format!("Invalid start cell reference: '{}'", parts[0]))?;
        let end = CellReference::from_string(parts[1])
            .ok_or_else(|| format!("Invalid end cell reference: '{}'", parts[1]))?;
        
        Ok(CellRange { start, end })
    }
    
    /// Convert range to string format
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.start.to_string(), self.end.to_string())
    }
    
    /// Check if range contains a specific cell
    pub fn contains(&self, cell_ref: &CellReference) -> bool {
        let min_row = self.start.row.min(self.end.row);
        let max_row = self.start.row.max(self.end.row);
        let min_col = self.start.col.min(self.end.col);
        let max_col = self.start.col.max(self.end.col);
        
        cell_ref.row >= min_row && cell_ref.row <= max_row &&
        cell_ref.col >= min_col && cell_ref.col <= max_col
    }
    
    /// Get all cell references in the range
    pub fn get_all_cells(&self) -> Vec<CellReference> {
        let mut cells = Vec::new();
        
        let min_row = self.start.row.min(self.end.row);
        let max_row = self.start.row.max(self.end.row);
        let min_col = self.start.col.min(self.end.col);
        let max_col = self.start.col.max(self.end.col);
        
        for row in min_row..=max_row {
            for col in min_col..=max_col {
                cells.push(CellReference { row, col });
            }
        }
        
        cells
    }
    
    /// Get the size of the range (rows, cols)
    pub fn size(&self) -> (usize, usize) {
        let rows = (self.start.row.max(self.end.row) - self.start.row.min(self.end.row)) + 1;
        let cols = (self.start.col.max(self.end.col) - self.start.col.min(self.end.col)) + 1;
        (rows, cols)
    }
}

// ===== DEPENDENCY GRAPH IMPLEMENTATION =====

impl DependencyGraph {
    /// Add a dependency relationship
    pub fn add_dependency(&mut self, dependent: &str, dependency: &str) {
        // Add to dependencies map
        self.dependencies
            .entry(dependent.to_string())
            .or_insert_with(Vec::new)
            .push(dependency.to_string());
        
        // Add to dependents map
        self.dependents
            .entry(dependency.to_string())
            .or_insert_with(Vec::new)
            .push(dependent.to_string());
    }
    
    /// Remove a dependency relationship
    pub fn remove_dependency(&mut self, dependent: &str, dependency: &str) {
        // Remove from dependencies map
        if let Some(deps) = self.dependencies.get_mut(dependent) {
            deps.retain(|d| d != dependency);
            if deps.is_empty() {
                self.dependencies.remove(dependent);
            }
        }
        
        // Remove from dependents map
        if let Some(deps) = self.dependents.get_mut(dependency) {
            deps.retain(|d| d != dependent);
            if deps.is_empty() {
                self.dependents.remove(dependency);
            }
        }
    }
    
    /// Remove all dependencies for a cell
    pub fn remove_cell(&mut self, cell_ref: &str) {
        // Remove as dependent
        if let Some(dependencies) = self.dependencies.remove(cell_ref) {
            for dep in dependencies {
                if let Some(dependents) = self.dependents.get_mut(&dep) {
                    dependents.retain(|d| d != cell_ref);
                    if dependents.is_empty() {
                        self.dependents.remove(&dep);
                    }
                }
            }
        }
        
        // Remove as dependency
        if let Some(dependents) = self.dependents.remove(cell_ref) {
            for dep in dependents {
                if let Some(dependencies) = self.dependencies.get_mut(&dep) {
                    dependencies.retain(|d| d != cell_ref);
                    if dependencies.is_empty() {
                        self.dependencies.remove(&dep);
                    }
                }
            }
        }
    }
    
    /// Get all dependencies for a cell
    pub fn get_dependencies(&self, cell_ref: &str) -> Vec<String> {
        self.dependencies.get(cell_ref).cloned().unwrap_or_default()
    }
    
    /// Get all dependents for a cell
    pub fn get_dependents(&self, cell_ref: &str) -> Vec<String> {
        self.dependents.get(cell_ref).cloned().unwrap_or_default()
    }
    
    /// Find circular reference starting from a cell
    pub fn find_circular_reference(&self, start_cell: &str) -> Option<Vec<String>> {
        let mut visited = std::collections::HashSet::new();
        let mut path = Vec::new();
        
        self.dfs_circular_check(start_cell, &mut visited, &mut path)
    }
    
    /// Depth-first search for circular references
    fn dfs_circular_check(
        &self,
        cell: &str,
        visited: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if path.contains(&cell.to_string()) {
            // Found circular reference
            let cycle_start = path.iter().position(|c| c == cell).unwrap();
            let mut cycle = path[cycle_start..].to_vec();
            cycle.push(cell.to_string());
            return Some(cycle);
        }
        
        if visited.contains(cell) {
            return None;
        }
        
        visited.insert(cell.to_string());
        path.push(cell.to_string());
        
        if let Some(dependencies) = self.dependencies.get(cell) {
            for dep in dependencies {
                if let Some(cycle) = self.dfs_circular_check(dep, visited, path) {
                    return Some(cycle);
                }
            }
        }
        
        path.pop();
        None
    }
    
    /// Get topological sort order for calculation
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut all_cells: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        // Collect all cells and calculate in-degrees
        for (cell, deps) in &self.dependencies {
            all_cells.insert(cell.clone());
            for dep in deps {
                all_cells.insert(dep.clone());
            }
        }
        
        // Initialize in-degrees
        for cell in &all_cells {
            in_degree.insert(cell.clone(), 0);
        }
        
        // Calculate in-degrees
        for (dependent, deps) in &self.dependencies {
            *in_degree.get_mut(dependent).unwrap() = deps.len();
        }
        
        // Kahn's algorithm
        let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        let mut result = Vec::new();
        
        // Start with cells that have no dependencies
        for (cell, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(cell.clone());
            }
        }
        
        while let Some(cell) = queue.pop_front() {
            result.push(cell.clone());
            
            // Reduce in-degree for dependents
            if let Some(dependents) = self.dependents.get(&cell) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }
        
        if result.len() != all_cells.len() {
            Err("Circular dependency detected in spreadsheet".to_string())
        } else {
            Ok(result)
        }
    }
}

// ===== VERSION HISTORY IMPLEMENTATION =====

impl VersionHistory {
    /// Add a new snapshot
    pub fn add_snapshot(&mut self, snapshot: SpreadsheetSnapshot) {
        // Remove any versions after current position (for redo functionality)
        self.versions.truncate(self.current_version);
        
        // Add new snapshot
        self.versions.push(snapshot);
        self.current_version = self.versions.len();
        
        // Maintain max versions limit
        if self.versions.len() > self.max_versions {
            self.versions.remove(0);
            self.current_version = self.versions.len();
        }
    }
    
    /// Undo to previous version
    pub fn undo(&mut self) -> Option<&SpreadsheetSnapshot> {
        if self.current_version > 1 {
            self.current_version -= 1;
            self.versions.get(self.current_version - 1)
        } else {
            None
        }
    }
    
    /// Redo to next version
    pub fn redo(&mut self) -> Option<&SpreadsheetSnapshot> {
        if self.current_version < self.versions.len() {
            self.current_version += 1;
            self.versions.get(self.current_version - 1)
        } else {
            None
        }
    }
    
    /// Check if undo is possible
    pub fn can_undo(&self) -> bool {
        self.current_version > 1
    }
    
    /// Check if redo is possible
    pub fn can_redo(&self) -> bool {
        self.current_version < self.versions.len()
    }
    
    /// Get current snapshot
    pub fn current_snapshot(&self) -> Option<&SpreadsheetSnapshot> {
        if self.current_version > 0 {
            self.versions.get(self.current_version - 1)
        } else {
            None
        }
    }
    
    /// Clear all history
    pub fn clear(&mut self) {
        self.versions.clear();
        self.current_version = 0;
    }
}

// ===== CELL TYPE PARSING AND VALIDATION =====

impl CellType {
    /// Parse cell type from input string with automatic type detection
    pub fn parse_from_input(input: &str) -> Self {
        if input.trim().is_empty() {
            return CellType::Empty;
        }
        
        let trimmed = input.trim();
        
        // Formula detection
        if trimmed.starts_with('=') {
            return CellType::Formula(trimmed.to_string());
        }
        
        // Boolean detection
        match trimmed.to_lowercase().as_str() {
            "true" | "yes" | "1" => return CellType::Boolean(true),
            "false" | "no" | "0" => return CellType::Boolean(false),
            _ => {}
        }
        
        // Uncertainty notation detection
        if let Some(uncertainty_cell) = Self::parse_uncertainty_notation(trimmed) {
            return uncertainty_cell;
        }
        
        // DateTime detection (ISO 8601 format)
        if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
            return CellType::DateTime(dt.with_timezone(&Utc));
        }
        
        // Date detection (YYYY-MM-DD)
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            return CellType::Date(date);
        }
        
        // Alternative date formats
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%m/%d/%Y") {
            return CellType::Date(date);
        }
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%d/%m/%Y") {
            return CellType::Date(date);
        }
        
        // Time detection (HH:MM:SS)
        if let Ok(time) = NaiveTime::parse_from_str(trimmed, "%H:%M:%S") {
            return CellType::Time(time);
        }
        if let Ok(time) = NaiveTime::parse_from_str(trimmed, "%H:%M") {
            return CellType::Time(time);
        }
        
        // Duration detection
        if let Some(duration) = Self::parse_duration(trimmed) {
            return CellType::Duration(duration);
        }
        
        // Number detection
        if let Ok(num) = trimmed.parse::<f64>() {
            return CellType::Number(num);
        }
        
        // Scientific notation
        if let Ok(num) = Self::parse_scientific_notation(trimmed) {
            return CellType::Number(num);
        }
        
        // Default to text
        CellType::Text(trimmed.to_string())
    }
    
    /// Parse uncertainty notation (5.2 ± 0.1, 5.2 +/- 0.1, 5.2 ± 2%)
    fn parse_uncertainty_notation(input: &str) -> Option<CellType> {
        // Try different uncertainty separators
        let separators = ["±", "+/-", "+-"];
        
        for sep in &separators {
            if let Some(pos) = input.find(sep) {
                let value_str = input[..pos].trim();
                let uncertainty_str = input[pos + sep.len()..].trim();
                
                if let Ok(value) = value_str.parse::<f64>() {
                    // Check if uncertainty is percentage
                    if uncertainty_str.ends_with('%') {
                        let percent_str = &uncertainty_str[..uncertainty_str.len() - 1];
                        if let Ok(percent) = percent_str.parse::<f64>() {
                            let uncertainty = value * percent / 100.0;
                            return Some(CellType::NumberWithUncertainty {
                                value,
                                uncertainty,
                                uncertainty_type: UncertaintyType::Percentage,
                            });
                        }
                    } else if let Ok(uncertainty) = uncertainty_str.parse::<f64>() {
                        return Some(CellType::NumberWithUncertainty {
                            value,
                            uncertainty,
                            uncertainty_type: UncertaintyType::Absolute,
                        });
                    }
                }
            }
        }
        None
    }
    
    /// Parse scientific notation (1.23e-4, 1.23E+5)
    fn parse_scientific_notation(input: &str) -> Result<f64, std::num::ParseFloatError> {
        // Handle both 'e' and 'E'
        let normalized = input.replace('E', "e");
        normalized.parse::<f64>()
    }
    
    /// Parse duration strings (2h 30m, 02:30:00, 150 minutes)
    fn parse_duration(input: &str) -> Option<Duration> {
        let input = input.to_lowercase();
        
        // Try HH:MM:SS format
        if let Ok(time) = NaiveTime::parse_from_str(&input, "%H:%M:%S") {
            let seconds = time.hour() as i64 * 3600 + time.minute() as i64 * 60 + time.second() as i64;
            return Duration::try_seconds(seconds);
        }
        
        // Try HH:MM format
        if let Ok(time) = NaiveTime::parse_from_str(&input, "%H:%M") {
            let seconds = time.hour() as i64 * 3600 + time.minute() as i64 * 60;
            return Duration::try_seconds(seconds);
        }
        
        // Try parsing components (2h 30m 15s)
        let mut total_seconds = 0i64;
        let re = Regex::new(r"(\d+)\s*([dhms])").ok()?;
        
        for cap in re.captures_iter(&input) {
            let value: i64 = cap[1].parse().ok()?;
            let unit = &cap[2];
            
            let seconds = match unit {
                "d" => value * 24 * 3600,
                "h" => value * 3600,
                "m" => value * 60,
                "s" => value,
                _ => continue,
            };
            total_seconds += seconds;
        }
        
        if total_seconds > 0 {
            Duration::try_seconds(total_seconds)
        } else {
            None
        }
    }
    
    /// Check if cell type supports uncertainty
    pub fn supports_uncertainty(&self) -> bool {
        matches!(self, CellType::Number(_) | CellType::NumberWithUncertainty { .. })
    }
    
    /// Check if cell type supports units
    pub fn supports_units(&self) -> bool {
        matches!(self, CellType::Number(_) | CellType::NumberWithUncertainty { .. })
    }
    
    /// Check if cell type supports time operations
    pub fn supports_time_operations(&self) -> bool {
        matches!(
            self,
            CellType::DateTime(_) | CellType::Date(_) | CellType::Time(_) | CellType::Duration(_)
        )
    }
    
    /// Convert cell type to another type if possible
    pub fn convert_to(&self, target_type: &str) -> Result<CellType, String> {
        match (self, target_type) {
            (CellType::Number(n), "text") => Ok(CellType::Text(n.to_string())),
            (CellType::Text(s), "number") => {
                s.parse::<f64>()
                    .map(CellType::Number)
                    .map_err(|_| format!("Cannot convert '{}' to number", s))
            }
            (CellType::Boolean(b), "text") => Ok(CellType::Text(b.to_string())),
            (CellType::Text(s), "boolean") => {
                match s.to_lowercase().as_str() {
                    "true" | "yes" | "1" => Ok(CellType::Boolean(true)),
                    "false" | "no" | "0" => Ok(CellType::Boolean(false)),
                    _ => Err(format!("Cannot convert '{}' to boolean", s)),
                }
            }
            _ => Err(format!("Conversion from {:?} to {} not supported", self, target_type)),
        }
    }
    
    /// Validate cell type against validation rules
    pub fn validate(&self, rules: &[ValidationRule]) -> Vec<String> {
        let mut errors = Vec::new();
        
        for rule in rules {
            match rule {
                ValidationRule::Range { min, max } => {
                    if let Some(value) = self.get_numeric_value() {
                        if let Some(min_val) = min {
                            if value < *min_val {
                                errors.push(format!("Value {} is below minimum {}", value, min_val));
                            }
                        }
                        if let Some(max_val) = max {
                            if value > *max_val {
                                errors.push(format!("Value {} is above maximum {}", value, max_val));
                            }
                        }
                    }
                }
                ValidationRule::DataType { expected_type } => {
                    if !self.matches_type(expected_type) {
                        errors.push(format!("Expected type {}, got {:?}", expected_type, self));
                    }
                }
                ValidationRule::Pattern { regex } => {
                    if let Ok(re) = Regex::new(regex) {
                        if let Some(text) = self.get_text_value() {
                            if !re.is_match(&text) {
                                errors.push(format!("Value '{}' does not match pattern '{}'", text, regex));
                            }
                        }
                    }
                }
                _ => {} // Other validation rules handled elsewhere
            }
        }
        
        errors
    }
    
    /// Get numeric value if applicable
    fn get_numeric_value(&self) -> Option<f64> {
        match self {
            CellType::Number(n) => Some(*n),
            CellType::NumberWithUncertainty { value, .. } => Some(*value),
            _ => None,
        }
    }
    
    /// Get text representation
    fn get_text_value(&self) -> Option<String> {
        match self {
            CellType::Text(s) => Some(s.clone()),
            CellType::Number(n) => Some(n.to_string()),
            CellType::Boolean(b) => Some(b.to_string()),
            _ => None,
        }
    }
    
    /// Check if cell type matches expected type string
    fn matches_type(&self, expected: &str) -> bool {
        match (self, expected.to_lowercase().as_str()) {
            (CellType::Number(_), "number") => true,
            (CellType::Text(_), "text") => true,
            (CellType::Boolean(_), "boolean") => true,
            (CellType::DateTime(_), "datetime") => true,
            (CellType::Date(_), "date") => true,
            (CellType::Time(_), "time") => true,
            (CellType::Duration(_), "duration") => true,
            (CellType::Formula(_), "formula") => true,
            (CellType::NumberWithUncertainty { .. }, "uncertainty") => true,
            _ => false,
        }
    }
}