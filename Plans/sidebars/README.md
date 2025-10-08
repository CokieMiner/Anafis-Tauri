# Sidebar Implementation Plans - Index

**Date**: January 2025  
**Branch**: univer-spreadsheet  
**Status**: Planning Complete

---

## Overview

This directory contains detailed implementation plans for all sidebars, tabs, and global UI components in the AnaFis application. Each component follows the established architecture pattern where **Univer.js** is the single source of truth and sidebars extract, process, and write back data.

---

## Component Files

### ✅ Already Implemented
- **Uncertainty Propagation Sidebar** - Propagate uncertainties through formulas
- **Unit Conversion Sidebar** - Convert between different units

### 📋 Planned Implementations

#### Sidebars

1. **[Quick Plot Sidebar](./01_quick_plot_sidebar.md)** 📊
   - **Priority**: High
   - **Complexity**: Low (Simplified)
   - **Features**: Simple 2D scatter/line plots, error bars, quick preview
   - **Dependencies**: Plotly.js, Data Library
   - **File**: `01_quick_plot_sidebar.md`

2. **[Statistical Analysis Sidebar](./02_statistical_analysis_sidebar.md)** 📈
   - **Priority**: High
   - **Complexity**: Medium
   - **Features**: Descriptive statistics, distribution analysis, correlation, confidence intervals
   - **Dependencies**: statrs (Rust), nalgebra (Rust)
   - **File**: `02_statistical_analysis_sidebar.md`
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

5. **[Data Validation Sidebar](./05_data_validation_sidebar.md)** ✔️

   - **Priority**: Medium
   - **Complexity**: Medium
   - **Features**: Validation rules (numeric, pattern, list, type check), real-time checking via Rust
   - **Dependencies**: regex, chrono (Rust backend)
   - **File**: `05_data_validation_sidebar.md`
   - **Note**: All validation logic in Rust for performance, TypeScript UI only

6. **[Metadata Manager Sidebar](./06_metadata_manager_sidebar.md)** 📋
   - **Priority**: Low
   - **Complexity**: Medium
   - **Features**: Experimental context, instrument info, calibration data, full-text search
   - **Dependencies**: SQLite (rusqlite), chrono, uuid, serde_json (Rust backend)
   - **File**: `06_metadata_manager_sidebar.md`
   - **Note**: All storage and search in Rust/SQLite with FTS5, TypeScript UI only

7. **[Export System](./07_export_system.md)** 💾
   - **Priority**: High
   - **Complexity**: Medium
   - **Features**: Multiple formats (Excel, ODS, CSV, JSON, Parquet, HDF5, MATLAB, LaTeX, HTML, Markdown), .anafispread format, export to Data Library (from where data can be used in other tabs)
   - **Dependencies**: rust_xlsxwriter, csv, parquet, hdf5, Data Library
   - **File**: `07_export_system.md`

8. **[Data Library Sidebar](./08_data_library_sidebar.md)** 📚
   - **Priority**: CRITICAL (Core Infrastructure)
   - **Complexity**: Medium
   - **Features**: Quick export FROM spreadsheet TO library with metadata, tags, uncertainties
   - **Dependencies**: SQLite (rusqlite), chrono, uuid - shares backend with Data Library Window
   - **File**: `08_data_library_sidebar.md`
   - **Note**: Sidebar for SAVING data, Window for MANAGING stored data, tabs for IMPORTING data

#### Tabs

9. **[Graphs & Fitting Tab](./09_graphs_and_fitting_tab.md)** 📊📉
   - **Priority**: High
   - **Complexity**: High
   - **Features**: Advanced 2D/3D plotting from Data Library, n-dimensional curve fitting, fit comparison, residuals
   - **Dependencies**: Plotly.js, Data Library, nalgebra, levenberg-marquardt
   - **File**: `09_graphs_and_fitting_tab.md`

#### Windows

10. **[Data Library Window](./10_data_library_window.md)** 🗄️
   - **Priority**: CRITICAL (Core Infrastructure)
   - **Complexity**: High
   - **Features**: Standalone window for data management, search/filter, statistics, preview, export/import, batch operations
   - **Dependencies**: SQLite (rusqlite), chrono, uuid, Tauri State, Plotly.js (preview only)
   - **File**: `10_data_library_window.md`
   - **Note**: All logic in Rust (SQLite + FTS5), TypeScript only for UI rendering

---

## Implementation Order

### Phase 0: Core Infrastructure (Week 1) 🔧
**MUST BE IMPLEMENTED FIRST**
1. ✅ **Data Library Window** (or Sidebar) - Persistent data storage with SQLite, required by all other components
   - Window approach recommended for better UX and multitasking
   - All logic in Rust (database, search, statistics)
   - See files: `10_data_library_window.md` (window) or `08_data_library_sidebar.md` (sidebar)

### Phase 1: Quick Visualization & Advanced Analysis (Weeks 2-3)
2. **Quick Plot Sidebar** - Simple 2D previews for rapid feedback
3. **Graphs & Fitting Tab** - Advanced plotting and curve fitting (depends on Data Library)

### Phase 2: Statistical Analysis (Week 4)
4. **Statistical Analysis Sidebar** - Complements plotting, essential for data analysis

### Phase 3: Data Quality (Weeks 5-6)
5. **Data Smoothing Sidebar** - Prepare data for analysis
6. **Outlier Detection Sidebar** - Data quality control

### Phase 4: Data Management (Weeks 7-8)
7. **Data Validation Sidebar** - Prevent bad data entry
8. **Metadata Manager Sidebar** - Track experimental context

### Phase 5: Export & Integration (Week 9)
9. **Export System** - Complete the workflow

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
npm install plotly.js react-plotly.js
npm install @types/plotly.js -D

# Utilities
npm install file-saver
npm install @types/file-saver -D
```

### Rust Crates
```toml
[dependencies]
statrs = "0.16"              # Statistics
rust_xlsxwriter = "0.64"     # Excel export
csv = "1.3"                  # CSV handling
serde_json = "1.0"           # JSON for .anafis format
nalgebra = "0.32"            # Linear algebra for fitting
levenberg-marquardt = "0.12" # Curve fitting algorithm
rusqlite = "0.31"            # SQLite for Data Library
uuid = "1.6"                 # UUID generation
chrono = "0.4"               # Date/time handling
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
2. **CRITICAL**: Implement Data Library (10 - window or 08 - sidebar) first - all other components depend on it
   - Use Rust + SQLite for all data logic
   - TypeScript only for UI rendering
3. Install dependencies (plotly.js, nalgebra, levenberg-marquardt, rusqlite)
4. Implement Quick Plot Sidebar (01) - simple, independent
5. Implement Graphs & Fitting Tab (09) - requires Data Library
6. Implement remaining sidebars following the phase order (02-07)
7. Update SpreadsheetTab UI with toolbar buttons for all sidebars
8. Implement Rust backend commands for fitting and statistics
9. Write tests for each component

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
