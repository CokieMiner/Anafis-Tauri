// SpreadsheetInterface.ts - Abstract interface for any spreadsheet library
//
// This file defines the minimal contract the app expects from any spreadsheet
// UI adapter. The project uses an adapter pattern so the UI component can be
// swapped (e.g. Univer, AG Grid, Handsontable, or a custom grid) while keeping
// the rest of the app unaware of the concrete implementation.
//
// Recommendation (non-breaking): when adding support for asynchronous
// formula evaluation (for example delegating formula parsing/evaluation to
// Rust via Tauri or to a WASM module), prefer keeping the current callback
// names but make the handler implementors return a Promise. That allows the
// UI adapter to await a computed value and then apply it back into the grid.
// Example pattern (JS/TS pseudo-signature):
//   async function onFormulaIntercept(cellRef: string, formula: string): Promise<string | number | null> { ... }
// The code in adapters can still support sync handlers for backward
// compatibility by checking whether the returned value is a Promise.

export interface CellValue {
  v?: string | number | boolean | null;
  f?: string; // formula
  meta?: {
    custom?: unknown; // Custom document data (cellData.p)
    cellType?: unknown; // Cell value type (cellData.t)
    formulaRef?: string; // Formula reference range for array formulas (cellData.ref)
    formulaId?: string; // Formula ID (cellData.si)
    customFields?: unknown; // User stored custom fields (cellData.custom)
    [key: string]: unknown;
  };
  style?: unknown;
  richText?: unknown[];
}

export interface SpreadsheetRef {
  // Update a single cell (A1-style reference). Implementations may accept
  // objects with either { v } for a value or { f } for a formula string.
  updateCell: (cellRef: string, value: CellValue) => Promise<void>;
  // Update multiple cells in a single batch operation (more efficient than calling updateCell in a loop)
  batchUpdateCells: (updates: Array<{ cellRef: string; value: CellValue }>) => Promise<void>;
  // Read the current (calculated) value of a single cell. Returns null when
  // the cell does not exist or has no value.
  getCellValue: (cellRef: string) => Promise<string | number | null>;
  // Read a rectangular range. Implementations should return rows in order
  // (top-to-bottom) and columns left-to-right.
  getRange: (rangeRef: string) => Promise<(string | number)[][]>;
  // Read a rectangular range with full cell data including formulas, formatting, etc.
  getRangeFull: (rangeRef: string) => Promise<CellValue[][]>;
  // Get all sheets data for multi-sheet export
  getAllSheetsData: () => Promise<{ name: string; data: CellValue[][] }[]>;
  // Get current selection range
  getSelection: () => Promise<string | null>;
  // Get used range of active sheet
  getUsedRange: () => Promise<string>;
}

export interface SpreadsheetProps {
  initialData: WorkbookData;
  // Fired when a cell's raw value (not a leading '=' formula) has changed in
  // the UI. Keep this synchronous for simple use cases; adapters may call it
  // from their command hooks.
  onCellChange: (cellRef: string, value: CellValue) => void;
  // Called when the user enters a value that looks like a formula
  // (typically starting with '='). Implementations can keep this synchronous
  // or return a Promise to support async evaluation via backend/Rust/WASM.
  onFormulaIntercept: (cellRef: string, formula: string) => void;
  onSelectionChange?: (cellRef: string) => void;
}

// Abstract data structure - can be adapted per library
export interface WorkbookData {
  id: string;
  name: string;
  appVersion?: string;
  locale?: string; // Abstract locale identifier
  styles?: unknown;
  sheets: Record<string, SheetData>;
  sheetOrder?: string[];
  // Allow additional properties for different spreadsheet libraries
  [key: string]: unknown;
}

export interface SheetData {
  id: string;
  name: string;
  cellData?: Record<string, CellValue>; // key is like "A1", "B2", etc.
  rowCount?: number;
  columnCount?: number;
  // Allow additional properties for different spreadsheet libraries
  [key: string]: unknown;
}