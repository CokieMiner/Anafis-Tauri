// facade.ts - Simplified facade wrapper using Facade API exclusively
import { FUniver } from '@univerjs/core/facade';
import {
  ExportService,
  type ExportResult,
  type ExportFormat
} from './exportService';
import { extractFormattedTable, type ExtractionOptions, type FormattedTable } from './tableFormatExtraction';
import {
  getWorkbook,
  updateCell,
  getCellValue,
  getRange,
  type UniverRef
} from './spreadsheetOperations';
import type { ExportOptions } from '../../../types/export';
import type { FWorkbook } from '@univerjs/sheets/facade';

/**
 * Simplified facade interface for spreadsheet operations
 */
export interface ISpreadsheetFacade {
  // Core operations
  getWorkbook(): Promise<FWorkbook | null>;
  updateCell(range: string, value: { v?: string | number; f?: string }): Promise<void>;
  getCellValue(range: string): Promise<string | number | null>;
  getRange(range: string): Promise<(string | number)[][]>;

  // Export operations
  exportToFile(options: ExportOptions): Promise<ExportResult>;
  exportToMultiple(formats: ExportFormat[], options: ExportOptions): Promise<ExportResult>;

  // Data extraction
  extractFormattedTable(options: ExtractionOptions): Promise<FormattedTable>;
}

/**
 * Simplified spreadsheet facade implementation using Facade API
 */
export class SpreadsheetFacade implements ISpreadsheetFacade {
  private univerAPI: ReturnType<typeof FUniver.newAPI>;

  constructor(univerAPI: ReturnType<typeof FUniver.newAPI>) {
    this.univerAPI = univerAPI;
  }

  // Core operations
  getWorkbook(): Promise<FWorkbook | null> {
    return Promise.resolve(getWorkbook({ current: this.univerAPI } as UniverRef));
  }

  async updateCell(range: string, value: { v?: string | number; f?: string }): Promise<void> {
    return updateCell({ current: this.univerAPI } as UniverRef, range, value);
  }

  async getCellValue(range: string): Promise<string | number | null> {
    return getCellValue({ current: this.univerAPI } as UniverRef, range);
  }

  async getRange(range: string): Promise<(string | number)[][]> {
    return getRange({ current: this.univerAPI } as UniverRef, range);
  }

  // Export operations
  async exportToFile(options: ExportOptions): Promise<ExportResult> {
    const exportService = new ExportService();
    return exportService.exportWithDialog(options, this.univerAPI);
  }

  async exportToMultiple(formats: ExportFormat[], options: ExportOptions): Promise<ExportResult> {
    const exportService = new ExportService();
    // Export to multiple formats sequentially
    for (const format of formats) {
      const formatOptions = { ...options, exportFormat: format };
      const result = await exportService.exportWithDialog(formatOptions, this.univerAPI);
      if (!result.success) {
        return result; // Return on first failure
      }
    }
    return { success: true, message: `Successfully exported to ${formats.length} formats` };
  }

  // Data extraction
  extractFormattedTable(options: ExtractionOptions): Promise<FormattedTable> {
    return Promise.resolve(extractFormattedTable(this.univerAPI, options));
  }

  /**
   * Get the underlying Univer API instance
   */
  getUniverAPI(): ReturnType<typeof FUniver.newAPI> {
    return this.univerAPI;
  }
}

/**
 * Create a spreadsheet facade instance
 */
export function createSpreadsheetFacade(univerAPI: ReturnType<typeof FUniver.newAPI>): SpreadsheetFacade {
  return new SpreadsheetFacade(univerAPI);
}

/**
 * Global facade instance
 */
let globalFacade: SpreadsheetFacade | null = null;

/**
 * Warn if attempting to initialize with a different Univer API instance
 */
function warnIfDifferentAPI(univerAPI: ReturnType<typeof FUniver.newAPI>): void {
  if (globalFacade && globalFacade.getUniverAPI() !== univerAPI) {
    console.warn(
      'Attempting to initialize spreadsheet facade with a different Univer API instance. ' +
      'The existing facade will be replaced. If this is intentional, consider calling resetFacade() first for clarity.'
    );
  }
}

/**
 * Get or create the global facade instance
 */
export function getSpreadsheetFacade(univerAPI?: ReturnType<typeof FUniver.newAPI>): SpreadsheetFacade | null {
  if (univerAPI) {
    // Check if we already have a facade with a different API
    warnIfDifferentAPI(univerAPI);

    // Create or replace the facade with the provided API
    if (!globalFacade || globalFacade.getUniverAPI() !== univerAPI) {
      globalFacade = new SpreadsheetFacade(univerAPI);
    }
  }
  return globalFacade;
}

/**
 * Initialize the global facade
 */
export function initializeFacade(univerAPI: ReturnType<typeof FUniver.newAPI>): void {
  // Check if we already have a facade with the same API - if so, do nothing
  if (globalFacade && globalFacade.getUniverAPI() === univerAPI) {
    return;
  }

  // Check if we already have a facade with a different API
  warnIfDifferentAPI(univerAPI);

  globalFacade = new SpreadsheetFacade(univerAPI);
}

/**
 * Reset the global facade
 */
export function resetFacade(): void {
  globalFacade = null;
}