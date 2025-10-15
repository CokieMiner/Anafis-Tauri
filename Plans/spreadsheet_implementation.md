 # Spreadsheet implementation plan — uncertainty-first, SymPy-for-derivatives

 Date: 2025-10-15
 Author: (generated)

 Executive summary
 -----------------
 - Goal: treat uncertainties as first-class data and provide automatic analytic uncertainty propagation using SymPy for symbolic derivatives and Rust for numeric evaluation.
 - Two complementary workflows are supported:
   1. "Separate-cell" workflow: UI generates Excel-style formulas (value and uncertainty in separate cells) and writes them into the sheet (current `UncertaintySidebar` + `generate_uncertainty_formulas`).
   2. "Same-cell" workflow: value and uncertainty are stored together (e.g. `{ v, u, f }`) and formulas are intercepted to compute and store `u` automatically (recommended for interactive editing via ReactDataGrid).
 - Short-term decision: migrate interactive editing to `react-data-grid` for custom renderers/editors; keep `Univer` integration for workbook fidelity and formula evaluation where needed.

 Quick decisions
 ---------------
 - Adapter: use `ReactDataGridAdapter` as primary interactive grid for features requiring custom renderers (`v ± u`), keep `UniverAdapter` as a fallback and for advanced workbook compatibility.
 - Evaluation: keep both strategies—emit sheet formulas for bulk propagation and also provide a numeric `evaluate_formula` path for single-cell inline propagation and Monte Carlo fallback.

 Table of contents
 -----------------
 - Executive summary
 - Quick decisions
 - How to use this document
 - Core contracts and data model
 - Propagation APIs and commands
 - Workflows
   - Separate-cell (sidebar) workflow
   - Same-cell (in-cell) auto-propagation workflow
 - Adapter & renderer strategy (ReactDataGrid vs Univer)
 - Formatting, styles, and conditional formatting (details below)
 - Implementation roadmap (prioritized)
 - Files to edit & concrete next steps
 - Tests & quality gates

 How to use this document
 ------------------------
 - Read the Executive summary and Quick decisions to get the short-form plan.
 - Use the Table of contents to jump to the detailed sections. The document is intentionally verbose below — the top-of-file is a compact entrypoint so you can scan quickly.

 (Detailed sections follow below.)

Cell formatting, styles and UI (rich formatting plan)
--------------------------------------------------
This section expands the spreadsheet implementation plan to cover rich cell formatting and UI features (font styling, colors, borders, number/date formats, rich text runs, conditional formatting, clipboard behavior, import/export, performance and tests). These features are complementary to the uncertainty-first spreadsheet plan above and reuse the same persistence and adapter wiring.

Goals
 - Treat formatting as first-class metadata attached to cells, but keep it sparse and deduplicable (named styles).
 - Support both simple cell-level styling (bold, italic, background) and advanced features (rich text runs, conditional formatting rules, number/date formats).
 - Provide a small, serializable, backwards-compatible data model that persists in workbook JSON.
 - Keep the renderer performant: deduplicate styles, use virtualization, and apply minimal DOM diffs.

Data model and types (summary)
 - Canonical cell persisted shape remains the single source of truth; extend it to include style and rich text.
 - Sheet-level metadata includes named styles and conditional formatting rule lists.
 - Conditional format rules are stored as small objects and can be evaluated by the UI or offloaded to Rust/worker for large ranges.

Suggested TypeScript interfaces (examples)

```ts
// Basic color / enumerations
type HexColor = `#${string}`; // e.g. '#RRGGBB' or '#RRGGBBAA'
type HAlign = 'left' | 'center' | 'right' | 'justify';
type VAlign = 'top' | 'middle' | 'bottom';

// Rich text run (within a single cell)
interface RichTextRun {
  text: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strike?: boolean;
  fontFamily?: string;
  fontSize?: number; // px or pt
  color?: HexColor;
  background?: HexColor;
  link?: string;
}

// Cell style: small, serializable
interface CellStyle {
  fontFamily?: string;
  fontSize?: number;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strike?: boolean;
  color?: HexColor;
  background?: HexColor;
  border?: {
    top?: BorderSpec;
    right?: BorderSpec;
    bottom?: BorderSpec;
    left?: BorderSpec;
  };
  horizontal?: HAlign;
  vertical?: VAlign;
  wrapText?: boolean;
  format?: string; // excel-like format string or internal key
  isMerged?: boolean;
  rotation?: number;
  namedStyle?: string; // reference to sheet/workbook style
}

interface BorderSpec { style?: 'none' | 'thin' | 'medium' | 'thick' | 'dashed' | 'dotted'; color?: HexColor }

interface ConditionalFormatRule {
  id?: string;
  range: string; // A1 notation or canonical range object
  priority?: number;
  stopIfTrue?: boolean;
  condition: {
    type: 'cellIs' | 'textContains' | 'dateIs' | 'customFormula';
    operator?: '>' | '<' | '>=' | '<=' | '=' | '<>';
    value?: string | number;
    formula?: string; // for custom formula
  };
  style: Partial<CellStyle>;
  enabled?: boolean;
}

interface NamedStyle { id: string; name: string; style: CellStyle; usageCount?: number }

// Extend existing CellValue
interface CellValue {
  v?: number | string | boolean | null;
  u?: number | null; // absolute uncertainty
  f?: string;
  meta?: Record<string, any>;
  style?: CellStyle;
  richText?: RichTextRun[];
  comment?: { author?: string; text: string; resolved?: boolean };
}
```

Persistence and migration
 - Persist `style` and `richText` at the cell level. Keep `meta` for compatibility; migrate legacy `meta.style` into `style` on load.
 - Store `namedStyles` and `conditionalFormatting` arrays at sheet-level in the workbook JSON. Keep these small and reference by `namedStyle` when possible.

UI / UX components
 - Formatting toolbar: bold, italic, underline, font family, font size, text color, fill color, alignment, wrap, number format dropdown, percent/currency buttons, border menu, merge cells, format painter.
 - Context menu and "Format cells" modal for advanced options.
 - Rich text inline editor for multi-style cells; for complex editing, open a modal.
 - Conditional formatting editor with interactive range selector and preview.

Rendering rules
 - For cells with `richText`, render runs (respecting run-level styles). For plain cells render `v` formatted according to `style.format`.
 - Uncertainty rendering: show `v ± u` when `u` is set. When `u === null` show a small `?` badge or configurable hint.
 - Map `CellStyle` to CSS with a small, stable mapping. Use CSS classes for common properties and inline styles for colors and sizes. Minimize DOM updates by diffing effective styles.

Runtime APIs and commands
 - Client-side actions: `applyCellStyle(range, stylePatch)`, `applyNamedStyle(range, namedStyleId)`, `clearFormats(range)`, `getEffectiveStyle(cellRef)`.
 - Tauri/Rust commands (suggested):
   - `apply_named_style` (batch apply)
   - `evaluate_conditional_formatting(sheetId, ranges?)` (optional offload)

Conditional formatting engine
 - Rules stored as `ConditionalFormatRule[]`. Evaluate incrementally on change and maintain reverse dependency mapping when rules use formula references.
 - Support `customFormula` rules using the same parser/evaluator as formulas.
 - Apply style patches non-destructively; provide priority and `stopIfTrue` semantics.

Clipboard and paste behavior
 - Copy payload includes `{ v, f, u, style, richText }` for each exported cell.
 - Paste special options: values, formats, formulas, uncertainties, values+formats.
 - Respect relative/absolute addressing when pasting formulas; support tiling behavior for mismatched ranges.
 - Format painter stores a `CellStyle` or `NamedStyle` and applies to selected range.

Import / Export
 - XLSX: map `CellStyle` <-> Excel style where possible. Persist uncertainties either in a custom property, comment, or separate metadata sheet if round-trip is required.
 - CSV/TSV: strip styles; export `v` and optionally `u` in adjacent columns or a `v ± u` string.

Performance & optimizations
 - Style deduplication via `namedStyles`; prefer storing a `namedStyle` ref on cells when many cells share the same style.
 - Renderer virtualization and batched updates: only update visible cells and apply style diffs.
 - Debounce conditional-format re-eval and offload large evaluations to a worker or Rust.
 - Benchmarks: measure applying a style to 10k cells, conditional rules on 100k cells.

Edge cases and considerations
 - Merged cells styling and edits.
 - Conflicting conditional rules: ensure deterministic priority handling.
 - Rich text vs numeric format conflicts.
 - Accessibility: color contrast and keyboard formatting shortcuts (Ctrl+B, etc.).

Tests and QA
 - Unit: serialization/migration, format parsing, conditional formatting evaluation.
 - Renderer snapshots for typical style combos.
 - Integration: toolbar -> cell JSON -> render; copy/paste/paste-special flows; named styles.
 - Accessibility checks: aria labels and contrast.

Prioritized roadmap (short to mid-term)
 1. Draft types and wire `CellValue` to include `style` + `richText` and add `NamedStyle` + `ConditionalFormatRule` (TS change).
 2. Update adapters to pass through `style`, `_cellTypes`, and `u`.
 3. Implement renderer CSS mapping and small examples for bold/italic/background, number formats, wrap and alignment.
 4. Add basic toolbar (bold/italic/underline, fill, font/size, number formats) and context menu.
 5. Implement copy/paste/paste-special and format painter.
 6. Implement conditional formatting engine and editor UI.
 7. Named styles, import/export mapping and performance tuning.
 8. Tests, accessibility, keyboard shortcuts, and docs.

Next immediate steps (recommended)
 - Implement the TypeScript types in `src/types` and update `SpreadsheetInterface.CellValue` to include `style` and `richText` (small, low-risk change).
 - Add a demo cell or a small story that lets you toggle bold/background to verify round-trip persistence and renderer mapping.
 - After types are in place, move to adapter wiring to ensure style and `u` flow through the adapters.

This formatting plan is intentionally practical: small, serializable types first, then renderer and toolbar, then clipboard/conditional rules and finally import/export and perf.

Contextualization with the existing codebase
-------------------------------------------
I reviewed the current spreadsheet adapters and interface in `AnaFis/src/components/spreadsheet/` so the formatting plan above is grounded in the actual code. Below are concrete mappings and low-risk changes that make the plan coherent and feasible with the current code.

Files discovered and key observations
 - `src/components/spreadsheet/SpreadsheetInterface.ts`
   - Current `CellValue` is minimal: `v?: string | number; f?: string`.
   - `WorkbookData` / `SheetData` shape already supports `cellData?: Record<string, CellValue>`.
 - `src/components/spreadsheet/univer/UniverAdapter.tsx`
   - Converts `WorkbookData.cellData` -> Univer `ICellData` by mapping `v` and `f` only.
   - Uses `UniverSpreadsheet` and its `UniverSpreadsheetRef` to invoke `updateCell`, `getCellValue`, `getRange`.
 - `src/components/spreadsheet/reactDataGrid/ReactDataGridAdapter.tsx`
   - Converts `WorkbookData` into rows where each cell is `sheet.cellData?.[key]?.v ?? ''` and includes an optional `_cellTypes` meta field on rows.
   - `updateCell` and `getCellValue` operate on `v` only.
 - `src/components/spreadsheet/univer/UniverSpreadsheet.tsx`
   - Internals already accept object cell values in some places: e.g. `const cellValue = typeof value === 'object' ? value : { v: value };` which makes it easier to carry richer cell objects through Univer.

Concrete change plan (file-level, low-risk, backwards-compatible)
 1) `SpreadsheetInterface.ts` (single small change)
    - Extend the exported `CellValue` interface to include optional fields but keep `v` and `f` as-is so existing code still compiles:
      - Add `u?: number | null`, `style?: CellStyle`, `richText?: RichTextRun[]`, `comment?: {...}`.
    - Rationale: this is a non-breaking extension type-wise and TypeScript optional fields are safe for current consumers.

 2) `UniverAdapter.tsx` (adapter wiring)
    - When converting abstract `cellData` -> `ICellData`, pass through the richer object when possible. Univer accepts `ICellData` with `v` and `f` at minimum; to preserve extra metadata either:
      - Put `u`, `style`, `richText` into `ICellData.meta` or `userMeta` if the Univer shape supports it, or
      - Keep the richer cell object in `cellData` (UniverSpreadsheet already has code to accept object values) so that downstream Univer rendering/serializing gets this payload.
    - On `handleCellChange` convert Univer change objects back into `CellValue` with `v`, `f`, and any metadata present.
    - Rationale: minimal change to the adapter; Univser internals already partially accept non-primitive cell values.

 3) `ReactDataGridAdapter.tsx` (adapter mapping for simple grid mode)
    - Update `toRows` to prefer `sheet.cellData?.[key]?.v ?? ''` but also attach the raw `CellValue` to a parallel structure (e.g., store `__cellMeta` on the Row) so editors can access `u` and `style` when needed.
    - Update `updateCell` to accept `value: CellValue` and set both the visible `v` and the underlying meta store.
    - Add paste-special / editor hooks that utilize the richer `CellValue` shape when committing changes.
    - Rationale: react-data-grid is a simpler adapter and can store a small `_cellMeta` to carry extra fields; it keeps visible cell text the same while preserving metadata for round-trip save.

 4) `UniverSpreadsheet.tsx` (renderer/engine side)
    - Since this component already checks for object cell values, ensure its `updateCell` and serialization persist `u`, `style`, and `richText` when writing workbook JSON.
    - If Univer rendering needs style-to-CSS mapping, add a thin mapping layer that reads `cell.style` and applies inline CSS or CSS classes.

Migration and backward compatibility
 - On load: if a workbook's `cellData` lacks `style` or `u`, treat those as undefined (no change). If older code wrote `meta.style` into `meta`, migrate it into `style` at parse/load time.
 - Adapters should tolerate cellValue being either a primitive value `{ v: 'foo' }` or the richer object; always prefer accessing optional fields safely.

Feasibility notes
 - The adapters and Univer internals already support object-style cell values in a few places, which makes adding `style` and `u` straightforward.
 - React-data-grid adapter is intentionally minimal; preserving a parallel per-row metadata store (`__cellMeta` or `_cellTypes`) is a pragmatic approach until a full renderer that understands `style` is implemented.
 - Heavy features (conditional-format evaluation across large ranges, XLSX import/export with styles) should be queued after the type + adapter plumbing is complete. They can be implemented incrementally and offloaded to a worker or Rust where needed.

Actionable repository edits to start (small patches)
 - Update `src/components/spreadsheet/SpreadsheetInterface.ts` to add optional `u`, `style`, `richText`, and `comment` fields to `CellValue`.
 - Update `src/components/spreadsheet/univer/UniverAdapter.tsx` to pass richer cell objects through `convertToUniverData` and to map `ICellData` back to `CellValue` including metadata in `handleCellChange`.
 - Update `src/components/spreadsheet/reactDataGrid/ReactDataGridAdapter.tsx` to preserve per-cell meta in rows and to pass `CellValue` objects into `onCellChange` instead of only `{ v }`.

After these edits the rest of the plan (renderer CSS mapping, toolbar, conditional rules) can be implemented incrementally and tested end-to-end using the existing adapters.

Adapter choice and rendering strategy (ReactDataGrid vs Univer)
-------------------------------------------------------------
Important: the repo currently contains a fully working `Univer` adapter and a lightweight `react-data-grid` adapter. However, `Univer` (the presets integration used) does not offer a straightforward way to add custom per-cell renderers for complex displays like "value ± uncertainty" inside a single cell. Because of that, this plan assumes a migration of the interactive spreadsheet UI to `react-data-grid` (or another grid that supports cell renderers) for the features that require custom rendering and in-cell uncertainty propagation.

Why switch to ReactDataGrid (or an equivalent grid):
 - Custom cell formatter/editor support — required to display "v ± u" compactly and to show `?` or a small badge when uncertainty is unknown.
 - Simpler integration for custom editors like the PlusMinusEditor, inline formula editing, and format toolbar.
 - Easier to incrementally extend for style application (CSS mapping) and virtualization control.

How this changes the implementation approach
 - Keep the `UniverAdapter` as a fallback and for cases where full workbook fidelity (formulas, complex spreadsheets) is needed. But make `ReactDataGridAdapter` the primary interactive editor for the new feature set (value+uncertainty-in-cell, rich formatting).
 - The `UncertaintySidebar` workflow that writes value formulas and separate uncertainty formulas into the sheet remains valuable for the scenario where value and uncertainty live in separate cells; keep it as-is and maintain compatibility.
 - For the new "value+uncertainty in the same cell + auto-propagate" mode, implement an analytic evaluation flow that intercepts formula entry, computes propagated `u` (via `derive_formula` + `evaluate_formula` or via `calculate_uncertainty`), and writes back a single `CellValue` object with `{ f, v, u }` to the grid. This requires ReactDataGrid to accept and preserve non-primitive `CellValue` objects (see adapter changes below).

Auto-propagation flow for value + uncertainty in the same cell (concrete)
 1. User types a formula into a cell (e.g. "=x+y" or an expression in the app that gets converted). The grid's editor emits a formula string (adapter's `onFormulaIntercept`).
 2. Adapter intercepts and normalizes the formula, and triggers a propagation routine:
   - Ensure symbolic derivatives are available for the formula. If needed, call `derive_formula({ formula, variables })` which will use Python/SymPy to return derivative strings. Cache derivatives keyed by normalized formula.
   - Build `context` by reading referenced variable cells from the sheet. Each referenced variable must be resolved to an object `{ v: number, u?: number | null }`. (Adapter should read cells via `getRange` or `getCellValue`; if the variable is a range, determine semantics.)
   - Call `evaluate_formula({ cellRef, formula, context, options })` implemented in Rust/Tauri. The recommended `evaluate_formula` returns `{ value?: number, uncertainty?: number | null, success: boolean, error?: string }`. Internally Rust will evaluate derivative expressions numerically and apply the analytic propagation u = sqrt(sum((df/dxi * ui)^2)). If analytic fails, `uncertainty` should be `null` and `success` false.
 3. Adapter receives the evaluation result and writes back to the grid with `updateCell(cellRef, { f: formula, v: result.value, u: result.uncertainty })`.
   - When `u === null`, renderer shows value and a small `?` badge; the user can click a button to run Monte Carlo on demand.
   - If the grid is `ReactDataGrid`, the custom renderer will show `v ± u` for numeric cells and provide hover or detail UI to show the formula and derivative info.

Editor/renderer implications for ReactDataGridAdapter
 - Update `ReactDataGridAdapter` to:
  1. Store both visible text (`v`) and full `CellValue` meta (including `u`, `f`, `style`) in the row object — e.g., column property stores display string while an internal `_cellMeta` map stores the full `CellValue` for each key.
  2. Provide a custom formatter component for cells that reads the meta and renders `v ± u` (with precision/locale) and a `?` marker when `u` is null.
  3. Provide a custom editor that supports either (a) formula editing when cell is formula, or (b) plus-minus editing when cell type is 'uncertainty' or user toggled to uncertainty mode.
 - This approach is incremental: the simplest first step is to store meta in `_cellMeta`, then create a custom cell renderer for `react-data-grid` that reads `_cellMeta` to display combined value/uncertainty.

Compatibility and migration notes
 - Keep `UncertaintySidebar` and `generate_uncertainty_formulas` unchanged for workflows where values and uncertainties are separate cells — they will still work when users prefer the separate-cell layout or when writing Excel-compatible propagated formulas is desired.
 - Gradually migrate sheets/users to the new same-cell shape `{ v, u, f }`. When loading older workbooks, adapters should detect legacy shapes and map them into the newer canonical form (e.g., if uncertainties were stored in separate columns, optionally convert them into same-cell shape during import or prompt the user).

Developer tasks specific to switching adapters
 - Update `SpreadsheetInterface.CellValue` (TS) to include `u` as a first-class optional field.
 - Update `ReactDataGridAdapter` to preserve full `CellValue` objects in `_cellMeta` and provide custom renderers/editors.
 - Enhance `PlusMinusEditor` to commit `{ v, u }` shaped `CellValue` objects when in uncertainty mode.
 - Add `derive_formula` (lightweight) and `evaluate_formula` (canonical numeric evaluation) Tauri commands or reuse `calculate_uncertainty` for the numeric path (but adapt its input/output shape to match the generic `evaluate_formula` contract).

With this clarification the plan is aligned with the repo constraints: keep Univer for workbook-level fidelity, use ReactDataGrid for interactive custom rendering and auto-propagation inside single cells, and keep both analytic (derive -> evaluate) and worksheet-formula generation (generate_uncertainty_formulas) workflows available.







