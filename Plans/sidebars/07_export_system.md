# Export System �

**Status**: Planned  
**Priority**: High  
**Complexity**: Medium  
**Dependencies**: rust_xlsxwriter, csv, parquet, hdf5, Data Library

---

## Purpose

Export spreadsheet data to two destinations:
- **External Files**: Multiple formats for use outside AnaFis (Excel, CSV, JSON, Parquet, HDF5, MATLAB, LaTeX, HTML, Markdown, ODS)
- **Data Library**: Save selected ranges as named sequences with metadata for reuse within AnaFis (from where they can be used in Graphs & Fitting, Monte Carlo, Solver tabs)
- **Spreadsheet Data Format**: .anafispread (preserves formulas, formatting, metadata, uncertainties) - spreadsheet data backup

**Note**: 
- Full application state (all tabs, settings, window state) is saved as .anafis format - handled separately by the main application.
- To use data in other tabs (Fitting, Monte Carlo, Solver), export to Data Library first, then select from library in those tabs.

---

## Features

### Export Destinations

#### 1. External File Formats

##### Excel (.xlsx, .xls)
- Full formatting preservation
- Multiple sheets
- Formulas preserved
- Charts (if implemented)
- Uses `rust_xlsxwriter` for .xlsx generation
- Legacy .xls support

##### OpenDocument Spreadsheet (.ods)
- Open source format
- Compatible with LibreOffice, OpenOffice
- Formatting and formulas preserved

##### Delimited Text (CSV, TSV, TXT)
- Values only (formulas evaluated)
- Custom delimiter support
- Optional header row
- Encoding selection (UTF-8, Latin-1, UTF-16)
- Quote character selection

##### JSON (.json)
- Array of objects format
- Each row as JSON object
- Column headers as keys
- Hierarchical data support
- Pretty print option

##### Parquet (.parquet)
- Columnar storage format
- Efficient compression
- Fast read/write
- Popular in data science workflows
- Supports large datasets

##### HDF5 (.h5, .hdf5)
- Hierarchical Data Format
- Efficient for large numerical datasets
- Metadata support
- Common in scientific computing
- Multiple datasets per file

##### MATLAB (.mat)
- MATLAB data format
- Array/matrix export
- Variable names from headers
- Compatible with Octave

##### LaTeX (.tex)
- Table format for LaTeX documents
- Customizable table style (tabular, booktabs)
- Column alignment options
- Caption and label support

##### HTML (.html)
- HTML table format
- Styling with CSS
- Responsive table option
- Good for reports and web display

##### Markdown (.md)
- Markdown table format
- GitHub-flavored markdown
- Plain text readable
- Good for documentation

##### Custom .anafispread Format
- JSON-based format **for spreadsheet data only**
- Preserves everything:
  - Cell values + formulas
  - Formatting (colors, fonts, borders)
  - Metadata (from Metadata Manager)
  - Uncertainties (from Uncertainty Sidebar)
  - Charts and plots
  - Validation rules
  - Sheet structure
- Allows full restore of spreadsheet work
- **Note**: Different from .anafis (full app state)

#### 2. Export to Data Library
- Save selected range as named sequence
- Specify X and Y columns (or single column)
- Include uncertainties column (optional)
- Add tags for organization
- Specify units
- Add description/metadata
- Statistics calculated automatically
- **Available for use in other tabs**: Once in Data Library, sequences can be selected in:
  - Graphs & Fitting tab (for plotting and curve fitting)
  - Monte Carlo tab (for simulations)
  - Solver tab (for optimization)
  - Any future analysis tabs
- **Workflow**: Spreadsheet → Data Library → Other Tabs

### Export Options
- Select range or entire sheet
- Include/exclude headers
- Include/exclude formulas
- Include/exclude formatting
- Compression (for .anafispread, .parquet, .h5)
- For Data Library: Add tags, units, uncertainties, description
- Format-specific options (delimiter, encoding, precision, etc.)

---

## UI Layout

```
┌─────────────────────────────────────┐
│ Export Data                     [X] │
├─────────────────────────────────────┤
│ Export Range:                       │
│  ⦿ Current Selection [A1:D100]     │
│  ○ Entire Sheet                    │
│  ○ All Sheets                      │
│  ○ Custom Range: [_____________]   │
│                                     │
│ Export To: [Excel File (.xlsx) ▼]  │
│  External Files:                    │
│  • Excel (.xlsx, .xls)             │
│  • OpenDocument (.ods)              │
│  • CSV (Comma-separated)           │
│  • TSV (Tab-separated)             │
│  • TXT (Custom delimiter)          │
│  • JSON (.json)                    │
│  • Parquet (.parquet)              │
│  • HDF5 (.h5, .hdf5)               │
│  • MATLAB (.mat)                   │
│  • LaTeX (.tex)                    │
│  • HTML (.html)                    │
│  • Markdown (.md)                  │
│  • AnaFis Spreadsheet (.anafispread)│
│  ─────────────────                 │
│  Within AnaFis:                     │
│  • Data Library                    │
│                                     │
│ ┌───────────────────────────────┐ │
│ │ Export Options                │ │
│ │                               │ │
│ │ [✓] Include header row        │ │
│ │ [✓] Include formulas          │ │
│ │ [✓] Include formatting        │ │
│ │ [✓] Include metadata          │ │
│ │ [✓] Include uncertainties     │ │
│ │ [ ] Evaluate formulas         │ │
│ │                               │ │
│ │ Encoding: [UTF-8 ▼]           │ │
│ │ Line ending: [CRLF ▼]         │ │
│ │                               │ │
│ │ (For .anafis only)            │ │
│ │ [✓] Compress file (gzip)      │ │
│ └───────────────────────────────┘ │
│                                     │
│ Preview (first 5 rows):             │
│ ┌─────────────────────────────┐   │
│ │ Time,Temperature,Pressure   │   │
│ │ 0.0,23.5,101.3              │   │
│ │ 1.0,24.1,101.2              │   │
│ │ 2.0,24.8,101.1              │   │
│ │ 3.0,25.2,101.0              │   │
│ └─────────────────────────────┘   │
│                                     │
│ Estimated file size: ~15 KB         │
│                                     │
│ [Preview Full Export]               │
│ [Export to File...] [Cancel]       │
│                                     │
│ ─── Or Export to Data Library ───  │
│                                     │
│ (Shows when "Data Library" selected)│
│ Name: [My Measurement__________]    │
│ Description: [Temperature series]   │
│ Tags: [temperature, exp1_______]    │
│ Unit: [°C______________________]    │
│ X column: [A ▼]                     │
│ Y column: [B ▼]                     │
│ Uncertainty column: [C ▼] (opt)    │
│                                     │
│ [Save to Data Library]              │
│                                     │
│ ───────────────────────────────────│
│ Note: To use data in other tabs    │
│ (Fitting, Monte Carlo, Solver):    │
│ 1. Export to Data Library here     │
│ 2. Open target tab                 │
│ 3. Select sequences from library   │
└─────────────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: Extract → Format/Store → Write/Save

### For External Files:
1. **Extract**: Read data from Univer spreadsheet
   - Get values, formulas, formatting
   - Retrieve metadata and uncertainties
2. **Format**: Convert to target format
   - Excel: Use rust_xlsxwriter
   - CSV/TSV: Format as delimited text
   - JSON/Parquet/HDF5/etc: Format to specific structure
   - .anafispread: Serialize to JSON
3. **Write**: Save to file
   - Use Tauri file dialog for save location
   - Write formatted data to disk

### For Data Library:
1. **Extract**: Read data from Univer spreadsheet
   - Get column values
   - Parse headers (if present)
2. **Validate**: Check data quality
   - Ensure X and Y columns have same length
   - Check for NaN/Infinity values
3. **Store**: Save to Data Library (SQLite via Rust)
   - Create DataSequence object
   - Call `save_sequence` Tauri command (Rust handles statistics calculation and SQLite storage)
   - Data persists in database and becomes available for all other tabs (Graphs & Fitting, Monte Carlo, Solver)

**Note**: Uses same Rust/SQLite backend as Data Library Sidebar and Window

### Workflow to Other Tabs:
```
Spreadsheet → Export to Data Library → [Data Library] → Select in Target Tab
                                                ↓
                                        Graphs & Fitting
                                        Monte Carlo
                                        Solver
```

### For Other Tabs:
1. **Extract**: Read data from specified ranges
2. **Pass**: Send data to target tab component
   - Direct in-memory transfer
   - Link back to source spreadsheet

---

## Technical Implementation

### TypeScript Interfaces

```typescript
interface ExportSystemProps {
  open: boolean;
  onClose: () => void;
  univerRef: UniverSpreadsheetRef;
  // Note: No Data Library store needed - uses Tauri invoke() directly
}

type ExportFormat = 
  | 'xlsx'              // Excel 2007+
  | 'xls'               // Excel 97-2003
  | 'ods'               // OpenDocument Spreadsheet
  | 'csv'               // Comma-separated values
  | 'tsv'               // Tab-separated values
  | 'txt'               // Custom delimiter text
  | 'json'              // JSON format
  | 'parquet'           // Apache Parquet
  | 'hdf5'              // HDF5 format
  | 'mat'               // MATLAB format
  | 'tex'               // LaTeX table
  | 'html'              // HTML table
  | 'markdown'          // Markdown table
  | 'anafispread'       // AnaFis spreadsheet format (data only)
  | 'data_library'      // Export to Data Library

interface ExportConfig {
  range: string; // 'selection' | 'sheet' | 'all' | specific range
  format: ExportFormat;
  
  options: {
    includeHeaders: boolean;
    includeFormulas: boolean;
    includeFormatting: boolean;
    includeMetadata: boolean;
    includeUncertainties: boolean;
    evaluateFormulas: boolean;
    
    // For text formats (CSV, TSV, TXT)
    delimiter?: string; // ',', '\t', '|', ';', etc.
    encoding?: 'utf8' | 'latin1' | 'utf16';
    lineEnding?: 'lf' | 'crlf';
    quoteChar?: '"' | "'" | '';
    
    // For JSON
    jsonFormat?: 'array' | 'object' | 'records';
    prettyPrint?: boolean;
    
    // For Parquet
    compression?: 'snappy' | 'gzip' | 'brotli' | 'lz4' | 'zstd' | 'none';
    
    // For HDF5
    datasetName?: string;
    
    // For MATLAB
    variableName?: string;
    
    // For LaTeX
    tableStyle?: 'tabular' | 'booktabs' | 'longtable';
    columnAlignment?: string; // e.g., 'lrc'
    caption?: string;
    label?: string;
    
    // For HTML
    cssClass?: string;
    responsive?: boolean;
    
    // For .anafispread
    compress?: boolean;
    lineEnding?: 'lf' | 'crlf';
    
    // For .anafis
    compress?: boolean;
  };
  
  // For Data Library export
  dataLibraryConfig?: {
    name: string;
    description?: string;
    tags: string[];
    unit?: string;
    xColumn: string; // Column letter (e.g., 'A')
    yColumn: string; // Column letter (e.g., 'B')
    uncertaintyColumn?: string; // Optional (e.g., 'C')
  };
}

// .anafispread format structure
interface AnaFisSpreadFormat {
  version: string; // e.g., "1.0.0"
  created: Date;
  application: string; // "AnaFis"
  
  sheets: Array<{
    name: string;
    rows: number;
    cols: number;
    
    // Cell data
    cells: Array<{
      row: number;
      col: number;
      value: any;
      formula?: string;
      
      // Formatting
      style?: {
        backgroundColor?: string;
        textColor?: string;
        fontFamily?: string;
        fontSize?: number;
        bold?: boolean;
        italic?: boolean;
        underline?: boolean;
        alignment?: 'left' | 'center' | 'right';
        numberFormat?: string;
      };
      
      // Uncertainty
      uncertainty?: {
        value: number;
        type: 'absolute' | 'relative' | 'percentage';
      };
      
      // Metadata
      metadata?: any; // From Metadata Manager
    }>;
    
    // Merged cells
    merges?: Array<{
      startRow: number;
      endRow: number;
      startCol: number;
      endCol: number;
    }>;
    
    // Column widths
    columnWidths?: Record<number, number>;
    
    // Row heights
    rowHeights?: Record<number, number>;
  }>;
  
  // Global metadata
  metadata?: any;
  
  // Validation rules
  validationRules?: any[];
}
```

### Rust Backend Commands

```rust
// src-tauri/src/export/mod.rs

use serde::{Deserialize, Serialize};
use rust_xlsxwriter::*;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    pub range: String,
    pub format: String,
    pub options: ExportOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_headers: bool,
    pub include_formulas: bool,
    pub include_formatting: bool,
    pub delimiter: Option<String>,
    pub encoding: Option<String>,
}

// Export to Excel
#[tauri::command]
pub async fn export_to_excel(
    data: Vec<Vec<serde_json::Value>>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    for (row_idx, row) in data.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let row = row_idx as u32;
            let col = col_idx as u16;
            
            match cell {
                serde_json::Value::Number(n) => {
                    if let Some(num) = n.as_f64() {
                        worksheet.write_number(row, col, num)?;
                    }
                }
                serde_json::Value::String(s) => {
                    worksheet.write_string(row, col, s)?;
                }
                serde_json::Value::Bool(b) => {
                    worksheet.write_boolean(row, col, *b)?;
                }
                _ => {}
            }
        }
    }
    
    workbook.save(&file_path)
        .map_err(|e| format!("Failed to save Excel file: {}", e))?;
    
    Ok(())
}

// Export to CSV/TSV
#[tauri::command]
pub async fn export_to_csv(
    data: Vec<Vec<serde_json::Value>>,
    file_path: String,
    delimiter: String,
) -> Result<(), String> {
    let mut file = File::create(&file_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    for row in data {
        let row_str: Vec<String> = row.iter().map(|cell| {
            match cell {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => {
                    // Escape quotes and wrap in quotes if contains delimiter
                    if s.contains(&delimiter) || s.contains('"') {
                        format!("\"{}\"", s.replace("\"", "\"\""))
                    } else {
                        s.clone()
                    }
                }
                serde_json::Value::Bool(b) => b.to_string(),
                _ => String::new(),
            }
        }).collect();
        
        writeln!(file, "{}", row_str.join(&delimiter))
            .map_err(|e| format!("Failed to write row: {}", e))?;
    }
    
    Ok(())
}

// Export to .anafis format
#[tauri::command]
pub async fn export_to_anafis(
    data: serde_json::Value, // Full AnaFisFormat structure
    file_path: String,
    compress: bool,
) -> Result<(), String> {
    let json_str = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    
    if compress {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        
        let file = File::create(&file_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(json_str.as_bytes())
            .map_err(|e| format!("Failed to write compressed data: {}", e))?;
    } else {
        let mut file = File::create(&file_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        file.write_all(json_str.as_bytes())
            .map_err(|e| format!("Failed to write data: {}", e))?;
    }
    
    Ok(())
}

// Import from .anafis format
#[tauri::command]
pub async fn import_from_anafis(
    file_path: String,
) -> Result<serde_json::Value, String> {
    use std::io::Read;
    
    let mut file = File::open(&file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    // Try to decompress first
    let json_str = if file_path.ends_with(".gz") {
        use flate2::read::GzDecoder;
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed)
            .map_err(|e| format!("Failed to decompress: {}", e))?;
        decompressed
    } else {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        contents
    };
    
    let data: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    Ok(data)
}
```

### Frontend Export Functions

```typescript
// Extract data from Univer
async function extractSpreadsheetData(
  univerRef: UniverSpreadsheetRef,
  range: string,
  config: ExportConfig
): Promise<any[][]> {
  const data = await univerRef.current?.getRange(range);
  
  if (config.options.evaluateFormulas) {
    // Return evaluated values
    return data;
  } else {
    // Return formulas where they exist
    const formulas = await univerRef.current?.getFormulas(range);
    return data.map((row, i) => 
      row.map((cell, j) => formulas?.[i]?.[j] || cell)
    );
  }
}

// Build .anafis format
async function buildAnaFisFormat(
  univerRef: UniverSpreadsheetRef,
  range: string,
  config: ExportConfig
): Promise<AnaFisFormat> {
  const data = await extractSpreadsheetData(univerRef, range, config);
  const formulas = await univerRef.current?.getFormulas(range);
  const styles = await univerRef.current?.getStyles(range);
  
  // Get metadata if included
  let metadata = {};
  if (config.options.includeMetadata) {
    // Retrieve from MetadataStore
  }
  
  // Get uncertainties if included
  let uncertainties = {};
  if (config.options.includeUncertainties) {
    // Retrieve from uncertainty data
  }
  
  return {
    version: '1.0.0',
    created: new Date(),
    application: 'AnaFis',
    sheets: [{
      name: 'Sheet1',
      rows: data.length,
      cols: data[0]?.length || 0,
      cells: data.flatMap((row, i) =>
        row.map((value, j) => ({
          row: i,
          col: j,
          value,
          formula: formulas?.[i]?.[j],
          style: styles?.[i]?.[j],
          uncertainty: uncertainties?.[`${i},${j}`],
          metadata: metadata?.[`${i},${j}`]
        }))
      )
    }]
  };
}

// Export to Data Library (simplified - calls Rust)
async function exportToDataLibrary(
  univerRef: UniverSpreadsheetRef,
  config: ExportConfig
): Promise<void> {
  const { dataLibraryConfig, range } = config;
  if (!dataLibraryConfig) throw new Error('Data Library config required');
  
  // Extract column data
  const xData = await univerRef.current?.getColumnValues(
    dataLibraryConfig.xColumn, 
    range
  );
  const yData = await univerRef.current?.getColumnValues(
    dataLibraryConfig.yColumn, 
    range
  );
  const uncertainties = dataLibraryConfig.uncertaintyColumn
    ? await univerRef.current?.getColumnValues(
        dataLibraryConfig.uncertaintyColumn, 
        range
      )
    : undefined;
  
  // Validate data
  if (xData.length !== yData.length) {
    throw new Error('X and Y columns must have the same length');
  }
  
  // Create DataSequence
  const sequence: DataSequence = {
    id: generateId(),
    name: dataLibraryConfig.name,
    description: dataLibraryConfig.description,
    values: yData,
    uncertainties,
    tags: dataLibraryConfig.tags,
    unit: dataLibraryConfig.unit,
    source: {
      type: 'spreadsheet',
      sheetName: await univerRef.current?.getActiveSheetName(),
      range,
      timestamp: new Date().toISOString()
    },
    isPinned: false
  };
  
  // Save to Data Library via Rust (statistics calculated there)
  await invoke('save_sequence', { sequence });
  
  // Show success notification
  console.log(`Saved "${sequence.name}" to Data Library`);
}

// Main export function
async function exportData(
  univerRef: UniverSpreadsheetRef,
  config: ExportConfig
): Promise<void> {
  // Handle Data Library export separately
  if (config.format === 'data_library') {
    await exportToDataLibrary(univerRef, config);
    return;
  }
  
  // Show file save dialog for file exports
  const filePath = await save({
    defaultPath: `export.${config.format}`,
    filters: [{
      name: 'Export file',
      extensions: [config.format]
    }]
  });
  
  if (!filePath) return; // User cancelled
  
  const data = await extractSpreadsheetData(univerRef, config.range, config);
  
  switch (config.format) {
    case 'xlsx':
    case 'xls':
      await invoke('export_to_excel', { data, filePath, config });
      break;
    case 'ods':
      await invoke('export_to_ods', { data, filePath, config });
      break;
    case 'csv':
      await invoke('export_to_csv', { 
        data, 
        filePath, 
        delimiter: config.options.delimiter || ',' 
      });
      break;
    case 'tsv':
      await invoke('export_to_csv', { 
        data, 
        filePath, 
        delimiter: '\t' 
      });
      break;
    case 'json':
      await invoke('export_to_json', { data, filePath, config });
      break;
    case 'parquet':
      await invoke('export_to_parquet', { data, filePath, config });
      break;
    case 'hdf5':
      await invoke('export_to_hdf5', { data, filePath, config });
      break;
    case 'mat':
      await invoke('export_to_matlab', { data, filePath, config });
      break;
    case 'tex':
      await invoke('export_to_latex', { data, filePath, config });
      break;
    case 'html':
      await invoke('export_to_html', { data, filePath, config });
      break;
    case 'markdown':
      await invoke('export_to_markdown', { data, filePath, config });
      break;
    case 'anafispread':
      const anafispreadData = await buildAnaFisSpreadFormat(univerRef, config.range, config);
      await invoke('export_to_anafispread', { 
        data: anafispreadData, 
        filePath,
        compress: config.options.compress 
      });
      break;
  }
}
```

---

## Dependencies

### Rust
```toml
[dependencies]
# Excel formats
rust_xlsxwriter = "0.64"
calamine = "0.24"        # For .xls reading

# OpenDocument
odf = "0.6"

# Text formats
csv = "1.3"

# JSON
serde_json = "1.0"

# Parquet
parquet = "50.0"
arrow = "50.0"

# HDF5
hdf5 = "0.8"

# MATLAB
matfile = "0.3"

# Compression
flate2 = "1.0"
```

### TypeScript
```bash
npm install @tauri-apps/api
npm install idb  # For Data Library integration
```

---

## File Location

- **Component**: `AnaFis/src/dialogs/ExportDialog.tsx`
- **Rust Modules**: 
  - `AnaFis/src-tauri/src/export/mod.rs` (main)
  - `AnaFis/src-tauri/src/export/excel.rs`
  - `AnaFis/src-tauri/src/export/ods.rs`
  - `AnaFis/src-tauri/src/export/text.rs` (CSV, TSV, TXT)
  - `AnaFis/src-tauri/src/export/json.rs`
  - `AnaFis/src-tauri/src/export/parquet.rs`
  - `AnaFis/src-tauri/src/export/hdf5.rs`
  - `AnaFis/src-tauri/src/export/matlab.rs`
  - `AnaFis/src-tauri/src/export/latex.rs`
  - `AnaFis/src-tauri/src/export/html.rs`
  - `AnaFis/src-tauri/src/export/markdown.rs`
  - `AnaFis/src-tauri/src/export/anafispread.rs`
- **Types**: `AnaFis/src/types/export.ts`
- **Data Library Integration**: Uses Rust `save_sequence` command (same backend as Sidebar and Window - see `08_data_library_sidebar.md` and `10_data_library_window.md`)

---

## Success Criteria

### External File Export
- ✓ Excel (.xlsx, .xls) export preserves formatting and formulas
- ✓ OpenDocument (.ods) export works correctly
- ✓ CSV/TSV export handles special characters and custom delimiters
- ✓ JSON export creates valid JSON with proper structure
- ✓ Parquet export creates valid compressed columnar files
- ✓ HDF5 export works with hierarchical data
- ✓ MATLAB .mat files load correctly in MATLAB/Octave
- ✓ LaTeX tables compile correctly
- ✓ HTML tables display properly with styling
- ✓ Markdown tables are properly formatted
- ✓ .anafispread format can fully restore spreadsheet data
- ✓ File dialogs work on all platforms
- ✓ Large exports (>100MB) don't freeze UI
- ✓ Format-specific options work correctly (compression, encoding, etc.)

### Data Library Export
- ✓ Can save selected ranges to Data Library
- ✓ X and Y columns correctly mapped
- ✓ Uncertainties column optional and validated
- ✓ Tags and metadata properly stored
- ✓ Statistics calculated automatically
- ✓ Exported sequences immediately available in all tabs (Graphs & Fitting, Monte Carlo, Solver)
- ✓ Validation prevents length mismatches
- ✓ Clear workflow: Spreadsheet → Data Library → Select in Target Tab

---

**Next Steps**: Implement early alongside Data Library (high priority utility)
- Data Library export depends on Data Library being implemented first
- External file exports can be implemented independently
- Prioritize common formats first: Excel, CSV, JSON, .anafispread
- Add specialized formats (Parquet, HDF5, MATLAB) as needed
- Other tabs (Fitting, Monte Carlo, Solver) will access data through Data Library
