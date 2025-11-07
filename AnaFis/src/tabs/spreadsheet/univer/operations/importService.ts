// importService.ts - Simplified import service
// Supports: CSV, TSV, TXT, Parquet, AnaFisSpread (no HTML/Markdown)
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { IStyleData } from '@univerjs/core';
import { SpreadsheetRef, CellValue, WorkbookSnapshot, SheetSnapshot } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import type { ImportOptions, ImportResult, ImportError } from '@/core/types/import';
import { ERROR_MESSAGES } from '@/tabs/spreadsheet/univer/utils/constants';
import { convertSimpleArrayToCellValues, parseRange, formatSpreadsheetError } from '@/tabs/spreadsheet/univer';
import { RangeValidator } from '@/tabs/spreadsheet/univer/utils/RangeValidator';
import { Result, ok, err } from '@/core/types/result';
import { extractStartCell } from '@/tabs/spreadsheet/utils/rangeUtils';
export type ImportFormat = 'csv' | 'tsv' | 'txt' | 'parquet' | 'anafispread';

export interface FileMetadata {
  path: string;
  size: number;
  extension: string;
  rowCount?: number;
  columnCount?: number;
}

const FORMAT_FILTERS = {
  csv: { name: 'CSV Files', extensions: ['csv'] },
  tsv: { name: 'TSV Files', extensions: ['tsv'] },
  txt: { name: 'Text Files', extensions: ['txt'] },
  parquet: { name: 'Parquet Files', extensions: ['parquet'] },
  anafispread: { name: 'AnaFis Spreadsheet', extensions: ['anafispread'] },
};


export class ImportService implements ImportService {
  /**
   * Select file and auto-detect format
   */
  async selectFile(): Promise<{ filePath: string; detectedFormat: ImportFormat } | null> {
    try {
      const file = await open({
        multiple: false,
        filters: [
          { name: 'All Supported', extensions: ['csv', 'tsv', 'txt', 'parquet', 'anafispread'] },
          FORMAT_FILTERS.anafispread,
          FORMAT_FILTERS.csv,
          FORMAT_FILTERS.tsv,
          FORMAT_FILTERS.txt,
          FORMAT_FILTERS.parquet,
        ]
      });

      if (!file) {
        return null;
      }

      const filePath = typeof file === 'string' ? file : (file as { path?: string }).path ?? '';
      if (!filePath) {
        throw new Error(ERROR_MESSAGES.NO_FILE_PATH);
      }

      // Auto-detect format from extension
      const detectedFormat = this.detectFormat(filePath);
      
      return { filePath, detectedFormat };
    } catch (error) {
      console.error('File selection failed:', error);
      return null;
    }
  }

  /**
   * Get supported formats with descriptions
   */
  getSupportedFormats(): { format: ImportFormat; description: string; extensions: string[] }[] {
    return [
      { 
        format: 'anafispread', 
        description: 'AnaFis Spreadsheet - Lossless native format', 
        extensions: ['anafispread'] 
      },
      { 
        format: 'csv', 
        description: 'CSV - Comma-separated values', 
        extensions: ['csv'] 
      },
      { 
        format: 'tsv', 
        description: 'TSV - Tab-separated values', 
        extensions: ['tsv'] 
      },
      { 
        format: 'txt', 
        description: 'TXT - Text with custom delimiter', 
        extensions: ['txt'] 
      },
      { 
        format: 'parquet', 
        description: 'Parquet - Columnar data format', 
        extensions: ['parquet'] 
      },
    ];
  }

  /**
   * Import from file path with options
   */
  async importFile(
    filePath: string,
    options: ImportOptions,
    spreadsheetRef: React.RefObject<SpreadsheetRef | null>
  ): Promise<Result<ImportResult, ImportError>> {
    try {
      const spreadsheetAPI = spreadsheetRef.current;
      if (!spreadsheetAPI) {
        return err({ message: ERROR_MESSAGES.SPREADSHEET_NOT_READY, code: 'SPREADSHEET_NOT_READY' });
      }

      // Special case: anafispread uses direct lossless import
      if (options.format === 'anafispread') {
        const snapshot = await invoke<unknown>('import_anafis_spread_direct', { filePath });

        // Handle anafispread mode: append (default) or replace
        const mode = options.anaFisMode ?? 'append';
        
        if (mode === 'replace') {
          await spreadsheetAPI.loadWorkbookSnapshot(snapshot);
        } else {
          await this.appendSheetsFromSnapshot(snapshot, spreadsheetAPI);
        }

        const sheetCount = this.getSheetCount(snapshot);
        return ok({ 
          message: `Successfully imported ${sheetCount} sheet(s)`,
          sheetCount 
        });
      }

      // Simple formats (CSV, TSV, TXT, Parquet): import as 2D array
      const importedData = await invoke<{ sheets: Record<string, (string | number | null)[][]> }>('import_spreadsheet_file', {
        filePath,
        options: {
          format: options.format,
          skipRows: options.skipRows ?? 0,
          delimiter: this.getDelimiter(options),
          encoding: options.encoding ?? 'utf8',
        }
      });

      // Get file dimensions for validation
      const fileDimensions = this.getFileDimensions(importedData);
      
      // Validate range if using currentRange mode
      let rangeValidation: ImportResult['rangeValidation'];
      if (options.targetMode === 'currentRange' && options.targetRange) {
        rangeValidation = this.validateRange(options.targetRange, fileDimensions);
      }

      // Load data into spreadsheet based on target mode
      const targetMode = options.targetMode ?? 'newSheet';
      
      for (const [sheetName, sheetData] of Object.entries(importedData.sheets)) {
        if (Array.isArray(sheetData)) {
          // Convert simple values to CellValue format using centralized utility
          const cellValues = convertSimpleArrayToCellValues(sheetData);

          // Handle different target modes
          switch (targetMode) {
            case 'newSheet':
              // Create new sheet (default) - sheet is automatically activated
              await spreadsheetAPI.createSheet(sheetName || 'Imported Data');
              await spreadsheetAPI.updateRange('A1', cellValues);
              break;
              
            case 'currentRange': {
              // Import to specified range (A1 default)
              const targetRange = options.targetRange ?? 'A1';
              
              // Extract starting cell from range (e.g., "J13:L16" -> "J13")
              const targetCell = extractStartCell(targetRange);
              
              // Apply range truncation if needed
              let dataToImport = cellValues;
              if (rangeValidation?.willTruncate && rangeValidation.selectedRange) {
                dataToImport = this.truncateDataToRange(cellValues, rangeValidation.selectedRange);
              }
              
              await spreadsheetAPI.updateRange(targetCell, dataToImport);
              break;
            }
          }
          
          break; // Only use first sheet for single-sheet formats
        }
      }

      const rowCount = this.getRowCount(importedData);
      const result: ImportResult = { 
        message: `Successfully imported ${rowCount} rows from ${filePath.split('/').pop() ?? 'file'}`,
        fileDimensions
      };

      // Add range validation info if applicable
      if (rangeValidation) {
        result.rangeValidation = rangeValidation;
      }

      return ok(result);
    } catch (error) {
      const errorMessage = this.formatError(error);
      return err({
        message: errorMessage,
        code: 'IMPORT_FAILED'
      });
    }
  }

  /**
   * Get file metadata
   * For TXT files, pass delimiter to get accurate column count with that delimiter
   */
  async getFileMetadata(filePath: string, delimiter?: string): Promise<FileMetadata | null> {
    try {
      return await invoke<FileMetadata>('get_file_metadata', { filePath, delimiter });
    } catch {
      return null;
    }
  }

  /**
   * Append sheets from a snapshot to the current workbook (for append mode)
   */
  private async appendSheetsFromSnapshot(
    snapshot: unknown,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<void> {
    const snapshotObj = this.validateSnapshot(snapshot);
    
    const existingNames = await this.getExistingSheetNames(spreadsheetAPI);
    const sheetIdMapping = await this.processSheetsFromSnapshot(
      snapshotObj, 
      spreadsheetAPI, 
      existingNames
    );

    await this.handleProtectionData(snapshotObj as { resources?: Array<{ name: string; data: string }> }, spreadsheetAPI, sheetIdMapping);
  }

  /**
   * Validate snapshot structure
   */
  private validateSnapshot(snapshot: unknown): unknown {
    if (typeof snapshot !== 'object' || snapshot === null) {
      throw new Error(ERROR_MESSAGES.INVALID_SNAPSHOT_DATA);
    }

    const snapshotObj = snapshot as Record<string, unknown>;

    // Required fields
    if (typeof snapshotObj.id !== 'string') {
      throw new Error('Snapshot missing required field: id');
    }
    if (typeof snapshotObj.name !== 'string') {
      throw new Error('Snapshot missing required field: name');
    }
    if (!snapshotObj.sheets || typeof snapshotObj.sheets !== 'object') {
      throw new Error(ERROR_MESSAGES.NO_SHEETS_FOUND);
    }

    // Validate sheets structure
    const sheets: Record<string, SheetSnapshot> = {};
    for (const [sheetId, sheetData] of Object.entries(snapshotObj.sheets as Record<string, unknown>)) {
      if (typeof sheetData === 'object' && sheetData !== null) {
        const sheet = sheetData as Record<string, unknown>;
        if (typeof sheet.id !== 'string' || typeof sheet.name !== 'string') {
          throw new Error(`Sheet ${sheetId} missing required fields: id or name`);
        }
        sheets[sheetId] = sheet as SheetSnapshot;
      } else {
        throw new Error(`Invalid sheet data for sheet ${sheetId}`);
      }
    }

    const result: WorkbookSnapshot = {
      id: snapshotObj.id,
      name: snapshotObj.name,
      sheets,
    };

    // Add optional properties only if they exist and are valid
    if (typeof snapshotObj.appVersion === 'string') {
      result.appVersion = snapshotObj.appVersion;
    }
    if (typeof snapshotObj.locale === 'string') {
      result.locale = snapshotObj.locale;
    }
    if (typeof snapshotObj.styles === 'object' && snapshotObj.styles !== null) {
      result.styles = snapshotObj.styles as Record<string, IStyleData>;
    }
    if (Array.isArray(snapshotObj.sheetOrder) && snapshotObj.sheetOrder.every(id => typeof id === 'string')) {
      result.sheetOrder = snapshotObj.sheetOrder;
    }
    if (Array.isArray(snapshotObj.resources)) {
      const validResources = snapshotObj.resources.filter((r): r is { name: string; data: string } => {
        if (typeof r !== 'object' || r === null) {return false;}
        const obj = r as Record<string, unknown>;
        return 'name' in obj && typeof obj.name === 'string' &&
               'data' in obj && typeof obj.data === 'string';
      });
      if (validResources.length > 0) {
        result.resources = validResources;
      }
    }

    // Include any additional properties
    for (const [key, value] of Object.entries(snapshotObj)) {
      if (!(key in result)) {
        (result as Record<string, unknown>)[key] = value;
      }
    }

    return result;
  }
  /**
   * Get existing sheet names to avoid duplicates
   */
  private async getExistingSheetNames(spreadsheetAPI: SpreadsheetRef): Promise<Set<string>> {
    const existingSheets = await spreadsheetAPI.getAllSheets();
    return new Set(existingSheets.map(s => s.name));
  }

  /**
   * Process each sheet from the snapshot
   */
  private async processSheetsFromSnapshot(
    snapshotObj: unknown,
    spreadsheetAPI: SpreadsheetRef,
    existingNames: Set<string>
  ): Promise<Map<string, string>> {
    const snapshot = snapshotObj as { sheets: Record<string, unknown>; sheetOrder?: unknown };
    const sheetOrder = (snapshot.sheetOrder as string[] | undefined) ?? Object.keys(snapshot.sheets);
    const sheetIdMapping = new Map<string, string>();
    
    for (const sheetId of sheetOrder) {
      try {
        const sheetData = snapshot.sheets[sheetId];
        if (!sheetData) {
          console.warn(`Sheet ${sheetId} not found in snapshot, skipping`);
          continue;
        }

        const newSheetId = await this.processSingleSheet(
          sheetId, 
          sheetData as SheetSnapshot, 
          spreadsheetAPI, 
          existingNames,
          sheetOrder.indexOf(sheetId)
        );
        
        if (newSheetId) {
          sheetIdMapping.set(sheetId, newSheetId);
        }
      } catch (error) {
        console.error(`Failed to append sheet ${sheetId}:`, error);
        // Continue with next sheet instead of stopping the entire import
      }
    }

    return sheetIdMapping;
  }

  /**
   * Process a single sheet from the snapshot
   */
  private async processSingleSheet(
    _sheetId: string,
    sheetData: SheetSnapshot,
    spreadsheetAPI: SpreadsheetRef,
    existingNames: Set<string>,
    sheetIndex: number
  ): Promise<string | null> {
    const sheet = sheetData as { 
      name?: string; 
      cellData?: Record<number, Record<number, unknown>>;
      mergeData?: Array<{
        startRow: number;
        startColumn: number;
        endRow: number;
        endColumn: number;
      }>;
      rowCount?: number;
      columnCount?: number;
    };

    const sheetName = this.generateUniqueSheetName(sheet.name ?? `Sheet ${sheetIndex + 1}`, existingNames);
    
    await spreadsheetAPI.createSheet(
      sheetName,
      sheet.rowCount ?? 1000,
      sheet.columnCount ?? 26
    );

    // Wait a bit for sheet creation to complete
    await new Promise(resolve => setTimeout(resolve, 100));

    const newSheet = await spreadsheetAPI.getNewlyCreatedSheet(sheetName);
    if (!newSheet) {
      return null;
    }

    const newSheetId = (newSheet as { getSheetId: () => string }).getSheetId();

    await this.loadSheetData(
      sheet, 
      newSheetId,
      spreadsheetAPI
    );

    return newSheetId;
  }

  /**
   * Generate a unique sheet name
   */
  private generateUniqueSheetName(baseName: string, existingNames: Set<string>): string {
    let sheetName = baseName;
    let nameCounter = 1;
    
    while (existingNames.has(sheetName)) {
      sheetName = `${baseName} (${nameCounter++})`;
    }
    
    existingNames.add(sheetName);
    return sheetName;
  }

  /**
   * Load data into the sheet using bulk load function
   */
  private async loadSheetData(
    sheet: {
      cellData?: Record<number, Record<number, unknown>>;
      mergeData?: Array<{
        startRow: number;
        startColumn: number;
        endRow: number;
        endColumn: number;
      }>;
    },
    sheetId: string,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<void> {
    try {
      interface ImportCellDataRecord {
        v?: string | number | boolean | null;
        f?: string;
        s?: unknown;
        t?: unknown;
        p?: unknown;
      }
      interface ImportMergeDataItem {
        startRow: number;
        startColumn: number;
        endRow: number;
        endColumn: number;
      }
      
      // Use the new abstraction method instead of direct Univer access
      await spreadsheetAPI.loadSheetDataBulk(
        sheetId,
        {
          name: `Sheet-${sheetId}`,
          cellDataMatrix: sheet.cellData as Record<number, Record<number, ImportCellDataRecord>>,
          mergeData: Array.isArray(sheet.mergeData) ? sheet.mergeData as ImportMergeDataItem[] : [],
        },
        {
          includeFormulas: true,
          includeFormatting: true,
        }
      );
    } catch (bulkLoadError) {
      console.error(`Failed to load data for sheet ${sheetId}:`, bulkLoadError);
      throw bulkLoadError;
    }
  }

  /**
   * Handle protection data from resources
   */
  private async handleProtectionData(
    snapshotObj: { resources?: Array<{ name: string; data: string }> },
    spreadsheetAPI: SpreadsheetRef,
    sheetIdMapping: Map<string, string>
  ): Promise<void> {
    if (snapshotObj.resources && Array.isArray(snapshotObj.resources)) {
      await this.applyProtectionFromResources(
        snapshotObj.resources,
        spreadsheetAPI,
        sheetIdMapping
      );
    }
  }

  /**
   * Apply protection rules from resources
   * 
   * Extracts and applies worksheet and range protection from Univer snapshot resources.
   * Protection data is stored in the `resources` array with names containing 'PROTECTION' or 'PERMISSION'.
   * 
   * Supports:
   * - Worksheet-level protection (entire sheet)
   * - Range-level protection (specific cell ranges)
   * 
   * Note: This is a best-effort operation - if protection fails, the main import still succeeds.
   */
  private async applyProtectionFromResources(
    resources: Array<{ name: string; data: string }>,
    spreadsheetAPI: SpreadsheetRef,
    sheetIdMapping?: Map<string, string>
  ): Promise<void> {
    try {
      // For each sheet in the mapping, apply protection if resources available
      for (const [originalSheetId, newSheetId] of (sheetIdMapping ?? new Map()).entries()) {
        // Find protection resources for this sheet (look for original sheet ID in resource data)
        const sheetProtectionResources = resources.filter(r => 
          r.name.includes('PROTECTION') || r.name.includes('PERMISSION') || r.data.includes(originalSheetId as string)
        );
        
        if (sheetProtectionResources.length > 0) {
          // Pass the mapping so the adapter can correctly map sheet IDs in the protection data
          await spreadsheetAPI.applySheetProtection(newSheetId as string, sheetProtectionResources, sheetIdMapping);
        }
      }
    } catch (error) {
      console.warn('Failed to apply protection rules from snapshot:', error);
      // Don't fail the entire import if protection fails - it's optional
    }
  }
  /**
   * Auto-detect format from file extension
   */
  private detectFormat(filePath: string): ImportFormat {
    const ext = filePath.split('.').pop()?.toLowerCase() ?? '';
    
    switch (ext) {
      case 'anafispread':
        return 'anafispread';
      case 'csv':
        return 'csv';
      case 'tsv':
        return 'tsv';
      case 'txt':
        return 'txt';
      case 'parquet':
        return 'parquet';
      default:
        return 'csv'; // Default fallback
    }
  }

  /**
   * Get delimiter based on format and options
   */
  private getDelimiter(options: ImportOptions): string {
    switch (options.format) {
      case 'csv':
        return ','; // Fixed comma for CSV
      case 'tsv':
        return '\t'; // Fixed tab for TSV
      case 'txt':
        return options.delimiter ?? ','; // Custom delimiter for TXT, default comma
      default:
        return ',';
    }
  }

  /**
   * Get sheet count from workbook snapshot
   */
  private getSheetCount(snapshot: unknown): number {
    try {
      if (typeof snapshot === 'object' && snapshot !== null && 'sheets' in snapshot) {
        const sheets = (snapshot as { sheets: unknown }).sheets;
        if (typeof sheets === 'object' && sheets !== null) {
          return Object.keys(sheets).length;
        }
      }
      return 1;
    } catch {
      return 1;
    }
  }

  /**
   * Get row count from imported data
   */
  private getRowCount(importedData: { sheets: Record<string, (string | number | null)[][]> } | undefined): number {
    if (!importedData?.sheets) {return 0;}
    
    const firstSheet = Object.values(importedData.sheets)[0];
    return Array.isArray(firstSheet) ? firstSheet.length : 0;
  }

  /**
   * Format errors for display
   */
  private formatError(err: unknown): string {
    return formatSpreadsheetError(err, 'import');
  }

  /**
   * Get file dimensions from imported data
   */
  private getFileDimensions(importedData: { sheets: Record<string, (string | number | null)[][]> }): { rows: number; columns: number } {
    const firstSheet = Object.values(importedData.sheets)[0];
    if (!Array.isArray(firstSheet) || firstSheet.length === 0) {
      return { rows: 0, columns: 0 };
    }

    const rows = firstSheet.length;
    const columns = Math.max(...firstSheet.map(row => Array.isArray(row) ? row.length : 0));

    return { rows, columns };
  }

  /**
   * Validate if the selected range can accommodate the file data
   */
  private validateRange(rangeRef: string, fileDimensions: { rows: number; columns: number }): ImportResult['rangeValidation'] {
    const warnings: string[] = [];
    let willTruncate = false;

    // Validate range format
    try {
      RangeValidator.validateFormat(rangeRef);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      return {
        isValid: false,
        warnings: [`Invalid range format: ${errorMessage}`],
        willTruncate: false
      };
    }

    // Parse the range to get dimensions
    const rangeBounds = parseRange(rangeRef);
    if (!rangeBounds) {
      return {
        isValid: false,
        warnings: ['Invalid range format'],
        willTruncate: false
      };
    }

    const selectedRows = rangeBounds.endRow - rangeBounds.startRow + 1;
    const selectedColumns = rangeBounds.endCol - rangeBounds.startCol + 1;

    // Check if range is too small (but allow single cell to import all data)
    if (selectedRows < fileDimensions.rows || selectedColumns < fileDimensions.columns) {
      // Don't truncate for single cell - use it as top-left corner for full data import
      if (!(selectedRows === 1 && selectedColumns === 1)) {
        willTruncate = true;
        warnings.push(`Selected range (${selectedRows}×${selectedColumns}) is smaller than file data (${fileDimensions.rows}×${fileDimensions.columns}). Data will be truncated to fit.`);
      }
    }

    // Check if range is much larger (just informational)
    if (selectedRows > fileDimensions.rows * 2 || selectedColumns > fileDimensions.columns * 2) {
      warnings.push(`Selected range is much larger than needed. Consider selecting a smaller range for better performance.`);
    }

    // Special case: single cell
    if (selectedRows === 1 && selectedColumns === 1) {
      warnings.push('Single cell selected. This will be treated as the top-left corner of the import area.');
    }

    return {
      isValid: true,
      warnings,
      willTruncate,
      selectedRange: {
        rows: selectedRows,
        columns: selectedColumns
      }
    };
  }

  /**
   * Truncate data to fit the selected range
   */
  private truncateDataToRange(data: CellValue[][], maxDimensions: { rows: number; columns: number }): CellValue[][] {
    return data.slice(0, maxDimensions.rows).map(row =>
      row.slice(0, maxDimensions.columns)
    );
  }
}
