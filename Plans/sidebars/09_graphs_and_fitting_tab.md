       # Graphs & Fitting Tab 📊�

**Status**: 🔄 **IN PROGRESS**  
**Priority**: High  
**Complexity**: High  
**Dependencies**: ECharts, Data Library, nalgebra, levenberg-marquardt

**Current Focus**: Implementing basic plotting component with ECharts integration, followed by fitting backend and UI components.

---

## Purpose

Unified tab for creating publication-quality plots and performing full regression analysis. Integrates data visualization with statistical regression analysis in a single workflow. Supports 2D and 3D plotting with n-dimensional regression analysis capabilities.

**Note**: Uses **ECharts 6.0** (already integrated in QuickPlotSidebar) for all visualizations. Extends existing plotting infrastructure with regression analysis capabilities.

**Current Implementation Tasks**:
- [ ] **Basic Plotting Component**: Implement ECharts 2D scatter/line plots with error bars from data library
- [ ] **Expression Parser**: Add math expression parser (fasteval/meval) for user-defined functions like 'a*exp(-x/b) + c'
- [ ] **Regression Analysis Backend**: Implement nonlinear least squares regression using argmin's Levenberg-Marquardt with uncertainty propagation
- [ ] **Regression UI Components**: Build UI for data selection, regression functions, initial guesses
- [ ] **Plotting-Regression Integration**: Overlay regression curves on data plots, show residuals, display regression parameters with uncertainties
- [ ] **Physics-Specific Regression Functions**: Add exponential decay, damped oscillation, power laws as presets

---

## Features

### Plot Sub-Tab
- **Create named plots** from Data Library sequences
- **2D plots**: Scatter, Line, Scatter+Line, Bar (ECharts types: 'scatter', 'line', 'bar')
- **3D plots**: 3D Scatter, 3D Surface (ECharts GL components)
- **Statistical plots**: QQ plots, Box plots, Histogram (integrated with statistical analysis)
- **Correlation plots**: Scatter plots with regression lines and correlation coefficients
- **Residual plots**: For regression diagnostics and model validation
- **Multi-plot support**: Multiple series on same ECharts instance with visibility toggles
- **Data validation**: Prevent length mismatches
- **Plot layers/groups**: Organize and bulk hide/show plot series
- **Interactive controls**: Zoom, pan, dataZoom, toolbox (ECharts built-in)
- **Export**: PNG, SVG (ECharts saveAsImage feature)

### Advanced Statistical Visualizations
- **QQ Plots**: Quantile-quantile plots for normality assessment with reference lines
- **Box Plots**: Box-and-whisker plots for distribution comparison and outlier detection
- **Correlation Scatter Plots**: Interactive scatter plots with regression lines, confidence bands, and correlation statistics
- **Residual Plots**: Diagnostic plots for regression analysis showing residuals vs fitted values, residuals vs predictors, and Q-Q plots of residuals
- **Distribution Overlays**: Multiple distribution curves on histograms for comparison
- **Statistical Annotations**: Automatic display of p-values, R², correlation coefficients on plots

**Note**: These advanced statistical visualizations complement the Statistical Analysis Sidebar by providing dedicated plotting interfaces for statistical diagnostics and correlation analysis.

### Fit Sub-Tab
- **Select plot to fit** from active plots (ECharts series)
- **Fit functions**: Linear, Polynomial, Exponential, Logarithmic, Power, Gaussian, Custom
- **User-defined regression functions**: Custom mathematical expressions with parameter fitting
- **N-dimensional regression analysis**: Full regression analysis for n-dimensional datasets with automatic detection for 2D (Y=f(X)) and 3D (Z=f(X,Y))
- **Parameter estimation**: Initial guess with auto-calculation
- **Uncertainty propagation**: Full uncertainty propagation in regression parameters and goodness-of-fit metrics using data uncertainties
- **Goodness-of-fit metrics**: R², χ², RMSE
- **Residuals plot**: Automatic residuals visualization (new ECharts series)
- **Fit comparison**: Compare multiple fit functions for same data
- **Non-intrusive warnings**: Alert when plot data changes after fit
- **Fit overlay**: Add fit curve as new ECharts series on existing chart

---

## UI Layout

### Plot Sub-Tab

```
┌──────────────────────────────────────────────────────────────────────┐
│ [Home] [Spreadsheet] [→ Graphs & Fitting] [Monte Carlo] [Solver]    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─ Control Panel ─────────────────┐  ┌─ Plot Area ────────────────┐│
│  │                                  │  │                            ││
│  │ [●📊 Plot] [○📐 Fit]            │  │                            ││
│  │ ═══════════════════════════      │  │   Interactive Plot         ││
│  │                                  │  │                            ││
│  │ ┌─ New Plot ─────────────────┐  │  │   (ECharts canvas)         ││
│  │ │                             │  │  │                            ││
│  │ │ Name:                       │  │  │   Multi-series overlay     ││
│  │ │ [Temp vs Time - Run 1____] │  │  │   with legend toggles      ││
│  │ │                             │  │  │                            ││
│  │ │ Type: [Scatter ▼]          │  │  │                            ││
│  │ │  • Scatter                  │  │  │                            ││
│  │ │  • Line                     │  │  │                            ││
│  │ │  • Scatter + Line           │  │  │                            ││
│  │ │  • Bar                      │  │  │                            ││
│  │ │  • 3D Scatter               │  │  │                            ││
│  │ │  • 3D Surface               │  │  │                            ││
│  │ │  • Contour                  │  │  │                            ││
│  │ │                             │  │  │                            ││
│  │ │ Dimensions: [2D ▼]         │  │  │                            ││
│  │ │  (2D or 3D only)            │  │  │                            ││
│  │ │                             │  │  └────────────────────────────┘│
│  │ │ ┌─ X-Axis ────────────────┐│  │                                │
│  │ │ │ [📚 From Library ▼]     ││  │  ┌─ Active Plots ─────────────┐│
│  │ │ │                         ││  │  │                            ││
│  │ │ │ 🔍 [Search: time_____] ││  │  │ Layer 1: Experiment Data   ││
│  │ │ │                         ││  │  │ ☑ Temp vs Time - Run 1     ││
│  │ │ │ Selected:               ││  │  │   Scatter, 100 pts         ││
│  │ │ │ ● Time Values           ││  │  │   [👁️] [✏️] [❌] [➡️ Fit]  ││
│  │ │ │   100 pts │ 0-99 s      ││  │  │                            ││
│  │ │ │   No uncertainties      ││  │  │ ☑ Temp vs Time - Run 2     ││
│  │ │ │                         ││  │  │   Line, 100 pts            ││
│  │ │ │ [✓] Include uncertainty││  │  │   [👁️] [✏️] [❌] [➡️ Fit]  ││
│  │ │ │     (disabled)          ││  │  │                            ││
│  │ │ └─────────────────────────┘│  │  │ Layer 2: Calibration       ││
│  │ │                             │  │  │ ☐ Calibration Curve        ││
│  │ │ ┌─ Y-Axis ────────────────┐│  │  │   Scatter, 20 pts          ││
│  │ │ │ [📚 From Library ▼]     ││  │  │   [👁️] [✏️] [❌] [➡️ Fit]  ││
│  │ │ │                         ││  │  │                            ││
│  │ │ │ Selected:               ││  │  │ [+ New Layer]              ││
│  │ │ │ ● Temp Sensor A         ││  │  │ [👁️ Hide All] [👁️ Show All]││
│  │ │ │   100 pts │23.5-35.2°C  ││  │  │ [🗑️ Clear Hidden]          ││
│  │ │ │   σ: ±0.1 °C            ││  │  └────────────────────────────┘│
│  │ │ │                         ││  │                                │
│  │ │ │ [✓] Include uncertainty││  │                                │
│  │ │ │     (±0.1 °C)           ││  │                                │
│  │ │ └─────────────────────────┘│  │                                │
│  │ │                             │  │                                │
│  │ │ ┌─ Z-Axis (3D) ───────────┐│  │                                │
│  │ │ │ [None ▼]                ││  │                                │
│  │ │ └─────────────────────────┘│  │                                │
│  │ │                             │  │                                │
│  │ │ ⚠️ Validation:              │  │                                │
│  │ │ ✓ X and Y have same length │  │                                │
│  │ │   (100 points each)         │  │                                │
│  │ │                             │  │                                │
│  │ │ ┌─ Style ─────────────────┐│  │                                │
│  │ │ │ Color: [🎨 #1f77b4]    ││  │                                │
│  │ │ │ Marker: [● ▼]           ││  │                                │
│  │ │ │ Size: [5─────10]        ││  │                                │
│  │ │ │ Opacity: [0.8────]      ││  │                                │
│  │ │ │ Line Width: [2──] (if line)││                                │
│  │ │ └─────────────────────────┘│  │                                │
│  │ │                             │  │                                │
│  │ │ [Create Plot]               │  │                                │
│  │ └─────────────────────────────┘  │                                │
│  │                                  │                                │
│  │ ┌─ Plot Settings ─────────────┐  │                                │
│  │ │ Title: [Temperature...____] │  │                                │
│  │ │ X Label: [Time (s)________] │  │                                │
│  │ │ Y Label: [Temperature (°C)] │  │                                │
│  │ │ [✓] Grid  [✓] Legend        │  │                                │
│  │ │ [Advanced ▼]                │  │                                │
│  │ └─────────────────────────────┘  │                                │
│  │                                  │                                │
│  │ [Export ▼] [Save Config]        │                                │
│  └──────────────────────────────────┘                                │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

### Fit Sub-Tab

```
┌──────────────────────────────────────────────────────────────────────┐
│ [Home] [Spreadsheet] [→ Graphs & Fitting] [Monte Carlo] [Solver]    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─ Control Panel ─────────────────┐  ┌─ Plot Area ────────────────┐│
│  │                                  │  │                            ││
│  │ [○📊 Plot] [●📐 Fit]            │  │   Plot + Fit Overlay       ││
│  │ ═══════════════════════════      │  │                            ││
│  │                                  │  │   • Original data          ││
│  │ Select Plot to Fit:              │  │   • Fit curve              ││
│  │                                  │  │   • Error bars             ││
│  │ (●) Temp vs Time - Run 1         │  │                            ││
│  │     100 pts, 2D (X→Y)            │  │                            ││
│  │     X: Time Values (s)           │  │                            ││
│  │     Y: Temp Sensor A (°C)        │  │                            ││
│  │     With Y uncertainties         │  │                            ││
│  │     [Show Data Table]            │  │                            ││
│  │                                  │  │                            ││
│  │ ( ) Temp vs Time - Run 2         │  │                            ││
│  │     100 pts, 2D (X→Y)            │  │                            ││
│  │     No uncertainties             │  │                            ││
│  │                                  │  │                            ││
│  │ ( ) Surface Data                 │  │                            ││
│  │     50 pts, 3D (X,Y→Z)           │  └────────────────────────────┘│
│  │     Z with uncertainties         │                                │
│  │                                  │  ┌─ Fit Results ──────────────┐│
│  │ ┌─ Fit Configuration ──────────┐│  │                            ││
│  │ │                               ││  │ ⚠️ Warning:                 ││
│  │ │ Selected: Temp vs Time - R1  ││  │ Plot data modified         ││
│  │ │ Fit: Y = f(X)                ││  │ 2024-10-08 15:30           ││
│  │ │                               ││  │ [Re-run Fit] [Dismiss]     ││
│  │ │ Fit Function:                ││  │ ────────────────────       ││
│  │ │ [Linear ▼]                   ││  │                            ││
│  │ │  • Linear                     ││  │ Function: Linear           ││
│  │ │  • Polynomial (deg 2)         ││  │   y = a·x + b              ││
│  │ │  • Polynomial (deg 3)         ││  │                            ││
│  │ │  • Polynomial (deg 4)         ││  │ Parameters:                ││
│  │ │  • Polynomial (deg 5)         ││  │   a = 0.2340 ± 0.0018      ││
│  │ │  • Exponential                ││  │   b = 23.45 ± 0.15         ││
│  │ │  • Logarithmic                ││  │                            ││
│  │ │  • Power Law                  ││  │ Goodness of Fit:           ││
│  │ │  • Gaussian                   ││  │   R² = 0.9985              ││
│  │ │  • Custom Formula...          ││  │   χ²_red = 1.023           ││
│  │ │                               ││  │   RMSE = 0.052 °C          ││
│  │ │ Formula: y = a·x + b         ││  │   Points: 100              ││
│  │ │                               ││  │   DOF: 98                  ││
│  │ │ Initial Guess:               ││  │                            ││
│  │ │  a: [1.0______] [Auto]       ││  │ [Show Residuals]           ││
│  │ │  b: [0.0______] [Auto]       ││  │ [Copy Parameters]          ││
│  │ │                               ││  │ [Save Fit to Library]      ││
│  │ │ Weighting:                   ││  │ [Export Report]            ││
│  │ │ (●) Use Y uncertainties      ││  │                            ││
│  │ │ ( ) Uniform (no weighting)   ││  │ ┌─ Compare Fits ──────┐   ││
│  │ │                               ││  │ │ Linear: R²=0.9985  │   ││
│  │ │ Constraints: [None ▼]        ││  │ │ Poly 2: R²=0.9992  │   ││
│  │ │                               ││  │ │ Exp: R²=0.9876     │   ││
│  │ │ [Run Fit]                    ││  │ │ [Clear History]    │   ││
│  │ │                               ││  │ └────────────────────┘   ││
│  │ └───────────────────────────────┘│  └────────────────────────────┘│
│  │                                  │                                │
│  └──────────────────────────────────┘                                │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

### 3D Fitting Example

```
┌─ Fit Configuration ───────────────┐
│                                   │
│ Selected: Surface Data            │
│ Fit: Z = f(X, Y)  [3D Fit]       │
│                                   │
│ Fit Function:                     │
│ [Planar ▼]                        │
│  • Planar: z = a·x + b·y + c      │
│  • Paraboloid: z = a·x² + b·y² + c│
│  • Polynomial 2D (order 2)        │
│  • Polynomial 2D (order 3)        │
│  • Custom 2D Formula...           │
│                                   │
│ Formula: z = a·x + b·y + c       │
│                                   │
│ Initial Guess:                    │
│  a: [1.0______] [Auto]            │
│  b: [1.0______] [Auto]            │
│  c: [0.0______] [Auto]            │
│                                   │
│ Weighting:                        │
│ (●) Use Z uncertainties           │
│ ( ) Uniform                       │
│                                   │
│ [Run 3D Fit]                      │
└───────────────────────────────────┘

Results:
┌─ Fit Results ────────────────────┐
│ Function: Planar                 │
│   z = a·x + b·y + c              │
│                                  │
│ Parameters:                      │
│   a = 1.234 ± 0.056              │
│   b = 2.345 ± 0.078              │
│   c = 10.123 ± 0.234             │
│                                  │
│ Goodness of Fit:                 │
│   R² = 0.9876                    │
│   χ²_red = 1.123                 │
│   RMSE = 0.234                   │
│                                  │
│ [Show 3D Residuals Surface]      │
└──────────────────────────────────┘
```

---

## Data Flow Pattern

### Plot Creation Flow
1. User switches to Plot sub-tab
2. Names the plot
3. Selects plot type and dimensionality
4. Selects X-axis data from Data Library
5. Selects Y-axis data from Data Library
6. (Optional) Selects Z-axis for 3D
7. System validates:
   - All sequences have same length
   - No NaN/invalid values
8. User customizes style (color, marker, etc.)
9. Clicks "Create Plot"
10. Plot appears in canvas and "Active Plots" list
11. Plot is now available for fitting

### Fitting Flow
1. User switches to Fit sub-tab
2. Selects plot to fit (radio buttons)
3. System shows plot details (dimensions, data sources)
4. User chooses regression function
5. (Optional) Sets initial parameter guesses
6. Chooses weighting method
7. Clicks "Run Regression"
8. Backend performs least-squares regression
9. Results displayed with uncertainties
10. Regression curve overlaid on plot
11. Residuals plot available
12. Can save regression results to Data Library

---

## Technical Implementation

### TypeScript Interfaces

```typescript
// AnaFis/src/types/graphsAndFitting.ts

interface PlotDefinition {
  id: string;
  name: string;
  type: 'scatter' | 'line' | 'bar' | '3d_scatter' | '3d_surface';  // ECharts types
  dimensionality: '2d' | '3d';
  
  // Data sources from library
  axes: {
    x: {
      sequenceId: string;
      includeUncertainty: boolean;
    };
    y: {
      sequenceId: string;
      includeUncertainty: boolean;
    };
    z?: {
      sequenceId: string;
      includeUncertainty: boolean;
    };
  };
  
  // ECharts series configuration
  echartsSeriesConfig: {
    type: 'scatter' | 'line' | 'bar' | 'scatter3D' | 'surface3D';
    name: string;
    data: [number, number][] | [number, number, number][];  // ECharts data format
    itemStyle: {
      color: string;
      borderWidth?: number;
    };
    symbolSize: number;
    lineStyle?: {
      width: number;
      type: 'solid' | 'dashed' | 'dotted';
    };
    // Error bars for ECharts
    errorBar?: {
      type: 'bar';
      xError: number[] | [number, number][];
      yError: number[] | [number, number][];
    };
  };
  
  // Visibility and organization
  visible: boolean;
  layer: string;  // Layer name for grouping
  
  // Fit results (if fitted)
  currentFit?: FitResult;
  fitHistory?: FitResult[];  // For comparison
  
  // Modification tracking
  dataModifiedSinceFit: boolean;
  lastModified: Date;
}

interface FitResult {
  id: string;
  plotId: string;
  timestamp: Date;
  
  function: {
    type: 'linear' | 'polynomial' | 'exponential' | 'logarithmic' | 'power' | 'gaussian' | 'custom';
    formula: string;
    degree?: number;  // For polynomials
    parameters: Array<{
      name: string;
      value: number;
      uncertainty: number;
      unit?: string;
    }>;
  };
  
  goodnessOfFit: {
    rSquared: number;
    chiSquaredReduced: number;
    rmse: number;
    residuals: number[];
    degreesOfFreedom: number;
  };
  
  // For n-dimensional fits
  dimensions: {
    independent: string[];  // ['x'] or ['x', 'y']
    dependent: string;      // 'y' or 'z'
  };
  
  // Weighting used
  weighted: boolean;
  
  // Fit settings
  initialGuess: Record<string, number>;
  constraints?: Record<string, { min?: number; max?: number }>;
}

interface PlotLayer {
  name: string;
  visible: boolean;
  plots: string[];  // Plot IDs in this layer
}

interface GraphsAndFittingState {
  // Plot management
  plots: Map<string, PlotDefinition>;
  layers: PlotLayer[];
  activePlotId: string | null;
  
  // Fit state
  selectedPlotIdForFit: string | null;
  currentFit: FitResult | null;
  
  // UI state
  activeTab: 'plot' | 'fit';
  plotSettings: {
    title: string;
    xLabel: string;
    yLabel: string;
    zLabel?: string;
    showGrid: boolean;
    showLegend: boolean;
  };
}
```

### Validation Functions

```typescript
// Validate data sequences before plotting
function validatePlotData(
  sequences: { x: DataSequence; y: DataSequence; z?: DataSequence }
): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  // Check X and Y have same length
  if (sequences.x.values.length !== sequences.y.values.length) {
    errors.push(
      `X and Y have different lengths: ` +
      `${sequences.x.values.length} vs ${sequences.y.values.length}`
    );
  }
  
  // Check Z if present
  if (sequences.z && sequences.z.values.length !== sequences.x.values.length) {
    errors.push(
      `Z has different length: ${sequences.z.values.length} ` +
      `(X and Y have ${sequences.x.values.length})`
    );
  }
  
  // Check for minimum points
  const minPoints = sequences.z ? 3 : 2;  // Need more points for 3D
  if (sequences.x.values.length < minPoints) {
    errors.push(`Insufficient points: need at least ${minPoints}, got ${sequences.x.values.length}`);
  }
  
  // Check for NaN/Infinity
  const checkInvalid = (seq: DataSequence, name: string) => {
    const invalid = seq.values.filter(v => !isFinite(v));
    if (invalid.length > 0) {
      errors.push(`${name} contains ${invalid.length} invalid values (NaN or Infinity)`);
    }
  };
  
  checkInvalid(sequences.x, 'X');
  checkInvalid(sequences.y, 'Y');
  if (sequences.z) checkInvalid(sequences.z, 'Z');
  
  // Check uncertainties match if present
  const checkUncertainties = (seq: DataSequence, name: string) => {
    if (seq.uncertainties && seq.uncertainties.length !== seq.values.length) {
      errors.push(
        `${name} uncertainties have wrong length: ` +
        `${seq.uncertainties.length} vs ${seq.values.length}`
      );
    }
  };
  
  checkUncertainties(sequences.x, 'X');
  checkUncertainties(sequences.y, 'Y');
  if (sequences.z) checkUncertainties(sequences.z, 'Z');
  
  return {
    valid: errors.length === 0,
    errors
  };
}

// Show validation error dialog
function showValidationError(errors: string[]): void {
  // Display modal with errors
  const errorList = errors.map((err, i) => `${i + 1}. ${err}`).join('\n');
  
  alert(`Cannot create plot:\n\n${errorList}\n\nPlease fix these issues and try again.`);
}
```

### Rust Fitting Backend

```rust
// src-tauri/src/fitting/mod.rs

use levenberg_marquardt::{LeastSquaresProblem, LevenbergMarquardt};
use nalgebra::{DVector, DMatrix};

#[derive(Debug, Serialize, Deserialize)]
pub struct FitRequest {
    pub x_data: Vec<f64>,
    pub y_data: Vec<f64>,
    pub y_uncertainties: Option<Vec<f64>>,
    pub function_type: String,
    pub initial_guess: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FitResponse {
    pub parameters: Vec<f64>,
    pub uncertainties: Vec<f64>,
    pub r_squared: f64,
    pub chi_squared_reduced: f64,
    pub rmse: f64,
    pub residuals: Vec<f64>,
}

#[tauri::command]
pub async fn fit_linear(request: FitRequest) -> Result<FitResponse, String> {
    // y = a*x + b
    let n = request.x_data.len();
    
    // Build design matrix
    let mut design = DMatrix::zeros(n, 2);
    for i in 0..n {
        design[(i, 0)] = request.x_data[i];
        design[(i, 1)] = 1.0;
    }
    
    let y = DVector::from_vec(request.y_data.clone());
    
    // Weighted least squares if uncertainties provided
    let weights = if let Some(ref unc) = request.y_uncertainties {
        DVector::from_vec(unc.iter().map(|u| 1.0 / (u * u)).collect())
    } else {
        DVector::from_element(n, 1.0)
    };
    
    // Solve weighted least squares
    let weighted_design = design.clone().component_mul(&weights.map(|w| w.sqrt()));
    let weighted_y = y.component_mul(&weights.map(|w| w.sqrt()));
    
    let params = (weighted_design.transpose() * &weighted_design)
        .try_inverse()
        .ok_or("Matrix inversion failed")?
        * weighted_design.transpose()
        * weighted_y;
    
    // Calculate residuals
    let y_fit = &design * &params;
    let residuals: Vec<f64> = (y - y_fit).iter().copied().collect();
    
    // Calculate goodness of fit
    let ss_res: f64 = residuals.iter().map(|r| r * r).sum();
    let y_mean: f64 = request.y_data.iter().sum::<f64>() / n as f64;
    let ss_tot: f64 = request.y_data.iter().map(|yi| (yi - y_mean).powi(2)).sum();
    let r_squared = 1.0 - ss_res / ss_tot;
    
    let dof = n - params.len();
    let chi_squared_reduced = ss_res / dof as f64;
    let rmse = (ss_res / n as f64).sqrt();
    
    // Calculate parameter uncertainties
    let cov_matrix = (weighted_design.transpose() * &weighted_design)
        .try_inverse()
        .ok_or("Covariance matrix inversion failed")?
        * chi_squared_reduced;
    
    let uncertainties: Vec<f64> = (0..params.len())
        .map(|i| cov_matrix[(i, i)].sqrt())
        .collect();
    
    Ok(FitResponse {
        parameters: params.iter().copied().collect(),
        uncertainties,
        r_squared,
        chi_squared_reduced,
        rmse,
        residuals,
    })
}

#[tauri::command]
pub async fn fit_polynomial(
    request: FitRequest,
    degree: usize
) -> Result<FitResponse, String> {
    // Similar to linear but with more columns in design matrix
    // ... implementation
}

#[tauri::command]
pub async fn fit_3d_planar(
    x_data: Vec<f64>,
    y_data: Vec<f64>,
    z_data: Vec<f64>,
    z_uncertainties: Option<Vec<f64>>
) -> Result<FitResponse, String> {
    // Fit z = a*x + b*y + c
    let n = x_data.len();
    
    let mut design = DMatrix::zeros(n, 3);
    for i in 0..n {
        design[(i, 0)] = x_data[i];
        design[(i, 1)] = y_data[i];
        design[(i, 2)] = 1.0;
    }
    
    // ... similar to linear fit but with 3 parameters
}
```

---

## Dependencies

**Frontend** (already installed):
```json
{
  "echarts": "6.0.0"  // ✅ Already in package.json
}
```

**Backend** (already added to Cargo.toml):
```toml
[dependencies]
nalgebra = "0.33"              # ✅ Already added
levenberg-marquardt = "0.14"   # ✅ Already added
statrs = "0.17"                # ✅ Already added (for statistics)
```

**Integration with QuickPlotSidebar**:
- Reuse existing ECharts instance and configuration
- Extend with curve fitting capabilities
- Share plot management logic

---

## File Location

- **Tab Component**: `AnaFis/src/pages/GraphsAndFittingTab.tsx`
- **Plot Panel**: `AnaFis/src/components/plotting/PlotPanel.tsx`
- **Fit Panel**: `AnaFis/src/components/plotting/FitPanel.tsx`
- **Plot Store**: `AnaFis/src/stores/plotStore.ts`
- **Types**: `AnaFis/src/types/graphsAndFitting.ts`
- **Rust Fitting**: `AnaFis/src-tauri/src/fitting/mod.rs`

---

## Success Criteria

- ✓ Can create named plots from Data Library
- ✓ Multiple series visible simultaneously with ECharts legend toggles
- ✓ Data validation prevents length mismatches
- ✓ Plot layers allow bulk show/hide of series
- ✓ 2D regression works for all function types (Rust backend)
- ✓ 3D regression works for planar and polynomial surfaces
- ✓ Parameter uncertainties calculated correctly (Rust)
- ✓ R², χ², RMSE metrics accurate
- ✓ Residuals plot displays correctly (ECharts series)
- ✓ Regression comparison shows multiple regression results
- ✓ Non-intrusive warning when plot data changes
- ✓ Can save regression results to Data Library
- ✓ Export works (PNG, SVG via ECharts saveAsImage)
- ✓ **Seamlessly integrates with existing ECharts infrastructure**
- ✓ **Reuses QuickPlotSidebar patterns and components**

---

**Next Steps**: Implement after Data Library is complete
