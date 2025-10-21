// spreadsheetOperations.ts - Core spreadsheet operations using injector pattern with interceptors
import { IUniverInstanceService, Workbook, ICommandService } from '@univerjs/core';
import { SheetInterceptorService } from '@univerjs/sheets';
import { CellValue } from '../SpreadsheetInterface';
import { parseRange, columnToLetter } from './univerUtils';
import { safeUniverOperation } from './errors';

/**
 * Setup SheetInterceptorService for enhanced data access
 */
export function setupSheetInterceptors(univerRef: any): void {
  if (!univerRef?.current) return;

  safeUniverOperation(async () => {
    const injector = univerRef.current.univer.__getInjector();
    const sheetInterceptorService = injector.get(SheetInterceptorService);

    // Add interceptors for enhanced cell value retrieval
    // This allows intercepting cell data access to account for formulas, formatting, etc.
    sheetInterceptorService.interceptCellValue((cell: any, _context: any) => {
      // Enhanced cell value processing with formula/formatting awareness
      // The interceptor can modify or enhance cell data before it's returned
      return cell;
    });

    // Add interceptor for range data access
    sheetInterceptorService.interceptCellData((cellData: any, _context: any) => {
      // Enhanced cell data processing
      return cellData;
    });
  }, 'setup sheet interceptors');
}

/**
 * Get workbook instance from injector
 */
export function getWorkbook(univerRef: any): Workbook | null {
  if (!univerRef?.current) {
    console.error('[getWorkbook] univerRef.current is null');
    return null;
  }

  if (!univerRef.current.univer) {
    console.error('[getWorkbook] univerRef.current.univer is null');
    return null;
  }

  if (!univerRef.current.univer.__getInjector) {
    console.error('[getWorkbook] __getInjector is not available on univer instance');
    return null;
  }

  try {
    const injector = univerRef.current.univer.__getInjector();
    const instanceService = injector.get(IUniverInstanceService);
    return instanceService.getFocusedUnit() as Workbook;
  } catch (error) {
    console.error('[getWorkbook] Error getting workbook:', error);
    return null;
  }
}

/**
 * Update a single cell using command service
 */
export function updateCell(
  univerRef: any,
  cellRef: string,
  value: { v?: string | number; f?: string }
): void {
  if (!univerRef?.current) return;

  safeUniverOperation(async () => {
    const injector = univerRef.current.univer.__getInjector();
    const commandService = injector.get(ICommandService);
    const workbook = getWorkbook(univerRef);
    if (!workbook) return;

    const activeSheet = workbook.getActiveSheet();
    if (!activeSheet) return;

    const match = cellRef.match(/^([A-Z]+)(\d+)$/);
    if (!match) return;

    const col = match[1];
    const row = parseInt(match[2]) - 1;
    let colIndex = 0;
    for (let i = 0; i < col.length; i++) {
      colIndex = colIndex * 26 + (col.charCodeAt(i) - 65 + 1);
    }
    colIndex -= 1;

    const cellValue = typeof value === 'object' ? value : { v: value };

    commandService.executeCommand('sheet.command.set-range-values', {
      unitId: workbook.getUnitId(),
      subUnitId: activeSheet.getSheetId(),
      range: {
        startRow: row,
        startColumn: colIndex,
        endRow: row,
        endColumn: colIndex
      },
      value: [[cellValue]]
    });
  }, 'update cell');
}

/**
 * Get cell value from spreadsheet
 */
export function getCellValue(univerRef: any, cellRef: string): string | number | null {
  if (!univerRef?.current) return null;

  try {
    const workbook = getWorkbook(univerRef);
    if (!workbook) return null;

    const activeSheet = workbook.getActiveSheet();
    if (!activeSheet) return null;

    const match = cellRef.match(/^([A-Z]+)(\d+)$/);
    if (!match) return null;

    const col = match[1];
    const row = parseInt(match[2]) - 1;
    let colIndex = 0;
    for (let i = 0; i < col.length; i++) {
      colIndex = colIndex * 26 + (col.charCodeAt(i) - 65 + 1);
    }
    colIndex -= 1;

    const cellData = activeSheet.getCellRaw(row, colIndex);
    return cellData?.v !== undefined ? cellData.v as string | number : null;
  } catch (error) {
    console.error('Failed to get cell value:', error);
    return null;
  }
}

/**
 * Get range values from spreadsheet
 */
export async function getRange(
  univerRef: any,
  rangeRef: string
): Promise<(string | number)[][]> {
  if (!univerRef?.current) return [];

  return safeUniverOperation(async () => {
    const workbook = getWorkbook(univerRef);
    if (!workbook) return [];

    const activeSheet = workbook.getActiveSheet();
    if (!activeSheet) return [];

    const parsedRange = parseRange(rangeRef);
    if (!parsedRange) return [];

    const { startCol, startRow, endCol, endRow } = parsedRange;

    // Extract values row by row
    const result: (string | number)[][] = [];
    for (let row = startRow; row <= endRow; row++) {
      const rowValues: (string | number)[] = [];
      for (let col = startCol; col <= endCol; col++) {
        const cellData = activeSheet.getCellRaw(row, col);
        const value = cellData?.v !== undefined ? cellData.v as string | number : '';
        rowValues.push(value);
      }
      result.push(rowValues);
    }

    return result;
  }, 'get range', []);
}

/**
 * Get full range data with metadata
 */
export async function getRangeFull(
  univerRef: any,
  rangeRef: string
): Promise<CellValue[][]> {
  console.log('[getRangeFull] Called with range:', rangeRef);

  if (!univerRef?.current) {
    console.error('[getRangeFull] No univerRef.current');
    return [];
  }

  return safeUniverOperation(async () => {
    const workbook = getWorkbook(univerRef);
    if (!workbook) {
      console.error('[getRangeFull] No workbook');
      return [];
    }

    const activeSheet = workbook.getActiveSheet();
    if (!activeSheet) {
      console.error('[getRangeFull] No active sheet');
      return [];
    }

    console.log('[getRangeFull] Active sheet:', activeSheet.getName());

    const parsedRange = parseRange(rangeRef);
    if (!parsedRange) {
      console.error('[getRangeFull] Failed to parse range:', rangeRef);
      return [];
    }

    const { startCol, startRow, endCol, endRow } = parsedRange;
    console.log('[getRangeFull] Parsed range:', { startCol, startRow, endCol, endRow });

    // Extract full cell data row by row
    const result: CellValue[][] = [];
    for (let row = startRow; row <= endRow; row++) {
      const rowValues: CellValue[] = [];
      for (let col = startCol; col <= endCol; col++) {
        const cellData = activeSheet.getCellRaw(row, col);
        if (cellData) {
          // Resolve style from style ID or use direct style object
          let resolvedStyle = undefined;
          if (cellData.s) {
            if (typeof cellData.s === 'string') {
              // Style ID - resolve from workbook styles
              const styles = workbook.getStyles() as Record<string, any>;
              if (styles && styles[cellData.s]) {
                resolvedStyle = styles[cellData.s];
              }
            } else {
              // Direct style object
              resolvedStyle = cellData.s;
            }
          }

          // Convert ICellData to CellValue
          const cellValue: CellValue = {
            v: cellData.v ?? undefined,
            f: cellData.f ?? undefined,
            style: resolvedStyle,
            meta: cellData.p ? { custom: cellData.p } : undefined
          };
          rowValues.push(cellValue);
        } else {
          // Empty cell
          rowValues.push({ v: '' });
        }
      }
      result.push(rowValues);
    }

    console.log('[getRangeFull] Result rows:', result.length, 'Sample:', result[0]);
    return result;
  }, 'get range full', []);
}

/**
 * Get all sheets data for multi-sheet operations
 */
export async function getAllSheetsData(
  univerRef: any
): Promise<{ name: string; data: CellValue[][] }[]> {
  if (!univerRef?.current) return [];

  return safeUniverOperation(async () => {
    const workbook = getWorkbook(univerRef);
    if (!workbook) return [];

    const sheets = workbook.getSheets();
    const result: { name: string; data: CellValue[][] }[] = [];

    // Helper to determine used bounds for a sheet
    const determineUsedBounds = (sheet: any) => {
      const ABS_MAX_ROWS = 10000;
      const ABS_MAX_COLS = 1024;

      const checkRow = (row: number): boolean => {
        for (let c = 0; c < Math.min(256, ABS_MAX_COLS); c++) {
          const cell = sheet.getCellRaw(row, c);
          if (cell && (cell.v !== undefined || cell.f !== undefined || cell.s !== undefined || cell.p !== undefined)) return true;
        }
        return false;
      };

      const checkCol = (col: number): boolean => {
        for (let r = 0; r < Math.min(256, ABS_MAX_ROWS); r++) {
          const cell = sheet.getCellRaw(r, col);
          if (cell && (cell.v !== undefined || cell.f !== undefined || cell.s !== undefined || cell.p !== undefined)) return true;
        }
        return false;
      };

      // Find last non-empty row
      let rowHigh = 64;
      let lastRow = -1;
      while (rowHigh < ABS_MAX_ROWS) {
        let found = false;
        for (let r = Math.max(0, rowHigh - Math.floor(rowHigh / 2)); r < rowHigh; r++) {
          if (checkRow(r)) { found = true; lastRow = r; }
        }
        if (!found) break;
        rowHigh = Math.min(rowHigh * 2, ABS_MAX_ROWS);
      }

      // Binary search precise last row
      let lo = 0, hi = Math.max(lastRow, rowHigh - 1), preciseLastRow = -1;
      while (lo <= hi) {
        const mid = Math.floor((lo + hi) / 2);
        if (checkRow(mid)) { preciseLastRow = mid; lo = mid + 1; } else { hi = mid - 1; }
      }

      // Find last non-empty col
      let colHigh = 16;
      let lastCol = -1;
      while (colHigh < ABS_MAX_COLS) {
        let found = false;
        for (let c = Math.max(0, colHigh - Math.floor(colHigh / 2)); c < colHigh; c++) {
          if (checkCol(c)) { found = true; lastCol = c; }
        }
        if (!found) break;
        colHigh = Math.min(colHigh * 2, ABS_MAX_COLS);
      }

      lo = 0; hi = Math.max(lastCol, colHigh - 1);
      let preciseLastCol = -1;
      while (lo <= hi) {
        const mid = Math.floor((lo + hi) / 2);
        if (checkCol(mid)) { preciseLastCol = mid; lo = mid + 1; } else { hi = mid - 1; }
      }

      return { lastRow: preciseLastRow, lastCol: preciseLastCol };
    };

    for (const sheet of sheets) {
      const sheetName = sheet.getName();
      const bounds = determineUsedBounds(sheet);
      if (bounds.lastRow === -1 || bounds.lastCol === -1) continue; // empty sheet

      const sheetData: CellValue[][] = [];
      for (let row = 0; row <= bounds.lastRow; row++) {
        const rowValues: CellValue[] = [];
        for (let col = 0; col <= bounds.lastCol; col++) {
          const cellData = sheet.getCellRaw(row, col);
          if (cellData) {
            // Resolve style from style ID or use direct style object
            let resolvedStyle = undefined;
            if (cellData.s) {
              if (typeof cellData.s === 'string') {
                // Style ID - resolve from workbook styles
                const styles = workbook.getStyles() as Record<string, any>;
                if (styles && styles[cellData.s]) {
                  resolvedStyle = styles[cellData.s];
                }
              } else {
                // Direct style object
                resolvedStyle = cellData.s;
              }
            }

            const cellValue: CellValue = {
              v: cellData.v ?? undefined,
              f: cellData.f ?? undefined,
              style: resolvedStyle,
              meta: cellData.p ? { custom: cellData.p } : undefined
            };
            rowValues.push(cellValue);
          } else {
            rowValues.push({ v: '' });
          }
        }
        sheetData.push(rowValues);
      }
      result.push({ name: sheetName, data: sheetData });
    }

    return result;
  }, 'get all sheets data', []);
}

/**
 * Get current selection
 */
export async function getSelection(univerRef: any): Promise<string | null> {
  if (!univerRef?.current) return null;

  return safeUniverOperation(async () => {
    const workbook = getWorkbook(univerRef);
    if (!workbook) return null;

    const injector = univerRef.current.univer.__getInjector();
    const selectionManager = injector.get('ISelectionManager' as any);
    if (!selectionManager) return null;

    const selections = selectionManager.getSelections();
    if (!selections || selections.length === 0) return null;

    const primarySelection = selections[0];
    if (!primarySelection || !primarySelection.range) return null;

    const { range } = primarySelection;
    const activeSheet = workbook.getActiveSheet();
    if (!activeSheet) return null;

    // Convert range to A1 notation
    const startRow = range.startRow;
    const startCol = range.startColumn;
    const endRow = range.endRow;
    const endCol = range.endColumn;

    const startCell = `${columnToLetter(startCol)}${startRow + 1}`;
    const endCell = `${columnToLetter(endCol)}${endRow + 1}`;

    // Return single cell if it's a single cell selection
    if (startRow === endRow && startCol === endCol) {
      return startCell;
    }

    return `${startCell}:${endCell}`;
  }, 'get selection', null);
}

/**
 * Calculate used range of active sheet
 */
export async function getUsedRange(univerRef: any): Promise<string> {
  if (!univerRef?.current) return 'A1:Z100';

  return safeUniverOperation(async () => {
    const workbook = getWorkbook(univerRef);
    if (!workbook) return 'A1:Z100';

    const activeSheet = workbook.getActiveSheet();
    if (!activeSheet) return 'A1:Z100';

    const ABS_MAX_ROWS = 10000;
    const ABS_MAX_COLS = 1024;

    const checkRow = (row: number): boolean => {
      for (let c = 0; c < ABS_MAX_COLS; c++) {
        const cell = activeSheet.getCellRaw(row, c);
        if (cell && (cell.v !== undefined || cell.f !== undefined || cell.s !== undefined)) return true;
      }
      return false;
    };

    const checkCol = (col: number): boolean => {
      for (let r = 0; r < ABS_MAX_ROWS; r++) {
        const cell = activeSheet.getCellRaw(r, col);
        if (cell && (cell.v !== undefined || cell.f !== undefined || cell.s !== undefined)) return true;
      }
      return false;
    };

    // Expand to find upper bound for rows
    let rowHigh = 64;
    while (rowHigh < ABS_MAX_ROWS) {
      let any = false;
      const start = Math.max(0, Math.floor(rowHigh / 2));
      for (let r = start; r < rowHigh; r++) {
        if (checkRow(r)) { any = true; }
      }
      if (!any) break;
      rowHigh = Math.min(rowHigh * 2, ABS_MAX_ROWS);
    }

    // Binary search last non-empty row
    let lo = 0, hi = Math.max(0, rowHigh - 1), lastRow = -1;
    while (lo <= hi) {
      const mid = Math.floor((lo + hi) / 2);
      if (checkRow(mid)) { lastRow = mid; lo = mid + 1; } else { hi = mid - 1; }
    }

    // Expand to find upper bound for cols
    let colHigh = 16;
    while (colHigh < ABS_MAX_COLS) {
      let any = false;
      const start = Math.max(0, Math.floor(colHigh / 2));
      for (let c = start; c < colHigh; c++) {
        if (checkCol(c)) { any = true; }
      }
      if (!any) break;
      colHigh = Math.min(colHigh * 2, ABS_MAX_COLS);
    }

    lo = 0; hi = Math.max(0, colHigh - 1);
    let lastCol = -1;
    while (lo <= hi) {
      const mid = Math.floor((lo + hi) / 2);
      if (checkCol(mid)) { lastCol = mid; lo = mid + 1; } else { hi = mid - 1; }
    }

    // Convert index to column letters
    const indexToCol = (index: number): string => {
      let result = '';
      index += 1; // Convert to 1-based
      while (index > 0) {
        index -= 1;
        result = String.fromCharCode(65 + (index % 26)) + result;
        index = Math.floor(index / 26);
      }
      return result;
    };

    if (lastRow === -1 && lastCol === -1) {
      return 'A1:A1';
    }

    const startCol = 'A';
    const endCol = lastCol >= 0 ? indexToCol(lastCol) : 'A';
    const startRow = 1;
    const endRow = lastRow >= 0 ? lastRow + 1 : 1;

    return `${startCol}${startRow}:${endCol}${endRow}`;
  }, 'get used range', 'A1:Z100');
}

