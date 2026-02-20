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

import type { ExportService } from '@/core/types/export';
import type { ImportService } from '@/core/types/import';

// ============================================================================
// ABSTRACT STYLE TYPES (Library-Independent)
// ============================================================================

/**
 * Abstract style data interface that doesn't depend on any specific spreadsheet library.
 * This allows the abstraction layer to be library-agnostic while still supporting
 * comprehensive styling capabilities.
 */
export interface SpreadsheetStyle {
  // Font properties
  fontFamily?: string;
  fontSize?: number;
  fontWeight?: 'normal' | 'bold' | 'lighter' | 'bolder' | number;
  fontStyle?: 'normal' | 'italic' | 'oblique';
  fontColor?: string;
  fontDecoration?: Array<'underline' | 'line-through' | 'overline'>;

  // Fill/background properties
  backgroundColor?: string;
  backgroundPattern?: {
    type: 'solid' | 'stripes' | 'dots' | 'grid';
    color?: string;
    patternColor?: string;
  };

  // Border properties
  borderTop?: BorderStyle;
  borderBottom?: BorderStyle;
  borderLeft?: BorderStyle;
  borderRight?: BorderStyle;

  // Alignment properties
  horizontalAlign?: 'left' | 'center' | 'right' | 'justify' | 'distributed';
  verticalAlign?: 'top' | 'middle' | 'bottom' | 'justify' | 'distributed';
  textWrap?: boolean;
  textRotation?: number; // degrees, -90 to 90

  // Number formatting
  numberFormat?: string; // Format pattern like "#,##0.00" or "YYYY-MM-DD"

  // Cell protection
  locked?: boolean;
  hidden?: boolean;

  // Additional style properties (extensible for different libraries)
  [key: string]: unknown;
}

/**
 * Border style specification
 */
export interface BorderStyle {
  style:
    | 'none'
    | 'thin'
    | 'medium'
    | 'dashed'
    | 'dotted'
    | 'thick'
    | 'double'
    | 'hair'
    | 'mediumDashed'
    | 'dashDot'
    | 'mediumDashDot'
    | 'dashDotDot'
    | 'mediumDashDotDot'
    | 'slantDashDot';
  color?: string;
}

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
  f?: string | null; // formula
  meta?: {
    custom?: unknown; // Custom document data (cellData.p) - keep flexible for various formats
    cellType?: unknown; // Cell value type (cellData.t) - keep flexible for various formats
    formulaRef?: string; // Formula reference range for array formulas (cellData.ref)
    formulaId?: string; // Formula ID (cellData.si)
    customFields?: unknown; // User stored custom fields (cellData.custom) - keep flexible
    [key: string]: unknown;
  };
  style?: unknown; // Keep flexible for various style formats
  s?: unknown; // Direct Univer style format (cellData.s) - for internal compatibility
  richText?: unknown[];
}

export interface SpreadsheetRef {
  // Core cell operations
  updateCell: (cellRef: string, value: CellValue) => Promise<void>;
  batchUpdateCells: (
    updates: Array<{ cellRef: string; value: CellValue }>
  ) => Promise<void>;
  getCellValue: (cellRef: string) => Promise<string | number | null>;

  // Range operations
  updateRange: (rangeRef: string, values: CellValue[][]) => Promise<void>;
  getRange: (rangeRef: string) => Promise<(string | number)[][]>;
  getRangeFull: (rangeRef: string) => Promise<CellValue[][]>;

  // Direct formula insertion (performance optimization)
  insertFormulas: (
    rangeOrStartCell: string,
    formulas: string[] | string[][],
    direction?: 'vertical' | 'horizontal'
  ) => Promise<void>;

  // Selection and state
  getSelection: () => Promise<string | null>;
  isReady: () => boolean;

  // Multi-sheet support (required for import/export operations)
  createSheet: (name: string, rows?: number, cols?: number) => Promise<string>;
  getAllSheets: () => Promise<Array<{ id: string; name: string }>>;
  deleteSheet: (sheetId: string) => Promise<void>;

  // Advanced snapshot operations (required for import/export operations)
  getWorkbookSnapshot: () => Promise<unknown>;
  loadWorkbookSnapshot: (snapshot: unknown) => Promise<void>;

  // Range utilities (added for proper abstraction)
  getUsedRange: () => Promise<string>;
  getSheetBounds: (sheetId?: string) => Promise<{
    startCol: number;
    startRow: number;
    endCol: number;
    endRow: number;
  }>;

  // Append mode operations (for importing sheets into existing workbook)
  getNewlyCreatedSheet: (sheetName: string) => Promise<unknown>;
  loadSheetDataBulk: (
    sheetId: string,
    sheetData: {
      name: string;
      cellDataMatrix?: Record<number, Record<number, unknown>> | null;
      mergeData?: Array<{
        startRow: number;
        startColumn: number;
        endRow: number;
        endColumn: number;
      }>;
    },
    options?: { includeFormulas?: boolean; includeFormatting?: boolean }
  ) => Promise<void>;
  applySheetProtection: (
    newSheetId: string,
    protectionData: Array<{ name: string; data: string }>,
    sheetIdMapping?: Map<string, string>
  ) => Promise<void>;

  // Service access (required for import/export operations)
  getExportService: () => ExportService;
  getImportService: () => ImportService;
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
  onReady?: () => void; // Called when the spreadsheet adapter is fully initialized and ready
  tabId?: string; // Optional tab ID for instance tracking
}

// Abstract data structure - can be adapted per library
export interface WorkbookData {
  id: string;
  name: string;
  appVersion?: string;
  locale?: string; // Abstract locale identifier
  styles?: Record<string, SpreadsheetStyle>;
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

// Typed snapshot structure for workbook snapshots (used by import/export)
export interface WorkbookSnapshot {
  id?: string;
  name?: string;
  appVersion?: string;
  locale?: string;
  styles?: Record<string, SpreadsheetStyle>;
  sheets: Record<string, unknown>; // More permissive to accept different sheet formats
  sheetOrder?: unknown;
  resources?: unknown;
  // Allow additional properties for different snapshot formats
  [key: string]: unknown;
}

export interface SheetSnapshot {
  id?: string;
  name?: string;
  cellData?: unknown; // More permissive to accept different cell data formats
  mergeData?: unknown;
  rowCount?: number;
  columnCount?: number;
  // Allow additional properties for different snapshot formats
  [key: string]: unknown;
}

// Cell data structure used in snapshots
export interface CellData {
  v?: string | number | boolean | null; // Value
  f?: string; // Formula
  s?: unknown; // Style - keep flexible for various formats
  t?: unknown; // Type - keep flexible for various formats
  p?: unknown; // Custom properties - keep flexible for various formats
  [key: string]: unknown; // Allow additional properties
}
