// facadeOperations.ts - Core spreadsheet operations using Facade API exclusively
import { CellValue } from '@/components/spreadsheet/SpreadsheetInterface';
import { safeSpreadsheetOperation } from '../index';
import type { FUniver } from '@univerjs/core/facade';
import type { FWorkbook } from '@univerjs/sheets/facade';
import { ERROR_MESSAGES } from '../utils/constants';

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
  return safeSpreadsheetOperation(() => {
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
  return safeSpreadsheetOperation(() => {
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
  return safeSpreadsheetOperation(() => {
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
 * Convert unknown cell data to safe CellValue with type guards
 */
function convertToCellValue(cell: unknown): CellValue {
  // Type guard: ensure cell is an object
  if (!cell || typeof cell !== 'object') {
    return { v: null };
  }

  const cellData = cell as Record<string, unknown>;
  const cellValue = cellData.v;

  // Type guard for cell value: must be string, number, boolean, or null
  const isValidValue = (val: unknown): val is string | number | boolean | null => {
    return val === null || typeof val === 'string' || typeof val === 'number' || typeof val === 'boolean';
  };

  const result: CellValue = {
    v: isValidValue(cellValue) ? cellValue : null
  };

  // Only set optional fields if they exist and are valid
  if (typeof cellData.f === 'string') {
    result.f = cellData.f;
  }
  if (cellData.s !== undefined) {
    result.style = cellData.s;
  }
  if (cellData.p !== undefined) {
    result.meta = { custom: cellData.p };
  }

  return result;
}

/**
 * Get range with full cell data using Facade API
 */
export async function getRangeFull(univerRef: UniverRef, rangeRef: string): Promise<CellValue[][]> {
  return safeSpreadsheetOperation(() => {
    const workbook = univerRef.current?.getActiveWorkbook();
    if (!workbook) {
      throw new Error(ERROR_MESSAGES.NO_ACTIVE_WORKBOOK);
    }

    const sheet = workbook.getActiveSheet();

    const range = sheet.getRange(rangeRef);

    const cellDatas = range.getCellDatas();

    // Convert ICellData[][] to CellValue[][] using safe conversion
    return cellDatas.map((row: unknown[]) =>
      row.map(convertToCellValue)
    );
  }, 'get range full', []);
}

/**
 * Get current selection using Facade API
 */
export async function getSelection(univerRef: UniverRef): Promise<string | null> {
  return safeSpreadsheetOperation(() => {
    const workbook = univerRef.current?.getActiveWorkbook();
    if (!workbook) {
      return null;
    }

    const sheet = workbook.getActiveSheet();

    const selection = sheet.getSelection();

    // Get the active range from selection
    const activeRange = selection!.getActiveRange();

    return activeRange!.getA1Notation();
  }, 'get selection');
}
