// bulkImportOperations.ts - Bulk data import operations using direct Univer instance access
import { Univer, ICommandService, IUniverInstanceService, Workbook } from '@univerjs/core';
import type { FWorksheet } from '@univerjs/sheets/facade';
import { letterToColumn } from '@/tabs/spreadsheet/univer';
// Import type augmentations
import '@/tabs/spreadsheet/types/univer-augmentations';

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
 * Unified bulk load data into a sheet using direct Univer commands
 * Handles all input formats and offset scenarios
 */
async function unifiedBulkLoadSheetData(
  univerInstance: Univer,
  sheet: FWorksheet,
  inputData: ImportSheetData | {
    name: string;
    cellDataMatrix?: Record<number, Record<number, ImportCellData>> | null;
    mergeData?: ImportMergeData[];
  },
  options: BulkImportOptions & {
    rowOffset?: number;
    colOffset?: number;
    inputFormat?: 'a1' | 'matrix';
  } = {}
): Promise<void> {
  const {
    includeFormulas = true,
    includeFormatting = true,
    rowOffset = 0,
    colOffset = 0,
    inputFormat = 'a1'
  } = options;

  const { commandService, instanceService } = getUniverServices(univerInstance);
  const workbook = instanceService.getFocusedUnit() as Workbook;
  const unitId = workbook.getUnitId();
  const sheetId = sheet.getSheetId();

  let cellDataMatrix: Record<number, Record<number, ImportCellData>>;
  const mergeData = (inputData as ImportSheetData).mergeData ??
                    (inputData as { mergeData?: ImportMergeData[] }).mergeData;

  // Convert input data to matrix format based on input type
  if (inputFormat === 'matrix') {
    // Direct matrix input
    const matrixInput = inputData as {
      cellDataMatrix?: Record<number, Record<number, ImportCellData>> | null;
    };
    if (!matrixInput.cellDataMatrix || Object.keys(matrixInput.cellDataMatrix).length === 0) {
      return; // No data to load
    }
    cellDataMatrix = matrixInput.cellDataMatrix;
  } else {
    // A1 notation input - convert to matrix
    const a1Input = inputData as ImportSheetData;
    if (!a1Input.cellData) {
      return; // No data to load
    }

    cellDataMatrix = {};
    let maxRow = rowOffset;
    let maxCol = colOffset;

    for (const [cellRef, cellValue] of Object.entries(a1Input.cellData)) {
      const cellMatch = cellRef.match(/^([A-Z]+)(\d+)$/);
      if (!cellMatch?.[1] || !cellMatch[2]) {
        continue;
      }

      const col = letterToColumn(cellMatch[1]) + colOffset;
      const row = parseInt(cellMatch[2], 10) - 1 + rowOffset;

      if (row > maxRow) { maxRow = row; }
      if (col > maxCol) { maxCol = col; }

      cellDataMatrix[row] ??= {};
      cellDataMatrix[row][col] = cellValue;
    }
  }

  // Convert matrix to 2D array for Univer
  const { value, maxRow, maxCol } = convertMatrixTo2DArray(cellDataMatrix, {
    includeFormulas,
    includeFormatting,
  });

  // Execute bulk command
  await commandService.executeCommand('sheet.command.set-range-values', {
    unitId,
    subUnitId: sheetId,
    range: {
      startRow: rowOffset,
      startColumn: colOffset,
      endRow: maxRow + rowOffset,
      endColumn: maxCol + colOffset,
    },
    value,
  });

  // Load merge data if present
  if (mergeData && mergeData.length > 0) {
    try {
      const worksheet = workbook.getSheetBySheetId(sheetId);
      if (worksheet) {
        const processedMergeData = mergeData.map(merge => ({
          startRow: merge.startRow + rowOffset,
          startColumn: merge.startColumn + colOffset,
          endRow: merge.endRow + rowOffset,
          endColumn: merge.endColumn + colOffset,
        }));

        // Use the worksheet's setMergeData method if available
        if (typeof worksheet.setMergeData === 'function') {
          worksheet.setMergeData(processedMergeData);
        } else {
          // Try accessing the merge model directly
          const config = worksheet.getConfig();
          if (config !== null && config !== undefined) {
            config.mergeData = config.mergeData ?? processedMergeData;
          }
        }
      }
    } catch (error) {
      // Handle errors consistently based on input format
      if (inputFormat === 'matrix') {
        console.error('Failed to apply merge data:', error);
      } else {
        console.warn('Failed to apply merge data with offset:', error);
      }
    }
  }
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
  return unifiedBulkLoadSheetData(univerInstance, sheet, sheetData, {
    ...options,
    inputFormat: 'a1'
  });
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
  return unifiedBulkLoadSheetData(univerInstance, sheet, sheetData, {
    ...options,
    inputFormat: 'matrix'
  });
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
  return unifiedBulkLoadSheetData(univerInstance, sheet, sheetData, {
    ...options,
    rowOffset,
    colOffset,
    inputFormat: 'a1'
  });
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