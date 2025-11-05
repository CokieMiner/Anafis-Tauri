// bulkImportOperations.ts - Bulk data import operations using direct Univer instance access
import { Univer, ICommandService, IUniverInstanceService, Workbook } from '@univerjs/core';
import type { FWorksheet } from '@univerjs/sheets/facade';
import { letterToColumn } from '../index';

/**
 * Cell data structure for import
 */
export interface ImportCellData {
  v?: string | number | boolean | null;
  f?: string;
  s?: unknown;
  t?: unknown;
  p?: unknown;
}

/**
 * Merge data structure for import
 */
export interface ImportMergeData {
  startRow: number;
  startColumn: number;
  endRow: number;
  endColumn: number;
}

/**
 * Sheet data structure for import
 */
export interface ImportSheetData {
  id: string;
  name: string;
  cellData?: Record<string, ImportCellData>;
  rowCount?: number;
  columnCount?: number;
  mergeData?: ImportMergeData[];
}

/**
 * Options for bulk import operations
 */
export interface BulkImportOptions {
  includeFormulas?: boolean;
  includeFormatting?: boolean;
}

/**
 * Get Univer services from Univer instance
 */
function getUniverServices(univerInstance: Univer) {
  const injector = univerInstance.__getInjector();
  const commandService = injector.get(ICommandService);
  const instanceService = injector.get(IUniverInstanceService);
  return { commandService, instanceService };
}

/**
 * Convert matrix format cellData directly to 2D array for bulk operations
 * This is more efficient than converting to A1 notation first
 */
function convertMatrixTo2DArray(
  matrixData: Record<number, Record<number, ImportCellData>>,
  options: { includeFormulas?: boolean; includeFormatting?: boolean } = {}
): { value: unknown[][]; maxRow: number; maxCol: number } {
  const { includeFormulas = true, includeFormatting = true } = options;
  
  let maxRow = 0;
  let maxCol = 0;
  
  // Find bounds
  for (const [rowStr, rowData] of Object.entries(matrixData)) {
    const row = parseInt(rowStr, 10);
    if (row > maxRow) {
      maxRow = row;
    }
    for (const colStr of Object.keys(rowData)) {
      const col = parseInt(colStr, 10);
      if (col > maxCol) {
        maxCol = col;
      }
    }
  }
  
  // Build 2D array directly from matrix
  const value: unknown[][] = [];
  for (let r = 0; r <= maxRow; r++) {
    const rowData: unknown[] = [];
    const rowCells = matrixData[r];
    
    for (let c = 0; c <= maxCol; c++) {
      const cellData = rowCells?.[c];
      if (cellData && typeof cellData === 'object') {
        const cellValue: Record<string, unknown> = {};
        
        if (cellData.v !== undefined) {
          cellValue.v = cellData.v;
        }
        if (cellData.f && includeFormulas) {
          cellValue.f = cellData.f;
        }
        if (cellData.s && includeFormatting) {
          cellValue.s = cellData.s;
        }
        if (cellData.t) {
          cellValue.t = cellData.t;
        }
        
        rowData.push(cellValue);
      } else {
        rowData.push({});
      }
    }
    value.push(rowData);
  }
  
  return { value, maxRow, maxCol };
}

/**
 * Bulk load data into a sheet using direct Univer commands
 */
export async function bulkLoadSheetData(
  univerInstance: Univer,
  sheet: FWorksheet,
  sheetData: ImportSheetData,
  options: BulkImportOptions = {}
): Promise<void> {
  if (!sheetData.cellData) {
    return;
  }

  const { includeFormulas = true, includeFormatting = true } = options;
  const { commandService, instanceService } = getUniverServices(univerInstance);

  const workbook = instanceService.getFocusedUnit() as Workbook;
  // workbook is guaranteed to be non-null from getFocusedUnit() in this context

  const unitId = workbook.getUnitId();
  const sheetId = sheet.getSheetId();

  // Convert cellData to 2D array
  const rowMap = new Map<number, Map<number, ImportCellData>>();
  let maxRow = 0;
  let maxCol = 0;

  for (const [cellRef, cellValue] of Object.entries(sheetData.cellData)) {
    const cellMatch = cellRef.match(/^([A-Z]+)(\d+)$/);
    if (!cellMatch?.[1] || !cellMatch[2]) {
      continue;
    }

    const col = letterToColumn(cellMatch[1]);
    const row = parseInt(cellMatch[2], 10) - 1;

    if (row > maxRow) { maxRow = row; }
    if (col > maxCol) { maxCol = col; }

    if (!rowMap.has(row)) {
      rowMap.set(row, new Map());
    }
    const rowData = rowMap.get(row);
    if (rowData) {
      rowData.set(col, cellValue);
    }
  }

  // Build 2D array
  const value: unknown[][] = [];
  for (let r = 0; r <= maxRow; r++) {
    const rowData: unknown[] = [];
    const rowCells = rowMap.get(r);

    for (let c = 0; c <= maxCol; c++) {
      if (rowCells?.has(c)) {
        const cellData = rowCells.get(c);
        if (!cellData) {
          rowData.push({});
          continue;
        }
        const cellValue: Record<string, unknown> = {};

        if (cellData.v !== undefined) {
          cellValue.v = cellData.v;
        }
        if (cellData.f && includeFormulas) {
          cellValue.f = cellData.f;
        }
        if (cellData.s && includeFormatting) {
          cellValue.s = cellData.s;
        }
        if (cellData.t) {
          cellValue.t = cellData.t;
        }

        rowData.push(cellValue);
      } else {
        rowData.push({});
      }
    }
    value.push(rowData);
  }

  // Execute bulk command
  await commandService.executeCommand('sheet.command.set-range-values', {
    unitId,
    subUnitId: sheetId,
    range: {
      startRow: 0,
      startColumn: 0,
      endRow: maxRow,
      endColumn: maxCol,
    },
    value,
  });

  // Load merge data if present
  if (sheetData.mergeData && sheetData.mergeData.length > 0) {
    try {
      // Access the underlying Worksheet model to set mergeData directly
      const worksheet = workbook.getSheetBySheetId(sheetId);
      if (worksheet) {
        const mergeData = sheetData.mergeData.map(merge => ({
          startRow: merge.startRow,
          startColumn: merge.startColumn,
          endRow: merge.endRow,
          endColumn: merge.endColumn,
        }));
        
        // Use the worksheet's setMergeData method if available
        const worksheetWithMerge = worksheet as unknown as { setMergeData?: (data: unknown) => void; getConfig: () => { mergeData?: unknown } | null | undefined };
        if (typeof worksheetWithMerge.setMergeData === 'function') {
          worksheetWithMerge.setMergeData(mergeData);
        } else {
          // Try accessing the merge model directly
          const config = worksheetWithMerge.getConfig();
          if (config !== null && config !== undefined) {
            config.mergeData = mergeData;
          }
        }
      }
    } catch {
      // Silently ignore merge data application errors
    }
  }
}

/**
 * Bulk load data from matrix format (optimized path for append mode)
 * Skips A1 notation conversion for better performance
 */
export async function bulkLoadSheetDataFromMatrix(
  univerInstance: Univer,
  sheet: FWorksheet,
  sheetData: {
    name: string;
    cellDataMatrix?: Record<number, Record<number, ImportCellData>> | null;
    mergeData?: ImportMergeData[];
  },
  options: BulkImportOptions = {}
): Promise<void> {
  // Early return if no data
  if (!sheetData.cellDataMatrix || Object.keys(sheetData.cellDataMatrix).length === 0) {
    return;
  }

  const { includeFormulas = true, includeFormatting = true } = options;
  const { commandService, instanceService } = getUniverServices(univerInstance);

  const workbook = instanceService.getFocusedUnit() as Workbook;
  const unitId = workbook.getUnitId();
  const sheetId = sheet.getSheetId();

  // Direct conversion: Matrix → 2D array (no A1 notation step!)
  const { value, maxRow, maxCol } = convertMatrixTo2DArray(sheetData.cellDataMatrix, {
    includeFormulas,
    includeFormatting,
  });

  // Execute bulk command
  await commandService.executeCommand('sheet.command.set-range-values', {
    unitId,
    subUnitId: sheetId,
    range: {
      startRow: 0,
      startColumn: 0,
      endRow: maxRow,
      endColumn: maxCol,
    },
    value,
  });

  // Load merge data if present
  if (sheetData.mergeData && sheetData.mergeData.length > 0) {
    try {
      const worksheet = workbook.getSheetBySheetId(sheetId);
      const worksheetWithMerge = worksheet as unknown as { setMergeData?: (data: unknown) => void };
      if (worksheet && typeof worksheetWithMerge.setMergeData === 'function') {
        const mergeData = sheetData.mergeData.map(merge => ({
          startRow: merge.startRow,
          startColumn: merge.startColumn,
          endRow: merge.endRow,
          endColumn: merge.endColumn,
        }));
        worksheetWithMerge.setMergeData(mergeData);
      }
    } catch (error) {
      console.error('Failed to apply merge data:', error);
    }
  }
}

/**
 * Bulk load data with offset (for insertAtCell mode)
 */
export async function bulkLoadSheetDataWithOffset(
  univerInstance: Univer,
  sheet: FWorksheet,
  sheetData: ImportSheetData,
  rowOffset: number,
  colOffset: number,
  options: BulkImportOptions = {}
): Promise<void> {
  if (!sheetData.cellData) {
    return;
  }

  const { includeFormulas = true, includeFormatting = true } = options;
  const { commandService, instanceService } = getUniverServices(univerInstance);

  const workbook = instanceService.getFocusedUnit() as Workbook;
  // workbook is guaranteed to be non-null from getFocusedUnit() in this context

  const unitId = workbook.getUnitId();
  const sheetId = sheet.getSheetId();

  // Convert cellData with offset
  const rowMap = new Map<number, Map<number, ImportCellData>>();
  let maxRow = rowOffset;
  let maxCol = colOffset;

  for (const [cellRef, cellValue] of Object.entries(sheetData.cellData)) {
    const cellMatch = cellRef.match(/^([A-Z]+)(\d+)$/);
    if (!cellMatch?.[1] || !cellMatch[2]) {
      continue;
    }

    const col = letterToColumn(cellMatch[1]) + colOffset;
    const row = parseInt(cellMatch[2], 10) - 1 + rowOffset;

    if (row > maxRow) { maxRow = row; }
    if (col > maxCol) { maxCol = col; }

    if (!rowMap.has(row)) {
      rowMap.set(row, new Map());
    }
    const rowData = rowMap.get(row);
    if (rowData) {
      rowData.set(col, cellValue);
    }
  }

  // Build 2D array
  const value: unknown[][] = [];
  const startRow = rowOffset;
  const startCol = colOffset;

  for (let r = startRow; r <= maxRow; r++) {
    const rowData: unknown[] = [];
    const rowCells = rowMap.get(r);

    for (let c = startCol; c <= maxCol; c++) {
      if (rowCells?.has(c)) {
        const cellData = rowCells.get(c);
        if (!cellData) {
          rowData.push({});
          continue;
        }
        const cellValue: Record<string, unknown> = {};

        if (cellData.v !== undefined) {
          cellValue.v = cellData.v;
        }
        if (cellData.f && includeFormulas) {
          cellValue.f = cellData.f;
        }
        if (cellData.s && includeFormatting) {
          cellValue.s = cellData.s;
        }
        if (cellData.t) {
          cellValue.t = cellData.t;
        }

        rowData.push(cellValue);
      } else {
        rowData.push({});
      }
    }
    value.push(rowData);
  }

  // Execute bulk command
  await commandService.executeCommand('sheet.command.set-range-values', {
    unitId,
    subUnitId: sheetId,
    range: {
      startRow,
      startColumn: startCol,
      endRow: maxRow,
      endColumn: maxCol,
    },
    value,
  });

  // Load merge data if present
  if (sheetData.mergeData && sheetData.mergeData.length > 0) {
    try {
      const worksheet = workbook.getSheetBySheetId(sheetId);
      if (worksheet) {
        const mergeData = sheetData.mergeData.map(merge => ({
          startRow: merge.startRow + rowOffset,
          startColumn: merge.startColumn + colOffset,
          endRow: merge.endRow + rowOffset,
          endColumn: merge.endColumn + colOffset,
        }));
        
        const worksheetWithMerge = worksheet as unknown as { setMergeData?: (data: unknown) => void; getConfig: () => { mergeData?: unknown } | null | undefined };
        if (typeof worksheetWithMerge.setMergeData === 'function') {
          worksheetWithMerge.setMergeData(mergeData);
        } else {
          const config = worksheetWithMerge.getConfig();
          if (config !== null && config !== undefined) {
            config.mergeData = mergeData;
          }
        }
      }
    } catch (error) {
      console.warn('Failed to apply merge data with offset:', error);
    }
  }
}


/**
 * Clear sheet data by filling with empty cells
 */
export async function clearSheetData(
  univerInstance: Univer,
  sheet: FWorksheet,
  maxRows: number = 1000,
  maxCols: number = 26
): Promise<void> {
  const { commandService, instanceService } = getUniverServices(univerInstance);

  const workbook = instanceService.getFocusedUnit() as Workbook;
  // workbook is guaranteed to be non-null from getFocusedUnit() in this context

  const unitId = workbook.getUnitId();
  const sheetId = sheet.getSheetId();

  // Create empty 2D array
  const emptyValue: unknown[][] = [];
  for (let r = 0; r < maxRows; r++) {
    const row: unknown[] = [];
    for (let c = 0; c < maxCols; c++) {
      row.push({});
    }
    emptyValue.push(row);
  }

  // Execute bulk clear command
  await commandService.executeCommand('sheet.command.set-range-values', {
    unitId,
    subUnitId: sheetId,
    range: {
      startRow: 0,
      startColumn: 0,
      endRow: maxRows - 1,
      endColumn: maxCols - 1,
    },
    value: emptyValue,
  });
}