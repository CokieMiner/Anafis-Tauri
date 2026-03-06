# Automatic Uncertainty Propagation - Plugin Architecture Design

## Executive Summary

**Status**: 🔄 **PLUGIN APPROACH - VIABLE ALTERNATIVE**

After deep architectural analysis of Univer's internals, automatic uncertainty propagation via cell modification is **NOT FEASIBLE** due to fundamental architectural constraints. However, a **plugin-based approach** provides a viable alternative that extends Univer's functionality without core modifications.

**New Approach**: Implement uncertainty propagation as a Univer plugin that:
- Adds custom data types for uncertainty values
- Provides custom renderers for uncertainty display
- Intercepts formula calculations for uncertainty propagation
- Integrates with existing Rust uncertainty backend
- Supports correlated uncertainties via covariance matrices

---

## Why Plugin Approach Works

**Plugin Advantages**:
- ✅ **No Core Modifications**: Works within Univer's extension architecture
- ✅ **Backward Compatible**: Doesn't break existing functionality
- ✅ **Maintainable**: Clean separation of concerns
- ✅ **Extensible**: Can add correlated uncertainty support
- ✅ **User Choice**: Can be enabled/disabled per spreadsheet

**Plugin Capabilities**:
- Custom cell data types (`UncertaintyValue`, `CorrelatedUncertainty`)
- Custom cell renderers with uncertainty notation
- Formula function extensions for uncertainty operations
- Integration with Rust backend for complex propagation
- Covariance matrix management for correlated variables

---

## Plugin Architecture Overview

### Core Components

```
Univer Plugin Structure:
├── UncertaintyPlugin.ts              # Main plugin class
├── types/
│   ├── UncertaintyValue.ts           # Custom data type
│   ├── CorrelatedUncertainty.ts      # Correlated uncertainty type
│   └── UncertaintyFormula.ts         # Formula extensions
├── renderers/
│   ├── UncertaintyCellRenderer.ts    # Cell display renderer
│   └── UncertaintyTooltipRenderer.ts # Tooltip renderer
├── controllers/
│   ├── UncertaintyController.ts      # Plugin lifecycle
│   ├── PropagationController.ts      # Uncertainty calculation
│   └── CorrelationController.ts      # Covariance management
├── services/
│   ├── RustPropagationService.ts     # Rust backend integration
│   └── FormulaParserService.ts       # Formula analysis
└── ui/
    ├── UncertaintySidebar.tsx        # Plugin configuration
    └── CorrelationMatrixEditor.tsx   # Covariance editor
```

### Data Flow

```
1. User enters: "5.0 ± 0.1" in cell A1
2. Plugin detects uncertainty notation
3. Stores as UncertaintyValue { value: 5.0, uncertainty: 0.1 }
4. Renderer displays: "5.0 ± 0.1" with blue border
5. User enters formula: "=A1 + B1" in C1
6. Plugin intercepts formula evaluation
7. Calls Rust backend for uncertainty propagation
8. Stores result as UncertaintyValue in C1
9. Renderer displays propagated uncertainty
```

---

## User Experience Design

### Input Methods

**Supported Input Formats**:
```
Standard notation:
5.0 ± 0.1
5.0 +/- 0.1

Compact notation:
5.0(1)    → Interpreted as 5.0 ± 0.1
12.34(5)  → Interpreted as 12.34 ± 0.05

Percentage:
5.0 ± 2%  → Interpreted as 5.0 ± 0.1

With units:
5.0 ± 0.1 m
5.0(1) kg

Correlated notation:
5.0 ± 0.1 [corr: 0.8 with B1]
```

**Visual Indicators**:
```
Uncertainty Cell:
┏━━━━━━━━━━━━━┓
┃ 5.0 ± 0.1   ┃  ← Blue border indicates uncertainty
┗━━━━━━━━━━━━━┛

Correlated Cell:
┏━━━━━━━━━━━━━┓
┃ 5.0 ± 0.1   ┃  ← Green border indicates correlations
┣━━━━━━━━━━━━━┫
┃ ρ=0.8 B1    ┃  ← Shows correlation info
┗━━━━━━━━━━━━━┛
```

### Formula Operations

**Basic Operations** (Automatic Propagation):
```
A1: 5.0 ± 0.1
B1: 3.0 ± 0.05
C1: =A1 + B1    → 8.0 ± 0.1118
C1: =A1 * B1    → 15.0 ± 0.8062
C1: =SQRT(A1)   → 2.236 ± 0.0224
```

**Correlated Operations**:
```
A1: 5.0 ± 0.1 [ρ=0.8 with B1]
B1: 3.0 ± 0.05 [ρ=0.8 with A1]
C1: =A1 + B1    → 8.0 ± 0.1118 (correlation reduces uncertainty)
C1: =A1 / B1    → 1.667 ± 0.0561 (correlation affects result)
```

### Correlation Management

**Covariance Matrix Editor**:
```
Correlation Matrix for Sheet1:
┌─────┬─────┬─────┬─────┐
│     │ A1  │ B1  │ C1  │
├─────┼─────┼─────┼─────┤
│ A1  │ 1.0 │ 0.8 │ 0.0 │
│ B1  │ 0.8 │ 1.0 │ 0.0 │
│ C1  │ 0.0 │ 0.0 │ 1.0 │
└─────┴─────┴─────┴─────┘

Edit correlations:
A1↔B1: [0.8] (drag slider or type)
A1↔C1: [0.0] (no correlation)
```

---

## Technical Implementation

### Plugin Registration

```typescript
// AnaFis/src/plugins/uncertainty/UncertainyPlugin.ts

import { Plugin, PluginType } from '@univerjs/core';
import { UncertaintyController } from './controllers/UncertaintyController';
import { UncertaintyCellRenderer } from './renderers/UncertainyCellRenderer';

export class UncertaintyPlugin extends Plugin {
    static override type = PluginType.Univer;

    override onStarting(): void {
        // Register custom data types
        this._registerDataTypes();
        
        // Register renderers
        this._registerRenderers();
        
        // Register formula functions
        this._registerFormulaFunctions();
        
        // Initialize controllers
        this._initializeControllers();
    }

    private _registerDataTypes(): void {
        // Register UncertaintyValue and CorrelatedUncertainty types
        const dataTypeRegistry = this._context.get(DataTypeRegistry);
        dataTypeRegistry.register('uncertainty', UncertaintyValue);
        dataTypeRegistry.register('correlatedUncertainty', CorrelatedUncertainty);
    }

    private _registerRenderers(): void {
        const rendererRegistry = this._context.get(RendererRegistry);
        rendererRegistry.register('uncertainty', UncertaintyCellRenderer);
    }
}
```

### Custom Data Types

```typescript
// AnaFis/src/plugins/uncertainty/types/UncertainyValue.ts

export interface UncertaintyValue {
    type: 'uncertainty';
    value: number;
    uncertainty: number;
    unit?: string;
    confidence?: number; // Default 0.95 for 95% CI
}

export interface CorrelatedUncertainty extends UncertaintyValue {
    type: 'correlatedUncertainty';
    correlations: Map<string, number>; // cellRef -> correlation coefficient
    covarianceMatrixId?: string; // Reference to global covariance matrix
}
```

### Cell Renderer

```typescript
// AnaFis/src/plugins/uncertainty/renderers/UncertainyCellRenderer.ts

export class UncertaintyCellRenderer extends BaseCellRenderer {
    override render(cell: ICellData): HTMLElement {
        const div = document.createElement('div');
        
        if (this._isUncertaintyCell(cell)) {
            div.className = 'uncertainty-cell';
            div.style.border = '2px solid #2196F3'; // Blue border
            
            const value = cell.custom?.value || cell.v;
            const uncertainty = cell.custom?.uncertainty;
            
            if (uncertainty !== undefined) {
                div.textContent = `${value} ± ${uncertainty}`;
            } else {
                div.textContent = String(value);
            }
            
            // Add tooltip
            div.title = this._getUncertaintyTooltip(cell);
        }
        
        return div;
    }

    private _isUncertaintyCell(cell: ICellData): boolean {
        return cell.custom?.type === 'uncertainty' || 
               cell.custom?.type === 'correlatedUncertainty';
    }

    private _getUncertaintyTooltip(cell: ICellData): string {
        const custom = cell.custom;
        if (!custom) return '';
        
        let tooltip = `Value: ${custom.value}\n`;
        tooltip += `Uncertainty: ±${custom.uncertainty}\n`;
        
        if (custom.confidence) {
            tooltip += `Confidence: ${custom.confidence * 100}%\n`;
        }
        
        if (custom.unit) {
            tooltip += `Unit: ${custom.unit}\n`;
        }
        
        return tooltip;
    }
}
```

### Formula Integration

```typescript
// AnaFis/src/plugins/uncertainty/services/FormulaParserService.ts

export class FormulaParserService {
    // Intercept formula evaluation
    interceptFormula(formula: string, context: FormulaContext): ICellData {
        // Check if formula references uncertainty cells
        const hasUncertaintyRefs = this._hasUncertaintyReferences(formula, context);
        
        if (hasUncertaintyRefs) {
            // Call Rust backend for uncertainty propagation
            return this._calculatePropagatedUncertainty(formula, context);
        }
        
        return null; // Let normal evaluation proceed
    }

    private async _calculatePropagatedUncertainty(
        formula: string, 
        context: FormulaContext
    ): Promise<ICellData> {
        // Extract uncertainty values from referenced cells
        const uncertaintyInputs = this._extractUncertaintyInputs(formula, context);
        
        // Call Rust backend
        const result = await invoke('propagate_uncertainty', {
            formula,
            inputs: uncertaintyInputs
        });
        
        // Return cell data with uncertainty
        return {
            v: result.value,
            custom: {
                type: 'uncertainty',
                value: result.value,
                uncertainty: result.uncertainty
            }
        };
    }
}
```

### Rust Backend Integration

```rust
// src-tauri/src/plugins/uncertainty.rs

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UncertaintyPropagationRequest {
    formula: String,
    inputs: Vec<UncertaintyInput>,
    correlations: Option<CorrelationMatrix>,
}

#[derive(Serialize)]
pub struct UncertaintyPropagationResponse {
    value: f64,
    uncertainty: f64,
    correlations: Option<Vec<f64>>, // For correlated outputs
}

#[tauri::command]
pub async fn propagate_uncertainty(
    request: UncertaintyPropagationRequest
) -> Result<UncertaintyPropagationResponse, String> {
    // Use existing uncertainty propagation logic
    // Enhanced with correlation support
    
    let result = if let Some(correlations) = request.correlations {
        // Correlated propagation using covariance matrix
        propagate_with_correlations(&request.formula, &request.inputs, &correlations)
    } else {
        // Standard uncorrelated propagation
        propagate_uncertainty_standard(&request.formula, &request.inputs)
    };
    
    Ok(result)
}
```

---

## Correlation Support

### Covariance Matrix Management

```typescript
// AnaFis/src/plugins/uncertainty/controllers/CorrelationController.ts

export class CorrelationController {
    private _covarianceMatrices: Map<string, CovarianceMatrix> = new Map();
    
    createCorrelationMatrix(cellRefs: string[]): string {
        const matrixId = generateId();
        const size = cellRefs.length;
        
        const matrix = new CovarianceMatrix(size);
        matrix.cellRefs = cellRefs;
        
        // Initialize with uncertainties from cells
        for (let i = 0; i < size; i++) {
            const uncertainty = this._getCellUncertainty(cellRefs[i]);
            matrix.setVariance(i, uncertainty * uncertainty);
        }
        
        this._covarianceMatrices.set(matrixId, matrix);
        return matrixId;
    }
    
    setCorrelation(matrixId: string, cellRef1: string, cellRef2: string, correlation: number): void {
        const matrix = this._covarianceMatrices.get(matrixId);
        if (!matrix) return;
        
        const idx1 = matrix.cellRefs.indexOf(cellRef1);
        const idx2 = matrix.cellRefs.indexOf(cellRef2);
        
        if (idx1 >= 0 && idx2 >= 0) {
            matrix.setCorrelation(idx1, idx2, correlation);
        }
    }
    
    getCovarianceMatrix(matrixId: string): CovarianceMatrix | null {
        return this._covarianceMatrices.get(matrixId) || null;
    }
}
```

### Enhanced Propagation with Correlations

```rust
// src-tauri/src/scientific/uncertainty/correlated_propagation.rs

pub fn propagate_with_correlations(
    formula: &str,
    inputs: &[UncertaintyInput],
    correlations: &CorrelationMatrix,
) -> UncertaintyPropagationResponse {
    // Parse formula to extract dependencies
    let dependencies = parse_formula_dependencies(formula);
    
    // Build covariance matrix for relevant variables
    let cov_matrix = build_covariance_matrix(inputs, correlations, &dependencies);
    
    // Calculate Jacobian matrix
    let jacobian = calculate_jacobian(formula, inputs);
    
    // Propagate uncertainty using: σ² = J * Σ * J^T
    // where Σ is the covariance matrix
    let output_variance = propagate_variance(&jacobian, &cov_matrix);
    let output_uncertainty = output_variance.sqrt();
    
    // Calculate nominal value
    let nominal_value = evaluate_formula(formula, inputs);
    
    UncertaintyPropagationResponse {
        value: nominal_value,
        uncertainty: output_uncertainty,
        correlations: None, // For now, single output
    }
}
```

---

## Plugin Configuration UI

### Settings Panel

```typescript
// AnaFis/src/plugins/uncertainty/ui/UncertainySidebar.tsx

export function UncertaintySidebar() {
    const [enabled, setEnabled] = useState(true);
    const [displayFormat, setDisplayFormat] = useState('compact');
    const [confidenceLevel, setConfidenceLevel] = useState(0.95);
    
    return (
        <div className="uncertainty-sidebar">
            <h3>Uncertainty Propagation</h3>
            
            <div className="setting-group">
                <label>
                    <input 
                        type="checkbox" 
                        checked={enabled}
                        onChange={e => setEnabled(e.target.checked)}
                    />
                    Enable automatic uncertainty propagation
                </label>
            </div>
            
            <div className="setting-group">
                <label>Display Format:</label>
                <select value={displayFormat} onChange={e => setDisplayFormat(e.target.value)}>
                    <option value="compact">5.0 ± 0.1</option>
                    <option value="percentage">5.0 ± 2%</option>
                    <option value="scientific">5.0e0 ± 1.0e-1</option>
                </select>
            </div>
            
            <div className="setting-group">
                <label>Confidence Level:</label>
                <input 
                    type="number" 
                    value={confidenceLevel * 100}
                    onChange={e => setConfidenceLevel(e.target.valueAsNumber / 100)}
                    min="68" 
                    max="99" 
                    step="0.1"
                />%
            </div>
            
            <div className="setting-group">
                <button onClick={() => openCorrelationEditor()}>
                    Edit Correlations
                </button>
            </div>
        </div>
    );
}
```

---

## Integration with Existing Systems

### Coexistence with Manual Sidebar

**Plugin Approach Benefits**:
- ✅ **Seamless Integration**: Plugin works alongside existing manual sidebar
- ✅ **User Choice**: Enable plugin for automatic, use sidebar for manual control
- ✅ **Data Compatibility**: Both systems can read/write uncertainty data
- ✅ **Migration Path**: Easy transition from manual to automatic workflows

**Workflow Integration**:
```
Manual Sidebar User:
1. Uses existing uncertainty sidebar for complex formulas
2. Enables plugin for simple automatic propagation
3. Both systems work on same spreadsheet
4. Can migrate formulas from manual to automatic

Plugin User:
1. Enables plugin for automatic propagation
2. Uses manual sidebar for complex cases or verification
3. Full interoperability between systems
```

### Data Library Integration

**Enhanced Data Sequences**:
```typescript
interface UncertaintyDataSequence extends DataSequence {
    uncertainties?: number[];
    correlations?: CorrelationMatrix;
    covarianceMatrixId?: string;
}

interface CorrelationMatrix {
    id: string;
    variables: string[]; // Variable names
    matrix: number[][];  // Correlation coefficients
}
```

**Import/Export Support**:
- CSV/TSV: Support uncertainty columns
- JSON: Include correlation metadata
- Parquet: Native covariance matrix support
- Excel: Export with uncertainty notation

---

## Implementation Roadmap

### Phase 1: Basic Plugin Framework
- [ ] Create plugin skeleton with basic registration
- [ ] Implement UncertaintyValue data type
- [ ] Add basic cell renderer with blue border
- [ ] Simple input parsing (5.0 ± 0.1 notation)

### Phase 2: Formula Integration
- [ ] Formula interception system
- [ ] Basic uncertainty propagation (Rust backend integration)
- [ ] Simple operations (+, -, *, /, SQRT, etc.)

### Phase 3: Correlation Support
- [ ] Correlation matrix data structure
- [ ] Covariance matrix editor UI
- [ ] Enhanced propagation with correlations
- [ ] Visual indicators for correlated cells

### Phase 4: Advanced Features
- [ ] Complex formula support (nested functions)
- [ ] Unit propagation
- [ ] Statistical distributions
- [ ] Export/import with correlations

### Phase 5: Polish & Testing
- [ ] Performance optimization
- [ ] Comprehensive testing
- [ ] Documentation and examples
- [ ] User acceptance testing

---

## Success Criteria

- ✅ Plugin loads without errors in Univer environment
- ✅ Can enter uncertainty values using ± notation
- ✅ Basic arithmetic operations propagate uncertainty automatically
- ✅ Visual indicators clearly show uncertainty cells
- ✅ Correlation matrix editor allows defining relationships
- ✅ Correlated propagation reduces uncertainty appropriately
- ✅ Coexists with existing manual uncertainty sidebar
- ✅ Data can be exported/imported with uncertainty metadata
- ✅ Performance acceptable for spreadsheets with 1000+ cells

---

## Benefits Over Cell-Based Approach

**Plugin Advantages**:
- **Architecturally Sound**: Works within Univer's extension model
- **Maintainable**: Clean separation, no core modifications
- **Future-Proof**: Compatible with Univer updates
- **Flexible**: Can be enabled/disabled per spreadsheet
- **Extensible**: Easy to add new features like correlations

**User Experience**:
- **Seamless**: Works like native spreadsheet functionality
- **Powerful**: Supports complex correlations and propagation
- **Compatible**: Works with existing workflows and tools
- **Educational**: Can show propagation formulas when requested

---

## Conclusion

The plugin approach transforms a technical limitation into an architectural advantage. By working within Univer's extension framework, we achieve automatic uncertainty propagation that is:

- **More maintainable** than core modifications
- **More robust** than cell-hacking approaches  
- **More extensible** for future enhancements
- **More user-friendly** than manual sidebar workflows

This approach delivers the desired automatic uncertainty propagation while maintaining system integrity and providing a solid foundation for correlated uncertainty support.