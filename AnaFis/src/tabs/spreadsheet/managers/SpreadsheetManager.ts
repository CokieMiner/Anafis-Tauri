import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { ExportService } from '@/tabs/spreadsheet/univer/operations/exportService';
import { ImportService } from '@/tabs/spreadsheet/univer/operations/importService';

/**
 * SpreadsheetManager explicitly unbinds the UI lifecycle from the Spreadsheet engine.
 * It holds the Global Reference to the active spreadsheet, and provides singletons
 * to the Export and Import pipelines.
 *
 * This enables cross-tab calculations, auto-saving logic, and external manipulation
 * without requiring the React component to mount Export/Import services constantly.
 */
class SpreadsheetManagerService {
  private activeSpreadsheet: SpreadsheetRef | null = null;
  private exportService: ExportService;
  private importService: ImportService;

  constructor() {
    this.exportService = new ExportService();
    this.importService = new ImportService();
  }

  /**
   * Registers a SpreadsheetRef wrapper instance
   */
  public register(spreadsheet: SpreadsheetRef): void {
    this.activeSpreadsheet = spreadsheet;
  }

  /**
   * Cleans up the active spreadsheet when the engine fundamentally unmounts
   */
  public deregister(spreadsheet: SpreadsheetRef): void {
    if (this.activeSpreadsheet === spreadsheet) {
      this.activeSpreadsheet = null;
    }
  }

  /**
   * Get the global adapter reference to the Spreadsheet
   * Throws an error if used when spreadsheet is unmounted (unless caught)
   */
  public getActiveSpreadsheet(): SpreadsheetRef | null {
    return this.activeSpreadsheet;
  }

  public getExportService(): ExportService {
    return this.exportService;
  }

  public getImportService(): ImportService {
    return this.importService;
  }
}

export const SpreadsheetManager = new SpreadsheetManagerService();
