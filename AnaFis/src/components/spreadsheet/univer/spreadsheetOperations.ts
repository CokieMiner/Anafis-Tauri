// spreadsheetOperations.ts - Core spreadsheet operations using Facade API exclusively
import { CellValue } from '../SpreadsheetInterface';
import { safeUniverOperation } from './errors';
import type { FUniver } from '@univerjs/core/facade';
import type { FWorkbook } from '@univerjs/sheets/facade';

// Type for the univer API reference
export type UniverRef = { current: ReturnType<typeof FUniver.newAPI> | null };

/**
 * Get workbook instance from Facade API
 */
export function getWorkbook(univerRef: UniverRef): FWorkbook | null {
  if (!univerRef.current) {
    console.error('[getWorkbook] univerRef.current is null');
    return null;
  }

  // Use Facade API exclusively
  try {
    return univerRef.current.getActiveWorkbook();
  } catch (error) {
    console.error('[getWorkbook] Facade API failed:', error);
    return null;
  }
}

/**
 * Update a single cell using Facade API
 */
export async function updateCell(
  univerRef: UniverRef,
  cellRef: string,
  value: { v?: string | number; f?: string }
): Promise<void> {
  return safeUniverOperation(() => {
    const workbook = univerRef.current!.getActiveWorkbook()!;
    const sheet = workbook.getActiveSheet();
    const range = sheet.getRange(cellRef);
    if (value.v !== undefined) {
      range.setValue(value.v);
    }
    if (value.f !== undefined) {
      range.setFormula(value.f);
    }
  }, 'update cell');
}

/**
 * Get cell value using Facade API
 */
export async function getCellValue(univerRef: UniverRef, cellRef: string): Promise<string | number | null> {
  return safeUniverOperation(() => {
    const workbook = univerRef.current!.getActiveWorkbook()!;
    const sheet = workbook.getActiveSheet();
    const range = sheet.getRange(cellRef);
    const value = range.getValue();

    // Convert boolean values to strings for consistency
    if (typeof value === 'boolean') {
      return value.toString();
    }

    return value;
  }, 'get cell value');
}

/**
 * Get range values using Facade API
 */
export async function getRange(univerRef: UniverRef, rangeRef: string): Promise<(string | number)[][]> {
  return safeUniverOperation(() => {
    const workbook = univerRef.current!.getActiveWorkbook()!;
    const sheet = workbook.getActiveSheet();
    const range = sheet.getRange(rangeRef);
    const values = range.getValues();

    // Convert any boolean values to strings and handle null/undefined
    return values.map((row: unknown[]) =>
      row.map((cell: unknown) => {
        if (cell === null || cell === undefined) {return '';}
        if (typeof cell === 'boolean') {return cell.toString();}
        return cell as string | number;
      })
    );
  }, 'get range', []);
}

/**
 * Get range with full cell data using Facade API
 */
export async function getRangeFull(univerRef: UniverRef, rangeRef: string): Promise<CellValue[][]> {
  return safeUniverOperation(() => {
    const workbook = univerRef.current!.getActiveWorkbook()!;
    const sheet = workbook.getActiveSheet();
    const range = sheet.getRange(rangeRef);
    const cellDatas = range.getCellDatas();

    // Convert ICellData[][] to CellValue[][]
    return cellDatas.map((row: unknown[]) =>
      row.map((cell: unknown) => {
        const cellData = cell as { v?: string | number | boolean | null; f?: string; s?: unknown; p?: unknown };
        const result: CellValue = {
          v: cellData.v ?? ''
        };
        if (cellData.f) {result.f = cellData.f;}
        if (cellData.s) {result.style = cellData.s;}
        if (cellData.p) {result.meta = { custom: cellData.p };}
        return result;
      })
    );
  }, 'get range full', []);
}

/**
 * Get all sheets data using Facade API
 */
export async function getAllSheetsData(univerRef: UniverRef): Promise<{ name: string; data: CellValue[][] }[]> {
  // NOTE: Should always be available when called from interactive sidebars
  return safeUniverOperation(() => {
    const workbook = univerRef.current!.getActiveWorkbook()!;
    const sheets = workbook.getSheets();
    const result: { name: string; data: CellValue[][] }[] = [];

    for (const sheet of sheets) {
      // For simplicity, get a reasonable range (A1:Z100)
      const range = sheet.getRange('A1:Z100');
      const cellDatas = range.getCellDatas();
      result.push({
        name: sheet.getSheetName(),
        data: cellDatas.map((row: unknown[]) =>
          row.map((cell: unknown) => {
            const cellData = cell as { v?: string | number | boolean | null; f?: string; s?: unknown; p?: unknown };
            const result: CellValue = {
              v: cellData.v ?? ''
            };
            if (cellData.f) {result.f = cellData.f;}
            if (cellData.s) {result.style = cellData.s;}
            if (cellData.p) {result.meta = { custom: cellData.p };}
            return result;
          })
        )
      });
    }

    return result;
  }, 'get all sheets data', []);
}

/**
 * Get current selection using Facade API
 */
export async function getSelection(univerRef: UniverRef): Promise<string | null> {
  return safeUniverOperation(() => {
    const workbook = univerRef.current!.getActiveWorkbook()!;
    const sheet = workbook.getActiveSheet();
    const selection = sheet.getSelection()!;
    // Get the active range from selection
    const activeRange = selection.getActiveRange();
    return activeRange!.getA1Notation();
  }, 'get selection');
}
