// UniverAdapter.tsx - Improved Facade API approach with proper plugin mode integration
import { forwardRef, useImperativeHandle, useRef, useMemo, useState, useCallback } from 'react';
import { ICellData, Univer } from '@univerjs/core';
import { FUniver } from '@univerjs/core/facade';
import { FSelection } from '@univerjs/sheets/facade';
import { SpreadsheetProps, SpreadsheetRef, CellValue } from '../SpreadsheetInterface';
import { safeUniverOperation } from './errors';
import { convertToUniverData, convertToUniverCellValue, convertFromUniverCellData } from './dataConversion';
import { LightweightErrorBoundary } from './UniverErrorBoundary';
import UniverSpreadsheet from './UniverSpreadsheet';


const UniverAdapterInner = forwardRef<SpreadsheetRef, SpreadsheetProps>(
  ({ initialData, onCellChange, onFormulaIntercept, onSelectionChange, tabId }, ref) => {
    const univerAPIRef = useRef<ReturnType<typeof FUniver.newAPI> | null>(null);
    // Runtime tracking of maximum bounds per sheet for efficient used range calculation
    const [sheetBounds, setSheetBounds] = useState<Record<string, { maxRow: number; maxCol: number }>>({});
    // Callback when Univer instance is ready
    const handleUniverReady = useCallback((univerInstance: Univer) => {
      univerAPIRef.current ??= FUniver.newAPI(univerInstance);
    }, []);

    // Helper function to convert column number to letter (0 = A, 1 = B, etc.)
    const columnToLetter = useCallback((col: number): string => {
      let temp, letter = '';
      let column = col;
      while (column >= 0) {
        temp = column % 26;
        letter = String.fromCharCode(temp + 65) + letter;
        column = Math.floor(column / 26) - 1;
      }
      return letter;
    }, []);

    // Helper function to parse A1 notation and return 0-indexed row/col
    const parseCellRef = useCallback((cellRef: string): { row: number; col: number } | null => {
      const match = cellRef.match(/^([A-Z]+)(\d+)$/);
      if (!match) { return null; }

      const colStr = match[1]!;
      const rowStr = match[2]!;

      // Convert column letters to number (A=0, B=1, etc.)
      let col = 0;
      for (let i = 0; i < colStr.length; i++) {
        col = col * 26 + (colStr.charCodeAt(i) - 65 + 1);
      }
      col -= 1; // Convert to 0-indexed

      const row = parseInt(rowStr, 10) - 1; // Convert to 0-indexed

      return { row, col };
    }, []);

    // Helper function to parse range or single cell reference and return all coordinates
    const parseRangeOrCell = useCallback((cellRef: string): { row: number; col: number }[] => {
      if (cellRef.includes(':')) {
        // Handle range like "A1:C3"
        const parts = cellRef.split(':');
        const startRef = parts[0];
        const endRef = parts[1];
        
        if (!startRef || !endRef) {
          return [];
        }
        
        const startCoords = parseCellRef(startRef);
        const endCoords = parseCellRef(endRef);
        
        if (!startCoords || !endCoords) {
          return [];
        }
        
        // Return both start and end coordinates for bounds calculation
        return [startCoords, endCoords];
      } else {
        // Handle single cell like "A1"
        const coords = parseCellRef(cellRef);
        return coords ? [coords] : [];
      }
    }, [parseCellRef]);

    // Initialize tracked bounds from initial data (removed - spreadsheets start empty)
    const memoizedOperations = useMemo(() => ({
      updateCell: (cellRef: string, value: CellValue) => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for cell updates');
        }

        return safeUniverOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheet = workbook.getActiveSheet();
          const range = sheet.getRange(cellRef);
          const univerValue = convertToUniverCellValue(value);

          range.setValue(univerValue);
        }, 'update cell');
      },

      batchUpdateCells: (updates: Array<{ cellRef: string; value: CellValue }>) => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for batch updates');
        }

        return safeUniverOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheet = workbook.getActiveSheet();

          for (const { cellRef, value } of updates) {
            const range = sheet.getRange(cellRef);
            const univerValue = convertToUniverCellValue(value);
            range.setValue(univerValue);
          }
        }, 'batch update cells');
      },

      // Simple range update method
      updateRange: (rangeRef: string, values: CellValue[][]) => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for range updates');
        }

        return safeUniverOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheet = workbook.getActiveSheet();
          const range = sheet.getRange(rangeRef);

          // Convert CellValue[][] to what Univer expects
          const univerValues = values.map(row =>
            row.map(cell => convertToUniverCellValue(cell))
          );

          range.setValues(univerValues);
        }, 'update range');
      },

      getCellValue: (cellRef: string): Promise<string | number | null> => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for getting cell value');
        }

        return safeUniverOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheet = workbook.getActiveSheet();
          const range = sheet.getRange(cellRef);
          const value = range.getValue();

          // Convert boolean values to strings to match interface
          if (typeof value === 'boolean') {
            return value.toString();
          }

          return value;
        }, 'get cell value');
      },

      getRange: (rangeRef: string): Promise<(string | number)[][]> => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for getting range');
        }

        return safeUniverOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheet = workbook.getActiveSheet();
          const range = sheet.getRange(rangeRef);
          const values = range.getValues();

          // Convert any boolean values to strings and handle null/undefined
          return values.map(row =>
            row.map(cell => {
              if (cell === null || cell === undefined) { return ''; }
              if (typeof cell === 'boolean') { return cell.toString(); }
              return cell;
            })
          );
        }, 'get range', []);
      },

      getRangeFull: (rangeRef: string): Promise<CellValue[][]> => {
        if (!univerAPIRef.current) {
          throw new Error('Facade API not ready for getting range full');
        }

        return safeUniverOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheet = workbook.getActiveSheet();
          const range = sheet.getRange(rangeRef);
          const cellDatas = range.getCellDatas();

          // Convert ICellData[][] to CellValue[][]
          return cellDatas.map(row =>
            row.map(cell => cell ? convertFromUniverCellData(cell) : { v: '' })
          );
        }, 'get range full', []);
      },

      getAllSheetsData: (): Promise<{ name: string; data: CellValue[][] }[]> => {
        if (!univerAPIRef.current) {
          return Promise.resolve([]);
        }

        return safeUniverOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheets = workbook.getSheets();
          const result: { name: string; data: CellValue[][] }[] = [];

          for (const sheet of sheets) {
            const sheetId = sheet.getSheetId();
            const bounds = sheetId ? sheetBounds[sheetId] : null;

            // Use actual used range instead of hardcoded A1:Z100
            let rangeStr = 'A1:A1'; // fallback for empty sheets
            if (bounds && bounds.maxRow >= 0 && bounds.maxCol >= 0) {
              const endCol = columnToLetter(bounds.maxCol);
              const endRow = bounds.maxRow + 1;
              rangeStr = `A1:${endCol}${endRow}`;
            }

            const range = sheet.getRange(rangeStr);
            const cellDatas = range.getCellDatas();

            // Get real sheet name with fallback
            let sheetName = '';
            try {
              // Use official Facade API to get sheet name
              sheetName = sheet.getSheetName();
            } catch {
              // Fallback to indexed name if API call fails
              sheetName = `Sheet ${sheets.indexOf(sheet) + 1}`;
            }

            result.push({
              name: sheetName,
              data: cellDatas.map(row =>
                row.map(cell => cell ? convertFromUniverCellData(cell) : { v: '' })
              )
            });
          }

          return result;
        }, 'get all sheets data', []);
      },

      getSelection: (): Promise<string | null> => {
        if (!univerAPIRef.current) {
          return Promise.resolve(null);
        }

        return safeUniverOperation(() => {
          const workbook = univerAPIRef.current!.getActiveWorkbook()!;
          const sheet = workbook.getActiveSheet();
          const selection: FSelection = sheet.getSelection()!;

          // Get the active range from the selection
          const activeRange = selection.getActiveRange();
          if (activeRange) {
            return activeRange.getA1Notation();
          }

          return null;
        }, 'get selection');
      },
      getUsedRange: (): string => {
        // Use runtime tracked bounds for instant calculation
        // NOTE: Bounds are grow-only for performance - may include empty cells
        // that were previously used but have since been cleared
        if (!univerAPIRef.current) {
          return 'A1:A1';
        }

        const workbook = univerAPIRef.current.getActiveWorkbook();
        if (!workbook) {
          return 'A1:A1';
        }

        const sheet = workbook.getActiveSheet();
        const sheetId = sheet.getSheetId();
        const bounds = sheetId ? sheetBounds[sheetId] : null;

        if (bounds && bounds.maxRow >= 0 && bounds.maxCol >= 0) {
          const endCol = columnToLetter(bounds.maxCol);
          const endRow = bounds.maxRow + 1;
          return `A1:${endCol}${endRow}`;
        }
        // Fallback for empty sheet
        return 'A1:A1';
      },

      getTrackedBounds: () => {
        // Ensure all sheets have at least A1 bounds (empty spreadsheets start with A1 available)
        if (univerAPIRef.current) {
          const workbook = univerAPIRef.current.getActiveWorkbook();
          if (workbook) {
            const sheets = workbook.getSheets();
            const updatedBounds = { ...sheetBounds };
            let hasChanges = false;
            
            for (const sheet of sheets) {
              const sheetId = sheet.getSheetId();
              if (sheetId && !updatedBounds[sheetId]) {
                // Initialize empty sheets with A1 bounds
                updatedBounds[sheetId] = { maxRow: 0, maxCol: 0 };
                hasChanges = true;
              }
            }
            
            if (hasChanges) {
              setSheetBounds(updatedBounds);
            }
          }
        }
        
        // Validate bounds before returning
        const validatedBounds: Record<string, { maxRow: number; maxCol: number }> = {};
        Object.entries(sheetBounds).forEach(([sheetId, bounds]) => {
          if (typeof bounds.maxRow === 'number' && typeof bounds.maxCol === 'number' && 
              bounds.maxRow >= 0 && bounds.maxCol >= 0) {
            validatedBounds[sheetId] = bounds;
          }
        });
        
        return validatedBounds;
      },
      isFacadeReady: () => !!univerAPIRef.current,
      getRawAPI: () => univerAPIRef.current
    }), [columnToLetter, sheetBounds]); 
    
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

      // Update tracked bounds when cell data changes
      // DESIGN DECISION: Bounds only grow (never shrink) for performance reasons
      // When cells are cleared/deleted, bounds remain at their historical maximum
      // This avoids expensive full-sheet scanning on every cell clear operation
      // Full bounds recalculation only occurs on explicit getUsedRange() calls
      // Trade-off: Slightly inflated bounds vs. maintaining responsive cell operations
      if (univerAPIRef.current) {
        const workbook = univerAPIRef.current.getActiveWorkbook();
        if (workbook) {
          const sheet = workbook.getActiveSheet();
          const sheetId = sheet.getSheetId();

          if (sheetId) {
            const coordinates = parseRangeOrCell(cellRef);
            if (coordinates.length > 0) {
              setSheetBounds(prev => {
                const currentBounds = prev[sheetId] ?? { maxRow: 0, maxCol: 0 };
                let newMaxRow = currentBounds.maxRow;
                let newMaxCol = currentBounds.maxCol;
                
                // Update bounds for all coordinates in the range
                coordinates.forEach(coord => {
                  newMaxRow = Math.max(newMaxRow, coord.row);
                  newMaxCol = Math.max(newMaxCol, coord.col);
                });
                
                const newBounds = {
                  ...prev,
                  [sheetId]: {
                    maxRow: newMaxRow,
                    maxCol: newMaxCol
                  }
                };
                

                
                return newBounds;
              });
            }
          }
        }
      }

      onCellChange(cellRef, abstractCellData);
    }, [onCellChange, parseRangeOrCell]);

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
    <LightweightErrorBoundary>
      <UniverAdapterInner {...props} ref={ref} />
    </LightweightErrorBoundary>
  )
);