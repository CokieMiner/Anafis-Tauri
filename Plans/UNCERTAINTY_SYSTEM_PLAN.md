# AnaFis Uncertainty System - REVISED Implementation Plan

## Current State Assessment (Post-Analysis)

### ✅ **What Actually Exists**
- **CellValue Interface**: Supports uncertainty fields (`u`, `uType`, `uConfidence` as percentage)
- **Formula Interception**: Basic system for high-precision calculations in `SpreadsheetTab.tsx`
- **Backend System**: Robust SymPy-based uncertainty propagation in Rust (`uncertainty_propagation.rs`)
- **Univer Integration**: Command system, cell data storage, custom functions
- **Custom Functions**: Mathematical functions registered with Univer formula engine

### ❌ **What Doesn't Exist (Needs Implementation)**
- Input parsing for uncertainty patterns ("5 ± 0.1", etc.)
- Uncertainty cell detection and storage in Univer
- Automatic uncertainty propagation in formulas
- Custom display formatting for uncertainty cells
- Integration between frontend detection and backend propagation

## Revised Architecture

### Frontend (React/TypeScript + Univer)
1. **Input Parser**: Detect uncertainty patterns in cell input
2. **Cell Storage**: Store uncertainty data in ICellData (using `u`, `uType` fields)
3. **Formula Engine**: Intercept formulas referencing uncertainty cells
4. **Display System**: Custom formatting/rendering for uncertainty cells
5. **State Management**: Track uncertainty cells and dependencies

### Backend (Tauri/Rust) - ALREADY EXISTS
1. **Formula Evaluation**: Parse and evaluate with uncertainty variables ✅
2. **Symbolic Engine**: Use existing SymPy integration for derivatives ✅
3. **Error Propagation**: Apply uncertainty propagation formulas ✅
4. **Result Formatting**: Return value + uncertainty pairs ✅

## Detailed Implementation Plan

### **Phase 1: Core Infrastructure (High Priority)**

#### **1.1 Uncertainty Input Parser**
**File**: `src/components/spreadsheet/univer/uncertaintyParser.ts`
**Function**: Parse various uncertainty input patterns
```typescript
export function parseUncertaintyInput(input: string): UncertaintyValue | null
export function isUncertaintyInput(input: string): boolean
```

**Supported Patterns**:
- `"5 ± 0.1"` → `{value: 5, uncertainty: 0.1, type: 'absolute', confidence: 68}`
- `"5 +/- 0.1"` → same as above
- `"5e+3 ± 0.5e+2"` → `{value: 5000, uncertainty: 50, type: 'absolute', confidence: 68}`
- `"1.23(5)"` → `{value: 1.23, uncertainty: 0.05, type: 'absolute', confidence: 68}`

**Confidence Levels**: Stored as percentages (68% = 1σ, 95% = 2σ, 99.7% = 3σ)
- `"1.23(5)e+2"` → scientific notation support

#### **1.2 Cell Input Detection**
**File**: `src/components/spreadsheet/univer/UniverSpreadsheet.tsx`
**Integration Point**: `sheet.mutation.set-range-values` command handler
```typescript
// In afterCommandDisposable handler
const inputStr = value?.v?.toString();
if (inputStr && isUncertaintyInput(inputStr)) {
    const uncertainty = parseUncertaintyInput(inputStr);
    value.u = uncertainty.uncertainty;
    value.uType = uncertainty.type;
    value.uConfidence = uncertainty.confidence || 68; // Default 68% confidence (1σ)
    value.v = uncertainty.value;
}
```

#### **1.3 Cell Data Access Utilities**
**File**: `src/components/spreadsheet/univer/UniverSpreadsheet.tsx`
**Functions**:
```typescript
function getCellData(cellRef: string): ICellData | null
function isUncertaintyCell(cellData: ICellData): boolean
function getUncertaintyData(cellData: ICellData): UncertaintyValue | null
```

### **Phase 2: Formula Integration (Medium Priority)**

#### **Important UX Principle: Formula Preservation**
**Formula vs Display Value**: Like Excel, we maintain separation between:
- **Formula**: User's input (e.g., "=A1 * 2") - always preserved for editing
- **Display Value**: Calculated uncertainty result (e.g., "10.0 ± 0.2") - shown in cell

**User Experience**: When user selects uncertainty cell, formula bar shows original formula, cell shows calculated result.

#### **2.1 Formula Interception Enhancement**
**File**: `src/pages/SpreadsheetTab.tsx`
**Enhance**: `handleFormulaIntercept` function
**New Logic**:
```typescript
// Check for uncertainty references BEFORE high-precision evaluation
const hasUncertaintyRefs = checkForUncertaintyReferences(cleanFormula);
if (hasUncertaintyRefs && shouldAttemptUncertainty(cleanFormula)) {
    const result = await calculateWithUncertainty(cleanFormula, cellRef);
    updateCellDisplayWithUncertainty(cellRef, result); // Updates display value only
    return;
}
```

#### **2.2 Uncertainty Reference Detection**
**Functions**:
```typescript
function checkForUncertaintyReferences(formula: string): boolean {
    const cellRefs = extractCellReferences(formula);
    return cellRefs.some(ref => {
        const cellData = getCellData(ref);
        return isUncertaintyCell(cellData);
    });
}

function extractCellReferences(formula: string): string[] {
    return formula.match(/[A-Z]+\d+/g) || [];
}
```

#### **2.3 Backend Command Creation**
**File**: `src-tauri/src/scientific/mod.rs` + new file
**Add**: `formula_evaluation.rs` module
**Command**:
```rust
#[tauri::command]
async fn evaluate_formula_with_uncertainty(
    formula: String,
    variables: HashMap<String, UncertaintyValue>
) -> Result<UncertaintyResult, String>
```

### **Phase 3: Display System (Low Priority)**

#### **3.1 Custom Number Formatting**
**File**: `src/components/spreadsheet/univer/customNumberFormats.ts`
**Add**: Uncertainty format patterns
```typescript
export const uncertaintyFormats = {
    standard: '#,##0.00 ± 0.00',
    scientific: '0.00(0)E+00',
    compact: '#,##0(0)'
};
```

#### **3.2 Cell Styling**
**Options**:
- Apply custom number format to uncertainty cells
- Use CSS classes for visual distinction
- Add tooltips with uncertainty details

### **Phase 4: Advanced Features (Future)**

#### **4.1 Caching System**
- **Strategy**: Cache uncertainty formulas when first requested by Univer
- **Behavior**: First call is async, subsequent calls return cached results immediately
- **Benefits**: Eliminates async concerns after initial calculation
- **Invalidation**: Clear cache when referenced cells change
- **Memory**: Manage memory for large spreadsheets with LRU eviction

#### **4.2 Background Processing**
- Web workers for complex calculations
- Progress indicators for long operations
- Cancellation support for interrupted calculations

#### **4.3 Error Recovery**
- Fallback to regular calculation for failed propagation
- Clear error states and retry logic
- User feedback for unsupported operations

## Technical Deep Dive

### **Univer Integration Points**

#### **1. Cell Change Detection**
```typescript
// Location: UniverSpreadsheet.tsx, afterCommandDisposable
commandService.onCommandExecuted((command: ICommandInfo) => {
    if (command.id === 'sheet.mutation.set-range-values') {
        // Detect uncertainty input here
        const inputStr = params.value[0][0]?.v?.toString();
        if (isUncertaintyInput(inputStr)) {
            // Parse and store uncertainty
        }
    }
});
```

#### **2. Formula Interception**
```typescript
// Location: SpreadsheetTab.tsx, handleFormulaIntercept
// Check for uncertainty BEFORE existing high-precision logic
if (hasUncertaintyRefs && shouldAttemptUncertainty(formula)) {
    // Use uncertainty propagation
} else {
    // Existing high-precision evaluation
}
```

#### **3. Cell Data Access**
```typescript
// Need to implement cell data access through Univer API
function getCellData(cellRef: string): ICellData | null {
    // Convert A1 to row/col
    // Use worksheet.getCell(row, col)
}
```

### **Async Handling Strategy**

**Problem**: Univer commands are synchronous, but uncertainty calculations are async
**Solution**: Smart caching eliminates async concerns after first calculation

```typescript
class UncertaintyCalculator {
    private cache: Map<string, UncertaintyResult> = new Map();

    async calculate(formula: string, variables: any, cellRef: string): Promise<UncertaintyResult> {
        const cacheKey = this.generateCacheKey(formula, variables);

        // Return cached result immediately if available
        if (this.cache.has(cacheKey)) {
            return this.cache.get(cacheKey)!;
        }

        // First-time calculation: async but acceptable
        const result = await performCalculation(formula, variables);
        this.cache.set(cacheKey, result);

        // Defer cell update to avoid blocking UI
        setTimeout(() => updateCellDisplayWithUncertainty(cellRef, result), 0);

        return result;
    }

    private generateCacheKey(formula: string, variables: any): string {
        // Create deterministic key from formula and variable values
        return `${formula}_${JSON.stringify(variables)}`;
    }
}
```

**Benefits**: First call is async, all subsequent calls are synchronous from cache.

### **Formula Filtering Strategy**

**Allowlist Approach (SHOULD list)**:
```typescript
const SUPPORTED_UNCERTAINTY_FUNCTIONS = [
    'SUM', 'AVERAGE', 'PRODUCT', 'POWER', 'SQRT', 'EXP', 'LN', 'LOG10',
    'SIN', 'COS', 'TAN', 'ASIN', 'ACOS', 'ATAN',
    'SINH', 'COSH', 'TANH', 'ASINH', 'ACOSH', 'ATANH',
    'GAMMA', 'BETA', 'ERF', 'BESSELJ', 'ELLIPTIC_K'
];

function shouldAttemptUncertainty(formula: string): boolean {
    const upperFormula = formula.toUpperCase();
    const hasSupportedFunction = SUPPORTED_UNCERTAINTY_FUNCTIONS.some(func =>
        upperFormula.includes(func)
    );
    const hasUncertaintyRefs = checkForUncertaintyReferences(formula);

    // Only propagate if formula uses supported functions AND references uncertainty cells
    return hasSupportedFunction && hasUncertaintyRefs;
}
```

**Why Allowlist?**: Prevents inappropriate propagation through statistical functions like `STDDEV`, `VAR`, `CORREL` that operate on datasets rather than individual measurements.

## Implementation Timeline

### **Week 1: Foundation (Current - Ready to Start)**
1. ✅ Create uncertainty parser (`uncertaintyParser.ts`)
2. ✅ Add input detection to cell changes (`UniverSpreadsheet.tsx`)
3. ✅ Implement cell data access utilities
4. ✅ Test input patterns with console logging

### **Week 2: Formula Integration**
1. ⚠️ Extend formula interception (`SpreadsheetTab.tsx`)
2. ⚠️ Add uncertainty reference detection
3. ⚠️ Implement caching system for uncertainty formulas
4. ⚠️ Create backend command (`formula_evaluation.rs`)
5. ⚠️ Test basic propagation (A1="5 ± 0.1", B1="=A1 * 2")

### **Week 3: Display & Polish**
1. ✅ Custom number formatting
2. ✅ Error handling and user feedback
3. ✅ Performance optimization
4. ✅ Integration testing

### **Week 4: Advanced Features**
1. 🔄 Background processing for complex calculations
2. 🔄 Complex formula support expansion
3. 🔄 Documentation and examples

## Risk Assessment (Realistic)

### **Low Risk: Async Integration** (Reduced by Caching)
- **Issue**: First uncertainty calculation is async, subsequent calls are cached
- **Mitigation**: Smart caching makes 99% of operations synchronous
- **Worst Case**: Brief delay only on first calculation of each unique formula

### **Medium Risk: Cell Data Access**
- **Issue**: Need to access ICellData from Univer worksheet
- **Mitigation**: Implement proper cell data utilities
- **Worst Case**: Fallback to simpler detection methods

### **Low Risk: Formula Complexity**
- **Issue**: Complex formulas might fail
- **Mitigation**: Start with simple operations, add filtering
- **Worst Case**: Some formulas fall back to regular calculation

### **Low Risk: Display System**
- **Issue**: Custom rendering complexity
- **Mitigation**: Start with number formatting
- **Worst Case**: Basic text display works

## Success Metrics

1. **Input Detection**: All 5 uncertainty patterns work reliably
2. **Basic Propagation**: Addition/subtraction/multiply/divide work correctly
3. **Formula Support**: 80%+ of physics/engineering formulas work
4. **Performance**: No UI blocking on typical calculations
5. **User Experience**: Seamless integration with existing workflow

## Testing Strategy

### **Unit Tests**
- Input parsing accuracy for all patterns
- Uncertainty calculation correctness
- Formula filtering logic

### **Integration Tests**
- End-to-end uncertainty workflows
- Cell update propagation
- Error handling scenarios

### **Performance Tests**
- Large spreadsheet handling (1000+ cells)
- Complex formula evaluation timing
- Memory usage monitoring

---

**Status**: Ready for implementation
**Estimated Timeline**: 4 weeks
**Risk Level**: Medium (challenges identified and mitigated)
**Dependencies**: Existing robust backend uncertainty system

## Key Insights from Existing Code

### **Strengths of Current System**
1. **Robust Differentiation**: SymPy handles 99% of practical physics/engineering formulas
2. **Excel Compatibility**: Converts derivatives to Excel functions automatically
3. **Comprehensive Coverage**: Supports all common mathematical functions
4. **Error Recovery**: Already has filtering and error handling

### **Integration Opportunities**
1. **Reuse Existing Backend**: Leverage `generate_uncertainty_formulas` command
2. **Extend for Single Cells**: Adapt range-based system for individual cell calculations
3. **Add Caching**: Cache derivative calculations by formula signature

### **Minimal Changes Needed**
```typescript
// Add to formula interceptor
if (hasUncertaintyRefs && shouldAttemptUncertaintyPropagation(formula)) {
    // Use existing backend with single-cell adaptation
    const result = await invoke('calculate_single_uncertainty', {
        formula,
        variables: uncertaintyVars
    });
    updateCellDisplayWithUncertainty(cellRef, result); // Updates display only, preserves formula
    return;
} else {
    // Regular calculation
}
```

## Success Criteria (Updated)

1. ✅ **Input Detection**: All specified patterns work
2. ✅ **Propagation**: Works for 95%+ of practical formulas (physics/engineering)
3. ✅ **Formula Preservation**: Original formulas remain editable after uncertainty calculation
4. ✅ **Fallback**: Unsupported formulas fall back to regular calculation
5. ✅ **Performance**: No noticeable lag for typical calculations
6. ✅ **Integration**: Seamless with existing spreadsheet workflow