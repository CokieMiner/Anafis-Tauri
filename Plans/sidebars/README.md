# Sidebar Implementation Plans - Index

**Date**: November 2025  
**Branch**: master  
**Status**: Core Infrastructure Complete, Basic Sidebars Implemented

**Latest Update**: 2025-11-17 - Updated with plugin architecture for uncertainty propagation and current graphs/fitting implementation tasks

---

## Overview

This directory contains detailed implementation plans for all sidebars, tabs, and global UI components in the AnaFis application. Each component follows the established architecture pattern where **Univer.js** is the single source of truth and sidebars extract, process, and write back data.

**Key Update**: Automatic uncertainty propagation will be implemented as a **Univer plugin** rather than cell modifications, enabling correlated uncertainty support while maintaining architectural integrity.

### 🎯 Key Decision: ECharts Migration

**Why**: Plotly export failures, WebKit errors, and 6x larger bundle size led to migration to Apache ECharts.

**Benefits**:
- ✅ Reliable PNG/SVG export via `getDataURL()` and `renderToSVGString()`
- ✅ Native timeline animation component for time-variable graphs
- ✅ 500KB vs Plotly's 3MB (6x smaller, no WebKit issues)
- ✅ Future 3D support via echarts-gl plugin

See **[ECHARTS_MIGRATION.md](./ECHARTS_MIGRATION.md)** for detailed rationale and implementation guide.

---

## Component Files

### ✅ Actually Implemented
- **Uncertainty Propagation Sidebar** - ✅ COMPLETE - Propagate uncertainties through formulas with Rust backend
- **Unit Conversion Sidebar** - ✅ COMPLETE - Convert between different units with comprehensive unit support
- **Data Library Window** - ✅ COMPLETE - SQLite-based persistent storage with FTS5 search, statistics, preview, CSV/JSON export
- **Quick Plot Sidebar** - ✅ COMPLETE - Simple 2D scatter/line plots with ECharts, error bars, PNG/SVG export, save to Data Library
- **Import Sidebar** - ✅ COMPLETE - Import from files (CSV, TSV, TXT, Parquet, AnaFisSpread) or Data Library with search/filter/tag UI
- **Export Sidebar** - ✅ COMPLETE - Export data in 10 formats (CSV, TSV, TXT, JSON, XLSX, Parquet, HTML, Markdown, LaTeX, AnaFisSpread)

### 📋 Not Yet Implemented

#### Sidebars

#### Sidebars

2. **[Statistical Analysis Sidebar](./02_statistical_analysis_sidebar.md)** 📈
   - **Priority**: High
   - **Complexity**: Medium
   - **Features**: Descriptive statistics, distribution analysis, correlation, confidence intervals
   - **Dependencies**: statrs (Rust), nalgebra (Rust)
   - **File**: `02_statistical_analysis_sidebar.md`
   - **Status**: 📋 **NOT YET IMPLEMENTED** - Rust backend exists, UI component missing
   - **Note**: All calculations in Rust, TypeScript UI only


3. **[Data Smoothing Sidebar](./03_data_smoothing_sidebar.md)** �
   - **Priority**: Medium
   - **Complexity**: Medium
   - **Features**: Noise reduction with moving average, Savitzky-Golay, Gaussian, low-pass filters
   - **Dependencies**: DSP libraries (Rust) - scipy algorithms
   - **File**: `03_data_smoothing_sidebar.md`
   - **Note**: All smoothing algorithms (moving average, Savitzky-Golay, Gaussian, FFT) implemented in Rust, TypeScript UI only

4. **[Outlier Detection Sidebar](./04_outlier_detection_sidebar.md)** 🔍
   - **Priority**: Medium
   - **Complexity**: Medium
   - **Features**: Z-score, modified Z-score, IQR methods; outlier actions (remove, interpolate, mark)
   - **Dependencies**: statrs (Rust) - statistical libraries
   - **File**: `04_outlier_detection_sidebar.md`
   - **Note**: All outlier detection algorithms (Z-Score, Modified Z-Score, IQR) and statistics (mean, median, MAD, IQR) in Rust, TypeScript UI only


7. **[Import System](./../../IMPORT_SYSTEM.md)** 📥
   - **Priority**: ✅ COMPLETE
   - **Complexity**: Medium
   - **Features**: Multiple formats (CSV, TSV, TXT, Parquet, AnaFisSpread), encoding detection, Data Library import with search/filter
   - **Dependencies**: arrow 57.0.0, parquet 57.0.0, encoding_rs 0.8, flate2, Data Library
   - **File**: `../../IMPORT_SYSTEM.md`
   - **Note**: Custom CSV parser with encoding detection, direct Arrow/Parquet usage (no Polars), file association system

8. **[Export System](./07_export_system.md)** 💾
   - **Priority**: ✅ COMPLETE
   - **Complexity**: Medium
   - **Features**: Multiple formats (Excel, ODS, CSV, JSON, Parquet, HDF5, MATLAB, LaTeX, HTML, Markdown), .anafispread format, export to Data Library (from where data can be used in other tabs)
   - **Dependencies**: rust_xlsxwriter, csv, parquet, hdf5, Data Library
   - **File**: `07_export_system.md`

8. **[Data Library Sidebar](./08_data_library_sidebar.md)** 📚
   - **Priority**: ✅ INTEGRATED (Import Sidebar)
   - **Complexity**: Low (shares backend with Window)
   - **Features**: Import FROM Data Library TO spreadsheet with metadata, tags, uncertainties, search/filter
   - **Dependencies**: ✅ SQLite backend already exists (shares with Data Library Window)
   - **File**: `08_data_library_sidebar.md`
   - **Note**: Integrated into Import Sidebar with dual mode (file/library), Window for MANAGING stored data

#### Monte Carlo Enhancements

10. **[Monte Carlo Uncertainty Function](./../../monte_carlo_uncertainty_function.md)** 🎲
   - **Priority**: High
   - **Complexity**: Medium-High
   - **Features**: Spreadsheet function for Monte Carlo uncertainty propagation, async result injection, confidence interval normalization
   - **Dependencies**: Enhanced Formula Engine, Async Result Injection
   - **File**: `../../monte_carlo_uncertainty_function.md`
   - **Note**: MONTECARLO() function for formulas without analytical derivatives

#### Plugin Architecture

13. **[Uncertainty Propagation Plugin](./../../uncertanty_cell_plan.md)** 🔬
   - **Priority**: High
   - **Complexity**: High
   - **Features**: Automatic uncertainty propagation via Univer plugin, correlated uncertainties, covariance matrix support
   - **Dependencies**: Univer.js plugin API, enhanced Rust uncertainty backend
   - **File**: `../../uncertanty_cell_plan.md`
   - **Status**: 🔄 DESIGN COMPLETE - Plugin architecture designed to replace deprecated cell-based approach

#### Tabs

11. **[Graphs & Fitting Tab](./09_graphs_and_fitting_tab.md)** 📊📉
   - **Priority**: High
   - **Complexity**: High
   - **Features**: Advanced 2D/3D plotting from Data Library, n-dimensional curve fitting, fit comparison, residuals
   - **Dependencies**: echarts, ✅ Data Library (DONE!), nalgebra, levenberg-marquardt
   - **File**: `09_graphs_and_fitting_tab.md`
   - **Status**: � **NOT YET IMPLEMENTED** - Placeholder component exists, full implementation pending

#### Windows

12. **Data Library Window** - ✅ FULLY IMPLEMENTED
   - **Status**: ✅ COMPLETE
   - **Features**: SQLite storage, FTS5 search, statistics, preview, multi-select, CSV/JSON export with metadata
   - **Documentation**: Removed (implementation complete)

---

## Implementation Order

### Phase 0: Core Infrastructure ✅ COMPLETE
**IMPLEMENTED**
1. ✅ **Data Library Window** - Persistent SQLite storage, FTS5 search, CSV/JSON export, multi-select
   - All Rust backend logic complete (database, search, statistics, export)
   - Full TypeScript UI with Material-UI
   - Global toolbar integration
   - See implementation: `src-tauri/src/data_library/`, `src/DataLibraryWindow.tsx`

### Phase 1: Basic Spreadsheet Sidebars ✅ MOSTLY COMPLETE
**IMPLEMENTED**
2. ✅ **Uncertainty Propagation Sidebar** - Error propagation through formulas
   - Rust backend with uncertainty calculation algorithms
   - TypeScript UI for formula input and result display
   - See implementation: `src/tabs/spreadsheet/components/sidebar/UncertaintySidebar.tsx`, `src-tauri/src/uncertainty_calculator/`

3. ✅ **Unit Conversion Sidebar** - Physical unit conversions
   - Comprehensive unit database with dimensional analysis
   - Real-time conversion preview and validation
   - See implementation: `src/tabs/spreadsheet/components/sidebar/UnitConversionSidebar.tsx`, `src-tauri/src/unit_conversion/`

4. ✅ **Quick Plot Sidebar** - Simple 2D previews for rapid feedback
   - Apache ECharts integration (500KB, reliable PNG/SVG export)
   - Scatter, line, and error bar plots
   - Dark/light theme support with auto-scaling axes
   - Save to Data Library integration
   - See implementation: `src/tabs/spreadsheet/components/sidebar/QuickPlotSidebar.tsx`

### Phase 1.5: Import/Export System ✅ COMPLETE
5. ✅ **Import Sidebar** - File and Data Library import
   - CSV, TSV, TXT, Parquet, AnaFisSpread format support
   - Custom CSV parser with encoding detection (UTF-8, Windows-1252)
   - Direct Arrow/Parquet usage (v57.0.0) without Polars
   - Data Library integration with search/filter/tag UI
   - File association system (.anafispread files open in AnaFis)
   - See implementation: `src/tabs/spreadsheet/components/sidebar/ImportSidebar.tsx`, `src-tauri/src/import/`
   
6. ✅ **Export Sidebar** - Multi-format export
   - 10 formats: CSV, TSV, TXT, JSON, XLSX, Parquet, HTML, Markdown, LaTeX, AnaFisSpread
   - Configurable options per format
   - Lossless exports (formulas, formatting, metadata)
   - See implementation: `src/tabs/spreadsheet/components/sidebar/ExportSidebar.tsx`, `src-tauri/src/export/`

### Phase 2: Code Quality & Linting ✅ COMPLETE
- ✅ **All ESLint errors fixed** (0 errors)
- ✅ **All runtime-affecting warnings fixed**
- ✅ **All TypeScript compilation errors resolved**
- ✅ **Rust Clippy warnings fixed** (3 issues resolved)
- ✅ **Type safety improved** (removed all `any` types)
- ✅ **React hooks dependencies corrected** (useCallback, useEffect)
- ✅ **Proper ECharts types** (CustomSeriesRenderItemParams, SeriesOption[])

### Phase 3: Advanced Analysis (Next)
7. **Statistical Analysis Sidebar** - Complements plotting, essential for data analysis
8. **Graphs & Fitting Tab** - Advanced plotting and curve fitting (depends on Data Library)

### Phase 3.5: Plugin Architecture (High Priority)
6. **Uncertainty Propagation Plugin** - Automatic uncertainty with correlations
   - ✅ **Design Complete**: Plugin architecture to work around Univer constraints
   - 🔄 **Basic Plugin Framework**: Data types, renderers, registration
   - 🔄 **Formula Integration**: Intercept calculations, Rust backend calls
   - 🔄 **Correlation Support**: Covariance matrices, enhanced propagation
   - 🔄 **Advanced Features**: Complex formulas, unit propagation

### Phase 4: Statistical Analysis (Next)
7. **Statistical Analysis Sidebar** - Complements plotting, essential for data analysis

### Phase 5: Data Quality (Future)
8. **Data Smoothing Sidebar** - Prepare data for analysis
9. **Outlier Detection Sidebar** - Data quality control

### Phase 6: Data Management (Weeks 7-8)
9. **Data Validation Sidebar** - Prevent bad data entry
10. **Metadata Manager Sidebar** - Track experimental context

---

## Common Architecture Patterns

All sidebars follow these patterns:

### Pattern A: Read → Process → Write
Used for: Unit conversion, data smoothing, formula application
```
Univer → Read Data → Process → Write Back → Univer
```

### Pattern B: Read → Analyze → Display
Used for: Statistics, outlier detection, data validation
```
Univer → Read Data → Analyze → Display in Sidebar (optional write-back)
```

### Pattern C: Monitor → Validate → Highlight
Used for: Data validation, quality control
```
Univer → Monitor Changes → Validate → Highlight Invalid Cells
```

### Pattern D: Extract → Visualize → Annotate
Used for: Quick Plot preview (2D only)
```
Univer → Extract Data → Generate Visualization → Display (no write-back)
```

### Pattern E: Persistent Storage → Visualization → Fitting
Used for: Data Library + Graphs & Fitting Tab
```
Univer → Save to SQLite (Rust) → Load from Library → Plot → Fit → Store Results
```

---

## Shared Components

### Selection Handler
Each sidebar registers a global selection handler:
```typescript
useEffect(() => {
  if (open) {
    window.__mySidebarSelectionHandler = (cellRef: string) => {
      // Handle selection
    };
  }
  return () => {
    delete window.__mySidebarSelectionHandler;
  };
}, [open]);
```

### Univer API Access
All sidebars receive `univerRef` for data access:
```typescript
const readData = async (range: string) => {
  if (!univerRef.current) return null;
  return await univerRef.current.getRange(range);
};

const writeData = async (range: string, values: any[][]) => {
  if (!univerRef.current) return;
  await univerRef.current.setRange(range, values);
};
```

---

## Dependencies Summary

### NPM Packages
```bash
# Visualization
npm install echarts
npm install @types/echarts -D

# Utilities
npm install file-saver
npm install @types/file-saver -D
```

### Rust Crates
```toml
[dependencies]
statrs = "0.18.0"            # Statistics
rust_xlsxwriter = "0.91.0"   # Excel export
csv = "1.3"                  # CSV handling
serde_json = "1.0"           # JSON for .anafis format
nalgebra = "0.34.1"          # Linear algebra for fitting
levenberg-marquardt = "0.15.0" # Curve fitting algorithm
rusqlite = "0.37.0"          # SQLite for Data Library
uuid = "1.6"                 # UUID generation
chrono = "0.4.42"            # Date/time handling
arrow = "57.0.0"             # Columnar data import
parquet = "57.0.0"           # Parquet file support
encoding_rs = "0.8"          # Character encoding detection
pyo3 = "0.27.1"              # Python integration
```

---

## File Structure

```
Plans/sidebars/
├── README.md (this file)
├── 01_quick_plot_sidebar.md               # Simple 2D preview sidebar
├── 02_statistical_analysis_sidebar.md     # Statistical calculations
├── 03_data_smoothing_sidebar.md           # Data filtering
├── 04_outlier_detection_sidebar.md        # Outlier detection
├── 05_data_validation_dialog.md           # Validation rules
├── 06_metadata_manager_sidebar.md         # Experimental metadata
├── 07_export_system.md                    # Export functionality
├── 08_data_library_sidebar.md             # Persistent data storage (Core Infrastructure)
└── 09_graphs_and_fitting_tab.md           # Advanced plotting + curve fitting
```

---

## Success Criteria

Each sidebar must:
- ✓ Follow the established architecture pattern
- ✓ Not duplicate data from Univer
- ✓ Handle errors gracefully
- ✓ Provide clear user feedback
- ✓ Have consistent UI/UX with existing sidebars
- ✓ Be documented with inline comments
- ✓ Include basic error handling and validation
- ✓ Have unit and integration tests

---

## Next Steps

1. ✅ Complete all sidebar and tab specification files
2. ✅ **Data Library Window** - FULLY IMPLEMENTED
   - ✅ Rust + SQLite backend with FTS5
   - ✅ TypeScript UI with Material-UI
   - ✅ Multi-select and CSV/JSON export with metadata
3. ✅ **Basic Sidebars** - MOSTLY IMPLEMENTED
   - ✅ Uncertainty Propagation Sidebar
   - ✅ Unit Conversion Sidebar  
   - ✅ Quick Plot Sidebar (ECharts)
   - ✅ Import Sidebar
   - ✅ Export Sidebar
4. ✅ **Code Quality** - COMPLETE
   - ✅ All ESLint errors fixed (0 errors, 0 warnings)
   - ✅ All TypeScript compilation errors resolved
   - ✅ Rust Clippy warnings fixed
   - ✅ Removed all `any` types, added proper ECharts types
   - ✅ Fixed React hooks dependencies
5. **NEXT**: Implement Statistical Analysis Sidebar (Rust backend exists, needs UI component)
6. **FUTURE**: Implement Graphs & Fitting Tab (requires Data Library integration)
7. ✅ Update SpreadsheetTab UI - already has toolbar buttons for all implemented sidebars
8. Write tests for each component

---

## Architecture Notes

- **Data Library (10/08)** is the core infrastructure using SQLite (Rust) for persistent storage
- **All business logic in Rust**: statistics, search, filtering, sorting, data validation
- **TypeScript for UI only**: rendering, user input, calling Tauri commands
- **Quick Plot (01)** is simplified to 2D scatter/line only for quick previews
- **Graphs & Fitting (09)** handles all advanced plotting (2D/3D) and curve fitting
- Named plots enable clear workflow: Create plot → Fit plot → Compare fits
- Each tab has one workbook (only sheet selection needed, not workbook selection)
- Data validation prevents length mismatches between sequences
- Non-intrusive warnings when fit data changes (preserves user control)

---

For detailed information about each sidebar, see the individual markdown files in this directory.
