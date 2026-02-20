// importService.ts - Simplified import service
// Supports: CSV, TSV, TXT, Parquet, AnaFisSpread (no HTML/Markdown)
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  ImportError,
  ImportOptions,
  ImportResult,
} from '@/core/types/import';
import { err, isErr, ok, type Result } from '@/core/types/result';
import type {
  CellValue,
  SheetSnapshot,
  SpreadsheetRef,
  SpreadsheetStyle,
  WorkbookSnapshot,
} from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import {
  convertSimpleArrayToCellValues,
  parseRange,
} from '@/tabs/spreadsheet/univer';
import { ERROR_MESSAGES } from '@/tabs/spreadsheet/univer/utils/constants';
import {
  ErrorCategory,
  ErrorSeverity,
  logError,
  normalizeError,
  SpreadsheetError,
  SpreadsheetErrorCode,
  SpreadsheetValidationError,
} from '@/tabs/spreadsheet/univer/utils/errors';
import { RangeValidator } from '@/tabs/spreadsheet/univer/utils/RangeValidator';
import { extractStartCell } from '@/tabs/spreadsheet/utils/rangeUtils';
export type ImportFormat = 'csv' | 'tsv' | 'txt' | 'parquet' | 'anafispread';

export interface FileMetadata {
  path: string;
  size: number;
  extension: string;
  rowCount?: number;
  columnCount?: number;
}

export interface TransactionLogEntry {
  timestamp: number;
  operation: string;
  phase?: string | undefined;
  details?: Record<string, unknown> | undefined;
  error?: string | undefined;
  duration?: number | undefined;
}

const FORMAT_FILTERS = {
  csv: { name: 'CSV Files', extensions: ['csv'] },
  tsv: { name: 'TSV Files', extensions: ['tsv'] },
  txt: { name: 'Text Files', extensions: ['txt'] },
  parquet: { name: 'Parquet Files', extensions: ['parquet'] },
  anafispread: { name: 'AnaFis Spreadsheet', extensions: ['anafispread'] },
};

/**
 * Transactional import manager for atomic multi-sheet operations
 * Ensures that either all sheets import successfully or none do (with rollback)
 */
class TransactionalImportManager {
  private createdSheets: string[] = [];
  private sheetIdMapping: Map<string, string> = new Map();
  private transactionLog: TransactionLogEntry[] = [];
  private transactionId: string;

  constructor() {
    this.transactionId = `import-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Get transaction audit trail
   */
  public getTransactionLog(): TransactionLogEntry[] {
    return [...this.transactionLog];
  }

  /**
   * Get transaction ID for tracking
   */
  public getTransactionId(): string {
    return this.transactionId;
  }

  /**
   * Log an operation with details
   */
  private logOperation(
    operation: string,
    phase?: string,
    details?: Record<string, unknown>,
    error?: string
  ): void {
    const entry: TransactionLogEntry = {
      timestamp: Date.now(),
      operation,
      phase,
      details,
    };

    if (error) {
      entry.error = error;
    }

    this.transactionLog.push(entry);

    // Log to console for debugging (can be disabled in production)
    const logPrefix = `[${this.transactionId}] ${operation}${phase ? ` (${phase})` : ''}`;

    if (error) {
      console.error(logPrefix, { error, details });
    } else if (operation.includes('FAILED') || operation.includes('ERROR')) {
      console.warn(logPrefix, details);
    } else {
      console.info(logPrefix, details);
    }
  }

  /**
   * Import sheets atomically with rollback capability
   */
  async importSheetsAtomic(
    snapshot: WorkbookSnapshot,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<Result<void, string>> {
    const startTime = Date.now();
    this.logOperation('START', 'importSheetsAtomic', {
      sheetCount: Object.keys(snapshot.sheets).length,
    });

    try {
      // Phase 1: Pre-validate all sheets can be imported
      this.logOperation('PHASE', 'validation', { phase: 1 });
      const validation = await this.validateAllSheets(snapshot, spreadsheetAPI);
      if (isErr(validation)) {
        this.logOperation('VALIDATION_FAILED', 'validateAllSheets', {
          error: validation.error,
        });
        return validation;
      }
      this.logOperation('VALIDATION_SUCCESS', 'validateAllSheets');

      // Phase 2: Import all sheets
      this.logOperation('PHASE', 'import', { phase: 2 });
      const importResult = await this.importAllSheets(snapshot, spreadsheetAPI);
      if (isErr(importResult)) {
        this.logOperation('IMPORT_FAILED', 'importAllSheets', {
          error: importResult.error,
        });
        await this.rollback(spreadsheetAPI);
        this.logOperation('ROLLBACK_COMPLETED', 'rollback');
        return importResult;
      }
      this.logOperation('IMPORT_SUCCESS', 'importAllSheets', {
        createdSheets: this.createdSheets.length,
      });

      // Phase 3: Apply protection data (optional, doesn't affect rollback)
      this.logOperation('PHASE', 'protection', { phase: 3 });
      try {
        this.applyProtectionData(snapshot, spreadsheetAPI);
        this.logOperation('PROTECTION_SUCCESS', 'applyProtectionData');
      } catch (error) {
        this.logOperation('PROTECTION_FAILED', 'applyProtectionData', {
          error: error instanceof Error ? error.message : String(error),
        });
        // Don't fail the import for protection issues
      }

      const duration = Date.now() - startTime;
      this.logOperation('SUCCESS', 'importSheetsAtomic', {
        duration,
        createdSheets: this.createdSheets.length,
      });
      return ok(undefined);
    } catch (error) {
      const duration = Date.now() - startTime;
      this.logOperation('UNEXPECTED_ERROR', 'importSheetsAtomic', {
        error: error instanceof Error ? error.message : String(error),
        duration,
      });

      // If anything unexpected happens, attempt rollback
      try {
        await this.rollback(spreadsheetAPI);
        this.logOperation('ROLLBACK_COMPLETED', 'rollback');
      } catch (rollbackError) {
        this.logOperation('ROLLBACK_FAILED', 'rollback', {
          error:
            rollbackError instanceof Error
              ? rollbackError.message
              : String(rollbackError),
        });
      }
      return err(
        `Import failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * Validate that all sheets can be imported before starting
   */
  private async validateAllSheets(
    snapshot: WorkbookSnapshot,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<Result<void, string>> {
    const existingNames = await this.getExistingSheetNames(spreadsheetAPI);
    const sheetOrder = Array.isArray(snapshot.sheetOrder)
      ? (snapshot.sheetOrder as string[])
      : Object.keys(snapshot.sheets);

    for (const sheetId of sheetOrder) {
      const sheetData = snapshot.sheets[sheetId] as SheetSnapshot;

      // Check if sheet name would conflict
      const sheetName =
        sheetData.name ?? `Sheet ${sheetOrder.indexOf(sheetId) + 1}`;
      if (existingNames.has(sheetName)) {
        // This is actually OK - we generate unique names, but let's validate the data structure
      }

      // Validate sheet data structure
      if (typeof sheetData.id !== 'string' || sheetData.id.trim() === '') {
        return err(`Sheet '${sheetId}' has invalid id`);
      }
      if (typeof sheetData.name !== 'string' || sheetData.name.trim() === '') {
        return err(`Sheet '${sheetId}' has invalid name`);
      }
    }

    return ok(undefined);
  }

  /**
   * Import all sheets (assumes validation passed)
   */
  private async importAllSheets(
    snapshot: WorkbookSnapshot,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<Result<void, string>> {
    const existingNames = await this.getExistingSheetNames(spreadsheetAPI);
    const sheetOrder = Array.isArray(snapshot.sheetOrder)
      ? (snapshot.sheetOrder as string[])
      : Object.keys(snapshot.sheets);

    this.logOperation('SHEET_IMPORT_START', undefined, {
      totalSheets: sheetOrder.length,
    });

    for (const sheetId of sheetOrder) {
      const sheetStartTime = Date.now();
      try {
        const sheetData = snapshot.sheets[sheetId] as SheetSnapshot;
        const sheetIndex = sheetOrder.indexOf(sheetId);

        this.logOperation('SHEET_PROCESSING', sheetId, {
          sheetName: sheetData.name,
          index: sheetIndex,
          rowCount: sheetData.rowCount,
          columnCount: sheetData.columnCount,
        });

        // Generate unique sheet name
        const baseName = sheetData.name ?? `Sheet ${sheetIndex + 1}`;
        const uniqueName = this.generateUniqueSheetName(
          baseName,
          existingNames
        );

        // Create the sheet
        const newSheetId = await spreadsheetAPI.createSheet(
          uniqueName,
          sheetData.rowCount ?? 1000,
          sheetData.columnCount ?? 26
        );

        // Track for potential rollback
        this.createdSheets.push(newSheetId);

        // Track sheet ID mapping for protection data
        this.sheetIdMapping.set(sheetId, newSheetId);

        // Load sheet data
        await this.loadSheetData(sheetData, newSheetId, spreadsheetAPI);

        // Update existing names set
        existingNames.add(uniqueName);

        const sheetDuration = Date.now() - sheetStartTime;
        this.logOperation('SHEET_SUCCESS', sheetId, {
          newSheetId,
          uniqueName,
          duration: sheetDuration,
        });
      } catch (error) {
        const sheetDuration = Date.now() - sheetStartTime;
        const errorMessage =
          error instanceof Error ? error.message : String(error);

        this.logOperation('SHEET_FAILED', sheetId, {
          error: errorMessage,
          duration: sheetDuration,
          sheetsProcessed: this.createdSheets.length,
        });

        // Return detailed error with context
        const sheetData = snapshot.sheets[sheetId] as SheetSnapshot;
        const sheetName = sheetData.name ?? 'Unknown';
        return err(
          `Failed to import sheet '${sheetId}' (${sheetName}): ${errorMessage}. ` +
            `${this.createdSheets.length} sheets were successfully imported before this failure.`
        );
      }
    }

    this.logOperation('ALL_SHEETS_SUCCESS', undefined, {
      totalSheets: sheetOrder.length,
    });
    return ok(undefined);
  }

  /**s
   * Rollback all created sheets
   */
  private async rollback(spreadsheetAPI: SpreadsheetRef): Promise<void> {
    if (this.createdSheets.length === 0) {
      this.logOperation('ROLLBACK_SKIPPED', undefined, {
        reason: 'No sheets to rollback',
      });
      return;
    }

    this.logOperation('ROLLBACK_START', undefined, {
      sheetsToDelete: this.createdSheets.length,
      sheetIds: this.createdSheets,
    });

    const rollbackStartTime = Date.now();
    let deletedCount = 0;
    const failedDeletions: Array<{ sheetId: string; error: string }> = [];

    // Delete all created sheets
    for (const sheetId of this.createdSheets) {
      try {
        await spreadsheetAPI.deleteSheet(sheetId);
        deletedCount++;
        this.logOperation('SHEET_DELETED', sheetId);
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        failedDeletions.push({ sheetId, error: errorMessage });
        this.logOperation('SHEET_DELETE_FAILED', sheetId, {
          error: errorMessage,
        });
      }
    }

    const rollbackDuration = Date.now() - rollbackStartTime;
    this.createdSheets = [];

    if (failedDeletions.length === 0) {
      this.logOperation('ROLLBACK_SUCCESS', undefined, {
        deletedCount,
        duration: rollbackDuration,
      });
    } else {
      this.logOperation('ROLLBACK_PARTIAL', undefined, {
        deletedCount,
        failedCount: failedDeletions.length,
        failedDeletions,
        duration: rollbackDuration,
      });
    }
  }

  /**
   * Apply protection data (best-effort, doesn't affect transaction success)
   */
  private applyProtectionData(
    snapshot: WorkbookSnapshot,
    spreadsheetAPI: SpreadsheetRef
  ): void {
    if (!snapshot.resources || !Array.isArray(snapshot.resources)) {
      return;
    }

    try {
      // Group protection resources by sheet
      const protectionBySheet = new Map<
        string,
        Array<{ name: string; data: string }>
      >();

      for (const resource of snapshot.resources) {
        const res = resource as { name?: string; data?: string };
        if (
          typeof res.name === 'string' &&
          (res.name.includes('PROTECTION') || res.name.includes('PERMISSION'))
        ) {
          // Parse sheet ID from resource data to apply protection to correct sheet
          // Protection resources contain sheet-specific data that needs to be mapped correctly
          const sheetIdFromResource =
            this.extractSheetIdFromProtectionResource(res);
          if (sheetIdFromResource) {
            // Map the original sheet ID to the new sheet ID
            const newSheetId = this.sheetIdMapping.get(sheetIdFromResource);
            if (newSheetId) {
              if (!protectionBySheet.has(newSheetId)) {
                protectionBySheet.set(newSheetId, []);
              }
              protectionBySheet
                .get(newSheetId)
                ?.push(res as { name: string; data: string });
            }
          } else {
            // Fallback: if we can't parse sheet ID, apply to all sheets (current behavior)
            for (const [
              _originalSheetId,
              newSheetId,
            ] of this.sheetIdMapping.entries()) {
              if (!protectionBySheet.has(newSheetId)) {
                protectionBySheet.set(newSheetId, []);
              }
              protectionBySheet
                .get(newSheetId)
                ?.push(res as { name: string; data: string });
            }
          }
        }
      }

      // Apply protection for each sheet
      for (const [
        newSheetId,
        protectionResources,
      ] of protectionBySheet.entries()) {
        if (protectionResources.length > 0) {
          void spreadsheetAPI.applySheetProtection(
            newSheetId,
            protectionResources,
            this.sheetIdMapping
          );
        }
      }
    } catch (error) {
      console.warn('Failed to apply protection data:', error);
      // Don't throw - protection is optional
    }
  }

  /**
   * Get existing sheet names
   */
  private async getExistingSheetNames(
    spreadsheetAPI: SpreadsheetRef
  ): Promise<Set<string>> {
    const existingSheets = await spreadsheetAPI.getAllSheets();
    return new Set(existingSheets.map((s) => s.name));
  }

  /**
   * Generate unique sheet name
   */
  private generateUniqueSheetName(
    baseName: string,
    existingNames: Set<string>
  ): string {
    let sheetName = baseName;
    let nameCounter = 1;

    while (existingNames.has(sheetName)) {
      sheetName = `${baseName} (${nameCounter++})`;
    }

    existingNames.add(sheetName);
    return sheetName;
  }

  /**
   * Extract sheet ID from protection resource data
   * Protection resources in Univer snapshots contain sheet-specific information
   */
  private extractSheetIdFromProtectionResource(resource: {
    name?: string;
    data?: string;
  }): string | null {
    try {
      if (!resource.data) {
        return null;
      }

      // Try to parse the resource data as JSON to find sheet ID
      const parsedData: unknown = JSON.parse(resource.data);

      // Look for sheet ID in common locations within protection data
      if (
        typeof parsedData === 'object' &&
        parsedData !== null &&
        'sheetId' in parsedData
      ) {
        const sheetId = (parsedData as { sheetId?: unknown }).sheetId;
        if (typeof sheetId === 'string') {
          return sheetId;
        }
      }

      // Check if the resource name contains sheet information
      if (resource.name?.includes('sheet')) {
        // Extract sheet ID from resource name if present
        const sheetMatch = resource.name.match(/sheet[_-]?([a-zA-Z0-9]+)/i);
        if (sheetMatch?.[1]) {
          return sheetMatch[1];
        }
      }

      // Look for sheet references in the data structure
      if (typeof parsedData === 'object' && parsedData !== null) {
        // Check for nested sheet references
        const findSheetId = (obj: unknown): string | null => {
          if (typeof obj === 'string' && obj.match(/^[a-zA-Z0-9]+$/)) {
            // Simple heuristic: if it looks like a sheet ID, return it
            return obj;
          }
          if (typeof obj === 'object' && obj !== null) {
            for (const key in obj) {
              if (
                key.toLowerCase().includes('sheet') &&
                typeof (obj as Record<string, unknown>)[key] === 'string'
              ) {
                return (obj as Record<string, unknown>)[key] as string;
              }
              const result = findSheetId((obj as Record<string, unknown>)[key]);
              if (result) {
                return result;
              }
            }
          }
          return null;
        };

        return findSheetId(parsedData);
      }

      return null;
    } catch (error) {
      // If parsing fails, return null
      console.warn('Failed to parse protection resource data:', error);
      return null;
    }
  }

  /**
   * Load sheet data
   */
  private async loadSheetData(
    sheet: SheetSnapshot,
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

      await spreadsheetAPI.loadSheetDataBulk(
        sheetId,
        {
          name: `Sheet-${sheetId}`,
          cellDataMatrix: sheet.cellData as Record<
            number,
            Record<number, ImportCellDataRecord>
          >,
          mergeData: Array.isArray(sheet.mergeData)
            ? (sheet.mergeData as ImportMergeDataItem[])
            : [],
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
}

export class ImportService implements ImportService {
  /**
   * Transactional import manager for atomic multi-sheet operations
   */
  private transactionalManager = new TransactionalImportManager();

  /**
   * Get the transaction log for the last import operation
   * Useful for debugging and monitoring import operations
   */
  public getLastTransactionLog(): TransactionLogEntry[] {
    return this.transactionalManager.getTransactionLog();
  }

  /**
   * Get the transaction ID for the last import operation
   */
  public getLastTransactionId(): string {
    return this.transactionalManager.getTransactionId();
  }
  /**
   * Select file and auto-detect format
   */
  async selectFile(): Promise<{
    filePath: string;
    detectedFormat: ImportFormat;
  } | null> {
    try {
      const file = await open({
        multiple: false,
        filters: [
          {
            name: 'All Supported',
            extensions: ['csv', 'tsv', 'txt', 'parquet', 'anafispread'],
          },
          FORMAT_FILTERS.anafispread,
          FORMAT_FILTERS.csv,
          FORMAT_FILTERS.tsv,
          FORMAT_FILTERS.txt,
          FORMAT_FILTERS.parquet,
        ],
      });

      if (!file) {
        return null;
      }

      const filePath =
        typeof file === 'string'
          ? file
          : ((file as { path?: string }).path ?? '');
      if (!filePath) {
        const error = new SpreadsheetError(
          ERROR_MESSAGES.NO_FILE_PATH,
          SpreadsheetErrorCode.INVALID_OPERATION,
          ErrorCategory.USER,
          ErrorSeverity.MEDIUM,
          { operation: 'selectFile' }
        );
        logError(error);
        return null;
      }

      // Auto-detect format from extension
      const detectedFormat = this.detectFormat(filePath);

      return { filePath, detectedFormat };
    } catch (error) {
      const spreadsheetError = normalizeError(error, 'selectFile');
      logError(spreadsheetError);
      return null;
    }
  }

  /**
   * Get supported formats with descriptions
   */
  getSupportedFormats(): {
    format: ImportFormat;
    description: string;
    extensions: string[];
  }[] {
    return [
      {
        format: 'anafispread',
        description: 'AnaFis Spreadsheet - Lossless native format',
        extensions: ['anafispread'],
      },
      {
        format: 'csv',
        description: 'CSV - Comma-separated values',
        extensions: ['csv'],
      },
      {
        format: 'tsv',
        description: 'TSV - Tab-separated values',
        extensions: ['tsv'],
      },
      {
        format: 'txt',
        description: 'TXT - Text with custom delimiter',
        extensions: ['txt'],
      },
      {
        format: 'parquet',
        description: 'Parquet - Columnar data format',
        extensions: ['parquet'],
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
        const error = new SpreadsheetError(
          ERROR_MESSAGES.SPREADSHEET_NOT_READY,
          SpreadsheetErrorCode.INVALID_OPERATION,
          ErrorCategory.USER,
          ErrorSeverity.HIGH,
          { operation: 'importFile', context: { filePath } }
        );
        logError(error);
        return err({
          message: error.message,
          code: error.code,
        });
      }

      // Special case: anafispread uses direct lossless import
      if (options.format === 'anafispread') {
        const snapshot = await invoke<unknown>('import_anafis_spread_direct', {
          filePath,
        });

        // Validate snapshot structure before processing
        const validatedSnapshot = this.validateSnapshot(snapshot);

        // Handle anafispread mode: append (default) or replace
        const mode = options.anaFisMode ?? 'append';

        if (mode === 'replace') {
          await spreadsheetAPI.loadWorkbookSnapshot(validatedSnapshot);
        } else {
          // Use transactional import manager for atomic multi-sheet operations
          const importResult =
            await this.transactionalManager.importSheetsAtomic(
              validatedSnapshot,
              spreadsheetAPI
            );
          if (isErr(importResult)) {
            const error = new SpreadsheetError(
              importResult.error,
              SpreadsheetErrorCode.OPERATION_FAILED,
              ErrorCategory.DATA,
              ErrorSeverity.HIGH,
              { operation: 'importSheetsAtomic', context: { filePath, mode } }
            );
            logError(error);
            return err({
              message: error.message,
              code: error.code,
            });
          }
        }

        const sheetCount = this.getSheetCount(validatedSnapshot);
        return ok({
          message: `Successfully imported ${sheetCount} sheet(s)`,
          sheetCount,
        });
      }

      // Simple formats (CSV, TSV, TXT, Parquet): import as 2D array
      const importedData = await invoke<{
        sheets: Record<string, (string | number | null)[][]>;
      }>('import_spreadsheet_file', {
        filePath,
        options: {
          format: options.format,
          skipRows: options.skipRows ?? 0,
          delimiter: this.getDelimiter(options),
          encoding: options.encoding ?? 'utf8',
        },
      });

      // Get file dimensions for validation
      const fileDimensions = this.getFileDimensions(importedData);

      // Validate range if using currentRange mode
      let rangeValidation: ImportResult['rangeValidation'];
      if (options.targetMode === 'currentRange' && options.targetRange) {
        rangeValidation = this.validateRange(
          options.targetRange,
          fileDimensions
        );
      }

      // Load data into spreadsheet based on target mode
      const targetMode = options.targetMode ?? 'newSheet';

      for (const [sheetName, sheetData] of Object.entries(
        importedData.sheets
      )) {
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
              if (
                rangeValidation?.willTruncate &&
                rangeValidation.selectedRange
              ) {
                dataToImport = this.truncateDataToRange(
                  cellValues,
                  rangeValidation.selectedRange
                );
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
        fileDimensions,
      };

      // Add range validation info if applicable
      if (rangeValidation) {
        result.rangeValidation = rangeValidation;
      }

      return ok(result);
    } catch (error) {
      const spreadsheetError = normalizeError(error, 'importFile');
      logError(spreadsheetError);
      return err({
        message: spreadsheetError.message,
        code: spreadsheetError.code,
      });
    }
  }

  /**
   * Get file metadata
   * For TXT files, pass delimiter to get accurate column count with that delimiter
   */
  async getFileMetadata(
    filePath: string,
    delimiter?: string
  ): Promise<FileMetadata | null> {
    try {
      return await invoke<FileMetadata>('get_file_metadata', {
        filePath,
        delimiter,
      });
    } catch {
      return null;
    }
  }

  /**
   * Validate snapshot structure and return typed snapshot
   */
  private validateSnapshot(snapshot: unknown): WorkbookSnapshot {
    if (typeof snapshot !== 'object' || snapshot === null) {
      throw new SpreadsheetValidationError(
        ERROR_MESSAGES.INVALID_SNAPSHOT_DATA,
        'snapshot',
        'validateSnapshot',
        { snapshotType: typeof snapshot }
      );
    }

    const snapshotObj = snapshot as Record<string, unknown>;

    // Required fields
    if (typeof snapshotObj.id !== 'string' || snapshotObj.id.trim() === '') {
      throw new SpreadsheetValidationError(
        'Snapshot missing required field: id (must be non-empty string)',
        'id',
        'validateSnapshot',
        { value: snapshotObj.id }
      );
    }
    if (
      typeof snapshotObj.name !== 'string' ||
      snapshotObj.name.trim() === ''
    ) {
      throw new SpreadsheetValidationError(
        'Snapshot missing required field: name (must be non-empty string)',
        'name',
        'validateSnapshot',
        { value: snapshotObj.name }
      );
    }
    if (!snapshotObj.sheets || typeof snapshotObj.sheets !== 'object') {
      throw new SpreadsheetValidationError(
        ERROR_MESSAGES.NO_SHEETS_FOUND,
        'sheets',
        'validateSnapshot',
        { valueType: typeof snapshotObj.sheets }
      );
    }

    // Validate sheets structure
    const sheets: Record<string, SheetSnapshot> = {};
    for (const [sheetId, sheetData] of Object.entries(
      snapshotObj.sheets as Record<string, unknown>
    )) {
      if (typeof sheetData === 'object' && sheetData !== null) {
        const sheet = sheetData as Record<string, unknown>;
        if (
          typeof sheet.id !== 'string' ||
          sheet.id.trim() === '' ||
          typeof sheet.name !== 'string' ||
          sheet.name.trim() === ''
        ) {
          throw new SpreadsheetValidationError(
            `Sheet '${sheetId}' missing required fields: id or name (must be non-empty strings)`,
            'id/name',
            'validateSnapshot',
            { sheetId, sheetData }
          );
        }

        // Additional validation for optional fields
        if (
          sheet.cellData !== undefined &&
          typeof sheet.cellData !== 'object'
        ) {
          throw new SpreadsheetValidationError(
            `Sheet '${sheetId}' has invalid cellData (must be object if present)`,
            'cellData',
            'validateSnapshot',
            { sheetId, valueType: typeof sheet.cellData }
          );
        }
        if (sheet.mergeData !== undefined && !Array.isArray(sheet.mergeData)) {
          throw new SpreadsheetValidationError(
            `Sheet '${sheetId}' has invalid mergeData (must be array if present)`,
            'mergeData',
            'validateSnapshot',
            { sheetId, valueType: typeof sheet.mergeData }
          );
        }
        if (
          sheet.rowCount !== undefined &&
          (typeof sheet.rowCount !== 'number' || sheet.rowCount < 0)
        ) {
          throw new SpreadsheetValidationError(
            `Sheet '${sheetId}' has invalid rowCount (must be non-negative number if present)`,
            'rowCount',
            'validateSnapshot',
            { sheetId, value: sheet.rowCount }
          );
        }
        if (
          sheet.columnCount !== undefined &&
          (typeof sheet.columnCount !== 'number' || sheet.columnCount < 0)
        ) {
          throw new SpreadsheetValidationError(
            `Sheet '${sheetId}' has invalid columnCount (must be non-negative number if present)`,
            'columnCount',
            'validateSnapshot',
            { sheetId, value: sheet.columnCount }
          );
        }

        sheets[sheetId] = sheet as SheetSnapshot;
      } else {
        throw new SpreadsheetValidationError(
          `Invalid sheet data for sheet '${sheetId}' (must be object)`,
          'sheetData',
          'validateSnapshot',
          { sheetId, sheetDataType: typeof sheetData }
        );
      }
    }

    // Validate that we have at least one valid sheet
    const sheetIds = Object.keys(sheets);
    if (sheetIds.length === 0) {
      throw new SpreadsheetValidationError(
        'Snapshot must contain at least one valid sheet',
        'sheets',
        'validateSnapshot',
        { sheetCount: sheetIds.length }
      );
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
      result.styles = snapshotObj.styles as Record<string, SpreadsheetStyle>;
    }
    if (
      Array.isArray(snapshotObj.sheetOrder) &&
      snapshotObj.sheetOrder.every((id) => typeof id === 'string')
    ) {
      result.sheetOrder = snapshotObj.sheetOrder;
    }
    if (Array.isArray(snapshotObj.resources)) {
      const validResources = snapshotObj.resources.filter(
        (r): r is { name: string; data: string } => {
          if (typeof r !== 'object' || r === null) {
            return false;
          }
          const obj = r as Record<string, unknown>;
          return (
            'name' in obj &&
            typeof obj.name === 'string' &&
            'data' in obj &&
            typeof obj.data === 'string'
          );
        }
      );
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
   * Get sheet count from workbook snapshot
   */

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
      if (
        typeof snapshot === 'object' &&
        snapshot !== null &&
        'sheets' in snapshot
      ) {
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
  private getRowCount(
    importedData:
      | { sheets: Record<string, (string | number | null)[][]> }
      | undefined
  ): number {
    if (!importedData?.sheets) {
      return 0;
    }

    const firstSheet = Object.values(importedData.sheets)[0];
    return Array.isArray(firstSheet) ? firstSheet.length : 0;
  }

  /**
   * Get file dimensions from imported data
   */
  private getFileDimensions(importedData: {
    sheets: Record<string, (string | number | null)[][]>;
  }): { rows: number; columns: number } {
    const firstSheet = Object.values(importedData.sheets)[0];
    if (!Array.isArray(firstSheet) || firstSheet.length === 0) {
      return { rows: 0, columns: 0 };
    }

    const rows = firstSheet.length;
    const columns = Math.max(
      ...firstSheet.map((row) => (Array.isArray(row) ? row.length : 0))
    );

    return { rows, columns };
  }

  /**
   * Validate if the selected range can accommodate the file data
   */
  private validateRange(
    rangeRef: string,
    fileDimensions: { rows: number; columns: number }
  ): ImportResult['rangeValidation'] {
    const warnings: string[] = [];
    let willTruncate = false;

    // Validate range format
    try {
      RangeValidator.validateFormat(rangeRef);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      return {
        isValid: false,
        warnings: [`Invalid range format: ${errorMessage}`],
        willTruncate: false,
      };
    }

    // Parse the range to get dimensions
    const rangeBounds = parseRange(rangeRef);
    if (!rangeBounds) {
      return {
        isValid: false,
        warnings: ['Invalid range format'],
        willTruncate: false,
      };
    }

    const selectedRows = rangeBounds.endRow - rangeBounds.startRow + 1;
    const selectedColumns = rangeBounds.endCol - rangeBounds.startCol + 1;

    // Check if range is too small (but allow single cell to import all data)
    if (
      selectedRows < fileDimensions.rows ||
      selectedColumns < fileDimensions.columns
    ) {
      // Don't truncate for single cell - use it as top-left corner for full data import
      if (!(selectedRows === 1 && selectedColumns === 1)) {
        willTruncate = true;
        warnings.push(
          `Selected range (${selectedRows}×${selectedColumns}) is smaller than file data (${fileDimensions.rows}×${fileDimensions.columns}). Data will be truncated to fit.`
        );
      }
    }

    // Check if range is much larger (just informational)
    if (
      selectedRows > fileDimensions.rows * 2 ||
      selectedColumns > fileDimensions.columns * 2
    ) {
      warnings.push(
        `Selected range is much larger than needed. Consider selecting a smaller range for better performance.`
      );
    }

    // Special case: single cell
    if (selectedRows === 1 && selectedColumns === 1) {
      warnings.push(
        'Single cell selected. This will be treated as the top-left corner of the import area.'
      );
    }

    return {
      isValid: true,
      warnings,
      willTruncate,
      selectedRange: {
        rows: selectedRows,
        columns: selectedColumns,
      },
    };
  }

  /**
   * Truncate data to fit the selected range
   */
  private truncateDataToRange(
    data: CellValue[][],
    maxDimensions: { rows: number; columns: number }
  ): CellValue[][] {
    return data
      .slice(0, maxDimensions.rows)
      .map((row) => row.slice(0, maxDimensions.columns));
  }
}
