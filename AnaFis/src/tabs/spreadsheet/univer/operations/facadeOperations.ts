// facadeOperations.ts - Core spreadsheet operations using Facade API exclusively
import { CellValue } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { safeSpreadsheetOperation, safeSpreadsheetOperationSync } from '@/tabs/spreadsheet/univer';
import type { FUniver } from '@univerjs/core/facade';
import type { FWorkbook } from '@univerjs/sheets/facade';
import { ERROR_MESSAGES } from '@/tabs/spreadsheet/univer/utils/constants';
import { convertFromUniverCellData } from '@/tabs/spreadsheet/univer/utils/dataConversion';
import {
  SpreadsheetError,
  SpreadsheetErrorCode,
  ErrorCategory,
  ErrorSeverity,
  normalizeError,
  logError
} from '@/tabs/spreadsheet/univer/utils/errors';

// Type for the univer API reference
export type UniverRef = { current: ReturnType<typeof FUniver.newAPI> | null };

/**
 * Get workbook instance from Facade API
 */
export function getWorkbook(univerRef: UniverRef): FWorkbook | null {
  if (!univerRef.current) {
    const error = new SpreadsheetError(
      'Univer reference is null',
      SpreadsheetErrorCode.SPREADSHEET_NOT_READY,
      ErrorCategory.SYSTEM,
      ErrorSeverity.HIGH,
      { operation: 'getWorkbook' }
    );
    logError(error);
    return null;
  }

  // Use Facade API exclusively
  try {
    return univerRef.current.getActiveWorkbook();
  } catch (error) {
    const spreadsheetError = normalizeError(error, 'getWorkbook');
    logError(spreadsheetError);
    return null;
  }
}

/**
 * Update a single cell using Facade API.
 *
 * Sets the value and/or formula of a cell identified by A1 notation reference.
 * Both value and formula can be set simultaneously. The operation is wrapped
 * in error handling and retry logic.
 *
 * @param univerRef - Reference to the Univer Facade API instance
 * @param cellRef - Cell reference in A1 notation (e.g., "A1", "B2:C10")
 * @param value - Object containing optional value (v) and formula (f) properties
 * @returns void (synchronous operation)
 * @throws {Error} If the spreadsheet operation fails after retries
 */
export function updateCell(
  univerRef: UniverRef,
  cellRef: string,
  value: { v?: string | number; f?: string }
): void {
  return safeSpreadsheetOperationSync(() => {
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
 * Get cell value using Facade API.
 *
 * Retrieves the value of a single cell identified by A1 notation reference.
 * Boolean values are converted to strings for consistency. The operation
 * is wrapped in error handling.
 *
 * @param univerRef - Reference to the Univer Facade API instance
 * @param cellRef - Cell reference in A1 notation (e.g., "A1")
 * @returns The cell value (string, number, or null)
 * @throws {Error} If the spreadsheet operation fails
 */
export function getCellValue(univerRef: UniverRef, cellRef: string): string | number | null {
  return safeSpreadsheetOperationSync(() => {
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
 * Get range values using Facade API.
 *
 * Retrieves a 2D array of values from a range identified by A1 notation reference.
 * Boolean values are converted to strings and null/undefined values are handled
 * for consistency. The operation is wrapped in error handling and retry logic.
 *
 * @param univerRef - Reference to the Univer Facade API instance
 * @param rangeRef - Range reference in A1 notation (e.g., "A1:C10")
 * @returns Promise resolving to 2D array of cell values
 * @throws {Error} If the spreadsheet operation fails after retries
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
  // Use centralized conversion function instead of manual conversion
  if (!cell || typeof cell !== 'object') {
    return { v: null };
  }
  return convertFromUniverCellData(cell as import('@univerjs/core').ICellData);
}

/**
 * Get range with full cell data using Facade API.
 *
 * Retrieves a 2D array of complete CellValue objects from a range, including
 * values, formulas, styles, and metadata. This provides richer data than
 * getRange() which only returns display values.
 *
 * @param univerRef - Reference to the Univer Facade API instance
 * @param rangeRef - Range reference in A1 notation (e.g., "A1:C10")
 * @returns Promise resolving to 2D array of CellValue objects
 * @throws {Error} If the spreadsheet operation fails after retries
 */
export async function getRangeFull(univerRef: UniverRef, rangeRef: string): Promise<CellValue[][]> {
  return safeSpreadsheetOperation(() => {
    const workbook = univerRef.current?.getActiveWorkbook();
    if (!workbook) {
      throw new SpreadsheetError(
        ERROR_MESSAGES.NO_ACTIVE_WORKBOOK,
        SpreadsheetErrorCode.SPREADSHEET_NOT_READY,
        ErrorCategory.SYSTEM,
        ErrorSeverity.HIGH,
        { operation: 'getRangeFull', context: { rangeRef } }
      );
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
 * Get current selection using Facade API.
 *
 * Retrieves the currently selected range in A1 notation from the active sheet.
 * Returns null if no workbook is available or no selection exists.
 *
 * @param univerRef - Reference to the Univer Facade API instance
 * @returns Selected range in A1 notation or null
 * @throws {Error} If the spreadsheet operation fails
 */
export function getSelection(univerRef: UniverRef): string | null {
  return safeSpreadsheetOperationSync(() => {
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
