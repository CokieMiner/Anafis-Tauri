// exportService.ts - Simplified export service
// Supports: CSV, TSV, TXT, Parquet, HTML, Markdown, TeX, AnaFisSpread
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { safeSpreadsheetOperation, formatSpreadsheetError } from '@/tabs/spreadsheet/univer';
import { RangeValidator } from '@/tabs/spreadsheet/univer/utils/RangeValidator';
import type { ExportOptions } from '@/core/types/export';
import { ERROR_MESSAGES } from '@/tabs/spreadsheet/univer/utils/constants';
import { Result, ok, err } from '@/core/types/result';

export type ExportFormat = 'csv' | 'tsv' | 'txt' | 'parquet' | 'html' | 'markdown' | 'tex' | 'anafispread';

export interface ExportResult {
  message?: string;
  filePath?: string;
}

export interface ExportError {
  message: string;
  code?: string;
}

// Simple file filter config
const FILE_FILTERS: Record<ExportFormat, { name: string; extensions: string[] }> = {
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
   * Main export with file dialog
   */
  async exportWithDialog(options: ExportOptions, spreadsheetAPI: SpreadsheetRef): Promise<Result<ExportResult, ExportError>> {
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
      return err({
        message: `Export failed: ${error instanceof Error ? error.message : String(error)}`,
        code: 'DIALOG_ERROR'
      });
    }
  }

  /**
   * Export to specific file path
   */
  async exportToFile(filePath: string, options: ExportOptions, spreadsheetAPI: SpreadsheetRef): Promise<Result<ExportResult, ExportError>> {
    try {
      // Extract data
      const data = await this.extractData(options, spreadsheetAPI);
      if (!data) {
        return err({ message: ERROR_MESSAGES.NO_DATA_TO_EXPORT, code: 'NO_DATA' });
      }

      // For anafispread, pass workbook snapshot directly
      if (options.format === 'anafispread') {
        await invoke('export_anafispread', { data, filePath });
        return ok({ message: `Successfully exported to ${filePath}`, filePath });
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
        }
      });

      const message = rowCount > 0 
        ? `Successfully exported ${rowCount} rows to ${filePath}`
        : `Successfully exported data to ${filePath}`;

      return ok({ message, filePath });
    } catch (error) {
      return err({
        message: this.formatError(error),
        code: 'EXPORT_FAILED'
      });
    }
  }

  /**
   * Extract data from spreadsheet
   */
  private async extractData(options: ExportOptions, spreadsheetAPI: SpreadsheetRef): Promise<unknown> {
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
   * Determine export range using direct internal API access
   * No fallbacks - forces proper fixes when internal API access fails
   */
  private async determineRange(options: ExportOptions, spreadsheetAPI: SpreadsheetRef): Promise<string> {
    if (options.rangeMode === 'custom') {
      if (!options.customRange) {
        throw new Error(ERROR_MESSAGES.CUSTOM_RANGE_REQUIRED);
      }
      try {
        RangeValidator.validateFormat(options.customRange);
        return options.customRange;
      } catch (error) {
        throw new Error(`Invalid range: ${error instanceof Error ? error.message : String(error)}`);
      }
    }

    // Always use direct internal API access for accurate used range detection
    // No fallbacks to tracked bounds or heuristics - broken internal API access is immediately visible
    try {
      return await spreadsheetAPI.getUsedRange();
    } catch (error) {
      // Fail fast - no graceful degradation that could hide real issues
      throw new Error(`Failed to determine used range via internal API: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Clean data: remove trailing empty rows/columns
   * Single-pass algorithm: O(rows×cols) instead of O(2×rows×cols)
   */
  private cleanData(data: (string | number | null)[][]): (string | number | null)[][] {
    if (!Array.isArray(data) || data.length === 0) {
      return [];
    }

    let lastRow = -1;
    let lastCol = -1;
    const result: (string | number | null)[][] = [];

    // Single pass: find bounds AND build result simultaneously
    for (let r = 0; r < data.length; r++) {
      if (!Array.isArray(data[r])) { continue; }
      const sourceRow = data[r] as (string | number | null)[];

      let rowLastCol = -1;
      const cleanRow: (string | number | null)[] = [];

      for (let c = 0; c < sourceRow.length; c++) {
        const value = sourceRow[c] ?? null;
        cleanRow.push(value);

        if (this.isNonEmpty(value)) {
          rowLastCol = c;
          lastRow = r;
        }
      }

      // Update global lastCol if this row extends further
      if (rowLastCol > lastCol) {
        lastCol = rowLastCol;
      }

      result.push(cleanRow);
    }

    if (lastRow === -1) { return []; }

    // Single slice at the end to trim empty rows and columns
    // Optimize padding to avoid unnecessary array copying
    const finalLastCol = lastCol;
    return result
      .slice(0, lastRow + 1)
      .map(row => {
        if (row.length > finalLastCol + 1) {
          // Row is longer than needed, slice it
          return row.slice(0, finalLastCol + 1);
        } else if (row.length < finalLastCol + 1) {
          // Row is shorter, pad with nulls
          const padded = [...row];
          while (padded.length <= finalLastCol) {
            padded.push(null);
          }
          return padded;
        } else {
          // Row is exactly the right length
          return row;
        }
      });
  }

  /**
   * Check if cell is non-empty
   */
  private isNonEmpty(value: string | number | null): boolean {
    if (value === null) {return false;}
    if (typeof value === 'string') {return value.trim().length > 0;}
    return true;
  }

  /**
   * Format errors for display
   */
  private formatError(err: unknown): string {
    return formatSpreadsheetError(err, 'export');
  }

  /**
   * Export to Data Library
   */
  async exportToDataLibrary(options: {
    libraryName: string;
    libraryDescription: string;
    libraryTags: string;
    libraryUnit: string;
    dataRange: string;
    uncertaintyRange: string;
  }, spreadsheetAPI: SpreadsheetRef): Promise<Result<ExportResult, ExportError>> {
    try {
      if (!options.libraryName.trim()) {
        return err({ message: 'Please enter a name', code: 'INVALID_INPUT' });
      }

      if (!options.dataRange.trim()) {
        return err({ message: 'Please specify a data range', code: 'INVALID_INPUT' });
      }

      // Validate ranges
      try {
        RangeValidator.validateFormat(options.dataRange.trim());
        if (options.uncertaintyRange.trim()) {
          RangeValidator.validateFormat(options.uncertaintyRange.trim());
        }
      } catch (error) {
        return err({ message: `Invalid range: ${error instanceof Error ? error.message : String(error)}`, code: 'INVALID_RANGE' });
      }

      // Extract data
      const dataExtraction = await this.extractRangeData(options.dataRange, spreadsheetAPI);
      if (dataExtraction.length === 0) {
        return err({ message: `No numeric data in range ${options.dataRange}`, code: 'NO_DATA' });
      }

      // Extract uncertainties if provided
      let uncertainties: number[] | undefined;
      if (options.uncertaintyRange.trim()) {
        uncertainties = await this.extractRangeData(options.uncertaintyRange, spreadsheetAPI);
        if (uncertainties.length !== dataExtraction.length) {
          return err({ message: 'Uncertainty range must match data range length', code: 'RANGE_MISMATCH' });
        }
      }

      // Save to library
      const tags = options.libraryTags
        .split(',')
        .map(t => t.trim())
        .filter(t => t.length > 0);

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
        }
      });

      return ok({
        message: `Saved '${options.libraryName}' (${dataExtraction.length} points)`
      });
    } catch (error) {
      return err({
        message: `Data Library save failed: ${error instanceof Error ? error.message : String(error)}`,
        code: 'LIBRARY_SAVE_FAILED'
      });
    }
  }

  /**
   * Extract numeric data from range
   */
  private async extractRangeData(range: string, spreadsheetAPI: SpreadsheetRef): Promise<number[]> {
    return safeSpreadsheetOperation(async () => {
      const data = await spreadsheetAPI.getRange(range);
      const numbers: number[] = [];

      for (const row of data) {
        if (!Array.isArray(row)) {continue;}
        for (const cell of row) {
          if (cell === '') {continue;}

          let num: number;
          if (typeof cell === 'number') {
            num = cell;
          } else {
            const str = String(cell).trim();
            if (str === '') {continue;}
            num = parseFloat(str);
          }

          if (!isNaN(num) && isFinite(num)) {
            numbers.push(num);
          }
        }
      }

      return numbers;
    }, 'numeric data extraction');
  }
}

export function getFileExtension(format: ExportFormat): string {
  return FILE_FILTERS[format].extensions[0] ?? 'txt';
}

export function getFilterName(format: ExportFormat): string {
  return FILE_FILTERS[format].name;
}
