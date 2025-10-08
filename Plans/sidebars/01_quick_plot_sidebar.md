# Quick Plot Sidebar 📊

**Status**: Planned  
**Priority**: High  
**Complexity**: Low (Simplified)  
**Dependencies**: Plotly.js (basic), Data Library

**Last Updated**: 2025-10-08 - Simplified to 2D preview only

---

## Purpose

Provide **quick 2D data visualization** directly from spreadsheet selections for rapid data exploration. Designed for fast, simple plotting while working in the spreadsheet.

For advanced features (3D, multi-dimensional, fitting), users should transition to the **Graphs & Fitting** tab.

---

## Features

### Simple 2D Plotting
- **Plot Types**: Scatter, Line, Scatter+Line
- **Data Input**: Direct range selection from current sheet
- **Error Bars**: Optional symmetric error bars for Y-axis
- **Interactive**: Zoom, pan, reset view (Plotly controls)
- **Quick Export**: Save as PNG

### Integration with Data Library
- **Save to Library**: Export X and Y data as named sequences
- **Open in Graphs & Fitting**: Transition to advanced plotting

### Instant Preview
- Real-time plot updates as ranges change
- No complex configuration needed
- 3-click workflow: Select X → Select Y → Plot

---

## UI Layout

```
┌─────────────────────────────────────┐
│ Quick Plot                      [X] │
├─────────────────────────────────────┤
│ Sheet: [Sheet1 ▼]                  │
│                                     │
│ X-Axis Data:                        │
│ Range: [A1:A100_] [Select from 📋] │
│ Label: [Time (s)_____________]     │
│                                     │
│ Y-Axis Data:                        │
│ Range: [B1:B100_] [Select from 📋] │
│ Label: [Temperature (°C)______]    │
│                                     │
│ Error Bars (optional):              │
│ Range: [C1:C100_] [Select from 📋] │
│ [✓] Show error bars                │
│                                     │
│ ┌─────────────────────────────┐   │
│ │                             │   │
│ │      PLOT PREVIEW           │   │
│ │      (Interactive)          │   │
│ │      - Zoom/Pan             │   │
│ │      - Hover values         │   │
│ │                             │   │
│ └─────────────────────────────┘   │
│                                     │
│ Plot Type:                          │
│ [● Scatter] [○ Line] [○ Both]      │
│                                     │
│ ⚠️ Validation: ✓ 100 points each   │
│                                     │
│ [Update Plot]                       │
│                                     │
│ ═══════════════════════════════    │
│                                     │
│ Actions:                            │
│ [💾 Save to Library]                │
│ [📊 Open in Graphs & Fitting →]    │
│ [📷 Export PNG]                     │
└─────────────────────────────────────┘
```

### Save to Library Dialog

```
┌──────────────────────────────┐
│ Save to Data Library     [X] │
├──────────────────────────────┤
│ This will save X and Y data  │
│ as separate sequences:       │
│                              │
│ X-Axis Sequence:             │
│ Name: [Time Values_______]   │
│ Unit: [s_________________]   │
│ Tags: [experiment_1, time]   │
│                              │
│ Y-Axis Sequence:             │
│ Name: [Temperature_______]   │
│ Unit: [°C________________]   │
│ Tags: [experiment_1, temp]   │
│ [✓] Include uncertainties    │
│                              │
│ [☐ Pin both sequences]       │
│                              │
│ [Save] [Cancel]              │
└──────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: Extract → Visualize (Read-only, no write-back)

1. User selects X range from sheet
2. User selects Y range from sheet
3. (Optional) User selects error range
4. System validates:
   - X and Y have same length
   - All values are numeric
   - No NaN/Infinity
5. Generate 2D plot with Plotly
6. User can:
   - Export as PNG
   - Save X/Y to Data Library
   - Open in Graphs & Fitting tab

---

## Features Intentionally Removed

These features are available in **Graphs & Fitting** tab:

❌ 3D, 4D, 5D plotting  
❌ Animation/time dimension  
❌ Color/size mapping  
❌ Contour/polar/heatmap plots  
❌ Log scales  
❌ Advanced uncertainty visualization (ellipses, bands)  
❌ Multiple series overlay  
❌ Curve fitting  
❌ Named plots  
❌ Plot layers  

**Rationale**: Sidebar should be simple and fast. Power users transition to full tab.

---

## Technical Implementation

### TypeScript Interfaces

```typescript
// AnaFis/src/types/quickPlot.ts

interface QuickPlotSidebarProps {
  open: boolean;
  onClose: () => void;
  univerAPI: UniverAPI;
  currentSheet: string;
}

interface QuickPlotData {
  x: {
    range: string;
    label: string;
    values: number[];
  };
  y: {
    range: string;
    label: string;
    values: number[];
  };
  errors?: {
    range: string;
    values: number[];
  };
}

interface QuickPlotConfig {
  type: 'scatter' | 'line' | 'both';
  showErrorBars: boolean;
  title?: string;
}
```

### Component Structure

```typescript
// AnaFis/src/components/spreadsheet/QuickPlotSidebar.tsx

import React, { useState, useEffect } from 'react';
import Plot from 'react-plotly.js';

export function QuickPlotSidebar({ open, onClose, univerAPI, currentSheet }: QuickPlotSidebarProps) {
  const [xRange, setXRange] = useState<string>('');
  const [yRange, setYRange] = useState<string>('');
  const [errorRange, setErrorRange] = useState<string>('');
  const [xLabel, setXLabel] = useState<string>('X');
  const [yLabel, setYLabel] = useState<string>('Y');
  const [plotType, setPlotType] = useState<'scatter' | 'line' | 'both'>('scatter');
  const [showErrorBars, setShowErrorBars] = useState<boolean>(false);
  const [plotData, setPlotData] = useState<QuickPlotData | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);
  
  const handleUpdatePlot = async () => {
    try {
      // Extract data
      const xValues = await univerAPI.getRange(currentSheet, xRange);
      const yValues = await univerAPI.getRange(currentSheet, yRange);
      
      const xFlat = xValues.flat().map(v => parseFloat(v));
      const yFlat = yValues.flat().map(v => parseFloat(v));
      
      // Validate
      if (xFlat.length !== yFlat.length) {
        setValidationError(`Length mismatch: X has ${xFlat.length} points, Y has ${yFlat.length}`);
        return;
      }
      
      if (xFlat.some(v => !isFinite(v)) || yFlat.some(v => !isFinite(v))) {
        setValidationError('Data contains invalid values (NaN or Infinity)');
        return;
      }
      
      // Get errors if enabled
      let errorValues: number[] | undefined;
      if (showErrorBars && errorRange) {
        const errors = await univerAPI.getRange(currentSheet, errorRange);
        errorValues = errors.flat().map(v => Math.abs(parseFloat(v)));
        
        if (errorValues.length !== yFlat.length) {
          setValidationError(`Error range length doesn't match: ${errorValues.length} vs ${yFlat.length}`);
          return;
        }
      }
      
      setValidationError(null);
      setPlotData({
        x: { range: xRange, label: xLabel, values: xFlat },
        y: { range: yRange, label: yLabel, values: yFlat },
        errors: errorValues ? { range: errorRange, values: errorValues } : undefined
      });
      
    } catch (error) {
      setValidationError(`Error: ${error.message}`);
    }
  };
  
  const handleSaveToLibrary = async () => {
    if (!plotData) return;
    
    // Open dialog to get metadata
    // ... implementation
  };
  
  const handleOpenInGraphsAndFitting = () => {
    // Save to library first, then navigate to tab
    // ... implementation
  };
  
  const handleExportPNG = () => {
    // Use Plotly's downloadImage
    const plotElement = document.querySelector('.js-plotly-plot');
    if (plotElement) {
      Plotly.downloadImage(plotElement, {
        format: 'png',
        filename: 'quick_plot'
      });
    }
  };
  
  // Build Plotly trace
  const trace: any = {
    x: plotData?.x.values || [],
    y: plotData?.y.values || [],
    mode: plotType === 'scatter' ? 'markers' : plotType === 'line' ? 'lines' : 'lines+markers',
    type: 'scatter',
    name: yLabel,
    error_y: plotData?.errors ? {
      type: 'data',
      array: plotData.errors.values,
      visible: showErrorBars
    } : undefined
  };
  
  return (
    <Drawer open={open} onClose={onClose} anchor="right" sx={{ width: 400 }}>
      <Box sx={{ width: 400, p: 2 }}>
        <Typography variant="h6">Quick Plot</Typography>
        
        {/* Sheet selector */}
        {/* Range inputs */}
        {/* Label inputs */}
        {/* Error bars toggle */}
        
        {/* Plot preview */}
        {plotData && (
          <Plot
            data={[trace]}
            layout={{
              title: `${yLabel} vs ${xLabel}`,
              xaxis: { title: xLabel },
              yaxis: { title: yLabel },
              autosize: true
            }}
            style={{ width: '100%', height: '300px' }}
            config={{ responsive: true }}
          />
        )}
        
        {/* Validation error */}
        {validationError && (
          <Alert severity="error">{validationError}</Alert>
        )}
        
        {/* Plot type selector */}
        {/* Action buttons */}
      </Box>
    </Drawer>
  );
}
```

### Validation Function

```typescript
function validateQuickPlotData(
  x: number[],
  y: number[],
  errors?: number[]
): { valid: boolean; error?: string } {
  // Check lengths
  if (x.length !== y.length) {
    return {
      valid: false,
      error: `X and Y have different lengths: ${x.length} vs ${y.length}`
    };
  }
  
  // Check minimum points
  if (x.length < 2) {
    return {
      valid: false,
      error: `Need at least 2 points, got ${x.length}`
    };
  }
  
  // Check for invalid values
  if (x.some(v => !isFinite(v))) {
    return { valid: false, error: 'X contains invalid values (NaN or Infinity)' };
  }
  
  if (y.some(v => !isFinite(v))) {
    return { valid: false, error: 'Y contains invalid values (NaN or Infinity)' };
  }
  
  // Check errors if present
  if (errors) {
    if (errors.length !== y.length) {
      return {
        valid: false,
        error: `Error bars length doesn't match: ${errors.length} vs ${y.length}`
      };
    }
    
    if (errors.some(v => !isFinite(v) || v < 0)) {
      return { valid: false, error: 'Error bars contain invalid values' };
    }
  }
  
  return { valid: true };
}
```

---

## Dependencies

```bash
npm install plotly.js react-plotly.js
npm install @types/plotly.js -D
```

---

## File Location

- **Component**: `AnaFis/src/components/spreadsheet/QuickPlotSidebar.tsx`
- **Types**: `AnaFis/src/types/quickPlot.ts`

---

## Integration Notes

### Data Library Integration
When saving to Data Library, use Rust backend:

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function saveToLibrary(xData: number[], yData: number[], metadata: any) {
  // Save X sequence
  await invoke('save_sequence', {
    sequence: {
      id: uuid(),
      name: `${metadata.name}_X`,
      description: metadata.description,
      tags: [...metadata.tags, 'quick_plot', 'x_axis'],
      unit: metadata.xUnit,
      values: xData,
      uncertainties: [],
      source: {
        type: 'spreadsheet',
        sheetName: metadata.sheetName,
        range: metadata.xRange,
        tabName: 'Spreadsheet',
        timestamp: new Date().toISOString()
      },
      isPinned: false
    }
  });
  
  // Save Y sequence
  await invoke('save_sequence', {
    sequence: {
      id: uuid(),
      name: `${metadata.name}_Y`,
      description: metadata.description,
      tags: [...metadata.tags, 'quick_plot', 'y_axis'],
      unit: metadata.yUnit,
      values: yData,
      uncertainties: yUncertainties || [],
      source: {
        type: 'spreadsheet',
        sheetName: metadata.sheetName,
        range: metadata.yRange,
        tabName: 'Spreadsheet',
        timestamp: new Date().toISOString()
      },
      isPinned: false
    }
  });
}
```

Uses same SQLite backend as Data Library Sidebar and Window.

---

## Success Criteria

- ✓ Can plot 2D scatter from spreadsheet ranges
- ✓ Can plot 2D line from spreadsheet ranges
- ✓ Can show Y-axis error bars
- ✓ Data validation prevents length mismatches
- ✓ Data validation catches NaN/Infinity
- ✓ Can export as PNG
- ✓ **Can save to Data Library via Rust invoke (SQLite backend)**
- ✓ "Open in Graphs & Fitting" navigates correctly
- ✓ Performance: <500ms for 1000 points
- ✓ Responsive plot (zoom, pan work)

---

## Testing Plan

### Unit Tests
- Range extraction from Univer
- Data validation logic
- Length mismatch detection

### Integration Tests
- Plot generation with real data
- Save to Data Library flow
- Export PNG

### E2E Tests
- Complete workflow: Select ranges → Plot → Export
- Save to library → Open in Graphs & Fitting

---

**Next Steps**: Implement as simple sidebar, then build advanced Graphs & Fitting tab
