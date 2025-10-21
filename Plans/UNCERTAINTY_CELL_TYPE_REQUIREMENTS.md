# Automatic Uncertainty Propagation in Cells - Feature Requirements

## Executive Summary

**Feature**: Automatic uncertainty propagation for spreadsheet cells with built-in uncertainty values.

**Goal**: When a user enters a formula that references cells containing uncertainty data, the result cell should automatically calculate and display the propagated uncertainty using symbolic differentiation.

**Status**: Design Phase
**Priority**: High

---

## Problem Statement

### Current Workflow (Manual)
```
1. User enters values with uncertainty in separate columns:
   A1: 5.0    B1: 0.1 (uncertainty)
   C1: 3.0    D1: 0.05 (uncertainty)

2. User enters formula: E1 = A1 + C1
   Result: 8.0

3. User must manually:
   - Open UncertaintySidebar
   - Define variables (a: A1:A1, B1:B1)
   - Enter formula: a + c
   - Specify output ranges
   - Click "Generate Formulas"
   - Get uncertainty formula: =SQRT(B1^2 + D1^2)
   - Manually enter in F1
```

**Problems**:
- ❌ 8+ manual steps
- ❌ Error-prone
- ❌ Doesn't scale to complex spreadsheets
- ❌ Uncertainty formulas not automatically updated when source formula changes

### Desired Workflow (Automatic)
```
1. User enters value with uncertainty:
   A1: 5.0 ± 0.1  (parsed as value + metadata)

2. User enters formula: C1 = A1 + B1
   Result: 8.0 ± 0.11  (automatically calculated)

3. Formula remains editable
4. Uncertainty updates automatically when dependencies change
```

**Benefits**:
- ✅ 1 step instead of 8+
- ✅ Automatic propagation
- ✅ Always up-to-date
- ✅ Scales to any spreadsheet size
- ✅ Professional scientific tool

---

## User Stories

### Story 1: Basic Uncertainty Input
**As a** scientist  
**I want to** enter a measurement with uncertainty directly in a cell  
**So that** I don't need separate columns for values and uncertainties

**Acceptance Criteria**:
- User can type `5.0 ± 0.1` or `5.0(1)` or `5.0 +/- 0.1`
- Cell displays the value with uncertainty notation
- Cell stores value (5.0) and uncertainty (0.1) separately
- Formula bar shows the original input
- Cell has visual indicator (e.g., blue border)

### Story 2: Automatic Propagation
**As a** scientist  
**I want** formulas to automatically propagate uncertainty  
**So that** I don't need to manually calculate uncertainty formulas

**Acceptance Criteria**:
- When formula references uncertainty cells, result automatically has uncertainty
- Uncertainty is calculated using proper propagation rules (symbolic differentiation)
- Propagation formula is stored for reference
- Works for any mathematical formula (addition, multiplication, functions, etc.)

### Story 3: Formula Editing
**As a** user  
**I want to** edit formulas normally  
**So that** uncertainty doesn't interfere with spreadsheet functionality

**Acceptance Criteria**:
- Double-clicking cell shows original formula in formula bar
- Editing formula works normally
- Uncertainty recalculates automatically when formula changes
- Undo/redo works correctly

### Story 4: Copy-Paste Preservation
**As a** user  
**I want** uncertainty to be preserved when copying cells  
**So that** I can reorganize my spreadsheet without losing data

**Acceptance Criteria**:
- Copying uncertainty cell preserves uncertainty metadata
- Pasting updates uncertainty if formula references change
- Cut operation removes uncertainty from source
- External paste (to Excel, etc.) shows notation like `5.0 ± 0.1`

### Story 5: Auto-Fill Propagation
**As a** user  
**I want** uncertainty to propagate when auto-filling  
**So that** I can quickly fill ranges with uncertainty data

**Acceptance Criteria**:
- Dragging fill handle propagates uncertainty
- Uncertainty formulas update with relative references
- Works for both values and formulas

---

## Technical Requirements

### 1. Data Storage

#### Cell Structure
```typescript
interface UncertaintyCellData extends ICellData {
  v: number                    // Value (e.g., 5.0)
  f?: string                   // Formula (e.g., "=A1+B1")
  custom: {
    cellType: 'uncertainty'
    uncertainty: number        // Uncertainty value (e.g., 0.1)
    confidence: number         // Confidence level (e.g., 95)
    unit?: string              // Optional unit (e.g., "m", "kg")
    propagationFormula?: string // Derived formula (e.g., "=SQRT(B1^2+D1^2)")
    sourceRefs?: string[]      // Referenced cells (e.g., ["A1", "B1"])
  }
  s?: IStyleData              // Visual indicator (blue border)
  note?: {                    // Tooltip with uncertainty info
    text: string
    visible: boolean
  }
}
```

#### Storage Location
- **Primary**: `cell.custom` field (Univer native support)
- **Backup**: Cell comment/note for external compatibility
- **Visual**: Cell style for border indicator

### 2. Input Parsing

#### Supported Formats
```
5.0 ± 0.1       → value: 5.0, uncertainty: 0.1
5.0 +/- 0.1     → value: 5.0, uncertainty: 0.1
5.0(1)          → value: 5.0, uncertainty: 0.1
5.0 ± 2%        → value: 5.0, uncertainty: 0.1 (2% of 5.0)
```

#### Parser Implementation
```typescript
function parseUncertaintyInput(input: string): UncertaintyCellData | null {
  const patterns = [
    /^([\d.]+)\s*±\s*([\d.]+)$/,           // 5.0 ± 0.1
    /^([\d.]+)\s*\+\/-\s*([\d.]+)$/,       // 5.0 +/- 0.1
    /^([\d.]+)\(([\d.]+)\)$/,              // 5.0(1) → 5.0 ± 0.1
    /^([\d.]+)\s*±\s*([\d.]+)%$/,          // 5.0 ± 2%
  ]
  
  for (const pattern of patterns) {
    const match = input.match(pattern)
    if (match) {
      const value = parseFloat(match[1])
      let uncertainty = parseFloat(match[2])
      
      // Handle percentage
      if (input.includes('%')) {
        uncertainty = value * (uncertainty / 100)
      }
      
      // Handle compact notation 5.0(1) → 5.0 ± 0.1
      if (input.includes('(') && !input.includes('.', input.indexOf('('))) {
        const decimals = (match[1].split('.')[1] || '').length
        uncertainty = uncertainty / Math.pow(10, decimals)
      }
      
      return {
        v: value,
        custom: {
          cellType: 'uncertainty',
          uncertainty,
          confidence: 95
        }
      }
    }
  }
  
  return null
}
```

### 3. Formula Interception

#### Command Interception
```typescript
// Intercept ALL formula executions
commandService.onCommandExecuted((command) => {
  if (command.id === 'sheet.mutation.set-range-values') {
    const params = command.params
    
    // Check if formula references uncertainty cells
    if (params.cellValue?.f) {
      handleUncertaintyPropagation(params)
    }
  }
})
```

#### Propagation Logic
```typescript
async function handleUncertaintyPropagation(params: any): Promise<void> {
  const formula = params.cellValue.f
  
  // 1. Extract cell references (A1, B2, etc.)
  const refs = extractCellReferences(formula)
  
  // 2. Check which refs have uncertainty
  const uncertaintyRefs = refs.filter(ref => hasUncertainty(ref))
  
  if (uncertaintyRefs.length === 0) {
    // No uncertainty - remove any existing uncertainty metadata
    removeUncertaintyMetadata(params)
    return
  }
  
  // 3. Prepare variables for backend
  const variables = uncertaintyRefs.map(ref => ({
    name: ref,
    value: getCellValue(ref),
    uncertainty: getCellUncertainty(ref),
    confidence: 95
  }))
  
  // 4. Call Rust backend for symbolic differentiation
  const result = await invoke('generate_uncertainty_formulas', {
    formula: formula.substring(1),  // Remove '='
    variables
  })
  
  // 5. Inject uncertainty metadata (non-invasive)
  await injectUncertaintyMetadata(params, result)
}
```

### 4. Backend Integration

#### Existing Rust Function
```rust
// Already implemented in uncertainty_propagation.rs
pub fn generate_uncertainty_formulas(
    formula: String,
    variables: Vec<Variable>,
) -> Result<UncertaintyResult, String>
```

**Capabilities**:
- ✅ Symbolic differentiation
- ✅ Partial derivatives calculation
- ✅ Combined uncertainty formula generation
- ✅ Supports complex formulas

**No backend changes needed!**

### 5. Visual Indicators

#### Custom Number Format
```typescript
// Display as "5.00 ± 0.10" in cell
numberFormat: {
  pattern: `0.00 "±" ${uncertainty.toFixed(2)}`
}
```

#### Tooltip/Note
```typescript
note: {
  text: `Value: 5.0
Uncertainty: ±0.1
Confidence: 95%

Propagation Formula:
=SQRT(B1^2 + D1^2)`,
  visible: false  // Show on hover
}
```

### 6. Copy-Paste Hook

```typescript
interface UncertaintyClipboardHook extends ISheetClipboardHook {
  id: 'uncertainty-clipboard'
  priority: 10
  
  onBeforeCopy: (unitId, subUnitId, range) => {
    // Cache uncertainty data
    cacheUncertaintyData(range)
  }
  
  onCopyCellContent: (row, col) => {
    // Add uncertainty notation to clipboard
    const uncertainty = getCellUncertainty(row, col)
    if (uncertainty) {
      return `${value} ± ${uncertainty}`
    }
  }
  
  onPasteCells: (pasteFrom, pasteTo, data, payload) => {
    // Restore uncertainty metadata
    return {
      undos: generateUndoMutations(),
      redos: generateRedoMutations()
    }
  }
}
```

### 7. Auto-Fill Hook

```typescript
interface UncertaintyAutoFillHook extends ISheetAutoFillHook {
  id: 'uncertainty-autofill'
  priority: 10
  type: AutoFillHookType.Append
  
  onFillData: (location, direction, applyType) => {
    // Propagate uncertainty during auto-fill
    return {
      undos: generateUndoMutations(),
      redos: generateRedoMutations()
    }
  }
}
```

---

## Architecture

### Plugin Structure
```
@univerjs/uncertainty-cell-plugin/
├── src/
│   ├── plugin.ts                          # Main plugin entry
│   ├── controllers/
│   │   ├── uncertainty-cell.controller.ts # Command interception
│   │   ├── uncertainty-clipboard.controller.ts
│   │   └── uncertainty-autofill.controller.ts
│   ├── services/
│   │   ├── uncertainty-parser.service.ts  # Input parsing
│   │   ├── uncertainty-storage.service.ts # Data management
│   │   └── uncertainty-backend.service.ts # Rust backend calls
│   ├── commands/
│   │   ├── set-uncertainty.command.ts
│   │   └── remove-uncertainty.command.ts
│   └── types/
│       └── uncertainty-cell.interface.ts
└── package.json
```

### Data Flow
```
User Input: "5.0 ± 0.1"
    ↓
Parser Service → Detect uncertainty notation
    ↓
Set Cell Command → Store in cell.custom
    ↓
Visual Indicator → Blue border + tooltip
    ↓
User enters formula: "=A1+B1"
    ↓
Univer evaluates formula → v: 10.0, f: "=A1+B1"
    ↓
Command Interceptor → Detect formula execution
    ↓
Check references → A1 and B1 have uncertainty
    ↓
Backend Service → Call Rust symbolic differentiation
    ↓
Inject Metadata → Add uncertainty to result cell
    ↓
Result Cell: { v: 10.0, f: "=A1+B1", custom: { uncertainty: 0.141 } }
    ↓
Visual Update → Blue border + tooltip on result cell
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1)
**Goal**: Basic plugin structure and data storage

**Tasks**:
- [ ] Create plugin boilerplate
- [ ] Implement uncertainty parser service
- [ ] Implement uncertainty storage service
- [ ] Add command for setting uncertainty
- [ ] Add visual indicators
- [ ] Unit tests for parser

**Deliverable**: Can manually set uncertainty on cells with visual feedback

### Phase 2: Formula Interception (Week 1-2)
**Goal**: Automatic propagation for simple formulas

**Tasks**:
- [ ] Implement command interceptor
- [ ] Implement cell reference extraction
- [ ] Implement uncertainty detection
- [ ] Integrate with Rust backend
- [ ] Implement metadata injection
- [ ] Handle async timing issues
- [ ] Unit tests for interception

**Deliverable**: Formulas automatically propagate uncertainty

### Phase 3: Copy-Paste & Auto-Fill (Week 2-3)
**Goal**: Uncertainty preserved during operations

**Tasks**:
- [ ] Implement clipboard hook
- [ ] Implement auto-fill hook
- [ ] Handle cut vs copy operations
- [ ] Handle external paste (to Excel)
- [ ] Integration tests

**Deliverable**: Copy-paste and auto-fill work correctly

### Phase 4: UI/UX Polish (Week 3-4)
**Goal**: Professional user experience

**Tasks**:
- [ ] Improve visual indicators
- [ ] Add tooltips with propagation info
- [ ] Add context menu items
- [ ] Add keyboard shortcuts
- [ ] Improve error messages
- [ ] Add user documentation
- [ ] Performance optimization

**Deliverable**: Production-ready feature

### Phase 5: Advanced Features (Week 4+)
**Goal**: Power user features

**Tasks**:
- [ ] Custom functions (UVALUE, UUNCERTAINTY, UMAKE)
- [ ] Uncertainty sidebar integration
- [ ] Batch operations
- [ ] Export with uncertainty notation
- [ ] Import from other formats
- [ ] Uncertainty visualization

**Deliverable**: Complete uncertainty system

---

## Technical Challenges & Solutions

### Challenge 1: Async Timing
**Problem**: Backend call is async, but command already completed  
**Solution**: Execute follow-up mutation to inject metadata  
**Risk**: Low - Univer's command system supports this

### Challenge 2: Formula Retention
**Problem**: Need to keep formula editable  
**Solution**: Only modify `custom` field, never touch `v` or `f`  
**Risk**: None - This is the correct approach

### Challenge 3: Performance
**Problem**: Backend call for every formula could be slow  
**Solution**: 
- Cache propagation formulas
- Batch process multiple cells
- Only recalculate when dependencies change  
**Risk**: Medium - Needs testing with large spreadsheets

### Challenge 4: Circular Dependencies
**Problem**: Uncertainty propagation might create cycles  
**Solution**: Track dependency graph, detect cycles, show error  
**Risk**: Low - Univer already handles formula cycles

### Challenge 5: External Compatibility
**Problem**: Excel doesn't understand uncertainty metadata  
**Solution**: Use clipboard hook to export as text notation  
**Risk**: Low - Copy-paste hook system is proven

---

## Performance Considerations

### Optimization Strategies

1. **Caching**
   - Cache parsed uncertainty data
   - Cache propagation formulas
   - Cache cell reference lookups

2. **Batching**
   - Batch multiple backend calls
   - Batch metadata injections
   - Debounce rapid changes

3. **Lazy Evaluation**
   - Only calculate uncertainty when needed
   - Skip cells without uncertainty
   - Skip non-formula cells

4. **Incremental Updates**
   - Only recalculate changed cells
   - Track dependency graph
   - Avoid redundant calculations


---

## User Documentation

### Quick Start Guide
```markdown
# Uncertainty Cells - Quick Start

## Entering Uncertainty
Type any of these formats:
- `5.0 ± 0.1`
- `5.0 +/- 0.1`
- `5.0(1)` (compact notation)
- `5.0 ± 2%` (percentage)

## Automatic Propagation
Just use formulas normally:
- `=A1+B1` → Automatically calculates uncertainty
- `=A1*B1` → Propagates correctly
- `=SIN(A1)` → Works with functions

## Visual Indicators
- Blue border = Cell has uncertainty
- Hover for tooltip with details
- Formula bar shows original formula

## Tips
- Copy-paste preserves uncertainty
- Auto-fill propagates uncertainty
- Undo/redo works normally
```

---

## Risks & Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Performance issues with large spreadsheets | Medium | High | Implement caching and batching |
| Univer API changes breaking plugin | Low | High | Pin Univer version, monitor releases |
| User confusion about notation | Medium | Medium | Clear documentation, tooltips |
| Backend errors breaking spreadsheet | Low | High | Graceful error handling, fallback |
| Circular dependency issues | Low | Medium | Detect and show clear error |

---

## Future Enhancements

### Phase 2 Features
- [ ] Correlation between variables
- [ ] Monte Carlo simulation
- [ ] Uncertainty budget analysis
- [ ] Sensitivity analysis
- [ ] Uncertainty visualization (charts)

### Phase 3 Features
- [ ] Multi-dimensional uncertainty
- [ ] Bayesian uncertainty
- [ ] Machine learning uncertainty

---

## Dependencies

### Required
- ✅ Univer Core (`@univerjs/core`)
- ✅ Univer Sheets (`@univerjs/sheets`)
- ✅ Univer Sheets UI (`@univerjs/sheets-ui`)
- ✅ Rust backend (already implemented)
- ✅ Tauri (for backend calls)

### Optional
- ⚠️ Univer Formula Engine (for custom functions)
- ⚠️ Univer Number Format (for custom display)

---

## Conclusion

**Key Strengths**:
- Univer provides all necessary extension points
- Backend uncertainty propagation already exists
- Non-invasive approach preserves formula editing
- Hook systems are proven and reliable

**Key Challenges**:
- Async timing requires careful handling
- Performance optimization needed for large spreadsheets
- Visual display requires creative solutions

