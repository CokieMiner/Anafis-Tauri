// SpreadsheetInterface.ts - Abstract interface for any spreadsheet library
export interface CellValue {
  v?: string | number;
  f?: string; // formula
}

export interface SpreadsheetRef {
  updateCell: (cellRef: string, value: CellValue) => void;
  getCellValue: (cellRef: string) => string | number | null;
  getRange: (rangeRef: string) => Promise<(string | number)[][]>;
}

export interface SpreadsheetProps {
  initialData: WorkbookData;
  onCellChange: (cellRef: string, value: CellValue) => void;
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