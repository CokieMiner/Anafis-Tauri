// UniverAdapter.tsx - Improved adapter using hybrid Facade/Injector approach with lifecycle awareness
import { forwardRef, useImperativeHandle, useRef, useMemo, useEffect, useState } from 'react';
import { ICellData } from '@univerjs/core';
import UniverSpreadsheet, { UniverSpreadsheetRef as OriginalUniverRef } from './UniverSpreadsheet';
import { UniverFacade } from './facade';
import { SpreadsheetProps, SpreadsheetRef, CellValue } from '../SpreadsheetInterface';
import { handleUniverError } from './errors';
import { getRange, getRangeFull, getAllSheetsData, getSelection, getUsedRange, setupSheetInterceptors, updateCell as updateCellOperation, getCellValue as getCellValueOperation } from './spreadsheetOperations';
import { convertToUniverData, convertToUniverCellValue } from './dataConversion';

export const UniverAdapter = forwardRef<SpreadsheetRef, SpreadsheetProps>(
  ({ initialData, onCellChange, onFormulaIntercept, onSelectionChange }, ref) => {
    const univerRef = useRef<OriginalUniverRef>(null);
    const facadeRef = useRef<UniverFacade | null>(null);
    const [isFacadeReady, setIsFacadeReady] = useState(false);

    // Initialize Facade when Univer instance is available
    useEffect(() => {
      if (univerRef.current) {
        // Setup interceptors
        setupSheetInterceptors(univerRef);
        
        // Initialize Facade - note: we need access to the univer instance
        // This will be passed via a callback from UniverSpreadsheet
      }
    }, []);

    useImperativeHandle(ref, () => ({
      updateCell: async (cellRef: string, value: CellValue) => {
        try {
          const univerValue = convertToUniverCellValue(value);
          
          // Use Facade API if ready, otherwise use injector operations
          if (isFacadeReady && facadeRef.current) {
            const success = await facadeRef.current.setCellValue(cellRef, univerValue);
            if (!success) {
              // Fallback to injector method
              updateCellOperation(univerRef, cellRef, univerValue);
            }
          } else {
            // Use injector-based operation
            updateCellOperation(univerRef, cellRef, univerValue);
          }
        } catch (error) {
          handleUniverError('update cell', error);
        }
      },

      batchUpdateCells: async (updates: Array<{ cellRef: string; value: CellValue }>) => {
        try {
          // Parallel updates for better performance than sequential
          await Promise.all(
            updates.map(({ cellRef, value }) => {
              const univerValue = convertToUniverCellValue(value);
              return updateCellOperation(univerRef, cellRef, univerValue);
            })
          );
        } catch (error) {
          handleUniverError('batch update cells', error);
          throw error;
        }
      },

      getCellValue: async (cellRef: string): Promise<string | number | null> => {
        try {
          // Use Facade API if ready, otherwise use injector operations
          if (isFacadeReady && facadeRef.current) {
            const value = await facadeRef.current.getCellValue(cellRef);
            if (value !== null) return value;
          }
          
          // Fallback to injector method
          return getCellValueOperation(univerRef, cellRef);
        } catch (error) {
          handleUniverError('get cell value', error);
          throw error; // Propagate error instead of silently returning null
        }
      },

      getRange: async (rangeRef: string): Promise<(string | number)[][]> => {
        try {
          // Use Facade API if ready, otherwise use injector operations
          if (isFacadeReady && facadeRef.current) {
            const values = await facadeRef.current.getRangeValues(rangeRef);
            if (values.length > 0) return values;
          }
          
          // Fallback to injector method
          return await getRange(univerRef, rangeRef) ?? [];
        } catch (error) {
          handleUniverError('get range', error);
          throw error;
        }
      },

      getRangeFull: async (rangeRef: string): Promise<CellValue[][]> => {
        console.log('[UniverAdapter.getRangeFull] Called with:', rangeRef);
        
        if (!univerRef.current) {
          console.error('[UniverAdapter.getRangeFull] univerRef.current is null!');
          return [];
        }
        
        try {
          // Complex operation - always use injector-based method
          console.log('[UniverAdapter.getRangeFull] Calling getRangeFull from operations...');
          const result = await getRangeFull(univerRef, rangeRef) ?? [];
          console.log('[UniverAdapter.getRangeFull] Result:', result);
          return result;
        } catch (error) {
          console.error('[UniverAdapter.getRangeFull] Error:', error);
          handleUniverError('get range full', error);
          throw error;
        }
      },

      getAllSheetsData: async (): Promise<{ name: string; data: CellValue[][] }[]> => {
        console.log('[UniverAdapter.getAllSheetsData] Called');
        try {
          // Complex multi-sheet operation - use injector-based method
          const result = await getAllSheetsData(univerRef) ?? [];
          console.log('[UniverAdapter.getAllSheetsData] Result:', result);
          return result;
        } catch (error) {
          console.error('[UniverAdapter.getAllSheetsData] Error:', error);
          handleUniverError('get all sheets data', error);
          throw error;
        }
      },

      getSelection: async (): Promise<string | null> => {
        try {
          // Use injector operations (Facade selection API is unreliable)
          return await getSelection(univerRef) ?? null;
        } catch (error) {
          handleUniverError('get selection', error);
          throw error;
        }
      },

      getUsedRange: async (): Promise<string> => {
        try {
          // Use injector-based calculation (more reliable than Facade)
          return await getUsedRange(univerRef) ?? 'A1:Z100';
        } catch (error) {
          handleUniverError('get used range', error);
          return 'A1:Z100'; // Safe fallback for this operation
        }
      }
    }));

    // Callback to receive Univer instance from UniverSpreadsheet
    const handleUniverReady = (univerInstance: any) => {
      try {
        // Initialize Facade with the Univer instance
        const facade = new UniverFacade(univerInstance);
        facadeRef.current = facade;
        setIsFacadeReady(true);
      } catch (error) {
        console.error('Failed to initialize Facade:', error);
        // Continue without Facade - injector methods will be used
      }
    };

    // Cleanup Facade on unmount
    useEffect(() => {
      return () => {
        if (facadeRef.current) {
          facadeRef.current.dispose();
          facadeRef.current = null;
        }
      };
    }, []);

    // Wrap callbacks to handle type conversion
    const handleCellChange = (cellRef: string, univerCellData: ICellData) => {
      const abstractCellData: CellValue = {
        v: typeof univerCellData.v === 'boolean' ? undefined : univerCellData.v || undefined,
        f: univerCellData.f || undefined,
        style: univerCellData.s,
        meta: univerCellData.p ? { custom: univerCellData.p } : undefined
      };
      onCellChange(cellRef, abstractCellData);
    };

    // Memoize the converted data to prevent unnecessary re-initialization
    const univerData = useMemo(() => convertToUniverData(initialData), [initialData]);

    return (
      <UniverSpreadsheet
        ref={univerRef}
        initialData={univerData}
        onCellChange={handleCellChange}
        onFormulaIntercept={onFormulaIntercept}
        onSelectionChange={onSelectionChange}
        onUniverReady={handleUniverReady}
      />
    );
  }
);