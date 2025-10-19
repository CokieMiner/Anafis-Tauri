// src/components/spreadsheet/univer/facade.ts - Facade API wrapper with lifecycle awareness
import { FUniver } from '@univerjs/core/facade';
import '@univerjs/sheets/facade';
import '@univerjs/ui/facade';
import { IDisposable } from '@univerjs/core';

export class UniverFacade {
  private univerAPI: ReturnType<typeof FUniver.newAPI> | null = null;
  private lifecycleDisposable: IDisposable | null = null;
  private isInitialized = false;

  constructor(univer: any) {
    this.initializeWithLifecycle(univer);
  }

  private initializeWithLifecycle(univer: any) {
    const tempAPI = FUniver.newAPI(univer);

    this.lifecycleDisposable = tempAPI.addEvent(tempAPI.Event.LifeCycleChanged, ({ stage }) => {
      if (stage === tempAPI.Enum.LifecycleStages.Rendered) {
        this.univerAPI = tempAPI;
        this.isInitialized = true;
        // Now safe to perform facade operations
      }
    });
  }

  // Check if facade is ready for operations
  private ensureInitialized(): boolean {
    return this.isInitialized && this.univerAPI !== null;
  }

  // Dispose lifecycle listener when no longer needed
  dispose() {
    if (this.lifecycleDisposable) {
      this.lifecycleDisposable.dispose();
      this.lifecycleDisposable = null;
    }
  }

  // Simple operations using Facade API
  getActiveWorkbook() {
    if (!this.ensureInitialized()) return null;
    return this.univerAPI!.getActiveWorkbook();
  }

  getActiveSheet() {
    if (!this.ensureInitialized()) return null;
    return this.univerAPI!.getActiveWorkbook()?.getActiveSheet();
  }

  async getCellValue(cellRef: string): Promise<string | number | null> {
    if (!this.ensureInitialized()) return null;

    try {
      const range = this.univerAPI!.getActiveWorkbook()?.getActiveSheet()?.getRange(cellRef);
      const value = await range?.getValue();
      if (value === null || value === undefined) return null;
      if (typeof value === 'boolean') return value ? 1 : 0;
      return value as string | number;
    } catch {
      return null;
    }
  }

  async setCellValue(cellRef: string, value: any): Promise<boolean> {
    if (!this.ensureInitialized()) return false;

    try {
      const range = this.univerAPI!.getActiveWorkbook()?.getActiveSheet()?.getRange(cellRef);
      if (!range) return false;

      await range.setValue(value);
      return true;
    } catch (error) {
      console.error('Failed to set cell value:', error);
      return false;
    }
  }

  async getRangeValues(rangeRef: string): Promise<(string | number)[][]> {
    if (!this.ensureInitialized()) return [];

    try {
      const range = this.univerAPI!.getActiveWorkbook()?.getActiveSheet()?.getRange(rangeRef);
      const values = await range?.getValues();
      if (!values) return [];

      // Convert CellValue[][] to (string | number)[][]
      return values.map(row =>
        row.map(cell => {
          if (cell === null || cell === undefined) return '';
          if (typeof cell === 'boolean') return cell ? 1 : 0;
          return cell as string | number;
        })
      );
    } catch {
      return [];
    }
  }

  async getRangeFull(rangeRef: string): Promise<any[][]> {
    if (!this.ensureInitialized()) return [];

    try {
      const range = this.univerAPI!.getActiveWorkbook()?.getActiveSheet()?.getRange(rangeRef);
      return await range?.getCellDatas() ?? [];
    } catch {
      return [];
    }
  }



  getUsedRange(): string {
    if (!this.ensureInitialized()) return 'A1:Z100';

    try {
      const sheet = this.univerAPI!.getActiveWorkbook()?.getActiveSheet();
      if (!sheet) return 'A1:Z100';

      // Use more efficient algorithm with early termination
      return this.calculateUsedBoundsEfficiently(sheet);
    } catch {
      return 'A1:Z100';
    }
  }

  private calculateUsedBoundsEfficiently(sheet: any): string {
    // Configurable limits for performance
    const MAX_CHECK_ROWS = 1000;
    const MAX_CHECK_COLS = 100;
    const SAMPLE_SIZE = 50; // Check every Nth cell for early termination

    // Helper function to check if a cell has content
    const hasContent = (row: number, col: number): boolean => {
      try {
        const cell = sheet.getRange(row, col).getValue();
        return cell !== null && cell !== undefined && cell !== '';
      } catch {
        return false;
      }
    };

    // Find last non-empty row using binary search with sampling
    let lastRow = -1;
    let rowHigh = Math.min(MAX_CHECK_ROWS, sheet.getMaxRows?.() || 1000);

    // First pass: find approximate bounds with sampling
    for (let row = 0; row < rowHigh; row += SAMPLE_SIZE) {
      for (let col = 0; col < Math.min(MAX_CHECK_COLS, 26); col++) { // Check A-Z first
        if (hasContent(row, col)) {
          lastRow = Math.max(lastRow, row);
        }
      }
    }

    // Second pass: binary search for precise last row
    if (lastRow >= 0) {
      let lo = Math.max(0, lastRow - SAMPLE_SIZE);
      let hi = Math.min(rowHigh, lastRow + SAMPLE_SIZE);

      while (lo <= hi) {
        const mid = Math.floor((lo + hi) / 2);
        let found = false;

        // Check a few columns in this row
        for (let col = 0; col < Math.min(MAX_CHECK_COLS, 10); col++) {
          if (hasContent(mid, col)) {
            found = true;
            break;
          }
        }

        if (found) {
          lastRow = mid;
          lo = mid + 1;
        } else {
          hi = mid - 1;
        }
      }
    }

    // Find last non-empty column using similar approach
    let lastCol = -1;
    let colHigh = Math.min(MAX_CHECK_COLS, sheet.getMaxColumns?.() || 100);

    // First pass: find approximate bounds
    for (let col = 0; col < colHigh; col += Math.floor(SAMPLE_SIZE / 5)) {
      for (let row = 0; row < Math.min(lastRow + 1, 100); row++) {
        if (hasContent(row, col)) {
          lastCol = Math.max(lastCol, col);
        }
      }
    }

    // Second pass: binary search for precise last column
    if (lastCol >= 0) {
      let lo = Math.max(0, lastCol - Math.floor(SAMPLE_SIZE / 5));
      let hi = Math.min(colHigh, lastCol + Math.floor(SAMPLE_SIZE / 5));

      while (lo <= hi) {
        const mid = Math.floor((lo + hi) / 2);
        let found = false;

        // Check a few rows in this column
        for (let row = 0; row < Math.min(lastRow + 1, 50); row++) {
          if (hasContent(row, mid)) {
            found = true;
            break;
          }
        }

        if (found) {
          lastCol = mid;
          lo = mid + 1;
        } else {
          hi = mid - 1;
        }
      }
    }

    if (lastRow === -1 && lastCol === -1) {
      return 'A1:A1';
    }

    const startCol = 'A';
    const endCol = lastCol >= 0 ? this.indexToColumn(lastCol) : 'A';
    const startRow = 1;
    const endRow = lastRow >= 0 ? lastRow + 1 : 1;

    return `${startCol}${startRow}:${endCol}${endRow}`;
  }

  private indexToColumn(index: number): string {
    let result = '';
    index += 1; // Convert to 1-based
    while (index > 0) {
      index -= 1;
      result = String.fromCharCode(65 + (index % 26)) + result;
      index = Math.floor(index / 26);
    }
    return result;
  }

  // Event system integration - simplified for now
  addEvent(eventName: string, callback: (params: any) => void): IDisposable | null {
    if (!this.ensureInitialized()) return null;

    try {
      // Use the core event system
      return this.univerAPI!.addEvent(eventName as any, callback);
    } catch (error) {
      console.error('Failed to add event listener:', error);
      return null;
    }
  }

  // Access to event constants
  get Event() {
    return this.univerAPI?.Event || {};
  }

  async getSelection(): Promise<string | null> {
    if (!this.ensureInitialized()) return null;

    try {
      const sheet = this.univerAPI!.getActiveWorkbook()?.getActiveSheet();
      if (!sheet) return null;

      const selection = sheet.getSelection();
      if (!selection) return null;

      const activeRange = selection.getActiveRange();
      if (!activeRange) return null;

      return activeRange.getA1Notation();
    } catch (error) {
      console.error('Failed to get selection:', error);
      return null;
    }
  }
}
