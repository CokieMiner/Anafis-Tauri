// importService.ts - Simplified import service
// Supports: CSV, TSV, TXT, Parquet, AnaFisSpread (no HTML/Markdown)
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { SpreadsheetRef, CellValue } from '@/components/spreadsheet/SpreadsheetInterface';
import type { ImportOptions, IImportService, ImportResult } from '@/types/import';
import { ERROR_MESSAGES } from '../utils/constants';
import { convertSimpleArrayToCellValues, extractStartCell, parseRange } from '../index';

export type ImportFormat = 'csv' | 'tsv' | 'txt' | 'parquet' | 'anafispread';

export interface FileMetadata {
  path: string;
  size: number;
  extension: string;
  rowCount?: number;
  columnCount?: number;
}

// Re-export ImportResult for backward compatibility
export type { ImportResult };

const FORMAT_FILTERS = {
  csv: { name: 'CSV Files', extensions: ['csv'] },
  tsv: { name: 'TSV Files', extensions: ['tsv'] },
  txt: { name: 'Text Files', extensions: ['txt'] },
  parquet: { name: 'Parquet Files', extensions: ['parquet'] },
  anafispread: { name: 'AnaFis Spreadsheet', extensions: ['anafispread'] },
};


export class ImportService implements IImportService {
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
  ): Promise<ImportResult> {
    try {
      const spreadsheetAPI = spreadsheetRef.current;
      if (!spreadsheetAPI) {
        throw new Error(ERROR_MESSAGES.SPREADSHEET_NOT_READY);
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
        return { 
          success: true, 
          message: `Successfully imported ${sheetCount} sheet(s)`,
          sheetCount 
        };
      }

      // Simple formats (CSV, TSV, TXT, Parquet): import as 2D array
      const importedData = await invoke<{ sheets: Record<string, unknown[][]> }>('import_spreadsheet_file', {
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
        success: true, 
        message: `Successfully imported ${rowCount} rows from ${filePath.split('/').pop()}`,
        fileDimensions
      };

      // Add range validation info if applicable
      if (rangeValidation) {
        result.rangeValidation = rangeValidation;
      }

      return result;
    } catch (error) {
      return {
        success: false,
        error: this.formatError(error)
      };
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
    const context = this.getImplementationContext(spreadsheetAPI);
    
    const { bulkLoadSheetDataFromMatrix } = await import('./bulkImportOperations');
    const existingNames = await this.getExistingSheetNames(spreadsheetAPI);
    const sheetIdMapping = await this.processSheetsFromSnapshot(
      snapshotObj, 
      spreadsheetAPI, 
      context, 
      bulkLoadSheetDataFromMatrix, 
      existingNames
    );

    await this.handleProtectionData(snapshotObj, context, sheetIdMapping);
  }

  /**
   * Validate snapshot structure
   */
  private validateSnapshot(snapshot: unknown): {
    sheets: Record<string, unknown>;
    sheetOrder?: string[];
    resources?: Array<{ name: string; data: string }>;
  } {
    if (typeof snapshot !== 'object' || snapshot === null) {
      throw new Error(ERROR_MESSAGES.INVALID_SNAPSHOT_DATA);
    }

    const snapshotObj = snapshot as { 
      sheets?: Record<string, unknown>; 
      sheetOrder?: string[];
      resources?: Array<{ name: string; data: string }>;
    };
    
    if (!snapshotObj.sheets) {
      throw new Error(ERROR_MESSAGES.NO_SHEETS_FOUND);
    }

    return {
      sheets: snapshotObj.sheets,
      sheetOrder: snapshotObj.sheetOrder,
      resources: snapshotObj.resources,
    } as {
      sheets: Record<string, unknown>;
      sheetOrder?: string[];
      resources?: Array<{ name: string; data: string }>;
    };
  }

  /**
   * Get implementation context for Univer API access
   */
  private getImplementationContext(spreadsheetAPI: SpreadsheetRef): {
    univerInstance: unknown;
    facadeInstance: unknown;
  } {
    const context = spreadsheetAPI.getImplementationContext() as { 
      univerInstance?: unknown; 
      facadeInstance?: unknown;
    } | undefined;
    
    if (!context?.univerInstance || !context.facadeInstance) {
      throw new Error('Cannot access Univer instance for append mode');
    }

    return {
      univerInstance: context.univerInstance,
      facadeInstance: context.facadeInstance,
    };
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
    snapshotObj: { sheets: Record<string, unknown>; sheetOrder?: string[] },
    spreadsheetAPI: SpreadsheetRef,
    context: { univerInstance: unknown; facadeInstance: unknown },
    bulkLoadSheetDataFromMatrix: unknown,
    existingNames: Set<string>
  ): Promise<Map<string, string>> {
    const sheetOrder = snapshotObj.sheetOrder ?? Object.keys(snapshotObj.sheets);
    const sheetIdMapping = new Map<string, string>();
    
    for (const sheetId of sheetOrder) {
      try {
        const newSheetId = await this.processSingleSheet(
          sheetId, 
          snapshotObj.sheets[sheetId], 
          spreadsheetAPI, 
          context, 
          bulkLoadSheetDataFromMatrix, 
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
    sheetData: unknown,
    spreadsheetAPI: SpreadsheetRef,
    context: { univerInstance: unknown; facadeInstance: unknown },
    bulkLoadSheetDataFromMatrix: unknown,
    existingNames: Set<string>,
    sheetIndex: number
  ): Promise<string | null> {
    if (typeof sheetData !== 'object' || sheetData === null) {
      return null;
    }

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

    const newSheet = this.getNewlyCreatedSheet(context.facadeInstance, sheetName);
    if (!newSheet) {
      return null;
    }

    const newSheetId = newSheet.getSheetId();

    await this.loadSheetData(
      sheet, 
      sheetName, 
      context.univerInstance, 
      newSheet, 
      bulkLoadSheetDataFromMatrix
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
   * Get the newly created sheet using Facade API
   */
  private getNewlyCreatedSheet(facadeInstance: unknown, sheetName: string): 
    { getSheetName: () => string; getSheetId: () => string } | null {
    
    const univerAPI = facadeInstance as { 
      getActiveWorkbook: () => { 
        getSheets: () => Array<{ getSheetName: () => string; getSheetId: () => string }> 
      } 
    };
    
    const workbook = univerAPI.getActiveWorkbook();
    const sheets = workbook.getSheets();
    return sheets.find(s => s.getSheetName() === sheetName) ?? null;
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
    sheetName: string,
    univerInstance: unknown,
    newSheet: unknown,
    bulkLoadSheetDataFromMatrix: unknown
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
      
      // Use type imports to avoid unused variable warnings
      // (types are used in the function call below)
      
      await (bulkLoadSheetDataFromMatrix as (
        univerInstance: unknown,
        worksheet: unknown,
        data: unknown,
        options: unknown
      ) => Promise<void>)(
        univerInstance,
        newSheet,
        {
          name: sheetName,
          cellDataMatrix: sheet.cellData as Record<number, Record<number, ImportCellDataRecord>>,
          mergeData: Array.isArray(sheet.mergeData) ? sheet.mergeData as ImportMergeDataItem[] : [],
        },
        {
          includeFormulas: true,
          includeFormatting: true,
        }
      );
    } catch (bulkLoadError) {
      console.error(`Failed to load data for sheet ${sheetName}:`, bulkLoadError);
      throw bulkLoadError;
    }
  }

  /**
   * Handle protection data from resources
   */
  private async handleProtectionData(
    snapshotObj: { resources?: Array<{ name: string; data: string }> },
    context: { univerInstance: unknown; facadeInstance: unknown },
    sheetIdMapping: Map<string, string>
  ): Promise<void> {
    if (snapshotObj.resources && Array.isArray(snapshotObj.resources)) {
      await this.applyProtectionFromResources(
        snapshotObj.resources,
        context.univerInstance,
        context.facadeInstance,
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
    univerInstance: unknown,
    _facadeInstance: unknown,
    sheetIdMapping?: Map<string, string>
  ): Promise<void> {
    try {
      // Import Univer types dynamically
      const { ICommandService, IUniverInstanceService, IPermissionService } = await import('@univerjs/core');
      const { 
        AddWorksheetProtectionMutation, 
        AddRangeProtectionMutation,
        WorksheetEditPermission,
        RangeProtectionPermissionEditPoint
      } = await import('@univerjs/sheets');

      // Get Univer services
      const univer = univerInstance as { __getInjector: () => { get: (token: unknown) => unknown } };
      const injector = univer.__getInjector();
      const commandService = injector.get(ICommandService) as { 
        executeCommand: (commandId: string, params: unknown) => Promise<boolean> 
      };
      const instanceService = injector.get(IUniverInstanceService) as{
        getFocusedUnit: () => { getUnitId: () => string } | null;
      };
      const permissionService = injector.get(IPermissionService) as {
        updatePermissionPoint: (pointId: string, value: boolean) => void;
      };

      const workbook = instanceService.getFocusedUnit();
      if (!workbook) {
        return;
      }

      const unitId = workbook.getUnitId();

      // Process each resource
      for (const resource of resources) {
        // Look for protection-related resources
        if (resource.name.includes('PROTECTION') || resource.name.includes('PERMISSION')) {
          try {
            const protectionData: unknown = JSON.parse(resource.data || '{}');
            
            // Validate that protectionData is an object
            if (typeof protectionData !== 'object' || protectionData === null) {
              continue;
            }
            
            for (const [, protections] of Object.entries(protectionData)) {
              if (!Array.isArray(protections)) {
                continue;
              }

              for (const protection of protections) {
                if (typeof protection !== 'object' || protection === null) {
                  continue;
                }

                const protectionObj = protection as {
                  unitType?: number;
                  subUnitId?: string;
                  permissionId?: string;
                  name?: string;
                };
                
                // Map old sheet ID to new sheet ID if mapping is provided (append mode)
                let targetSubUnitId = protectionObj.subUnitId;
                let protectionToApply: unknown = protection;
                
                if (targetSubUnitId !== undefined && sheetIdMapping !== undefined) {
                  const mappedId = sheetIdMapping.get(targetSubUnitId);
                  if (mappedId !== undefined) {
                    targetSubUnitId = mappedId;
                    
                    // Update the protection rule with the new sheet ID
                    protectionToApply = { 
                      ...(protection as Record<string, unknown>),
                      subUnitId: mappedId,
                      unitId
                    };
                  }
                }

                const isWorksheetProtection = protectionObj.unitType === 2;
                const isRangeProtection = protectionObj.unitType === 3;

                if (isWorksheetProtection && targetSubUnitId !== undefined) {
                  const success = await commandService.executeCommand(AddWorksheetProtectionMutation.id, {
                    unitId,
                    subUnitId: targetSubUnitId,
                    rule: protectionToApply,
                  });
                  
                  if (success) {
                    interface WorksheetEditPermissionConstructor {
                      new (unitId: string, subUnitId: string): { id: string };
                    }
                    const editPermission = new (WorksheetEditPermission as unknown as WorksheetEditPermissionConstructor)(unitId, targetSubUnitId);
                    permissionService.updatePermissionPoint(editPermission.id, false);
                  }
                } else if (isRangeProtection && targetSubUnitId !== undefined) {
                  const success = await commandService.executeCommand(AddRangeProtectionMutation.id, {
                    unitId,
                    subUnitId: targetSubUnitId,
                    rules: [protectionToApply],
                  });
                  
                  if (success && protectionObj.permissionId !== undefined) {
                    interface RangeProtectionPermissionEditPointConstructor {
                      new (unitId: string, subUnitId: string, permissionId: string): { id: string };
                    }
                    const editPermission = new (RangeProtectionPermissionEditPoint as unknown as RangeProtectionPermissionEditPointConstructor)(
                      unitId, 
                      targetSubUnitId, 
                      protectionObj.permissionId
                    );
                    permissionService.updatePermissionPoint(editPermission.id, false);
                  }
                }
              }
            }
          } catch {
            // Silently ignore protection parsing errors
          }
        }
      }
    } catch {
      // Don't throw - protection is optional, main data import should succeed
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
  private getRowCount(importedData: { sheets: Record<string, unknown[][]> } | undefined): number {
    if (!importedData?.sheets) {return 0;}
    
    const firstSheet = Object.values(importedData.sheets)[0];
    return Array.isArray(firstSheet) ? firstSheet.length : 0;
  }

  /**
   * Format errors for display
   */
  private formatError(err: unknown): string {
    const msg = err instanceof Error ? err.message : String(err);

    if (msg.includes('Invalid range')) {return `Range error: ${msg}`;}
    if (msg.includes('No such file')) {return 'File not found';}
    if (msg.includes('permission') || msg.includes('denied')) {
      return 'Permission denied: Cannot read file';
    }
    if (msg.includes('encoding') || msg.includes('charset')) {
      return `Encoding error: ${msg}`;
    }

    return `Import failed: ${msg}`;
  }

  /**
   * Get file dimensions from imported data
   */
  private getFileDimensions(importedData: { sheets: Record<string, unknown[][]> }): { rows: number; columns: number } {
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
