# AnaFis Sidebar/Dialog Architecture
## Integration with Univer.js Spreadsheet

**Design Philosophy**: All advanced scientific features are implemented as **sidebars** or **dialogs** that extract data from Univer, process it (frontend or backend), and write results back to Univer cells.

---

## Core Architecture Principles

### 1. **Univer as Single Source of Truth**
- Univer.js handles ALL data storage, display, and basic spreadsheet operations
- All cell values, formulas, and formatting live in Univer
- No parallel data storage systems
- Univer's formula engine evaluates standard Excel-like formulas

### 2. **Sidebars/Dialogs as Processing Tools**
- Each scientific feature is a separate UI component (sidebar or dialog)
- Components extract data from Univer via API calls
- Processing happens in frontend (TypeScript) or backend (Rust/Tauri)
- Results are written back to Univer via API calls
- Components are stateless - they don't store spreadsheet data

### 3. **Standard Data Flow Pattern**

```
┌──────────────────────────────────────────────────────────┐
│                    Univer Spreadsheet                     │
│  (stores all cell values, formulas, formatting)          │
└──────────────────┬───────────────────────┬───────────────┘
                   │                       │
            READ   │                       │   WRITE
         (extract) │                       │  (insert)
                   ▼                       ▲
         ┌─────────────────────────────────────┐
         │     Sidebar/Dialog Component        │
         │  - User selects ranges              │
         │  - Configures parameters            │
         │  - Triggers processing              │
         └──────────┬──────────────────────────┘
                    │
         ┌──────────▼──────────┐
         │  Processing Layer   │
         │  (Frontend/Backend) │
         │  - Calculations     │
         │  - Validation       │
         │  - Transformations  │
         └─────────────────────┘
```

---

## Standard Interface Patterns

### Pattern A: **Read → Process → Write**
Used for: Unit conversion, data smoothing, formula application

**Steps:**
1. User selects range in Univer (e.g., A1:A10)
2. Sidebar reads current values from Univer
3. User configures operation parameters
4. Processing happens (frontend or backend)
5. Results written back to Univer (same range or new range)

**Example: Unit Converter**
```typescript
// 1. Read from Univer
const values = univerAPI.getRange('A1:A10'); // [5, 10, 15, ...]

// 2. Process (convert kg to lb)
const converted = values.map(v => v * 2.20462);

// 3. Write back to Univer
univerAPI.setRange('B1:B10', converted);
```

---

### Pattern B: **Read → Analyze → Display**
Used for: Statistics, outlier detection, data validation

**Steps:**
1. User selects range in Univer
2. Sidebar reads values
3. Analysis performed
4. Results displayed in sidebar (not written to spreadsheet)
5. User can optionally export summary to new range

**Example: Statistical Analysis**
```typescript
// 1. Read from Univer
const data = univerAPI.getRange('A1:A50');

// 2. Analyze
const stats = {
  mean: calculateMean(data),
  std: calculateStd(data),
  ci95: calculateConfidence(data, 0.95)
};

// 3. Display in sidebar UI
setSidebarResults(stats);

// 4. Optional: User clicks "Write Summary"
if (userWantsExport) {
  univerAPI.setRange('C1:C3', [stats.mean, stats.std, stats.ci95]);
}
```

---

### Pattern C: **Monitor → Validate → Highlight**
Used for: Data validation, quality control

**Steps:**
1. User defines validation rules in dialog
2. Dialog monitors specified Univer ranges
3. Validation runs on data changes
4. Invalid cells are highlighted using Univer's formatting API
5. No data modification - purely visual feedback

**Example: Data Validation**
```typescript
// 1. User sets rule: "A1:A10 must be 0-100"
const rule = { range: 'A1:A10', min: 0, max: 100 };

// 2. Monitor changes
univerAPI.onCellChange((range) => {
  if (isInRange(range, rule.range)) {
    // 3. Validate
    const value = univerAPI.getValue(range);
    const isValid = value >= rule.min && value <= rule.max;
    
    // 4. Highlight invalid cells
    if (!isValid) {
      univerAPI.setCellStyle(range, { backgroundColor: 'rgba(244,67,54,0.2)' });
    }
  }
});
```

---

### Pattern D: **Extract → Visualize → Annotate**
Used for: Plotting, graphing

**Steps:**
1. User selects data ranges (X and Y)
2. Sidebar extracts values from Univer
3. Plot generated in sidebar
4. User interactions (zoom, annotations) stay in sidebar
5. Optional: Save plot image/metadata to separate store

**Example: Quick Plot**
```typescript
// 1. Read data
const xData = univerAPI.getRange('A1:A10');
const yData = univerAPI.getRange('B1:B10');

// 2. Generate plot in sidebar
const plotConfig = {
  data: [{ x: xData, y: yData, type: 'scatter' }],
  layout: { title: 'Data Plot' }
};
renderPlot(plotConfig);

// 3. No write-back to Univer (plot lives in sidebar)
```

---

## Univer API Integration

### Required Univer.js Methods

```typescript
interface UniverAPI {
  // Read operations
  getValue(cellRef: string): any;
  getRange(range: string): any[][];
  getFormula(cellRef: string): string;
  getSelection(): string; // e.g., "A1:B10"
  
  // Write operations
  setValue(cellRef: string, value: any): void;
  setRange(range: string, values: any[][]): void;
  setFormula(cellRef: string, formula: string): void;
  
  // Formatting
  setCellStyle(cellRef: string, style: CellStyle): void;
  setRangeStyle(range: string, style: CellStyle): void;
  
  // Events
  onCellChange(callback: (cellRef: string) => void): void;
  onSelectionChange(callback: (range: string) => void): void;
}
```

### Wrapper Layer
Create a `univerService.ts` that wraps Univer API calls for consistent usage across all sidebars:

```typescript
// src/services/univerService.ts
export class UniverService {
  constructor(private univerInstance: Univer) {}
  
  // Standardized methods
  async readRange(range: string): Promise<CellData[][]> { ... }
  async writeRange(range: string, data: any[][]): Promise<void> { ... }
  async getCurrentSelection(): Promise<string> { ... }
  async applyFormulaToRange(range: string, formula: string): Promise<void> { ... }
}
```

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- ✅ Uncertainty Propagation sidebar (current - nearly complete)
- [ ] Create `univerService.ts` wrapper
- [ ] Document Univer API methods we need
- [ ] Create sidebar component template

### Phase 2: Core Scientific Tools (Week 3-4)
- [ ] Unit Converter sidebar
- [ ] Formula Builder dialog
- [ ] Statistical Analysis sidebar

### Phase 3: Data Quality (Week 5-6)
- [ ] Data Validation dialog
- [ ] Outlier Detection sidebar
- [ ] Data Smoothing dialog

### Phase 4: Visualization & Metadata (Week 7-8)
- [ ] Quick Plot sidebar
- [ ] Metadata Manager sidebar
- [ ] Export/Import tools

---

## Sidebar Component Template

Standard structure for all sidebars:

```typescript
interface SidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef: UniverSpreadsheetRef; // Reference to Univer instance
}

export const ScientificToolSidebar: React.FC<SidebarProps> = ({ 
  open, 
  onClose, 
  univerRef 
}) => {
  // 1. State for user inputs
  const [selectedRange, setSelectedRange] = useState<string>('');
  const [parameters, setParameters] = useState<any>({});
  const [results, setResults] = useState<any>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState('');

  // 2. Read data from Univer
  const readData = async () => {
    try {
      const data = await univerRef.current?.getRange(selectedRange);
      return data;
    } catch (err) {
      setError('Failed to read data');
    }
  };

  // 3. Process data (frontend or call backend)
  const processData = async () => {
    setIsProcessing(true);
    try {
      const inputData = await readData();
      
      // Option A: Frontend processing
      const result = performCalculation(inputData, parameters);
      
      // Option B: Backend processing
      // const result = await invoke('process_data', { data: inputData, params: parameters });
      
      setResults(result);
    } catch (err) {
      setError(err.toString());
    } finally {
      setIsProcessing(false);
    }
  };

  // 4. Write results back to Univer
  const writeResults = async () => {
    try {
      await univerRef.current?.setRange(parameters.outputRange, results);
    } catch (err) {
      setError('Failed to write results');
    }
  };

  // 5. UI renders inputs, controls, results
  return (
    <Paper sx={{ /* sidebar styling */ }}>
      {/* Range selection */}
      {/* Parameter inputs */}
      {/* Process button */}
      {/* Results display */}
      {/* Write back button */}
    </Paper>
  );
};
```

---

## Backend Processing (Tauri Commands)

When calculations are complex or require Rust libraries:

```rust
// src-tauri/src/scientific/mod.rs

#[tauri::command]
pub async fn process_scientific_data(
    data: Vec<Vec<f64>>,
    operation: String,
    parameters: serde_json::Value,
) -> Result<Vec<Vec<f64>>, String> {
    match operation.as_str() {
        "uncertainty_propagation" => uncertainty::propagate(data, parameters),
        "outlier_detection" => outliers::detect(data, parameters),
        "data_smoothing" => smoothing::apply(data, parameters),
        _ => Err(format!("Unknown operation: {}", operation))
    }
}
```

---

## Data Storage Strategy

### What Lives in Univer:
- ✅ Raw measurement values
- ✅ Calculated values (formulas)
- ✅ Cell formatting (colors, fonts)
- ✅ Standard Excel-like formulas

### What Lives Outside Univer:
- ❌ Uncertainty metadata (stored separately, linked by cell reference)
- ❌ Unit definitions (stored in backend)
- ❌ Validation rules (stored in separate state)
- ❌ Experimental metadata (separate database)
- ❌ Plot configurations (separate state)

### Linking External Data to Univer:
```typescript
// External metadata storage
interface CellMetadata {
  cellRef: string; // "A5"
  uncertainty?: { type: 'absolute' | 'percentage', value: number };
  unit?: string;
  experimentalContext?: {
    instrument: string;
    date: Date;
    operator: string;
  };
}

// Metadata store (separate from Univer)
const metadataStore = new Map<string, CellMetadata>();

// When reading from Univer, enrich with metadata
const cellValue = univerAPI.getValue('A5');
const metadata = metadataStore.get('A5');
const enrichedData = { value: cellValue, ...metadata };
```

---

## Benefits of This Architecture

### ✅ Advantages:
1. **Leverage Univer's strengths**: Excel compatibility, formula engine, UI
2. **Modular**: Each scientific tool is independent
3. **Maintainable**: Clear separation of concerns
4. **Flexible**: Easy to add new tools as sidebars
5. **User-friendly**: Familiar spreadsheet interface
6. **No data duplication**: Single source of truth
7. **Interoperable**: Can export to Excel/CSV with standard data

### ⚠️ Limitations:
1. Univer's formula engine doesn't understand uncertainty notation
2. Need separate metadata storage for scientific context
3. Advanced features (units, uncertainty) require custom UI
4. Performance depends on Univer API efficiency

### 🎯 Solutions:
1. Use formulas for calculations, store uncertainties as metadata
2. Sidebars display both value and uncertainty together
3. Backend handles complex scientific calculations
4. Cache frequently accessed data in frontend

---

## Example: Complete Uncertainty Propagation Flow

```typescript
// 1. User opens uncertainty sidebar
<UncertaintySidebar open={true} univerRef={spreadsheetRef} />

// 2. User configures variables
variables = [
  { name: 'a', valueRange: 'A1:A10', uncertaintyRange: 'B1:B10' },
  { name: 'b', valueRange: 'C1:C10', uncertaintyRange: 'D1:D10' }
];
formula = 'a + b';
outputValueRange = 'E1:E10';
outputUncertaintyRange = 'F1:F10';

// 3. Read data from Univer
const aValues = await univerRef.getRange('A1:A10'); // [5.2, 5.3, ...]
const aUncertainties = await univerRef.getRange('B1:B10'); // [0.1, 0.1, ...]
const bValues = await univerRef.getRange('C1:C10'); // [3.1, 3.2, ...]
const bUncertainties = await univerRef.getRange('D1:D10'); // [0.05, 0.05, ...]

// 4. Send to backend for processing
const result = await invoke('propagate_uncertainty', {
  variables: [
    { name: 'a', values: aValues, uncertainties: aUncertainties },
    { name: 'b', values: bValues, uncertainties: bUncertainties }
  ],
  formula: 'a + b'
});

// 5. Write results back to Univer
await univerRef.setRange('E1:E10', result.values); // [8.3, 8.5, ...]
await univerRef.setRange('F1:F10', result.uncertainties); // [0.11, 0.11, ...]

// 6. Done! Univer displays the results
```

---

## Next Steps

1. **Complete uncertainty propagation sidebar** (current task)
2. **Create `univerService.ts`** wrapper for API standardization
3. **Implement unit converter sidebar** (simpler, good second example)
4. **Build formula builder dialog** (more complex, templates for other tools)
5. **Add statistical analysis sidebar** (read-only analysis example)

This architecture allows us to build powerful scientific tools while leveraging Univer's robust spreadsheet foundation! 🎯
