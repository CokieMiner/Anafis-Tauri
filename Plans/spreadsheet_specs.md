# AnaFis Spreadsheet Specifications - Organized by Architecture

## 1. CORE ARCHITECTURE (Backend + Frontend)

### Unified Cell System **[Backend + Frontend]**
- **Single Cell Type**: Combined number/uncertainty cell that handles both direct values and formulas
- **Uncertainty Support**:
  - Absolute uncertainty: `value ± uncertainty` (e.g., "5.2 ± 0.1")
  - Percentage uncertainty: `value ± percentage%` (e.g., "5.2 ± 2%")
  - Automatic conversion between absolute and percentage representations
- **Formula Support**: All cells support formulas with `=` prefix
- **Scientific Notation**: Support for exponential notation with configurable precision

### Data Structures **[Backend]**
```typescript
interface UnifiedCell {
  content: string;        // Display value, uncertainty notation, or formula with '=' prefix
  metadata: {
    unit?: string;        // Unit specification if applicable
    uncertainty?: {
      type: 'absolute' | 'percentage';
      value: number;      // Absolute uncertainty or percentage
    };
    format: {
      notation: 'standard' | 'scientific';
      precision: number;  // Significant figures
    };
    isFormula: boolean;   // Whether content is a formula
  };
}
```

### Calculation Engine **[Backend]**
- **Rust-Based Formula Evaluator**: Native Rust evaluation engine with unit awareness
- **Performance Optimization**: Leverage Rust's speed for complex calculations
- **Uncertainty Propagation**: Built-in uncertainty calculations through formula chains
- **Dimensional Analysis**: Automatic unit compatibility checking
- **Cell Reference Substitution**: Support for `#` placeholder in range operations

## 2. USER INTERFACE & INPUT (Frontend)

### Data Input & Editing **[Frontend]**
- **Smart Parsing**: Automatic detection of uncertainty notation and formulas
- **Unified Editor**: Single input field that handles values, uncertainties, and formulas
- **Visual Indicators**: Clear display of uncertainty type and formula status
- **Content-Type Preservation**: Maintains cell characteristics during editing operations

### Toolbar System Components **[Frontend]**

#### Unit Input Component **[Frontend + Backend]**
- **Flexible Input**: Type custom unit strings or select from predefined list
- **Real-time Validation**: Parser checks if unit is supported by existing system
- **Auto-complete**: Suggestions from 1000+ supported units across 25+ categories
- **Syntax Support**: Complex unit expressions (kg·m²/s²) with proper formatting

#### Formatting Controls **[Frontend]**
- **Number Format Selector**: Scientific, decimal, percentage, fraction formats
- **Precision Adjustment**: Significant figures control (1-15 digits)
- **Scientific Notation Toggle**: Exponential notation with configurable precision
- **Bulk Operations**: Apply formatting to selected ranges or entire columns

#### Enhanced Range Operations **[Frontend + Backend]**
- **Formula Application**: Set same formula for all selected cells with cell reference substitution:
  - Use `A#` to substitute row index (e.g., A5, A6, A7)
  - Use `#12` to substitute column index (e.g., A12, B12, C12)
  - Use `#` to substitute both row and column as needed
- **Unit Management**:
  - Apply units to selection with validation
  - Convert between compatible units in selection
  - Clear units from selection while preserving values
- **Formatting Operations**:
  - Bulk apply number formats
  - Bulk set precision levels
  - Bulk toggle scientific notation

#### Calculation Controls **[Frontend + Backend]**
- **Recalculation**: Manual trigger for formula recalculation
- **Uncertainty Propagation**: Calculate uncertainty through formula chains
- **Dimensional Analysis**: Validate unit consistency across calculations
- **Dependency Viewer**: Menu showing formula dependencies and precedents

### Scientific Keyboard & Input **[Frontend]**

#### Special Character Palette **[Frontend]**
- **Mathematical Symbols**: Easy access to ∑, ∫, ∂, ∇, and other symbols
- **Greek Letters**: Full Greek alphabet (α, β, γ, μ, σ, etc.)
- **Unit Symbols**: °C, °F, °, %, ‰, and scientific unit abbreviations

#### Formula Auto-complete **[Frontend]**
- **Intelligent Suggestions**: Context-aware function and unit recommendations
- **Syntax Assistance**: Real-time formula validation and correction
- **Function Documentation**: Inline help for scientific functions

#### Keyboard Shortcuts **[Frontend]**
- **Scientific Workflow**: Optimized shortcuts for common scientific operations
- **Formula Entry**: Quick access to mathematical operators and functions
- **Navigation**: Efficient movement through large datasets

#### Touch Interface Support **[Frontend]**
- **Tablet Optimization**: Large touch targets for laboratory environments
- **Gesture Support**: Pinch-to-zoom, swipe navigation
- **Stylus Input**: Precise input for handwritten formulas and annotations

## 3. DATA MANAGEMENT & STORAGE (Backend + Frontend)

### Cross-Tab Integration System **[Backend + Frontend]**

#### Shared Data Storage **[Backend]**
- **Tagged Data Sets**: Ability to select ranges and store with descriptive tags/names
- **Application-wide Access**: All tabs can access and utilize stored data sets
- **Metadata Preservation**: Full unit and uncertainty metadata maintained in shared storage
- **Version Management**: Track revisions to stored data sets

### Cross-Tab Integration System **[Backend + Frontend]**

#### Shared Data Storage **[Backend]**
- **Tagged Data Sets**: Ability to select ranges and store with descriptive tags/names
- **Application-wide Access**: All tabs can access and utilize stored data sets
- **Metadata Preservation**: Full unit and uncertainty metadata maintained in shared storage
- **Version Management**: Track revisions to stored data sets

#### Local Data Sharing **[Backend]**
- **Cross-Tab Data Exchange**: Fast sharing of datasets between different spreadsheet tabs
- **Temporary Data Buffers**: Store intermediate results accessible across windows
- **Shared Calculations**: Cache expensive computations for reuse across tabs
- **Window Synchronization**: Coordinate data updates between multiple open windows

#### Integration Points **[Frontend + Backend]**
- **Solver Integration**: Pass tagged data sets to solver tab for optimization problems
- **Plotting Integration**: Generate graphs from tagged data sets with proper units
- **Report Generation**: Export tagged data to reports with formatting preserved
- **Extrapolation Tools**: Project trends beyond existing data ranges (no curve fitting)

#### Integration Points **[Frontend + Backend]**
- **Solver Integration**: Pass tagged data sets to solver tab for optimization problems
- **Plotting Integration**: Generate graphs from tagged data sets with proper units
- **Report Generation**: Export tagged data to reports with formatting preserved
- **Extrapolation Tools**: Project trends beyond existing data ranges (no curve fitting)

### Metadata Enhancement **[Backend + Frontend]**

#### Experimental Context **[Backend]**
- **Instrument Calibration**: Storage of calibration data, dates, and certificates
- **Environmental Conditions**: Temperature, pressure, humidity tracking
- **Operator Information**: User identification and methodology documentation
- **Measurement Protocol**: Detailed experimental procedures and conditions

#### Provenance Tracking **[Backend]**
- **Full History**: Complete audit trail of all data transformations and calculations
- **Transformation Chain**: Step-by-step record of data processing operations
- **Reproducibility**: Ability to recreate exact analysis conditions

### Metadata Enhancement **[Backend + Frontend]**

#### Experimental Context **[Backend]**
- **Instrument Calibration**: Storage of calibration data, dates, and certificates
- **Environmental Conditions**: Temperature, pressure, humidity tracking
- **Operator Information**: User identification and methodology documentation
- **Measurement Protocol**: Detailed experimental procedures and conditions

#### Provenance Tracking **[Backend]**
- **Full History**: Complete audit trail of all data transformations and calculations
- **Transformation Chain**: Step-by-step record of data processing operations
- **Reproducibility**: Ability to recreate exact analysis conditions

#### Measurement Uncertainty Budget **[Backend + Frontend]**
- **Detailed Breakdown**: Individual uncertainty sources and their contributions
- **Uncertainty Propagation**: Mathematical combination of different uncertainty types
- **Budget Visualization**: Graphical representation of uncertainty components

#### Local Metadata Caching **[Backend]**
- **Frequently Accessed Metadata**: Cache commonly used unit definitions and conversion factors
- **User Preferences**: Store personalized settings and recent activity
- **Lookup Tables**: Fast access to scientific constants and reference data
- **Temporary Annotations**: Cache cell comments and temporary notes

### Version Control Integration **[Backend + Frontend]**

#### Change Tracking **[Backend]**
- **Detailed Audit Trail**: All modifications with timestamps and authorship
- **Cell-Level History**: Individual cell change tracking
- **Bulk Operation Logging**: Recording of range operations and transformations

#### Experiment Snapshotting **[Backend]**
- **Complete State Saving**: Full spreadsheet state with all metadata
- **Version Tagging**: Descriptive labels for different experimental states
- **Quick Restore**: One-click restoration of previous states

#### Comparison Tools **[Frontend]**
- **Side-by-Side Comparison**: Visual diff of different versions
- **Change Highlighting**: Clear indication of modified cells and ranges
- **Difference Quantification**: Statistical summary of changes between versions

#### Annotation System **[Frontend + Backend]**
- **Cell Comments**: Notes and explanations attached to specific cells
- **Range Annotations**: Documentation for data ranges and calculations
- **Personal Notes**: User annotations for analysis and documentation

## 4. SCIENTIFIC FUNCTIONS & ANALYSIS (Backend)

### Evaluation Features **[Backend]**
- **Formula Parsing**: Advanced parsing of mathematical expressions with unit support
- **Function Library**: Scientific, statistical, and engineering functions
- **Error Handling**: Comprehensive error reporting with detailed messages
- **Extrapolation Support**: Mathematical functions for trend projection

### Specialized Scientific Functions **[Backend]**

#### Mathematical & Statistical Operations **[Backend]**
- **Complex Number Support**: Full operations with complex numbers (a+bi format)
- **Matrix Operations**: Multiplication, inversion, determinant with unit preservation
- **Statistical Tests**: t-tests, ANOVA, chi-square with uncertainty propagation
- **Distribution Functions**: Normal, Poisson, binomial with parameter estimation

#### Interpolation Methods **[Backend]**
- **Multiple Algorithms**: Linear, cubic spline, polynomial interpolation
- **Error Estimates**: Uncertainty quantification for interpolated values
- **Extrapolation Bounds**: Safe extrapolation limits based on data quality

#### Unit Algebra **[Backend]**
- **Automatic Simplification**: Complex unit expressions (kg·m/s² → N)
- **Dimensional Analysis**: Validation of physical relationships
- **Unit Conversion**: Intelligent conversion between equivalent units

#### Dimensionless Quantities **[Backend]**
- **Proper Handling**: Reynolds, Prandtl, Nusselt numbers with context
- **Automatic Calculation**: Derived dimensionless parameters from base measurements
- **Validation Rules**: Physical constraints for dimensionless quantities

### Advanced Uncertainty Handling **[Backend]**

#### Monte Carlo Simulation **[Backend]**
- **Probabilistic Analysis**: Monte Carlo methods for complex uncertainty propagation
- **Distribution Sampling**: Support for various probability distributions (normal, uniform, triangular)
- **Result Statistics**: Mean, standard deviation, confidence intervals from simulation results

#### Correlation Handling **[Backend]**
- **Correlated Uncertainties**: Support for dependencies between measurement variables
- **Covariance Matrix**: Mathematical representation of uncertainty correlations
- **Correlation Coefficients**: Calculation and display of correlation relationships

#### Confidence Interval Calculation **[Backend]**
- **Multiple Levels**: Support for 1σ, 2σ, 3σ confidence intervals
- **Proper Interpretation**: Statistical guidance for confidence level selection
- **Visual Representation**: Error bars and confidence bands on plots

### Advanced Uncertainty Handling **[Backend]**

#### Monte Carlo Simulation **[Backend]**
- **Probabilistic Analysis**: Monte Carlo methods for complex uncertainty propagation
- **Distribution Sampling**: Support for various probability distributions (normal, uniform, triangular)
- **Result Statistics**: Mean, standard deviation, confidence intervals from simulation results

#### Correlation Handling **[Backend]**
- **Correlated Uncertainties**: Support for dependencies between measurement variables
- **Covariance Matrix**: Mathematical representation of uncertainty correlations
- **Correlation Coefficients**: Calculation and display of correlation relationships

#### Confidence Interval Calculation **[Backend]**
- **Multiple Levels**: Support for 1σ, 2σ, 3σ confidence intervals
- **Proper Interpretation**: Statistical guidance for confidence level selection
- **Visual Representation**: Error bars and confidence bands on plots

#### Systematic vs Random Uncertainty **[Backend]**
- **Separate Tracking**: Independent handling of systematic and random components
- **Combined Uncertainty**: Proper mathematical combination using RSS method
- **Uncertainty Budget**: Detailed breakdown by uncertainty type and source

#### Local Uncertainty Analysis **[Backend]**
- **Simulation Result Caching**: Store Monte Carlo results for similar parameter sets
- **Statistical Distribution Cache**: Cache pre-computed distribution functions
- **Correlation Matrix Storage**: Efficient storage of large covariance matrices
- **Multi-threaded Processing**: Utilize multiple CPU cores for uncertainty calculations

## 5. VISUALIZATION & GRAPHICS (Frontend + Backend)

### Integrated Data Visualization **[Frontend]**

#### Quick Plot Preview **[Frontend]**
- **Instant Visualization**: Real-time plotting of selected data ranges
- **Multiple Plot Types**: Line, scatter, bar, histogram charts
- **Interactive Preview**: Direct manipulation within spreadsheet interface

#### Error Bar Support **[Frontend]**
- **Proper Display**: Uncertainty visualization on all plot types
- **Configurable Levels**: 1σ, 2σ, 3σ error bars with legend
- **Statistical Interpretation**: Clear labeling of confidence levels

#### Residual Analysis **[Frontend]**
- **Model Fit Examination**: Visual comparison of data vs. model predictions
- **Discrepancy Highlighting**: Identification of outliers and systematic errors
- **Goodness-of-Fit Metrics**: R², chi-square, and other statistical measures

#### Interactive Graphs **[Frontend]**
- **Linked Selections**: Bidirectional selection between spreadsheet and plots
- **Dynamic Updates**: Real-time plot updates as data changes
- **Export Options**: High-quality image and vector format export

## 6. QUALITY CONTROL & VALIDATION (Backend + Frontend)

### Data Integrity & Quality Control **[Backend + Frontend]**

#### Data Validation Rules **[Backend]**
- **Range Constraints**: Configurable min/max values for data entry
- **Unit Compatibility Checks**: Automatic validation of unit consistency in calculations
- **Mathematical Consistency Validation**: Cross-checks between related measurements

#### Outlier Detection **[Backend + Frontend]**
- **Statistical Methods**: Grubbs' test and Chauvenet's criterion for anomaly detection
- **Visual Indicators**: Highlighting of potential outliers with configurable thresholds
- **Interactive Review**: Ability to accept/reject flagged outliers with justification

#### Data Smoothing **[Backend]**
- **Moving Average**: Configurable window sizes for noise reduction
- **Savitzky-Golay Filtering**: Polynomial smoothing with adjustable parameters
- **Preservation of Metadata**: Maintains uncertainty and unit information through smoothing

#### Quality Flags **[Frontend + Backend]**
- **Visual Indicators**: Color-coded flags for questionable data, missing values, estimated values
- **Quality Metrics**: Automated calculation of data quality scores
- **Filtering Options**: Ability to hide/show data based on quality flags

## 7. PERFORMANCE & OPTIMIZATION (Backend + Frontend)

## 7. PERFORMANCE & OPTIMIZATION (Backend + Frontend)

### Local Caching System **[Backend]**

#### Formula Result Caching **[Backend]**
- **Computational Results**: Cache expensive formula calculations with automatic invalidation
- **Intermediate Values**: Store intermediate calculation results to avoid redundant computations
- **Dependency Tracking**: Smart cache invalidation when dependencies change
- **Memory Management**: Configurable cache sizes with LRU eviction policies

#### Auto-save & Recovery **[Backend]**
- **Automatic Saving**: Periodic background saving of work-in-progress
- **Crash Recovery**: Restore unsaved work after application crashes
- **Backup Management**: Multiple backup versions with configurable retention
- **File Integrity**: Verify file integrity on load and save operations

#### Multi-threading & Background Processing **[Backend]**
- **Parallel Calculations**: Utilize multiple CPU cores for complex computations
- **Background Tasks**: Non-blocking execution of long-running operations
- **Progress Tracking**: Real-time progress updates with cancellation support
- **Resource Management**: CPU and memory usage monitoring and limits

### Calculation Profiling **[Backend + Frontend]**
- **Performance Analysis**: Identification of computational bottlenecks
- **Timing Reports**: Detailed execution time breakdown by operation
- **Optimization Suggestions**: Automated recommendations for performance improvement

### Selective Recalculation **[Backend]**
- **Smart Updates**: Only recalculate affected cells and dependencies
- **Incremental Processing**: Progressive calculation of large datasets
- **Dependency Optimization**: Efficient handling of complex calculation chains

### Memory Management **[Backend]**
- **Intelligent Caching**: Automatic caching of intermediate results
- **Memory Optimization**: Efficient memory usage for large spreadsheets
- **Garbage Collection**: Smart cleanup of unused data structures
- **Virtual Memory**: Handle datasets larger than available RAM

### Background Processing **[Backend + Frontend]**
- **Non-blocking UI**: Maintain responsive interface during long calculations
- **Progress Indicators**: Real-time progress feedback for long operations
- **Cancellation Support**: Ability to interrupt long-running calculations
- **Priority Queuing**: Manage multiple concurrent operations

### Calculation Profiling **[Backend + Frontend]**
- **Performance Analysis**: Identification of computational bottlenecks
- **Timing Reports**: Detailed execution time breakdown by operation
- **Optimization Suggestions**: Automated recommendations for performance improvement

### Selective Recalculation **[Backend]**
- **Smart Updates**: Only recalculate affected cells and dependencies
- **Incremental Processing**: Progressive calculation of large datasets
- **Dependency Optimization**: Efficient handling of complex calculation chains

### Cache Management **[Backend]**
- **Intelligent Caching**: Automatic caching of intermediate results
- **Memory Optimization**: Efficient memory usage for large spreadsheets
- **Cache Invalidation**: Smart cache clearing when data changes

### Background Processing **[Backend + Frontend]**
- **Non-blocking UI**: Maintain responsive interface during long calculations
- **Progress Indicators**: Real-time progress feedback for long operations
- **Cancellation Support**: Ability to interrupt long-running calculations

## 8. IMPLEMENTATION EXAMPLES

### Formula Application with Substitution **[Backend]**
```typescript
// Example: Applying "=A#*2" to cells B5, B6, B7 would create:
// B5: "=A5*2"
// B6: "=A6*2"
// B7: "=A7*2"

// Example: Applying "=#12*2" to cells C12, D12, E12 would create:
// C12: "=C12*2"
// D12: "=D12*2"
// E12: "=E12*2"
```

### Extrapolation Implementation **[Backend]**
- **Trend Projection**: Simple linear extrapolation from existing data
- **Uncertainty Propagation**: Estimate uncertainty in extrapolated values
- **Visual Indicators**: Clear marking of extrapolated versus measured data
- **Limitation Controls**: Set bounds on reasonable extrapolation ranges

---

## Desktop Application Optimization Summary

### Performance Improvements
- **50-90% faster** formula recalculation through intelligent local caching
- **Real-time responsiveness** for large spreadsheets with complex calculations
- **Reduced computational load** by avoiding redundant calculations
- **Multi-threaded processing** for Monte Carlo simulations and data analysis

### User Experience Enhancements
- **Seamless multi-window workflow** with cross-tab data sharing
- **Automatic crash recovery** with work preservation
- **Responsive interface** during heavy calculations through background processing
- **Cross-tab data sharing** with instant synchronization

### Scientific Workflow Optimization
- **Accelerated uncertainty analysis** through cached Monte Carlo results
- **Fast metadata access** for unit conversions and scientific constants
- **Efficient data sharing** between analysis tabs and visualization tools
- **Background processing** for long-running scientific computations

### System Reliability
- **Automatic backup** and recovery for critical calculations
- **File integrity verification** to prevent data corruption
- **Resource monitoring** for safe computational limits
- **Audit trails** and change tracking for scientific reproducibility

### Desktop-Specific Features
- **Native file system integration** with proper document handling
- **System resource optimization** for local hardware capabilities
- **Offline operation** without network dependencies
- **Local data security** with user-controlled access permissions

**Architecture Summary:**
- **[Backend]**: Core calculation engine, data processing, scientific functions, storage
- **[Frontend]**: User interface, visualization, input handling, display formatting
- **[Backend + Frontend]**: Data structures, validation, integration points, optimization
- **[Local Caching]**: High-performance caching, session management, background processing

This specification provides a comprehensive blueprint for implementing a scientific spreadsheet optimized for desktop application usage with native performance and reliability features.
