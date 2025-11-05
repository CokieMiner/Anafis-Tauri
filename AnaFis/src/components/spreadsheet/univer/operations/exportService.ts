// exportService.ts - Simplified export service
// Supports: CSV, TSV, TXT, Parquet, HTML, Markdown, TeX, AnaFisSpread
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { SpreadsheetRef } from '@/components/spreadsheet/SpreadsheetInterface';
import { safeSpreadsheetOperation, normalizeRangeRef, SpreadsheetValidationError, determineUsedRange } from '../index';
import type { ExportOptions, IExportService } from '@/types/export';
import { ERROR_MESSAGES } from '../utils/constants';

export type ExportFormat = 'csv' | 'tsv' | 'txt' | 'parquet' | 'html' | 'markdown' | 'tex' | 'anafispread';

export interface ExportResult {
  success: boolean;
  message?: string;
  error?: string;
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

export class ExportService implements IExportService {
  /**
   * Main export with file dialog
   */
  async exportWithDialog(options: ExportOptions, spreadsheetAPI: SpreadsheetRef): Promise<ExportResult> {
    try {
      const filter = FILE_FILTERS[options.format];
      const filePath = await save({
        filters: [filter],
        defaultPath: `export.${filter.extensions[0]}`,
      });

      if (!filePath) {
        return { success: false, message: 'Export cancelled' };
      }

      return this.exportToFile(filePath, options, spreadsheetAPI);
    } catch (error) {
      return {
        success: false,
        error: `Export failed: ${error instanceof Error ? error.message : String(error)}`
      };
    }
  }

  /**
   * Export to specific file path
   */
  async exportToFile(filePath: string, options: ExportOptions, spreadsheetAPI: SpreadsheetRef): Promise<ExportResult> {
    try {
      // Extract data
      const data = await this.extractData(options, spreadsheetAPI);
      if (!data) {
        throw new Error(ERROR_MESSAGES.NO_DATA_TO_EXPORT);
      }

      // For anafispread, pass workbook snapshot directly
      if (options.format === 'anafispread') {
        await invoke('export_anafispread', { data, filePath });
        return { success: true, message: `Successfully exported to ${filePath}` };
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

      return { success: true, message };
    } catch (error) {
      return {
        success: false,
        error: this.formatError(error)
      };
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
      const range = this.determineRange(options, spreadsheetAPI);
      const values = await spreadsheetAPI.getRange(range);
      return this.cleanData(values);
    }, 'data extraction');
  }

  /**
   * Determine export range using direct internal API access
   * No fallbacks - forces proper fixes when internal API access fails
   */
  private determineRange(options: ExportOptions, spreadsheetAPI: SpreadsheetRef): string {
    if (options.rangeMode === 'custom') {
      if (!options.customRange) {
        throw new Error(ERROR_MESSAGES.CUSTOM_RANGE_REQUIRED);
      }
      try {
        return normalizeRangeRef(options.customRange);
      } catch (error) {
        if (error instanceof SpreadsheetValidationError) {
          throw new Error(`Invalid range: ${error.message}`);
        }
        throw error;
      }
    }

    // Always use direct internal API access for accurate used range detection
    // No fallbacks to tracked bounds or heuristics - broken internal API access is immediately visible
    const context = spreadsheetAPI.getImplementationContext();
    if (!context.facadeInstance) {
      throw new Error('Internal API access required for determining used range - cannot use fallbacks');
    }

    try {
      return determineUsedRange(context.facadeInstance as ReturnType<typeof import('@univerjs/core/facade').FUniver.newAPI>);
    } catch (error) {
      // Fail fast - no graceful degradation that could hide real issues
      throw new Error(`Failed to determine used range via internal API: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Clean data: remove trailing empty rows/columns
   */
  private cleanData(data: unknown[][]): unknown[][] {
    if (!Array.isArray(data) || data.length === 0) {
      return [];
    }

    // Find last row and column with data
    let lastRow = -1;
    let lastCol = -1;

    for (let r = 0; r < data.length; r++) {
      if (!Array.isArray(data[r])) {continue;}
      const row = data[r] as unknown[];
      for (let c = 0; c < row.length; c++) {
        if (this.isNonEmpty(row[c])) {
          lastRow = Math.max(lastRow, r);
          lastCol = Math.max(lastCol, c);
        }
      }
    }

    if (lastRow === -1) {return [];}

    // Build rectangular result
    const result: unknown[][] = [];
    for (let r = 0; r <= lastRow; r++) {
      const sourceRow = Array.isArray(data[r]) ? (data[r] as unknown[]) : [];
      const row: unknown[] = [];
      for (let c = 0; c <= lastCol; c++) {
        row.push(c < sourceRow.length ? sourceRow[c] : null);
      }
      result.push(row);
    }

    return result;
  }

  /**
   * Check if cell is non-empty
   */
  private isNonEmpty(value: unknown): boolean {
    if (value === null || value === undefined) {return false;}
    if (typeof value === 'string') {return value.trim().length > 0;}
    return true;
  }

  /**
   * Format errors for display
   */
  private formatError(err: unknown): string {
    const msg = err instanceof Error ? err.message : String(err);

    if (msg.includes('Invalid range')) {return `Range error: ${msg}`;}
    if (msg.includes('permission') || msg.includes('denied')) {
      return 'Permission denied: Cannot write to file';
    }
    if (msg.includes('disk') || msg.includes('space')) {
      return 'Insufficient disk space';
    }
    if (msg.includes('timeout')) {
      return 'Export timed out - try smaller range';
    }

    return `Export failed: ${msg}`;
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
  }, spreadsheetAPI: SpreadsheetRef): Promise<ExportResult> {
    try {
      if (!options.libraryName.trim()) {
        throw new Error('Please enter a name');
      }

      if (!options.dataRange.trim()) {
        throw new Error('Please specify a data range');
      }

      // Validate ranges
      try {
        normalizeRangeRef(options.dataRange.trim());
        if (options.uncertaintyRange.trim()) {
          normalizeRangeRef(options.uncertaintyRange.trim());
        }
      } catch (error) {
        if (error instanceof SpreadsheetValidationError) {
          throw new Error(`Invalid range: ${error.message}`);
        }
        throw error;
      }

      // Extract data
      const dataExtraction = await this.extractRangeData(options.dataRange, spreadsheetAPI);
      if (dataExtraction.length === 0) {
        throw new Error(`No numeric data in range ${options.dataRange}`);
      }

      // Extract uncertainties if provided
      let uncertainties: number[] | undefined;
      if (options.uncertaintyRange.trim()) {
        uncertainties = await this.extractRangeData(options.uncertaintyRange, spreadsheetAPI);
        if (uncertainties.length !== dataExtraction.length) {
          throw new Error('Uncertainty range must match data range length');
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

      return {
        success: true,
        message: `Saved '${options.libraryName}' (${dataExtraction.length} points)`
      };
    } catch (error) {
      return {
        success: false,
        error: `Data Library save failed: ${error instanceof Error ? error.message : String(error)}`
      };
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
