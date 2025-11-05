// SpreadsheetInterface.ts - Abstract interface for any spreadsheet library
//
// This file defines the minimal contract the app expects from any spreadsheet
// UI adapter. The project uses an adapter pattern so the UI component can be
// swapped (e.g. Univer, AG Grid, Handsontable, or a custom grid) while keeping
// the app unaware of the concrete implementation.
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

import { IExportService } from '@/types/export';
import { IImportService } from '@/types/import';

export interface SpreadsheetCapabilities {
  // Feature support flags
  supportsFormulas: boolean;
  supportsMultipleSheets: boolean;
  supportsFormatting: boolean;
  supportsMergedCells: boolean;
  supportsCharts: boolean;
  supportsComments: boolean;
  
  // Performance characteristics
  maxRows: number;
  maxColumns: number;
  maxSheets: number;
  
  // Implementation metadata
  libraryName: string;
  libraryVersion: string;
  adapterVersion: string;
}

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
  // Core cell operations
  updateCell: (cellRef: string, value: CellValue) => Promise<void>;
  batchUpdateCells: (updates: Array<{ cellRef: string; value: CellValue }>) => Promise<void>;
  getCellValue: (cellRef: string) => Promise<string | number | null>;

  // Range operations
  updateRange: (rangeRef: string, values: CellValue[][]) => Promise<void>;
  getRange: (rangeRef: string) => Promise<(string | number)[][]>;
  getRangeFull: (rangeRef: string) => Promise<CellValue[][]>;

  // Selection and state
  getSelection: () => Promise<string | null>;
  isReady: () => boolean;

  // Multi-sheet support (required for import/export operations)
  createSheet: (name: string, rows?: number, cols?: number) => Promise<string>;
  getAllSheets: () => Promise<Array<{ id: string; name: string }>>;

  // Advanced snapshot operations (required for import/export operations)
  getWorkbookSnapshot: () => Promise<unknown>;
  loadWorkbookSnapshot: (snapshot: unknown) => Promise<void>;

  // Internal API access (required for advanced operations like used range detection)
  getImplementationContext: () => { univerInstance?: unknown; facadeInstance?: unknown };

  // Service access (required for import/export operations)
  getExportService: () => IExportService;
  getImportService: () => IImportService;
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
  tabId?: string; // Optional tab ID for instance tracking
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