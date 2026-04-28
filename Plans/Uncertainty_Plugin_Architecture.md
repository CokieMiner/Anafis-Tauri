# Univer In-Cell Uncertainty Plugin Architecture

This document serves as the comprehensive engineering blueprint for building "Value ± Error" natively into the Univer spreadsheet core.

## Core Philosophy
Spreadsheets are fundamentally designed to hold a single nominal number per cell. To support uncertainties seamlessly without breaking native math (`+`, `-`, `*`), we separate the **nominal value** from its **uncertainty metadata**. We leverage Univer's native Number Formatting engine to visually merge them on the screen, creating a magical "in-cell" experience while bypassing the need to write complex Canvas WebGL rendering extensions.

---

## 1. Data Storage & Rendering (The Dual-Value System)

### Data Model
- **Nominal Value:** Saved to the cell's standard `v` property.
- **Uncertainty Metadata:** Saved invisibly to the cell's `custom` metadata object. We use a clean, flattened format to support all permutations (including asymmetric percentages):
  ```typescript
  custom: {
    upperBound: number;
    lowerBound?: number;              // Only present for asymmetric errors
    upperType: 'abs' | 'rel';
    lowerType?: 'abs' | 'rel';        // Only present for asymmetric errors
    upperSource: 'manual' | 'propagated'; // Behavior on recalculation for upper bound
    lowerSource?: 'manual' | 'propagated';// Behavior on recalculation for lower bound
  }
  ```

### Rendering & Synchronization
- We dynamically generate a Univer Custom Number Format for the cell using `FRange.setNumberFormat()`. 
- **CRITICAL:** Every time `custom` is updated (via calculation, scaling, or manual input), the plugin MUST regenerate and apply the format string to prevent the visual display from desynchronizing with the data model.
- **GUM Rounding:** Before generating the format string, the plugin applies standard GUM rounding: the uncertainty is rounded to 1-2 significant figures, and the nominal value's display precision is truncated to match the uncertainty's decimal place (e.g., `5.123456` and `0.007` becomes `5.123 ± 0.007`).

### Undo/Redo Integration
- Both `v` and `custom` metadata must be updated within the same `Mutation` or `Command` execution context to ensure Univer's native Undo/Redo stack captures them as a single user action.

---

## 2. The Input Interceptor (Regex Parser & Write Hook)

We hook into Univer's command system (`ICommandService` listening for `SetRangeValuesCommand`) to catch scientific string formats before they are committed to the data model.

**Parsing Rules & Regex Examples:**
To avoid greedy matching and catastrophic backtracking, the nominal group strictly matches floating-point numbers: `[-+]?[\d\.eE]+(?:[eE][-+]?\d+)?`

1. **Standard Delimiters (`+-`, `+/-`, `-+`, `-/+`, `±`)**
   - *Example:* `5.0 +- 0.1`
   - *Behavior:* Captures nominal and sets `custom.upperBound = 0.1`, `upperType = 'abs'`, `upperSource = 'manual'`.
2. **Concise Scientific Notation (`()`)**
   - *Example:* `5.00(12)`
   - *Behavior:* Calculates absolute uncertainty based on decimal position. The `12` is aligned with the last digit, resulting in an uncertainty of `0.12`, `upperType = 'abs'`.
3. **Percentage Uncertainty**
   - *Example:* `5.0 +- 2%`
   - *Behavior:* Captures nominal and sets `custom.upperBound = 2`, `upperType = 'rel'`, `upperSource = 'manual'`.
4. **Asymmetric Bounds**
   - *Example:* `5.0 +0.1/-0.2` or `5.0 +1%/-2%`
   - *Behavior:* Sets `custom.upperBound = 0.1`, `custom.lowerBound = 0.2`, and maps `%` detection independently to `upperType` and `lowerType` (e.g., `'abs'` and `'rel'` respectively, though usually symmetric in type).

---

## 3. Dynamic Formula Wrapping & Recalculation

### The `=UNCERT()` Function
For dynamic, auto-updating uncertainty bounds, we register a custom function `=UNCERT(value, upperBound, [lowerBound])`.
- **Side-Effect Limitation:** Univer formula functions are pure and cannot directly write cell metadata during evaluation.
- **Solution:** `UNCERT()` returns only the nominal value. A `PostCalculationInterceptor` (or an event listener on the Formula Engine) detects cells utilizing `UNCERT` in their AST and explicitly dispatches a metadata update mutation afterwards.
- **Loop Prevention:** To prevent the metadata mutation from triggering an infinite recalculation loop, the plugin uses a lock/flag (`isWritingUncertainty = true`) during the metadata update.

### Standard Error Propagation (AST)
When the user types `=A1*B1`:
1. The formula engine computes the nominal math.
2. A calculation listener sends the AST to the Rust backend (Tauri IPC).
3. Rust calculates the propagated error using partial derivatives.
4. The result cell is updated with `custom.upperSource = 'propagated'` (and `lowerSource = 'propagated'` if asymmetric).

**Statistical Limitations & CAS Compatibility:** 
- **Correlated Inputs:** The propagation engine currently assumes statistical independence between variables. Formulas referencing the same cell multiple times (e.g., `=A1+A1`) or aggregations (`=SUM(A1:A10)`) where inputs are correlated will incorrectly sum errors in quadrature (underestimating the true error), currently without a warning to the user.
- **Asymmetric Multi-Variable Propagation:** The parallel upper/lower chains method may be mathematically incorrect for general multi-variable or non-monotonic functions, as the maximum deviation could occur at mixed combinations of bounds.
- **Non-Differentiable Functions:** Functions like `=IF()` or `=ABS()` lack a defined analytical derivative at their discontinuities. 
- **Fallback Behavior:** If an AST contains functions that are non-differentiable or generally incompatible with the Rust CAS (Computer Algebra System), the engine will not perform propagation (or will attempt to transform it into a compatible form).

### Manual Overrides on Formulas
To inject manual uncertainty into a formula, the `|` syntax (`=A1*B1 | +- 0.5`) may be rejected by Univer's initial formula parser. Instead, we provide the `=UNCERT_MANUAL(formula, upperBound, [lowerBound])` function. This explicitly sets `upperSource = 'manual'` (and `lowerSource = 'manual'` if a third argument is provided), ignoring automatic propagation from dependencies.

---

## 4. The Edit Lifecycle Interceptor (Seamless UX)

When a user double-clicks an uncertainty cell:
1. `SheetInterceptorService` intercepts the `BEFORE_CELL_EDIT` hook.
2. For simple values, the plugin reconstructs the string by appending the delimiter (e.g., `5 ± 0.1`).
3. For formulas with manual uncertainty, the plugin reconstructs the `=UNCERT_MANUAL(...)` wrapper to ensure valid syntax.
4. The user edits this string naturally in the text cursor box.
5. On Enter, the Input Interceptor (Step 2) catches the string (or formula parser catches the function) and commits the split values.

---

## 5. Confidence Scaling (Workflow 5)

A global UI Tool scales the numeric uncertainty column-wide via a Student's t-distribution calculator.
- **Direct Overwrite:** The scaling operation is a destructive edit that directly multiplies and overwrites the `upperBound` (and `lowerBound` if present) with the calculated t-factor. It functions similarly to a "Paste Special -> Multiply" action in traditional spreadsheets.
- **Reversion & Workflow Limitation:** If a user wishes to change the confidence level again (e.g., from 95% to 99%), they must rely on the native Undo (Ctrl+Z) functionality to revert to the 1-sigma baseline. Because Undo operates sequentially, applying confidence scaling should ideally be the **final operation before export**, as any intermediate data entry between scaling steps would be lost upon reverting.
- **Sync:** The format string is explicitly regenerated after scaling.

---

## 6. Export Serialization

When exporting the workbook out of the native `.anafispread` ecosystem (which is lossless for `custom` metadata):
- **CSV Export:** Exposes user-configurable options:
  - *Option A (Scientific standard):* Split into two columns `value` and `uncertainty` (easiest for Pandas/Numpy).
  - *Option B (Concise GUM):* Single column as `5.0(1)`.
  - *Option C (Literal String):* Single column as `5 ± 0.1`.
