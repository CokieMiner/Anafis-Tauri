// exportService.ts - Simplified export service
// Supports: CSV, TSV, TXT, Parquet, HTML, Markdown, TeX, AnaFisSpread
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import type { ExportOptions } from '@/core/types/export';
import { err, isErr, ok, type Result } from '@/core/types/result';
import type {
  SpreadsheetRef,
  WorkbookSnapshot,
} from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { safeSpreadsheetOperation } from '@/tabs/spreadsheet/univer';
import { ERROR_MESSAGES } from '@/tabs/spreadsheet/univer/utils/constants';
import {
  logError,
  normalizeError,
  SpreadsheetOperationError,
  SpreadsheetValidationError,
} from '@/tabs/spreadsheet/univer/utils/errors';
import { RangeValidator } from '@/tabs/spreadsheet/univer/utils/RangeValidator';

// Type definitions for better type safety
type CellValue = string | number | null;
type DataTable = CellValue[][];

export type ExportFormat =
  | 'csv'
  | 'tsv'
  | 'txt'
  | 'parquet'
  | 'html'
  | 'markdown'
  | 'tex'
  | 'anafispread';

export interface ExportResult {
  message?: string;
  filePath?: string;
}

export interface ExportError {
  message: string;
  code?: string;
}

// Simple file filter config
const FILE_FILTERS: Record<
  ExportFormat,
  { name: string; extensions: string[] }
> = {
  csv: { name: 'CSV Files', extensions: ['csv'] },
  tsv: { name: 'TSV Files', extensions: ['tsv'] },
  txt: { name: 'Text Files', extensions: ['txt'] },
  parquet: { name: 'Parquet Files', extensions: ['parquet'] },
  html: { name: 'HTML Files', extensions: ['html'] },
  markdown: { name: 'Markdown Files', extensions: ['md'] },
  tex: { name: 'LaTeX Files', extensions: ['tex'] },
  anafispread: { name: 'AnaFis Spreadsheet', extensions: ['anafispread'] },
};

export class ExportService implements ExportService {
  /**
   * Export spreadsheet data with a native file save dialog.
   *
   * Shows a system file dialog for the user to choose where to save the exported file,
   * then performs the export operation. Supports all export formats with appropriate
   * file filters and default extensions.
   *
   * @param options - Export configuration including format, range mode, and custom range
   * @param spreadsheetAPI - Reference to the spreadsheet API for data access
   * @returns Promise resolving to Result with export success details or error
   */
  async exportWithDialog(
    options: ExportOptions,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<Result<ExportResult, ExportError>> {
    try {
      const filter = FILE_FILTERS[options.format];
      const filePath = await save({
        filters: [filter],
        defaultPath: `export.${filter.extensions[0]}`,
      });

      if (!filePath) {
        return err({ message: 'Export cancelled' });
      }

      return this.exportToFile(filePath, options, spreadsheetAPI);
    } catch (error) {
      const spreadsheetError = normalizeError(error, 'exportWithDialog');
      logError(spreadsheetError);
      return err({
        message: spreadsheetError.message,
        code: spreadsheetError.code,
      });
    }
  }

  /**
   * Export spreadsheet data directly to a specified file path.
   *
   * Performs the complete export pipeline: data extraction, cleaning, and backend
   * processing. Handles special cases like AnaFisSpread format (workbook snapshots)
   * and standard formats (2D array data).
   *
   * @param filePath - Absolute path where the exported file will be saved
   * @param options - Export configuration including format, range mode, and custom range
   * @param spreadsheetAPI - Reference to the spreadsheet API for data access
   * @returns Promise resolving to Result with export success details or error
   */
  async exportToFile(
    filePath: string,
    options: ExportOptions,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<Result<ExportResult, ExportError>> {
    try {
      // Extract data
      const data = await this.extractData(options, spreadsheetAPI);
      if (!data) {
        return err({
          message: ERROR_MESSAGES.NO_DATA_TO_EXPORT,
          code: 'NO_DATA',
        });
      }

      // For anafispread, validate and pass workbook snapshot directly
      if (options.format === 'anafispread') {
        // Validate snapshot structure before export
        const validation = this.validateWorkbookSnapshot(data);
        if (isErr(validation)) {
          return err({ message: validation.error, code: 'INVALID_SNAPSHOT' });
        }

        await invoke('export_anafispread', { data, filePath });
        return ok({
          message: `Successfully exported to ${filePath}`,
          filePath,
        });
      }

      // For other formats, ensure we have 2D array data
      const arrayData = Array.isArray(data) ? data : [];
      const rowCount = arrayData.length;

      // Call backend
      await invoke('export_data', {
        data: arrayData,
        filePath,
        format: options.format,
        config: {
          delimiter: options.delimiter ?? ',',
        },
      });

      const message =
        rowCount > 0
          ? `Successfully exported ${rowCount} rows to ${filePath}`
          : `Successfully exported data to ${filePath}`;

      return ok({ message, filePath });
    } catch (error) {
      const spreadsheetError = normalizeError(error, 'exportToFile');
      logError(spreadsheetError);
      return err({
        message: spreadsheetError.message,
        code: spreadsheetError.code,
      });
    }
  }

  /**
   * Extract data from spreadsheet
   */
  private async extractData(
    options: ExportOptions,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<unknown> {
    return safeSpreadsheetOperation(async () => {
      // Special case: anafispread format needs full workbook snapshot
      if (options.format === 'anafispread') {
        return spreadsheetAPI.getWorkbookSnapshot();
      }

      // All other formats: get 2D array data
      const range = await this.determineRange(options, spreadsheetAPI);
      const values = await spreadsheetAPI.getRange(range);
      return this.cleanData(values);
    }, 'data extraction');
  }

  /**
   * Validate workbook snapshot structure before export
   * Prevents corrupted or malformed snapshots from being saved
   */
  private validateWorkbookSnapshot(
    snapshot: unknown
  ): Result<WorkbookSnapshot, string> {
    try {
      // Basic type check
      if (!snapshot || typeof snapshot !== 'object') {
        return err('Snapshot is not a valid object');
      }

      const obj = snapshot as Record<string, unknown>;

      // Required fields validation
      if (typeof obj.id !== 'string' || obj.id.trim() === '') {
        return err(
          'Snapshot missing required field: id (must be non-empty string)'
        );
      }

      if (typeof obj.name !== 'string' || obj.name.trim() === '') {
        return err(
          'Snapshot missing required field: name (must be non-empty string)'
        );
      }

      if (!obj.sheets || typeof obj.sheets !== 'object') {
        return err('Snapshot missing required field: sheets (must be object)');
      }

      const sheets = obj.sheets as Record<string, unknown>;
      const sheetIds = Object.keys(sheets);

      if (sheetIds.length === 0) {
        return err('Snapshot must contain at least one sheet');
      }

      // Validate each sheet
      for (const [sheetId, sheetData] of Object.entries(sheets)) {
        if (!sheetData || typeof sheetData !== 'object') {
          return err(`Sheet '${sheetId}' is not a valid object`);
        }

        const sheet = sheetData as Record<string, unknown>;

        if (typeof sheet.id !== 'string' || sheet.id.trim() === '') {
          return err(`Sheet '${sheetId}' missing required field: id`);
        }

        if (typeof sheet.name !== 'string' || sheet.name.trim() === '') {
          return err(`Sheet '${sheetId}' missing required field: name`);
        }

        // Optional fields validation (if present, must be valid)
        if (
          sheet.cellData !== undefined &&
          typeof sheet.cellData !== 'object'
        ) {
          return err(
            `Sheet '${sheetId}' has invalid cellData (must be object if present)`
          );
        }

        if (sheet.mergeData !== undefined && !Array.isArray(sheet.mergeData)) {
          return err(
            `Sheet '${sheetId}' has invalid mergeData (must be array if present)`
          );
        }

        if (
          sheet.rowCount !== undefined &&
          (typeof sheet.rowCount !== 'number' || sheet.rowCount < 0)
        ) {
          return err(
            `Sheet '${sheetId}' has invalid rowCount (must be non-negative number if present)`
          );
        }

        if (
          sheet.columnCount !== undefined &&
          (typeof sheet.columnCount !== 'number' || sheet.columnCount < 0)
        ) {
          return err(
            `Sheet '${sheetId}' has invalid columnCount (must be non-negative number if present)`
          );
        }
      }

      // Optional workbook-level fields validation
      if (obj.appVersion !== undefined && typeof obj.appVersion !== 'string') {
        return err(
          'Snapshot has invalid appVersion (must be string if present)'
        );
      }

      if (obj.locale !== undefined && typeof obj.locale !== 'string') {
        return err('Snapshot has invalid locale (must be string if present)');
      }

      if (obj.styles !== undefined && typeof obj.styles !== 'object') {
        return err('Snapshot has invalid styles (must be object if present)');
      }

      if (obj.sheetOrder !== undefined && !Array.isArray(obj.sheetOrder)) {
        return err(
          'Snapshot has invalid sheetOrder (must be array if present)'
        );
      }

      if (obj.resources !== undefined && !Array.isArray(obj.resources)) {
        return err('Snapshot has invalid resources (must be array if present)');
      }

      // If we get here, validation passed
      return ok(snapshot as WorkbookSnapshot);
    } catch (error) {
      return err(
        `Snapshot validation failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * Determine export range using direct internal API access
   * No fallbacks - forces proper fixes when internal API access fails
   */
  private async determineRange(
    options: ExportOptions,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<string> {
    if (options.rangeMode === 'custom') {
      if (!options.customRange) {
        throw new SpreadsheetValidationError(
          ERROR_MESSAGES.CUSTOM_RANGE_REQUIRED,
          'customRange',
          'determineRange',
          { rangeMode: options.rangeMode }
        );
      }
      try {
        RangeValidator.validateFormat(options.customRange);
        return options.customRange;
      } catch (error) {
        const originalError =
          error instanceof Error ? error : new Error(String(error));
        throw new SpreadsheetValidationError(
          `Invalid range: ${originalError.message}`,
          'customRange',
          'determineRange',
          { rangeMode: options.rangeMode, customRange: options.customRange }
        );
      }
    }

    // Always use direct internal API access for accurate used range detection
    // No fallbacks to tracked bounds or heuristics - broken internal API access is immediately visible
    try {
      return await spreadsheetAPI.getUsedRange();
    } catch (error) {
      const originalError =
        error instanceof Error ? error : new Error(String(error));
      throw new SpreadsheetOperationError('determineRange', originalError, {
        rangeMode: options.rangeMode,
        operation: 'getUsedRange',
      });
    }
  }

  /**
   * Clean data: remove trailing empty rows/columns
   * Single-pass algorithm: O(rows×cols) - mark data locations then build result
   */
  private cleanData(data: DataTable): DataTable {
    if (!Array.isArray(data) || data.length === 0) {
      return [];
    }

    let firstRow = -1;
    let lastRow = -1;
    const rowHasData: boolean[] = new Array<boolean>(data.length).fill(false);
    const colHasData: (boolean | undefined)[] = [];

    // Single pass: mark all rows and columns that contain data
    for (let r = 0; r < data.length; r++) {
      if (!Array.isArray(data[r])) {
        continue;
      }

      const row = data[r] as CellValue[];
      for (let c = 0; c < row.length; c++) {
        const value = row[c] ?? null;
        // Empty strings and whitespace are considered content
        if (value !== null) {
          if (firstRow === -1) {
            firstRow = r;
          }
          lastRow = Math.max(lastRow, r);
          rowHasData[r] = true;
          colHasData[c] = true;
        }
      }
    }

    if (firstRow === -1) {
      return [];
    }

    // Find actual last column with data (trim trailing empty columns)
    let actualLastCol = -1;
    for (let c = colHasData.length - 1; c >= 0; c--) {
      if (colHasData[c]) {
        actualLastCol = c;
        break;
      }
    }

    if (actualLastCol === -1) {
      return [];
    }

    // Build result: include only rows from firstRow to lastRow, columns 0 to actualLastCol
    const result: DataTable = [];
    for (let r = firstRow; r <= lastRow; r++) {
      if (!Array.isArray(data[r])) {
        // Pad missing rows with nulls
        result.push(new Array(actualLastCol + 1).fill(null) as CellValue[]);
        continue;
      }

      const sourceRow = data[r] as CellValue[];
      const cleanRow: CellValue[] = [];

      // Include all columns up to actualLastCol, padding with nulls if necessary
      for (let c = 0; c <= actualLastCol; c++) {
        const value = c < sourceRow.length ? (sourceRow[c] ?? null) : null;
        cleanRow.push(value);
      }

      result.push(cleanRow);
    }

    return result;
  } /**
   * Export spreadsheet data to the AnaFis Data Library.
   *
   * Extracts numeric data and uncertainties from specified ranges, validates the data,
   * and saves it as a named sequence in the Data Library with metadata like tags,
   * units, and description.
   *
   * @param options - Data Library export configuration including name, description, ranges, etc.
   * @param spreadsheetAPI - Reference to the spreadsheet API for data access
   * @returns Promise resolving to Result with save confirmation or error
   */
  async exportToDataLibrary(
    options: {
      libraryName: string;
      libraryDescription: string;
      libraryTags: string;
      libraryUnit: string;
      dataRange: string;
      uncertaintyRange: string;
    },
    spreadsheetAPI: SpreadsheetRef
  ): Promise<Result<ExportResult, ExportError>> {
    try {
      if (!options.libraryName.trim()) {
        return err({ message: 'Please enter a name', code: 'INVALID_INPUT' });
      }

      if (!options.dataRange.trim()) {
        return err({
          message: 'Please specify a data range',
          code: 'INVALID_INPUT',
        });
      }

      // Validate ranges
      try {
        RangeValidator.validateFormat(options.dataRange.trim());
        if (options.uncertaintyRange.trim()) {
          RangeValidator.validateFormat(options.uncertaintyRange.trim());
        }
      } catch (error) {
        return err({
          message: `Invalid range: ${error instanceof Error ? error.message : String(error)}`,
          code: 'INVALID_RANGE',
        });
      }

      // Extract data
      const dataExtraction = await this.extractRangeData(
        options.dataRange,
        spreadsheetAPI
      );
      if (dataExtraction.length === 0) {
        return err({
          message: `No numeric data in range ${options.dataRange}`,
          code: 'NO_DATA',
        });
      }

      // Extract uncertainties if provided
      let uncertainties: number[] | undefined;
      if (options.uncertaintyRange.trim()) {
        uncertainties = await this.extractRangeData(
          options.uncertaintyRange,
          spreadsheetAPI
        );
        if (uncertainties.length !== dataExtraction.length) {
          return err({
            message: 'Uncertainty range must match data range length',
            code: 'RANGE_MISMATCH',
          });
        }
      }

      // Save to library
      const tags = options.libraryTags
        .split(',')
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

      await invoke('save_sequence', {
        request: {
          name: options.libraryName.trim(),
          description: options.libraryDescription.trim(),
          tags,
          unit: options.libraryUnit.trim(),
          source: `Range: ${options.dataRange}${options.uncertaintyRange ? ` (unc: ${options.uncertaintyRange})` : ''}`,
          data: dataExtraction,
          uncertainties: uncertainties ?? null,
          is_pinned: false,
        },
      });

      return ok({
        message: `Saved '${options.libraryName}' (${dataExtraction.length} points)`,
      });
    } catch (error) {
      const spreadsheetError = normalizeError(error, 'exportToDataLibrary');
      logError(spreadsheetError);
      return err({
        message: spreadsheetError.message,
        code: spreadsheetError.code,
      });
    }
  }

  /**
   * Extract numeric data from range
   */
  private async extractRangeData(
    range: string,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<number[]> {
    return safeSpreadsheetOperation(async () => {
      const data = await spreadsheetAPI.getRange(range);
      const numbers: number[] = [];

      for (const row of data) {
        if (!Array.isArray(row)) {
          continue;
        }
        for (const cell of row) {
          if (cell === '') {
            continue;
          }

          let num: number;
          if (typeof cell === 'number') {
            num = cell;
          } else {
            const str = String(cell).trim();
            if (str === '') {
              continue;
            }
            num = parseFloat(str);
          }

          if (!Number.isNaN(num) && Number.isFinite(num)) {
            numbers.push(num);
          }
        }
      }

      return numbers;
    }, 'numeric data extraction');
  }
}
