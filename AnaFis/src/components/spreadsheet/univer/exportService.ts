// exportService.ts - Backend-integrated export service using Facade API
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { FUniver } from '@univerjs/core/facade';
import {
  extractFormattedTable,
  type ExtractionOptions
} from './tableFormatExtraction';
import { columnToLetter } from './univerUtils';
import { safeUniverOperation, normalizeRangeRef, UniverValidationError } from './errors';
import type { ExportOptions } from '../../../types/export';

/**
 * Interface for Univer sheet objects (from Facade API)
 */
interface UniverSheet {
  getRange: (range: string) => {
    getValues: () => unknown[][];
    getCellDatas?: () => unknown[][];
  };
  getSheetId?: () => string;
  getName?: () => string;
}

/**
 * Multi-sheet data structure for backend
 */
interface SheetData {
  name: string;
  data: unknown[][];
}

/**
 * Interface for validating sheet objects
 */
interface SheetObject {
  name: string;
  data: unknown;
}

/**
 * Interface for multi-sheet JSON format
 */
interface MultiSheetObject {
  _multiSheet: true;
  data: Record<string, unknown>;
}

/**
 * Export format types (matching backend enum)
 */
export type ExportFormat =
  | 'csv'
  | 'tsv'
  | 'txt'
  | 'json'
  | 'xlsx'
  | 'parquet'
  | 'tex'
  | 'html'
  | 'markdown'
  | 'anafispread';

/**
 * Export range mode (public interface - matches sidebar)
 */
export type ExportRangeMode = 'sheet' | 'all' | 'custom';

/**
 * Export result
 */
export interface ExportResult {
  success: boolean;
  message?: string;
  error?: string;
}

/**
 * File type filters for save dialog
 */
const FILE_FILTERS: Record<ExportFormat, { name: string; extensions: string[] }> = {
  csv: { name: 'CSV Files', extensions: ['csv'] },
  tsv: { name: 'TSV Files', extensions: ['tsv'] },
  txt: { name: 'Text Files', extensions: ['txt'] },
  json: { name: 'JSON Files', extensions: ['json'] },
  xlsx: { name: 'Excel Files', extensions: ['xlsx'] },
  anafispread: { name: 'AnaFis Spreadsheet', extensions: ['anafispread'] },
  parquet: { name: 'Parquet Files', extensions: ['parquet'] },
  tex: { name: 'LaTeX Files', extensions: ['tex'] },
  html: { name: 'HTML Files', extensions: ['html'] },
  markdown: { name: 'Markdown Files', extensions: ['md'] },
};

/**
 * Backend-integrated export service using Facade API
 */
export class ExportService {
  constructor() { }

  /**
   * Main export method with file dialog
   */
  async exportWithDialog(options: ExportOptions, univerAPI: ReturnType<typeof FUniver.newAPI>): Promise<ExportResult> {
    try {
      // Show save dialog
      const filePath = await save({
        filters: [FILE_FILTERS[options.exportFormat]],
        defaultPath: `export.${FILE_FILTERS[options.exportFormat].extensions[0]}`,
      });

      if (!filePath) {
        return { success: false, message: 'Export cancelled by user' };
      }

      return await this.exportToFile(filePath, options, univerAPI);
    } catch (error) {
      return {
        success: false,
        error: `Export failed: ${error instanceof Error ? error.message : 'Unknown error'}`
      };
    }
  }

  /**
   * Export to specific file path
   */
  async exportToFile(filePath: string, options: ExportOptions, univerAPI: ReturnType<typeof FUniver.newAPI>): Promise<ExportResult> {
    try {
      // Extract data using Facade API
      const data = await this.extractData(options, univerAPI);
      if (!data) {
        throw new Error('No data to export');
      }

      // Validate and type-check the extracted data
      const validatedData = this.validateAndTypeCheckData(data);

      // Prepare export configuration for backend
      const exportConfig = this.buildExportConfig(options);

      // Call appropriate backend export command
      await this.callBackendExport(options.exportFormat, validatedData, filePath, exportConfig);

      // Generate success message based on data type
      let message: string;
      if (Array.isArray(validatedData) && validatedData.length > 0) {
        // Check if it's multi-sheet data
        const firstItem = validatedData[0];
        if (firstItem && typeof firstItem === 'object' &&
          (('_multiSheet' in firstItem) || ('name' in firstItem && 'data' in firstItem))) {
          message = `Successfully exported ${validatedData.length} sheets to ${filePath}`;
        } else {
          message = `Successfully exported ${validatedData.length} rows to ${filePath}`;
        }
      } else {
        message = `Successfully exported data to ${filePath}`;
      }

      return {
        success: true,
        message
      };
    } catch (error) {
      return this.handleExportError(error);
    }
  }

  /**
   * Type guard to check if an object is a valid SheetObject
   */
  private isValidSheetObject(obj: unknown): obj is SheetObject {
    return (
      obj !== null &&
      typeof obj === 'object' &&
      'data' in obj &&
      'name' in obj &&
      typeof (obj as Record<string, unknown>).name === 'string'
    );
  }

  /**
   * Validate and type-check extracted data before sending to backend
   */
  private validateAndTypeCheckData(data: unknown): unknown[] {
    if (!data) {
      throw new Error('No data extracted for export');
    }

    // Handle multi-sheet data formats
    if (Array.isArray(data)) {
      // Check for multi-sheet object format: [{ _multiSheet: true, data: {...} }]
      if (data.length === 1 &&
        typeof data[0] === 'object' &&
        data[0] !== null &&
        '_multiSheet' in data[0] &&
        'data' in data[0]) {
        // Validate multi-sheet JSON format
        const multiSheetObj = data[0] as MultiSheetObject;
        if (typeof multiSheetObj.data !== 'object') {
          throw new Error('Invalid multi-sheet format: data property must be an object');
        }
        // Multi-sheet JSON format - return as-is for backend processing
        return data;
      }

      // Check for sheet array format: [{ name: "Sheet1", data: [[...]] }]
      if (data.length > 0 &&
        typeof data[0] === 'object' &&
        data[0] !== null &&
        'name' in data[0] &&
        'data' in data[0]) {
        // Multi-sheet array format - validate each sheet's data
        for (let i = 0; i < data.length; i++) {
          const sheetItem: unknown = data[i];
          if (!this.isValidSheetObject(sheetItem)) {
            throw new Error(`Invalid sheet data at index ${i}: missing required properties`);
          }

          // Now we know sheetItem is a valid SheetObject
          if (!Array.isArray(sheetItem.data)) {
            throw new Error(`Invalid sheet data at index ${i}: data must be an array`);
          }
        }
        return data;
      }

      // Regular 2D array format - validate structure
      if (data.length === 0) {
        throw new Error('No data rows to export');
      }

      // Validate that it's a proper 2D array
      for (let i = 0; i < data.length; i++) {
        if (!Array.isArray(data[i])) {
          throw new Error(`Invalid data structure: row ${i} is not an array`);
        }
      }

      return data;
    }

    throw new Error('Invalid data format: expected array or multi-sheet object');
  }

  /**
   * Extract data using Facade API
   */
  private async extractData(options: ExportOptions, univerAPI: ReturnType<typeof FUniver.newAPI>): Promise<unknown> {
    return safeUniverOperation(() => {
      // Handle multi-sheet export for 'all' range mode
      if (options.rangeMode === 'all' && (options.exportFormat === 'xlsx' || options.exportFormat === 'anafispread' || options.exportFormat === 'json')) {
        return this.extractAllSheetsData(options, univerAPI);
      }

      // Determine the range to export for single sheet
      const range = this.determineExportRange(options, univerAPI);

      // Check if this is a multi-sheet export request
      if (range === '__ALL_SHEETS__') {
        return this.extractAllSheetsData(options, univerAPI);
      }

      if (options.losslessExtraction) {
        // Use formatted table extraction for lossless export
        const extractionOptions: ExtractionOptions = {
          range,
          includeFormulas: options.includeFormulas,
          includeFormatting: options.includeFormatting,
          includeMetadata: options.includeMetadata
        };

        const formattedTable = extractFormattedTable(univerAPI, extractionOptions);
        return formattedTable.data;
      } else {
        // Use simple data extraction
        const workbook = univerAPI.getActiveWorkbook();
        if (!workbook) { throw new Error('No active workbook'); }

        const sheet = workbook.getActiveSheet();

        const sheetRange = sheet.getRange(range);
        const values = sheetRange.getValues();

        // Filter out empty rows and columns
        return this.filterEmptyData(values);
      }
    }, 'data extraction');
  }

  /**
   * Determine the range to export based on options
   */
  private determineExportRange(options: ExportOptions, univerAPI: ReturnType<typeof FUniver.newAPI>): string {
    switch (options.rangeMode) {

      case 'custom':
        if (!options.customRange) {
          throw new Error('Please specify a custom range');
        }
        // Use existing validation functions from errors module
        try {
          return normalizeRangeRef(options.customRange);
        } catch (error) {
          if (error instanceof UniverValidationError) {
            throw new Error(`Invalid range format: ${error.message}`);
          }
          throw error;
        }

      case 'sheet': {
        // Use runtime tracked bounds for instant range calculation

        const workbook = univerAPI.getActiveWorkbook();
        if (!workbook) {
          return 'A1:A1';
        }

        const sheet = workbook.getActiveSheet();

        const sheetId = sheet.getSheetId();
        if (!sheetId) {
          return 'A1:A1';
        }

        if (!options.trackedBounds) {
          return 'A1:A1';
        }

        const bounds = options.trackedBounds[sheetId];
        if (!bounds || typeof bounds.maxRow !== 'number' || typeof bounds.maxCol !== 'number' || bounds.maxRow < 0 || bounds.maxCol < 0) {
          return 'A1:A1';
        }

        const endCol = columnToLetter(bounds.maxCol);
        const endRow = bounds.maxRow + 1;
        return `A1:${endCol}${endRow}`;
      }

      case 'all': {
        // Multi-sheet export: return a special marker to indicate all sheets should be processed
        return '__ALL_SHEETS__';
      }
    }
  }

  /**
   * Extract data from all sheets for multi-sheet export formats
   * 
   * Supports the following formats:
   * - JSON: Returns { _multiSheet: true, data: { "Sheet1": [[...]], "Sheet2": [[...]] } }
   * - Excel/AnaFis: Returns [{ name: "Sheet1", data: [[...]] }, { name: "Sheet2", data: [[...]] }]
   * 
   * Features:
   * - Automatic sheet name detection (uses sheet ID or falls back to Sheet1, Sheet2, etc.)
   * - Smart used range detection for each sheet
   * - Error handling - continues processing other sheets if one fails
   * - Support for both lossless and standard extraction modes
   */
  private extractAllSheetsData(options: ExportOptions, univerAPI: ReturnType<typeof FUniver.newAPI>): unknown {
    const workbook = univerAPI.getActiveWorkbook();
    if (!workbook) {
      throw new Error('No active workbook found');
    }

    const sheets = workbook.getSheets();
    if (sheets.length === 0) {
      throw new Error('No sheets found in workbook');
    }

    // For JSON format, return structured data with sheet names
    if (options.exportFormat === 'json') {
      const allSheetsData: Record<string, unknown[][]> = {};

      for (let i = 0; i < sheets.length; i++) {
        const sheet = sheets[i];
        if (!sheet) {
          console.warn(`Skipping undefined sheet at index ${i}`);
          continue;
        }

        try {
          const sheetName = this.getSheetName(sheet, i);
          const usedRange = this.getSheetUsedRange(sheet, options.trackedBounds);

          console.log(`Processing sheet "${sheetName}" with range ${usedRange}`);

          if (options.losslessExtraction) {
            // Use formatted extraction for lossless
            const extractionOptions: ExtractionOptions = {
              range: usedRange,
              includeFormulas: options.includeFormulas,
              includeFormatting: options.includeFormatting,
              includeMetadata: options.includeMetadata
            };

            const formattedTable = extractFormattedTable(univerAPI, extractionOptions);
            allSheetsData[sheetName] = formattedTable.data;
          } else {
            // Use simple extraction
            const sheetRange = sheet.getRange(usedRange);
            const values = sheetRange.getValues();
            allSheetsData[sheetName] = this.filterEmptyData(values);
          }
        } catch (error) {
          console.error(`Error processing sheet ${i}:`, error);
          // Don't silently replace data - let user know about the error
          const sheetName = this.getSheetName(sheet, i);
          throw new Error(`Failed to process sheet "${sheetName}": ${error instanceof Error ? error.message : String(error)}. Export cancelled to prevent data loss.`);
        }
      }

      // For JSON multi-sheet export, we need to serialize this as a special format
      // The backend will handle converting this to the appropriate JSON structure
      return [{ _multiSheet: true, data: allSheetsData } as unknown];
    }

    // For Excel and AnaFis formats, combine all sheets into a single data structure
    // The backend will handle creating multiple sheets in the output file
    const allSheetsData: SheetData[] = [];

    for (let i = 0; i < sheets.length; i++) {
      const sheet = sheets[i];
      if (!sheet) {
        console.warn(`Skipping undefined sheet at index ${i}`);
        continue;
      }

      try {
        const sheetName = this.getSheetName(sheet, i);
        const usedRange = this.getSheetUsedRange(sheet, options.trackedBounds);

        console.log(`Processing sheet "${sheetName}" with range ${usedRange}`);

        let sheetData: unknown[][];

        if (options.losslessExtraction) {
          // Use formatted extraction for lossless
          const extractionOptions: ExtractionOptions = {
            range: usedRange,
            includeFormulas: options.includeFormulas,
            includeFormatting: options.includeFormatting,
            includeMetadata: options.includeMetadata
          };

          const formattedTable = extractFormattedTable(univerAPI, extractionOptions);
          sheetData = formattedTable.data;
        } else {
          // Use simple extraction
          const sheetRange = sheet.getRange(usedRange);
          const values = sheetRange.getValues();
          sheetData = this.filterEmptyData(values);
        }

        // Add sheet metadata for backend processing (using format expected by backend)
        allSheetsData.push({
          name: sheetName,
          data: sheetData
        });
      } catch (error) {
        console.error(`Error processing sheet ${i}:`, error);
        // Don't silently replace data - let user know about the error
        const sheetName = this.getSheetName(sheet, i);
        throw new Error(`Failed to process sheet "${sheetName}": ${error instanceof Error ? error.message : String(error)}. Export cancelled to prevent data loss.`);
      }
    }

    return allSheetsData;
  }

  /**
   * Get the sheet name from a sheet object
   */
  private getSheetName(sheet: UniverSheet, index: number): string {
    try {
      // Try to get the sheet ID first
      if (sheet.getSheetId) {
        const sheetId = sheet.getSheetId();
        if (sheetId && typeof sheetId === 'string') {
          return sheetId;
        }
      }

      // Try to get sheet name if available
      if (sheet.getName) {
        const name = sheet.getName();
        if (name && typeof name === 'string') {
          return name;
        }
      }

      // Fallback to index-based naming
      return `Sheet${index + 1}`;
    } catch (error) {
      console.warn('Error getting sheet name:', error);
      return `Sheet${index + 1}`;
    }
  }

  /**
   * Get the used range for a specific sheet
   *
   * Uses per-sheet tracked bounds for instant calculation instead of scanning
   */
  private getSheetUsedRange(sheet: UniverSheet, trackedBounds?: Record<string, { maxRow: number; maxCol: number }> | null): string {
    try {
      if (!sheet.getSheetId || !trackedBounds) {
        return 'A1:A1';
      }

      const sheetId = sheet.getSheetId();
      if (!sheetId) {
        return 'A1:A1';
      }

      const bounds = trackedBounds[sheetId];
      if (!bounds || bounds.maxRow < 0 || bounds.maxCol < 0) {
        return 'A1:A1';
      }

      const endCol = columnToLetter(bounds.maxCol);
      const endRow = bounds.maxRow + 1;
      return `A1:${endCol}${endRow}`;
    } catch (error) {
      console.warn('Error getting sheet used range:', error);
      return 'A1:A1';
    }
  }


  /**
   * Filter out trailing empty rows and columns while preserving rectangular data structure
   * 
   * This method:
   * 1. Finds the actual data bounds (last row and column with any data)
   * 2. Preserves all cells within those bounds (including empty cells)
   * 3. Only removes trailing empty rows and columns beyond the data bounds
   * 4. Maintains rectangular array structure for proper spreadsheet semantics
   * 
   * Example:
   * Input:  [["Name", "Age", ""], ["John", "", ""], ["", "", ""], ["", "", "Notes"], [], []]
   * Output: [["Name", "Age", ""], ["John", "", ""], ["", "", ""], ["", "", "Notes"]]
   *         (Preserves empty row 3, removes trailing empty rows 5-6)
   */
  private filterEmptyData(data: unknown[][]): unknown[][] {
    if (data.length === 0) {
      return [];
    }

    // Step 1: Find the actual data bounds
    let lastDataRow = -1;
    let lastDataCol = -1;

    // Scan all data to find the furthest row and column with actual data
    for (let rowIdx = 0; rowIdx < data.length; rowIdx++) {
      const row = data[rowIdx];
      if (!Array.isArray(row)) {
        continue;
      }

      // Check each cell in this row
      for (let colIdx = 0; colIdx < row.length; colIdx++) {
        if (this.isCellNonEmpty(row[colIdx])) {
          lastDataRow = Math.max(lastDataRow, rowIdx);
          lastDataCol = Math.max(lastDataCol, colIdx);
        }
      }
    }

    // If no data found, return empty array
    if (lastDataRow === -1 || lastDataCol === -1) {
      return [];
    }

    // Step 2: Create rectangular output preserving structure within bounds
    const result: unknown[][] = [];
    
    for (let rowIdx = 0; rowIdx <= lastDataRow; rowIdx++) {
      const sourceRow = data[rowIdx];
      const outputRow: unknown[] = [];
      
      // Build row with proper length, filling missing cells with null
      for (let colIdx = 0; colIdx <= lastDataCol; colIdx++) {
        if (Array.isArray(sourceRow) && colIdx < sourceRow.length) {
          outputRow.push(sourceRow[colIdx]);
        } else {
          // Fill missing cells with null to maintain rectangular structure
          outputRow.push(null);
        }
      }
      
      result.push(outputRow);
    }

    return result;
  }


  /*
   Comprehensive check if a cell value is non-empty
   Handles null, undefined, empty strings, and whitespace-only strings
  */
  private isCellNonEmpty(value: unknown): boolean {
    if (value === null || value === undefined) {
      return false;
    }

    if (typeof value === 'string') {
      return value.trim().length > 0;
    }

    // Numbers (including 0), booleans, and other types are considered non-empty
    return true;
  }

  /**
   * Build export configuration for backend
   * Creates the nested structure expected by Rust ExportConfig struct
   */
  private buildExportConfig(options: ExportOptions): Record<string, unknown> {
    // Determine data structure based on range mode and format
    // This explicit marker eliminates the need for implicit detection in backend
    let dataStructure: string;
    if (options.rangeMode === 'all') {
      // Multi-sheet export
      if (options.exportFormat === 'json') {
        dataStructure = 'multisheetjson';  // JSON uses special format with _multiSheet marker
      } else if (options.exportFormat === 'xlsx' || options.exportFormat === 'anafispread') {
        dataStructure = 'multisheetarray';  // Excel/AnaFis use array of sheet objects
      } else {
        // CSV/TSV/TXT/HTML/Markdown/LaTeX/Parquet don't support multi-sheet
        // But we still set this so backend can provide clear error message
        dataStructure = 'array2d';
      }
    } else {
      // Single sheet export (sheet or custom range)
      dataStructure = 'array2d';
    }

    const config = {
      range: options.rangeMode === 'custom' ? options.customRange : options.rangeMode,
      format: options.exportFormat,
      dataStructure,  // Explicit data structure marker - no detection needed
      options: {
        // General options (camelCase for Rust serde)
        includeHeaders: options.includeHeaders,
        includeFormulas: options.includeFormulas,
        includeFormatting: options.includeFormatting,
        includeMetadata: options.includeMetadata,
        
        // Text format options (CSV, TSV, TXT)
        delimiter: options.delimiter,
        encoding: options.encoding,
        lineEnding: options.lineEnding,
        quoteChar: options.quoteChar,
        
        // JSON options
        jsonFormat: options.jsonFormat,
        prettyPrint: options.prettyPrint,
        
        // Compression options
        compress: options.compress,
      }
    };

    return config;
  }

  /**
   * Call appropriate backend export command
   */
  private async callBackendExport(
    format: ExportFormat,
    data: unknown,
    filePath: string,
    config: Record<string, unknown>
  ): Promise<void> {
    switch (format) {
      case 'csv':
      case 'tsv':
      case 'txt':
        await invoke('export_to_text', { data, filePath, config });
        break;
      case 'json':
        await invoke('export_to_json', { data, filePath, config });
        break;
      case 'html':
        await invoke('export_to_html', { data, filePath, config });
        break;
      case 'markdown':
        await invoke('export_to_markdown', { data, filePath, config });
        break;
      case 'tex':
        await invoke('export_to_latex', { data, filePath, config });
        break;
      case 'parquet':
        await invoke('export_to_parquet', { data, filePath, config });
        break;
      case 'xlsx':
        await invoke('export_to_excel', { data, filePath, config });
        break;
      case 'anafispread':
        await invoke('export_to_anafis_spread', { data, filePath, config });
        break;
      default:
        throw new Error(`Export format '${format as string}' not supported`);
    }
  }

  /**
   * Handle export errors with specific messages
   */
  private handleExportError(err: unknown): ExportResult {
    const errorMessage = err instanceof Error ? err.message : String(err);

    // Categorize errors for better user experience
    // Order matters: more specific patterns must be checked before general ones

    // Specific error patterns (check first)
    if (errorMessage.includes('Invalid range format') || errorMessage.includes('Invalid data range format') || errorMessage.includes('Invalid uncertainty range format')) {
      return {
        success: false,
        error: `Range validation error: ${errorMessage}. Please use formats like A1:B10, A:A, or 1:1.`
      };
    } else if (errorMessage.includes('Failed to process sheet')) {
      return {
        success: false,
        error: `Sheet processing error: ${errorMessage}. Some sheets may contain invalid data or formulas.`
      };
    } else if (errorMessage.includes('Invalid data structure') || errorMessage.includes('No data extracted')) {
      return {
        success: false,
        error: `Data extraction error: ${errorMessage}. Please check that your spreadsheet contains valid data.`
      };
    }

    // Permission and access errors
    else if (errorMessage.includes('permission') || errorMessage.includes('access') || errorMessage.includes('denied')) {
      return {
        success: false,
        error: 'Permission denied: Cannot write to the selected file. Please check file permissions and try a different location.'
      };
    }

    // System resource errors
    else if (errorMessage.includes('disk') || errorMessage.includes('space') || errorMessage.includes('insufficient')) {
      return {
        success: false,
        error: 'Insufficient disk space to save the file.'
      };
    } else if (errorMessage.includes('memory') || errorMessage.includes('out of memory')) {
      return {
        success: false,
        error: 'Insufficient memory to process the export. Please try with a smaller data range.'
      };
    }

    // Encoding errors
    else if (errorMessage.includes('encoding') || errorMessage.includes('charset') || errorMessage.includes('unicode')) {
      return {
        success: false,
        error: `Text encoding error: ${errorMessage}`
      };
    }

    // Timeout errors
    else if (errorMessage.includes('timeout') || errorMessage.includes('timed out')) {
      return {
        success: false,
        error: 'Export timed out. Please try with a smaller data range.'
      };
    }

    // More specific format/data errors (before generic "format" check)
    else if (errorMessage.includes('data format') || errorMessage.includes('file format') || errorMessage.includes('export format')) {
      return {
        success: false,
        error: `Data format error: ${errorMessage}`
      };
    }

    // Generic format/invalid errors (catch remaining cases)
    else if (errorMessage.includes('invalid') || errorMessage.includes('malformed') || errorMessage.includes('corrupt')) {
      return {
        success: false,
        error: `Data validation error: ${errorMessage}`
      };
    }

    // Fallback for uncategorized errors
    else {
      return {
        success: false,
        error: `Export failed: ${errorMessage}`
      };
    }
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
  }, univerAPI: ReturnType<typeof FUniver.newAPI>): Promise<ExportResult> {
    try {
      // Validate inputs
      if (!options.libraryName.trim()) {
        throw new Error('Please enter a name for the data sequence');
      }

      if (!options.dataRange.trim()) {
        throw new Error('Please specify a data range');
      }

      // Validate data range format
      try {
        normalizeRangeRef(options.dataRange.trim());
      } catch (error) {
        if (error instanceof UniverValidationError) {
          throw new Error(`Invalid data range format: ${error.message}`);
        }
        throw error;
      }

      // Extract range data with analysis
      console.log('Extracting data from range:', options.dataRange);
      const dataExtraction = await this.extractRangeDataWithAnalysis(options.dataRange, univerAPI);
      const { dataValues, analysis } = dataExtraction;
      console.log('Extracted data values:', dataValues);
      console.log('Data analysis:', analysis);

      if (dataValues.length === 0) {
        throw new Error(`No valid numeric data found in range ${options.dataRange}. Make sure the range contains numeric values.`);
      }

      // Warn about data loss if there are non-numeric values
      if (analysis.nonNumericCount > 0 || analysis.emptyCount > 0) {
        const warnings: string[] = [];
        if (analysis.nonNumericCount > 0) {
          warnings.push(`${analysis.nonNumericCount} non-numeric cells will be excluded`);
        }
        if (analysis.emptyCount > 0) {
          warnings.push(`${analysis.emptyCount} empty cells will be skipped`);
        }
        console.warn(`Data Library Import Warning: ${warnings.join(', ')}. Total cells: ${analysis.totalCells}, Numeric values: ${dataValues.length}`);
      }

      // Extract uncertainty data if specified
      let uncertainties: number[] | undefined;
      if (options.uncertaintyRange.trim()) {
        // Validate uncertainty range format
        try {
          normalizeRangeRef(options.uncertaintyRange.trim());
        } catch (error) {
          if (error instanceof UniverValidationError) {
            throw new Error(`Invalid uncertainty range format: ${error.message}`);
          }
          throw error;
        }

        const uncertaintyExtraction = await this.extractRangeDataWithAnalysis(options.uncertaintyRange, univerAPI);
        uncertainties = uncertaintyExtraction.dataValues;
        
        // Warn about data loss in uncertainty values
        if (uncertaintyExtraction.analysis.nonNumericCount > 0 || uncertaintyExtraction.analysis.emptyCount > 0) {
          const warnings: string[] = [];
          if (uncertaintyExtraction.analysis.nonNumericCount > 0) {
            warnings.push(`${uncertaintyExtraction.analysis.nonNumericCount} non-numeric uncertainty values excluded`);
          }
          if (uncertaintyExtraction.analysis.emptyCount > 0) {
            warnings.push(`${uncertaintyExtraction.analysis.emptyCount} empty uncertainty cells skipped`);
          }
          console.warn(`Data Library Import Warning (Uncertainty): ${warnings.join(', ')}. Total cells: ${uncertaintyExtraction.analysis.totalCells}, Valid values: ${uncertainties.length}`);
        }
        
        if (uncertainties.length !== dataValues.length) {
          throw new Error(`Uncertainty range (${uncertainties.length} values) must have the same length as data range (${dataValues.length} values)`);
        }
      }

      // Parse tags
      const tags = options.libraryTags
        .split(',')
        .map(tag => tag.trim())
        .filter(tag => tag.length > 0);

      // Build save request
      const request = {
        name: options.libraryName.trim(),
        description: options.libraryDescription.trim(),
        tags,
        unit: options.libraryUnit.trim(),
        source: `Range: ${options.dataRange}${options.uncertaintyRange ? ` (uncertainty: ${options.uncertaintyRange})` : ''}`,
        data: dataValues,
        uncertainties: uncertainties && uncertainties.length > 0 ? uncertainties : null,
        is_pinned: false,
      };

      // Save to Data Library
      await invoke('save_sequence', { request });

      return {
        success: true,
        message: `Successfully saved '${options.libraryName}' to Data Library (${dataValues.length} data points)`
      };
    } catch (error) {
      console.error('Data Library export error:', error);

      const errorMessage: string = (() => {
        if (error instanceof Error) {
          return error.message;
        }
        if (typeof error === 'string') {
          return error;
        }
        if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
          return error.message;
        }
        return 'Unknown error';
      })();

      return {
        success: false,
        error: `Failed to save to Data Library: ${errorMessage}`
      };
    }
  }

  /**
   * Extract range data with analysis of data types and losses
   * Returns both numeric values and analysis of what was excluded
   */
  private async extractRangeDataWithAnalysis(range: string, univerAPI: ReturnType<typeof FUniver.newAPI>): Promise<{ dataValues: number[], analysis: { totalCells: number, numericCount: number, nonNumericCount: number, emptyCount: number } }> {
    return safeUniverOperation(() => {
      const workbook = univerAPI.getActiveWorkbook();
      if (!workbook) {
        throw new Error('No active workbook');
      }

      const sheet = workbook.getActiveSheet();
      const sheetRange = sheet.getRange(range);
      const rangeData = sheetRange.getValues();

      const dataValues: number[] = [];
      let totalCells = 0;
      let emptyCount = 0;
      let nonNumericCount = 0;

      // Extract numeric values and track all types
      for (const row of rangeData) {
        if (!Array.isArray(row)) { continue; }

        for (const cell of row) {
          totalCells++;

          // Track empty cells
          if (cell === null || cell === undefined || cell === '') {
            emptyCount++;
            continue;
          }

          // Try to convert cell value to number
          let num: number;
          if (typeof cell === 'number') {
            num = cell;
          } else {
            // Convert string representations to numbers
            const str = String(cell).trim();
            if (str === '') {
              emptyCount++;
              continue;
            }
            num = parseFloat(str);
          }

          // Only include valid numbers (not NaN)
          if (!isNaN(num) && isFinite(num)) {
            dataValues.push(num);
          } else {
            // Non-numeric value that couldn't be converted
            nonNumericCount++;
          }
        }
      }

      return {
        dataValues,
        analysis: {
          totalCells,
          numericCount: dataValues.length,
          nonNumericCount,
          emptyCount
        }
      };
    }, 'range data extraction with analysis');
  }
}

/**
 * Utility functions
 */
export function getFileExtension(format: ExportFormat): string {
  const filter = FILE_FILTERS[format];
  return filter.extensions[0]!;
}

export function getFilterName(format: ExportFormat): string {
  const filter = FILE_FILTERS[format];
  return filter.name;
}