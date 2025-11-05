// UniverAdapter.tsx - Improved Facade API approach with proper plugin mode integration
import { forwardRef, useImperativeHandle, useRef, useMemo, useCallback } from 'react';
import { ICellData, Univer } from '@univerjs/core';
import { FUniver } from '@univerjs/core/facade';
import { SpreadsheetProps, SpreadsheetRef, CellValue, WorkbookData } from '@/components/spreadsheet/SpreadsheetInterface';
import { 
  safeSpreadsheetOperation,
  convertToUniverData,
  convertToUniverDataMultiSheet,
  convertToUniverCellValue,
  UniverErrorBoundary,
  // Use facade operations instead of duplicating logic
  updateCell,
  getCellValue,
  getRange,
  getRangeFull,
  getSelection,
  // Import services
  ExportService,
  ImportService,
  // Import utilities
  columnToLetter,
  letterToColumn,
} from '../index';
import { getSheetNameSafely } from '../utils/sheetUtils';
import UniverSpreadsheet from './UniverSpreadsheet';


const UniverAdapterInner = forwardRef<SpreadsheetRef, SpreadsheetProps>(
  ({ initialData, onCellChange, onFormulaIntercept, onSelectionChange, tabId }, ref) => {
    const univerAPIRef = useRef<ReturnType<typeof FUniver.newAPI> | null>(null);
    const univerInstanceRef = useRef<Univer | null>(null);
    
    // Callback when Univer instance is ready
    const handleUniverReady = useCallback((univerInstance: Univer) => {
      univerAPIRef.current ??= FUniver.newAPI(univerInstance);
      univerInstanceRef.current = univerInstance;
    }, []);

    // Initialize tracked bounds from initial data (removed - spreadsheets start empty)
    const memoizedOperations = useMemo(() => ({
      updateCell: (cellRef: string, value: CellValue) => {
        // Convert CellValue to facade operation format
        const facadeValue: { v?: string | number; f?: string } = {};
        if (value.v !== null && value.v !== undefined && typeof value.v !== 'boolean') {
          facadeValue.v = value.v;
        }
        if (value.f) {
          facadeValue.f = value.f;
        }
        return updateCell({ current: univerAPIRef.current }, cellRef, facadeValue);
      },

      batchUpdateCells: (updates: Array<{ cellRef: string; value: CellValue }>) => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for batch updates');
        }

        return safeSpreadsheetOperation(async () => {
          for (const { cellRef, value } of updates) {
            const facadeValue: { v?: string | number; f?: string } = {};
            if (value.v !== null && value.v !== undefined && typeof value.v !== 'boolean') {
              facadeValue.v = value.v;
            }
            if (value.f) {
              facadeValue.f = value.f;
            }
            await updateCell({ current: univerAPIRef.current }, cellRef, facadeValue);
          }
        }, 'batch update cells');
      },

      // Simple range update method
      updateRange: (rangeRef: string, values: CellValue[][]) => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for range updates');
        }

        return safeSpreadsheetOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheet = workbook.getActiveSheet();

          // Calculate the range dimensions based on the data
          const numRows = values.length;
          const numCols = values.length > 0 ? (values[0]?.length ?? 0) : 0;

          if (numRows === 0 || numCols === 0) {
            return; // Nothing to update
          }

          // Parse the starting cell reference
          const startCell = rangeRef.match(/^([A-Z]+)(\d+)$/);
          if (!startCell?.[1] || !startCell[2]) {
            throw new Error(`Invalid range reference: ${rangeRef}`);
          }

          const startCol = startCell[1];
          const startRow = parseInt(startCell[2], 10);

          // Calculate end cell
          const endColIndex = columnToLetter(letterToColumn(startCol) + numCols - 1);
          const endRow = startRow + numRows - 1;
          const endCell = `${endColIndex}${endRow}`;

          // Create range from start to end
          const range = sheet.getRange(`${rangeRef}:${endCell}`);

          // Convert CellValue[][] to what Univer expects
          const univerValues = values.map(row =>
            row.map(cell => convertToUniverCellValue(cell))
          );

          range.setValues(univerValues);
        }, 'update range');
      },

      getCellValue: (cellRef: string): Promise<string | number | null> => {
        return getCellValue({ current: univerAPIRef.current }, cellRef);
      },

      getRange: (rangeRef: string): Promise<(string | number)[][]> => {
        return getRange({ current: univerAPIRef.current }, rangeRef);
      },

      getRangeFull: (rangeRef: string): Promise<CellValue[][]> => {
        return getRangeFull({ current: univerAPIRef.current }, rangeRef);
      },

      getSelection: (): Promise<string | null> => {
        return getSelection({ current: univerAPIRef.current });
      },
      isReady: () => !!univerAPIRef.current,

      // Sheet management operations
      createSheet: (name: string, rows = 100, cols = 20) => {
        if (!univerAPIRef.current) {
          return Promise.reject(new Error('Univer API not ready'));
        }
        const workbook = univerAPIRef.current.getActiveWorkbook();
        if (!workbook) {
          return Promise.reject(new Error('No active workbook'));
        }
        
        const newSheet = workbook.create(name, rows, cols);
        // Activate the newly created sheet
        workbook.setActiveSheet(newSheet);
        
        return Promise.resolve(newSheet.getSheetId());
      },
      
      getAllSheets: () => {
        if (!univerAPIRef.current) {
          return Promise.resolve([]);
        }
        const workbook = univerAPIRef.current.getActiveWorkbook();
        if (!workbook) {
          return Promise.resolve([]);
        }
        
        const sheets = workbook.getSheets();
        return Promise.resolve(sheets.map(sheet => ({
          id: sheet.getSheetId(),
          name: getSheetNameSafely(sheet, sheets)
        })));
      },
      
      getWorkbookSnapshot: () => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for snapshot export');
        }
        const workbook = univerAPIRef.current.getActiveWorkbook();
        if (!workbook) {
          throw new Error('No active workbook for snapshot export');
        }
        // Use the save() method to get the complete IWorkbookData snapshot
        // This includes all sheets, styles, protection rules, resources, etc.
        // Wrap in Promise.resolve since interface expects Promise<unknown>
        return Promise.resolve(workbook.save());
      },

      loadWorkbookSnapshot: (snapshot: unknown) => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for snapshot import');
        }
        
        // Validate that snapshot is an object
        if (!snapshot || typeof snapshot !== 'object') {
          throw new Error('Invalid snapshot data');
        }
        
        // Use Univer's createWorkbook to load the snapshot as a new workbook
        // This replaces the entire workbook with the snapshot data
        const univerAPI = univerAPIRef.current;
        const currentWorkbook = univerAPI.getActiveWorkbook();
        
        if (currentWorkbook) {
          // Dispose of current workbook using disposeUnit
          univerAPI.disposeUnit(currentWorkbook.getId());
        }
        
        // Determine format: check for explicit marker first, then simple heuristic
        const snapshotObj = snapshot as Record<string, unknown>;
        const explicitFormat = (snapshotObj as { __format?: string }).__format;
        
        let isAbstractFormat = false;
        if (explicitFormat) {
          isAbstractFormat = explicitFormat === 'abstract';
        } else {
          // Simple heuristic: check if first sheet has A1-style keys
          const firstSheet = Object.values(snapshotObj.sheets ?? {})[0] as Record<string, unknown> | undefined;
          if (firstSheet?.cellData && typeof firstSheet.cellData === 'object') {
            const cellDataKeys = Object.keys(firstSheet.cellData);
            // If any key looks like A1, B2, etc., it's abstract format
            isAbstractFormat = cellDataKeys.some(key => /^[A-Z]+\d+$/.test(key));
          }
        }

        // Convert if needed
        let snapshotToLoad = snapshot;
        
        if (isAbstractFormat) {
          // Convert abstract WorkbookData format to Univer IWorkbookData
          snapshotToLoad = convertToUniverDataMultiSheet(snapshot as WorkbookData);
        }        // Create new workbook from snapshot
        univerAPI.createWorkbook(snapshotToLoad);
        
        // Apply protection from resources if present
        const snapshotWithResources = snapshotToLoad as { resources?: Array<{ name: string; data: string }> };
        
        // Use univerAPIRef to get the Univer instance since we just created a workbook
        const univerInstanceForProtection = univerInstanceRef.current ?? (univerAPIRef.current as unknown as Record<string, Univer>).__univerInstance;
        
        if (snapshotWithResources.resources?.length && univerInstanceForProtection) {
          // Create AbortController for cancellation support
          const abortController = new AbortController();
          
          // Import protection utility and apply rules
          import('../utils/protectionUtils').then(({ applyProtectionRules }) => {
            return applyProtectionRules(snapshotWithResources.resources!, univerInstanceForProtection, abortController.signal)
              .catch((error: unknown) => {
                console.error('Failed to apply protection rules:', error);
                // Don't re-throw here as this is a best-effort operation
              })
              .finally(() => {
                // Clean up abort controller after protection rules are applied
                if (!abortController.signal.aborted) {
                  abortController.abort();
                }
              });
          }).catch((error: unknown) => {
            console.error('Failed to import protection utilities:', error);
          });
        }
        
        // Wrap in Promise.resolve since interface expects Promise<void>
        return Promise.resolve();
      },

      // Internal API access for advanced operations
      getImplementationContext: () => ({
        univerInstance: univerInstanceRef.current,
        facadeInstance: univerAPIRef.current
      }),

      // Service access for import/export operations
      getExportService: () => new ExportService(),
      getImportService: () => new ImportService()
    }), []); 
    
    useImperativeHandle(ref, () => memoizedOperations, [memoizedOperations]);

    // Memoized callback to handle type conversion
    const handleCellChange = useCallback((cellRef: string, univerCellData: ICellData) => {
      const abstractCellData: CellValue = {};

      if (univerCellData.v !== undefined && univerCellData.v !== null) {
        if (typeof univerCellData.v === 'boolean' || typeof univerCellData.v === 'string' || typeof univerCellData.v === 'number') {
          abstractCellData.v = univerCellData.v;
        }
      }

      if (univerCellData.f) {
        abstractCellData.f = univerCellData.f;
      }

      if (univerCellData.s) {
        abstractCellData.style = univerCellData.s;
      }

      if (univerCellData.p) {
        abstractCellData.meta = { custom: univerCellData.p };
      }

      onCellChange(cellRef, abstractCellData);
    }, [onCellChange]);

    // Memoized the converted data to prevent unnecessary re-initialization
    const univerData = useMemo(() => convertToUniverData(initialData), [initialData]);

    return (
      <UniverSpreadsheet
        initialData={univerData}
        onCellChange={handleCellChange}
        onFormulaIntercept={onFormulaIntercept}
        onSelectionChange={onSelectionChange ?? (() => { })}
        onUniverReady={handleUniverReady}
        {...(tabId && { tabId })}
      />
    );
  }
);

// Export wrapped component with lightweight error boundary
export const UniverAdapter = forwardRef<SpreadsheetRef, SpreadsheetProps>(
  (props, ref) => (
    <UniverErrorBoundary>
      <UniverAdapterInner {...props} ref={ref} />
    </UniverErrorBoundary>
  )
);