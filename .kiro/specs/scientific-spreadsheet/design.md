# Design Document

## Overview

The Scientific Spreadsheet is a sophisticated data analysis tool built on the Tauri framework, combining a React/TypeScript frontend with a high-performance Rust backend. The design emphasizes uncertainty handling, unit support, and scientific computing capabilities while maintaining the familiar spreadsheet interface that scientists expect.

The system uses react-datasheet-grid as the foundation for the grid interface, enhanced with custom cell types that support dual-input uncertainty cells, unit validation, and formula evaluation. The Rust backend provides the computational engine for formula parsing, uncertainty propagation through numerical differentiation, and scientific function evaluation.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React/TypeScript)              │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Grid Component │  │  Toolbar System │  │ Visualization   │ │
│  │ (react-datasheet-│  │                 │  │   Components    │ │
│  │      grid)       │  │                 │  │                 │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Custom Cell     │  │  Formula Bar    │  │  Quality Control│ │
│  │    Types        │  │                 │  │    Components   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                      Tauri IPC Layer                           │
├─────────────────────────────────────────────────────────────────┤
│                    Backend (Rust)                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Formula Engine  │  │ Uncertainty     │  │ Unit System     │ │
│  │                 │  │  Propagation    │  │                 │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Scientific      │  │ Data Storage    │  │ Performance     │ │
│  │  Functions      │  │                 │  │   Optimization  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Component Architecture

#### Frontend Components (TypeScript/React) - Visual Only
- **SpreadsheetGrid**: Visual grid rendering using react-datasheet-grid (display only)
- **UncertaintyCellRenderer**: Visual rendering of dual-input uncertainty cells
- **FormulaBarRenderer**: Visual display of formula bar with syntax highlighting
- **ToolbarRenderer**: Visual rendering of toolbar components and controls
- **VisualizationRenderer**: Chart and plot rendering using Plotly.js
- **QualityControlRenderer**: Visual indicators for data quality and outliers
- **UIComponents**: Pure visual components for styling and layout
- **EventCapture**: Capture user events and forward to Rust backend

#### Backend Modules (Rust) - All Logic
- **ApplicationState**: Complete application state management
- **CellManager**: All cell logic, editing, validation, and state
- **FormulaEngine**: Expression parsing, evaluation, and dependency tracking
- **UncertaintyEngine**: Numerical differentiation and Monte Carlo methods
- **UnitSystem**: Dimensional analysis, unit conversion, and validation
- **ScientificFunctions**: Mathematical and statistical function library
- **ToolbarLogic**: All toolbar functionality and range operations
- **QualityController**: Outlier detection, validation rules, and data quality
- **VisualizationEngine**: Data preparation and plot configuration
- **DataManager**: Persistent storage, file I/O, and metadata management
- **PerformanceOptimizer**: Caching, background processing, and threading
- **UserInteractionHandler**: Process all user interactions and update UI state

## Logic Distribution Strategy

### Frontend Logic (TypeScript/React) - Visual Only
**Responsibilities:**
- Component rendering and DOM manipulation
- CSS styling and visual state (hover, focus, selection highlights)
- Animation and transition effects
- Layout management and responsive design
- Event capture and forwarding to Rust backend
- Display formatting (colors, fonts, visual indicators)

### Backend Logic (Rust) - All Business Logic
**Responsibilities:**
- All application state management
- Input validation and processing
- Cell editing logic and data management
- Formula parsing and mathematical evaluation
- Uncertainty propagation calculations
- Unit system operations and conversions
- Scientific function implementations
- Toolbar logic and range operations
- Data visualization data preparation
- Quality control and outlier detection
- File I/O and data persistence
- Performance optimization and caching
- Background processing and threading
- User interaction logic and business rules

### Communication Pattern
```typescript
// Frontend only handles visual events, sends everything to Rust
const handleCellClick = async (row: number, col: number, clickX: number) => {
  const result = await invoke('handle_cell_click', {
    row, col, clickX,
    currentState: 'any_needed_visual_state'
  });
  
  // Rust returns complete UI update instructions
  updateUI(result.uiUpdates);
};

// Rust handles all logic and returns UI instructions
interface UIUpdateInstructions {
  cellUpdates: CellUpdate[];
  selectionChanges: SelectionUpdate[];
  toolbarUpdates: ToolbarUpdate[];
  errorMessages?: string[];
}
```

## Components and Interfaces

### Custom Cell Types

#### Adaptive Cell Rendering System
The system provides intelligent cell rendering that adapts to the data type and user interaction:

```typescript
interface AdaptiveCellRenderer {
  // Visual rendering based on cell type
  renderCell(cellType: CellType, metadata: CellMetadata): JSX.Element;
  
  // Input handling
  handleInput(input: string, currentType: CellType): CellType;
  
  // Visual modes
  getVisualMode(cellType: CellType): 'text' | 'number' | 'uncertainty' | 'boolean' | 'date';
}
```

**Supported Cell Types and Rendering:**

1. **Text Cells**: Standard text input and display
2. **Number Cells**: Numeric input with formatting options
3. **Uncertainty Cells**: Dual-input interface with "±" separator
4. **Boolean Cells**: Checkbox or dropdown (true/false/null)
5. **DateTime Cells**: Full date-time picker with timezone support
6. **Date Cells**: Date-only picker with various format options
7. **Time Cells**: Time-only picker with 12/24 hour formats
8. **Duration Cells**: Duration input (hours, minutes, seconds, days)
9. **Formula Cells**: Formula bar integration with syntax highlighting
10. **Error Cells**: Error display with diagnostic information

**Date/Time Features:**
- **Automatic Parsing**: "2024-01-15 14:30:00", "Jan 15, 2024 2:30 PM", "15/01/2024"
- **Multiple Formats**: ISO 8601, US, European, custom formats
- **Time Zones**: UTC storage with local display options
- **Duration Arithmetic**: Add/subtract durations from dates
- **Business Day Calculations**: Exclude weekends and holidays
- **Relative Dates**: "today", "yesterday", "+7 days", "next Monday"

**Key Features:**
- Automatic type detection from user input
- Seamless switching between cell types
- Type-specific validation and formatting
- Uncertainty support only for numeric types
- Unit support for numeric and uncertainty types

#### Implementation Strategy:
1. Create custom cell renderers for each type in react-datasheet-grid
2. Implement type detection logic in Rust backend
3. Use conditional rendering based on cell type
4. Maintain visual consistency across all cell types
5. Support keyboard and mouse navigation for all types

### Formula Engine

#### Expression Parser
```rust
pub struct FormulaEngine {
    parser: ExpressionParser,
    evaluator: ExpressionEvaluator,
    dependency_tracker: DependencyGraph,
}

pub struct Expression {
    ast: ExpressionNode,
    dependencies: Vec<CellReference>,
    uncertainty_method: UncertaintyMethod,
}

pub enum UncertaintyMethod {
    NumericalDifferentiation,
    MonteCarlo { samples: usize },
    Analytical, // For simple cases
}
```

**Parsing Strategy:**
1. Use `nom` crate for robust expression parsing
2. Build Abstract Syntax Tree (AST) for expressions
3. Identify cell references and function calls
4. Determine optimal uncertainty propagation method

#### Numerical Differentiation Engine
```rust
pub struct UncertaintyPropagator {
    step_size_optimizer: StepSizeOptimizer,
    derivative_calculator: NumericalDerivatives,
    monte_carlo_fallback: MonteCarloEngine,
}

impl UncertaintyPropagator {
    pub fn propagate_uncertainty(
        &self,
        expression: &Expression,
        inputs: &[UncertainValue],
    ) -> Result<UncertainValue, UncertaintyError> {
        match self.try_numerical_differentiation(expression, inputs) {
            Ok(result) => Ok(result),
            Err(_) => self.monte_carlo_fallback.propagate(expression, inputs),
        }
    }
}
```

**Numerical Methods:**
- Forward, backward, and central difference methods
- Adaptive step size optimization
- Richardson extrapolation for improved accuracy
- Automatic fallback to Monte Carlo for complex cases

### Unit System Integration

#### Leveraging Existing Unit System
The design will build upon the already implemented comprehensive unit conversion system in `src-tauri/src/unit_conversion/`:

```rust
// Existing system provides:
pub struct UnitConverter {
    base_units: HashMap<String, BaseUnit>,     // 1000+ units across 25+ categories
    prefixes: HashMap<String, f64>,            // SI prefixes (Y, Z, E, P, T, G, M, k, etc.)
    categories: HashMap<String, Vec<String>>,  // Organized unit categories
    quick_conversions: HashMap<String, HashMap<String, f64>>, // Fast conversions
}

pub struct Dimension {
    pub mass: i32, pub length: i32, pub time: i32, pub current: i32,
    pub temperature: i32, pub amount: i32, pub luminosity: i32,
}
```

**Existing Capabilities to Leverage:**
- **1000+ Units**: Comprehensive coverage across length, mass, time, temperature, energy, power, etc.
- **Dimensional Analysis**: Full dimensional compatibility checking with SI base units
- **Complex Unit Parsing**: Supports expressions like "kg·m²/s²", "m/s^2", Unicode superscripts
- **Temperature Handling**: Special logic for temperature conversions (°C, °F, K, °R, °Ré)
- **Quick Conversions**: Pre-calculated common conversions for performance
- **Prefix Support**: Full SI prefix system (yotta to yocto)

#### Spreadsheet Integration Strategy
```rust
// Extend existing commands for spreadsheet use
#[command]
pub async fn validate_cell_unit(unit: String) -> Result<bool, String> {
    // Use existing validate_unit_string
}

#[command]
pub async fn convert_cell_value(value: f64, from_unit: String, to_unit: String) -> Result<ConversionResult, String> {
    // Use existing convert_value with enhanced error handling
}

#[command]
pub async fn get_unit_suggestions(partial: String) -> Result<Vec<UnitInfo>, String> {
    // New command for auto-complete using existing unit registry
}
```

### Data Models

#### Core Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCell {
    pub content: String,
    pub metadata: CellMetadata,
    pub computed_value: Option<ComputedValue>,
    pub cell_type: CellType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellType {
    Empty,
    Text(String),
    Number(f64),
    NumberWithUncertainty { value: f64, uncertainty: f64, uncertainty_type: UncertaintyType },
    Boolean(bool),
    DateTime(chrono::DateTime<chrono::Utc>),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
    Duration(chrono::Duration),
    Formula(String),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    pub unit: Option<String>,
    pub format: CellFormat,
    pub is_formula: bool,
    pub quality_flags: Vec<QualityFlag>,
    pub experimental_context: Option<ExperimentalContext>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellFormat {
    Auto,
    Number { precision: usize, notation: NumberNotation },
    Scientific { precision: usize },
    Percentage { precision: usize },
    Boolean,
    Text,
    DateTime { format: String }, // e.g., "YYYY-MM-DD HH:mm:ss", "MM/DD/YYYY h:mm AM/PM"
    Date { format: String },     // e.g., "YYYY-MM-DD", "MM/DD/YYYY", "DD/MM/YYYY"
    Time { format: String },     // e.g., "HH:mm:ss", "h:mm AM/PM", "HH:mm"
    Duration { format: String }, // e.g., "HH:mm:ss", "D days H hours", "seconds"
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NumberNotation {
    Standard,
    Scientific,
    Engineering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UncertaintyType {
    Absolute,
    Percentage,
    StandardDeviation,
    StandardError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputedValue {
    Number(f64),
    NumberWithUncertainty { value: f64, uncertainty: f64 },
    Boolean(bool),
    Text(String),
    DateTime(chrono::DateTime<chrono::Utc>),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
    Duration(chrono::Duration),
    Array(Vec<ComputedValue>),
    Error(String),
}
```

#### Cell Type Intelligence
The system automatically determines cell type based on input:

```rust
impl CellType {
    pub fn parse_from_input(input: &str) -> CellType {
        // Auto-detection logic:
        // "5.2 ± 0.1" -> NumberWithUncertainty
        // "true" or "false" -> Boolean  
        // "2024-01-15 14:30:00" -> DateTime
        // "2024-01-15" -> Date
        // "14:30:00" -> Time
        // "2h 30m" or "02:30:00" -> Duration
        // "=A1+B1" -> Formula
        // "123.45" -> Number
        // Everything else -> Text
    }
    
    pub fn supports_uncertainty(&self) -> bool {
        matches!(self, CellType::Number(_) | CellType::NumberWithUncertainty { .. })
    }
    
    pub fn supports_units(&self) -> bool {
        matches!(self, CellType::Number(_) | CellType::NumberWithUncertainty { .. })
    }
    
    pub fn supports_time_operations(&self) -> bool {
        matches!(self, CellType::DateTime(_) | CellType::Date(_) | CellType::Time(_) | CellType::Duration(_))
    }
}
```

#### Spreadsheet State Management
```rust
pub struct SpreadsheetState {
    cells: HashMap<CellReference, UnifiedCell>,
    dependency_graph: DependencyGraph,
    calculation_cache: CalculationCache,
    metadata_store: MetadataStore,
    version_history: VersionHistory,
}
```

### Performance Optimization

#### Caching Strategy
```rust
pub struct CalculationCache {
    formula_results: LruCache<FormulaHash, ComputedValue>,
    uncertainty_derivatives: LruCache<DerivativeKey, f64>,
    unit_conversions: LruCache<ConversionKey, f64>,
    monte_carlo_results: LruCache<MonteCarloKey, UncertaintyResult>,
}
```

**Caching Levels:**
1. **Formula Results**: Cache computed values with dependency tracking
2. **Numerical Derivatives**: Cache partial derivatives for reuse
3. **Unit Conversions**: Cache conversion factors
4. **Monte Carlo Results**: Cache simulation results for similar parameters

#### Background Processing
```rust
pub struct BackgroundProcessor {
    calculation_queue: Arc<Mutex<VecDeque<CalculationTask>>>,
    worker_pool: ThreadPool,
    progress_tracker: ProgressTracker,
}

pub enum CalculationTask {
    FormulaEvaluation { cell: CellReference, priority: Priority },
    UncertaintyPropagation { cells: Vec<CellReference> },
    MonteCarloSimulation { parameters: MonteCarloParams },
    QualityAnalysis { range: CellRange },
}
```

## Error Handling

### Error Types and Recovery
```rust
#[derive(Debug, thiserror::Error)]
pub enum SpreadsheetError {
    #[error("Formula parsing error: {message} at position {position}")]
    FormulaParseError { message: String, position: usize },
    
    #[error("Unit compatibility error: cannot convert {from} to {to}")]
    UnitCompatibilityError { from: String, to: String },
    
    #[error("Uncertainty propagation failed: {reason}")]
    UncertaintyError { reason: String },
    
    #[error("Circular reference detected: {chain:?}")]
    CircularReferenceError { chain: Vec<CellReference> },
    
    #[error("Numerical instability in calculation")]
    NumericalInstabilityError,
}
```

**Error Recovery Strategies:**
- Graceful degradation for uncertainty calculations
- Alternative calculation methods for numerical issues
- Clear error messages with correction suggestions
- Partial results when possible

## Testing Strategy

### Unit Testing
- **Formula Engine**: Test expression parsing and evaluation
- **Uncertainty Propagation**: Validate numerical differentiation accuracy
- **Unit System**: Test conversion accuracy and dimensional analysis
- **Cell Types**: Test custom cell behavior and state management

### Integration Testing
- **Frontend-Backend Communication**: Test Tauri IPC reliability
- **End-to-End Workflows**: Test complete user scenarios
- **Performance Testing**: Validate large dataset handling
- **Cross-Platform Testing**: Ensure compatibility across operating systems

### Validation Testing
- **Scientific Accuracy**: Compare results with reference implementations
- **Uncertainty Validation**: Test against analytical solutions where available
- **Unit Conversion Accuracy**: Validate against NIST standards
- **Statistical Function Validation**: Compare with established libraries

## Security and Data Integrity

### Data Protection
- **Input Validation**: Sanitize all user inputs
- **Formula Sandboxing**: Prevent malicious formula execution
- **File Integrity**: Checksums and validation for saved files
- **Memory Safety**: Leverage Rust's memory safety guarantees

### Audit Trail
- **Change Tracking**: Record all modifications with timestamps
- **User Attribution**: Track individual contributions
- **Version Control**: Maintain complete history of changes
- **Compliance Support**: Generate reports for regulatory requirements

## Deployment and Distribution

### Build Configuration
- **Tauri Bundle**: Native application packaging
- **Cross-Platform Support**: Windows, macOS, Linux builds
- **Dependency Management**: Automated dependency resolution
- **Update System**: Secure automatic updates

### Performance Monitoring
- **Metrics Collection**: Track calculation performance and accuracy
- **Error Reporting**: Automated error collection and analysis
- **Usage Analytics**: Understand user workflows and bottlenecks
- **Resource Monitoring**: Track memory and CPU usage patterns

This design provides a comprehensive foundation for implementing the scientific spreadsheet with advanced uncertainty handling, while maintaining the flexibility to adapt and extend functionality as requirements evolve.