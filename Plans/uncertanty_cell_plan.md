# Automatic Uncertainty Propagation - User Experience Design

## Executive Summary

**Status**: ⚠️ **PLAN DEPRECATED - NOT VIABLE**

After deep architectural analysis of Univer's internals, automatic uncertainty propagation is **NOT FEASIBLE** due to fundamental architectural constraints.

---

## Why This Plan Was Removed

**Original Goal**: When a user enters a formula that references cells containing uncertainty data, the result cell should automatically calculate and display the propagated uncertainty.

**Why It Can't Work**: Univer's architecture fundamentally conflicts with this approach.

---

## Historical Reference: Original User Experience Design

The sections below document how users would have interacted with automatic uncertainty propagation. 
**DO NOT IMPLEMENT** - kept only for historical reference and to inform future feature design.

### Current Implementation (Manual Sidebar - WORKS)
**File**: `AnaFis/src/components/sidebars/UncertaintySidebar.tsx` (863 lines)

```
1. Double-click E1
2. Change =A1+C1 to =A1*C1
3. Press Enter
4. Open Uncertainty Sidebar again
5. Regenerate uncertainty formula
6. F1 updated with new propagation
```

**Automatic Method Would Have Been**:
```
1. Double-click E1
2. Change =A1+C1 to =A1*C1
3. Press Enter
4. E1 automatically updates: "15.0 ± 0.35"
5. Uncertainty recalculated automatically
```

**What User Sees**:
- Normal formula editing experience
- Uncertainty updates without manual intervention
- Can undo/redo as normal
- Blue border remains on cell

---

#### Story 4: Copy and Paste

**Scenario**: User wants to reorganize spreadsheet

**Current Manual Method** (Works):
```
1. Select A1:B1 (value and uncertainty)
2. Copy
3. Paste to D1:E1
4. Both value and uncertainty copied
5. Formulas maintain references correctly
```

**Automatic Method Would Have Been**:
```
1. Select A1 (shows "5.0 ± 0.1")
2. Copy
3. Paste to D1
4. D1 shows "5.0 ± 0.1" with blue border
5. Uncertainty preserved in single cell

OR paste to Excel:
1. Copy A1
2. Paste to Excel
3. Excel shows: "5.0 ± 0.1" (as text)
```

**What User Sees**:
- Single cell contains both value and uncertainty
- Copy-paste works as expected
- External apps see formatted text
- No loss of data

---

#### Story 5: Auto-Fill

**Scenario**: User wants to fill down a series

**Current Manual Method** (Works):
```
1. A1: 5.0, B1: 0.1
2. A2: Formula =A1+1, B2: =B1
3. Select A1:B2
4. Drag fill handle down
5. Series fills with values and uncertainties
```

**Automatic Method Would Have Been**:
```
1. A1: Type "5.0 ± 0.1"
2. A2: Type "=A1+1"
3. Select A2
4. Drag fill handle down
5. Each cell shows updated value ± 0.1
6. Blue borders indicate uncertainty preserved
```

**What User Sees**:
- Drag fill handle as normal
- Uncertainty propagates down the series
- Each cell maintains uncertainty
- Formulas update with relative references

---

### Visual Design

#### Cell Appearance

**Standard Cell**:
```
┌─────────┐
│  5.0    │  ← Normal cell, no indicator
└─────────┘
```

**Uncertainty Cell (Manual Method)**:
```
┌─────────┬─────────┐
│  5.0    │  0.1    │  ← Two cells needed
└─────────┴─────────┘
```

**Uncertainty Cell (Would Have Been)**:
```
┏━━━━━━━━━┓
┃ 5.0±0.1 ┃  ← Blue border indicates uncertainty
┗━━━━━━━━━┛
```

---

#### Tooltip Display

**Hovering over uncertainty cell would have shown**:
```
┌────────────────────────────────┐
│ Value: 5.0                     │
│ Uncertainty: ±0.1              │
│ Confidence: 95%                │
│ Unit: m                        │
│                                │
│ [View Details]                 │
└────────────────────────────────┘
```

**Hovering over calculated cell would have shown**:
```
┌────────────────────────────────┐
│ Result: 8.0 ± 0.11             │
│ Confidence: 95%                │
│                                │
│ Propagation Formula:           │
│ √(0.1² + 0.05²)                │
│                                │
│ Source Cells:                  │
│ • A1: 5.0 ± 0.1                │
│ • C1: 3.0 ± 0.05               │
│                                │
│ [View Details] [Edit Sources]  │
└────────────────────────────────┘
```

---

#### Formula Bar Display

**Would have shown**:
```
When cell E1 selected (shows "8.0 ± 0.11"):
Formula bar: =A1+C1

When editing E1:
Formula bar: =A1+C1  (cursor blinking, normal editing)

No difference from regular formulas - uncertainty is invisible metadata
```

---

### Input Methods

#### Supported Notation Formats

**Would have accepted any of these formats**:

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
```

**What user types** → **What appears** → **What's stored**:
- `5.0 ± 0.1` → "5.0 ± 0.1" → Value: 5.0, Uncertainty: 0.1
- `5.0(1)` → "5.0 ± 0.1" → Value: 5.0, Uncertainty: 0.1
- `5.0 ± 2%` → "5.0 ± 0.1" → Value: 5.0, Uncertainty: 0.1
- `5.0` → "5.0" → Value: 5.0, No uncertainty

---

### Settings & Configuration

#### Enable/Disable Feature

**Would have had global setting**:
```
Settings → Spreadsheet → Uncertainty Propagation
☐ Enable automatic uncertainty propagation

When enabled:
- Cells can store uncertainty data
- Formulas automatically propagate uncertainty
- Visual indicators shown

When disabled:
- Feature completely inactive
- Falls back to manual sidebar workflow
- No performance impact
```

**Why Toggle Needed**:
- Some users prefer explicit manual control
- Learning curve for new users
- Compatibility with existing workflows
- Option to disable if performance issues

---

#### Display Options

**Would have offered customization**:
```
Settings → Spreadsheet → Uncertainty Display

Display Format:
○ Compact (5.0 ± 0.1)
○ Percentage (5.0 ± 2%)
○ Scientific (5.0e0 ± 1.0e-1)
○ Custom format: _______

Visual Indicators:
☑ Show blue border on uncertainty cells
☑ Show tooltip on hover
☐ Show in formula bar
☑ Show in status bar

Confidence Level (default):
[95] % (Standard: 95%, can be 68%, 99%)
```

---

### Context Menu Actions

**Would have added right-click options**:

**On uncertainty cell**:
```
┌────────────────────────────────┐
│ Cut                            │
│ Copy                           │
│ Paste                          │
├────────────────────────────────┤
│ ▶ Uncertainty                 │
│   ├ View Details               │  ← Opens detailed view
│   ├ Edit Uncertainty           │  ← Change value
│   ├ Remove Uncertainty         │  ← Keep value, remove ±
│   └ Copy as Text               │  ← Copy "5.0 ± 0.1"
├────────────────────────────────┤
│ Format Cell                    │
│ Insert Comment                 │
└────────────────────────────────┘
```

**On calculated cell with uncertainty**:
```
┌─────────────────────────────────┐
│ ▶ Uncertainty                  │
│   ├ View Propagation            │  ← See how calculated
│   ├ View Source Cells           │  ← Highlight A1, C1
│   ├ Copy Result with Uncertainty│ ← Copy "8.0 ± 0.11"
│   └ Export Uncertainty Report   │  ← Detailed breakdown
└─────────────────────────────────┘
```

---

### Keyboard Shortcuts

**Would have provided quick access**:

```
Alt+U          → Toggle uncertainty mode (type ± easily)
Ctrl+Shift+U   → Add/Edit uncertainty for selected cell
Alt+Shift+U    → View uncertainty details
Ctrl+Alt+U     → Remove uncertainty from selection

In uncertainty mode (Alt+U):
Type: 5.0 + 0.1 → Automatically converts to "5.0 ± 0.1"
```

---

### Status Bar Integration

**Would have shown at bottom of spreadsheet**:

```
When no selection:
Ready | Sheet1 | Uncertainty: 15 cells

When cell with uncertainty selected:
A1: 5.0 ± 0.1 (95%) | Sheet1

When range with uncertainties selected:
Selection: 10 cells (5 with uncertainty) | Average: 5.2 ± 0.08

When calculated cell selected:
E1: 8.0 ± 0.11 (propagated from 2 sources) | Sheet1
```

---

### Error Handling & User Feedback

#### Invalid Input

**User types**: `5.0 ± abc`
**Behavior**: 
- Cell shows error indicator (red border)
- Tooltip: "Invalid uncertainty format. Expected number, got 'abc'"
- Formula bar shows original input
- User can edit to fix

---

#### Missing Uncertainty in Formula

**Scenario**: `=A1+B1` where A1 has uncertainty but B1 doesn't

**Behavior**:
- Result shows uncertainty only from A1
- Tooltip: "Partial uncertainty: only A1 has uncertainty data"
- Warning icon in cell corner
- Can click to add uncertainty to B1

---

#### Circular Dependency

**Scenario**: A1 references B1, B1 references A1

**Behavior**:
- Shows same circular reference error as normal Univer
- Tooltip: "Circular dependency detected"
- Uncertainty calculation skipped
- User must fix circular reference first

---

#### Backend Calculation Error

**Scenario**: Rust backend fails (complex formula not supported)

**Behavior**:
- Cell shows value without uncertainty
- Yellow warning border
- Tooltip: "Could not calculate uncertainty propagation. Formula may be too complex."
- Option to manually specify uncertainty
- Link to open manual sidebar for this calculation

---

### Integration with Manual Sidebar

**Both systems would have worked together**:

#### Scenario 1: Manual → Automatic
```
1. User creates formula with manual sidebar
2. Formula: =A1+C1 (value)
3. Formula: =SQRT(B1^2+D1^2) (uncertainty)
4. User enables automatic propagation
5. Edits formula: =A1+C1 becomes =A1*C1
6. Automatic system recalculates uncertainty
7. Both formulas updated automatically
```

#### Scenario 2: Automatic → Manual
```
1. User has automatic uncertainty cells
2. A1: 5.0 ± 0.1 (single cell)
3. Opens manual sidebar
4. Sidebar detects uncertainty in A1
5. Automatically fills variable fields
6. User can generate additional formulas
7. Both systems work on same data
```

#### Scenario 3: Mixed Workflow
```
1. Some cells use automatic (simple formulas)
2. Some cells use manual (complex formulas)
3. Both have blue borders
4. Both show tooltips
5. User chooses appropriate tool for each case
6. No conflicts, seamless coexistence
```

---

### Export & Import

#### Exporting Spreadsheet

**Would have offered options**:
```
File → Export → Excel

Dialog:
┌────────────────────────────────┐
│ Export Options                 │
├────────────────────────────────┤
│ Uncertainty Notation:          │
│ ○ Keep as formulas             │
│ ● Convert to text (5.0 ± 0.1)  │
│ ○ Separate columns (value, ±)  │
│                                │
│ ☑ Include uncertainty metadata │
│ ☑ Include propagation formulas │
│                                │
│ [Export] [Cancel]              │
└────────────────────────────────┘
```

**Result in Excel**:
```
Option 1 (formulas): Cell shows calculated value
Option 2 (text): Cell shows "5.0 ± 0.1" as text
Option 3 (columns): A1=5.0, B1=0.1, C1="±"
```

---

#### Importing from Excel

**Would have detected patterns**:
```
Excel file has:
A1: "5.0 ± 0.1"  (text)

AnaFis import detects pattern:
┌──────────────────────────────────┐
│ Import Uncertainty Data          │
├──────────────────────────────────┤
│ Detected 10 cells with ± notation│
│                                  │
│ ● Parse as uncertainty cells     │
│ ○ Import as plain text           │
│                                  │
│ Preview:                         │
│ A1: "5.0 ± 0.1" → 5.0 ± 0.1      │
│                                  │
│ [Import] [Cancel]                │
└──────────────────────────────────┘
```

---

### Help & Documentation

**Would have provided in-app help**:

#### Quick Help Tooltip
```
Hover over blue-bordered cell:
┌────────────────────────────────┐
│ This cell contains uncertainty │
│ data.                          │
│                                │
│ [Learn More] [Don't Show Again]│
└────────────────────────────────┘
```

#### First-Time User Guide
```
On first use:
┌────────────────────────────────────────┐
│ Welcome to Uncertainty Propagation!    │
├────────────────────────────────────────┤
│                                        │
│ 1. Enter values with ± notation        │
│    Example: 5.0 ± 0.1                  │
│                                        │
│ 2. Write formulas normally             │
│    Example: =A1+B1                     │
│                                        │
│ 3. Uncertainty calculates automatically│
│    Result: 8.0 ± 0.11                  │
│                                        │
│ [Try Example] [Skip Tutorial]          │
└────────────────────────────────────────┘
```

---

## Comparison: Manual vs Automatic

### Side-by-Side Workflow Comparison

| Step | Manual Sidebar (Current) | Automatic (Would Have Been) |
|------|-------------------------|----------------------------|
| **Enter Data** | A1: 5.0, B1: 0.1 (2 cells) | A1: 5.0 ± 0.1 (1 cell) |
| **Write Formula** | E1: =A1+C1 | E1: =A1+C1 |
| **Calculate Uncertainty** | Open sidebar, define vars, generate | Automatic |
| **View Result** | E1: 8.0, F1: 0.11 (2 cells) | E1: 8.0 ± 0.11 (1 cell) |
| **Edit Formula** | Change E1, regenerate F1 | Change E1, auto-updates |
| **Copy Result** | Copy E1:F1 (2 cells) | Copy E1 (1 cell) |
| **Export** | Two columns | One cell with notation |

---

### User Effort Comparison

**Task**: Add two measurements with uncertainty

**Manual Method** (Current):
```
Clicks: 8
- Enter A1 value (1)
- Enter B1 uncertainty (1)
- Enter C1 value (1)  
- Enter D1 uncertainty (1)
- Enter formula E1 (1)
- Open sidebar (1)
- Setup & generate (1)
- Close sidebar (1)

Time: ~30 seconds
Columns used: 4 (2 values + 2 uncertainties)
```

**Automatic Method** (Would Have Been):
```
Clicks: 3
- Enter A1: "5.0 ± 0.1" (1)
- Enter C1: "3.0 ± 0.05" (1)
- Enter formula E1: "=A1+C1" (1)

Time: ~5 seconds
Columns used: 2 (automatic uncertainty)
```

**Reduction**: 63% fewer clicks, 83% faster, 50% less space

---

### When to Use Each Method

#### Use Manual Sidebar When:
- ✅ Learning uncertainty propagation concepts
- ✅ Need to see the propagation formula explicitly
- ✅ Working with complex multi-variable formulas
- ✅ Need fine control over confidence levels
- ✅ Want to verify calculation steps
- ✅ Creating educational materials

#### Would Have Used Automatic When:
- ✅ Routine data analysis
- ✅ Many simple calculations
- ✅ Quick what-if scenarios
- ✅ Space-constrained spreadsheets
- ✅ Exporting to other applications
- ✅ Confident in uncertainty concepts

---

## Conclusion

### What Users Would Have Experienced

**Benefits**:
- ✅ **Seamless**: Works like normal spreadsheet
- ✅ **Automatic**: No sidebar needed for simple cases
- ✅ **Space-efficient**: One cell instead of two
- ✅ **Portable**: Copy-paste to Excel preserves notation
- ✅ **Fast**: No manual formula generation
- ✅ **Visual**: Clear indicators show uncertainty cells
- ✅ **Flexible**: Toggle on/off based on preference

**Trade-offs**:
- ⚠️ **Less explicit**: Calculation happens behind scenes
- ⚠️ **Learning curve**: Need to understand notation
- ⚠️ **Hidden formulas**: Propagation not visible in formula bar

**Best Approach Would Have Been**:
- **Beginners**: Start with manual sidebar (educational)
- **Power users**: Enable automatic (efficient)
- **Complex cases**: Use manual sidebar (control)
- **Simple cases**: Use automatic (speed)
- **Both available**: User chooses best tool for each task

---