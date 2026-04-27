import type { IWorkbookData, Univer } from '@univerjs/core';
import { FUniver } from '@univerjs/core/facade';
import type { FWorksheet } from '@univerjs/sheets/facade';
import '@univerjs/sheets/facade'; // Side effect import for type augmentations

import {
  forwardRef,
  useCallback,
  useImperativeHandle,
  useMemo,
  useRef,
} from 'react';
import type {
  CellValue,
  SpreadsheetProps,
  SpreadsheetRef,
  WorkbookData,
} from '@/tabs/spreadsheet/types/SpreadsheetInterface';
// Import type augmentations
import '@/tabs/spreadsheet/types/univer-augmentations';
import UniverSpreadsheet from '@/tabs/spreadsheet/univer/core/UniverSpreadsheet';
import {
  // Import utilities
  convertToUniverData,
  determineUsedRange,
  // Import services
  getCellValue,
  getRange,
  getRangeFull,
  getSelection,
  parseCellRef,
  safeSpreadsheetOperation,
  UniverErrorBoundary,
  // Use facade operations instead of duplicating logic
  updateCell,
} from '@/tabs/spreadsheet/univer/index';
import {
  DEFAULT_SHEET_COLS,
  DEFAULT_SHEET_ROWS,
} from '@/tabs/spreadsheet/univer/utils/constants';
import { SequentialSpreadsheetQueue } from '@/tabs/spreadsheet/univer/utils/SequentialSpreadsheetQueue';
import { getSheetNameSafely } from '@/tabs/spreadsheet/univer/utils/sheetUtils';
import { GlobalDataMapper } from '@/tabs/spreadsheet/univer/workers/DataMapperClient';

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/**
 * Converts CellValue to facade operation format
 */
const convertCellValueToFacade = (
  value: CellValue
): { v?: string | number; f?: string } => {
  const facadeValue: { v?: string | number; f?: string } = {};
  if (
    value.v !== null &&
    value.v !== undefined &&
    typeof value.v !== 'boolean'
  ) {
    facadeValue.v = value.v;
  }
  if (value.f) {
    facadeValue.f = value.f;
  }
  return facadeValue;
};

// ============================================================================
// MAIN COMPONENT
// ============================================================================

const UniverAdapterInner = forwardRef<SpreadsheetRef, SpreadsheetProps>(
  ({ initialData, onSelectionChange, onReady, tabId }, ref) => {
    const univerAPIRef = useRef<ReturnType<typeof FUniver.newAPI> | null>(null);
    const univerInstanceRef = useRef<Univer | null>(null);
    const operationQueueRef = useRef<SequentialSpreadsheetQueue | null>(null);

    // Callback when Univer instance is ready
    const handleUniverReady = useCallback(
      (univerInstance: Univer) => {
        univerAPIRef.current ??= FUniver.newAPI(univerInstance);
        univerInstanceRef.current = univerInstance;
        // Initialize the operation queue
        operationQueueRef.current ??= new SequentialSpreadsheetQueue();
        // Notify parent component that the adapter is ready
        onReady?.();
      },
      [onReady]
    );

    // ============================================================================
    // MEMOIZED OPERATIONS - ORGANIZED BY FUNCTIONALITY
    // ============================================================================

    const memoizedOperations = useMemo(
      () => ({
        // ========================================================================
        // BASIC CELL & RANGE OPERATIONS
        // ========================================================================

        updateCell: (cellRef: string, value: CellValue) => {
          if (!operationQueueRef.current) {
            throw new Error('Operation queue not initialized');
          }
          return operationQueueRef.current.enqueue(() => {
            const facadeValue = convertCellValueToFacade(value);
            updateCell({ current: univerAPIRef.current }, cellRef, facadeValue);
            return Promise.resolve();
          });
        },

        batchUpdateCells: (
          updates: Array<{ cellRef: string; value: CellValue }>
        ) => {
          if (!operationQueueRef.current) {
            throw new Error('Operation queue not initialized');
          }
          return operationQueueRef.current.enqueue(() => {
            if (!univerAPIRef.current) {
              throw new Error('Facade API not ready for batch updates');
            }

            return safeSpreadsheetOperation(() => {
              for (const { cellRef, value } of updates) {
                const facadeValue = convertCellValueToFacade(value);
                updateCell(
                  { current: univerAPIRef.current },
                  cellRef,
                  facadeValue
                );
              }
            }, 'batch update cells');
          });
        },

        updateRange: (rangeRef: string, values: CellValue[][]) => {
          if (!operationQueueRef.current) {
            throw new Error('Operation queue not initialized');
          }
          return operationQueueRef.current.enqueue(() => {
            if (!univerAPIRef.current) {
              throw new Error('Facade API not ready for range updates');
            }

            return safeSpreadsheetOperation(async () => {
              const workbook = univerAPIRef.current?.getActiveWorkbook();
              if (!workbook) return;
              const sheet = workbook.getActiveSheet();

              const numRows = values.length;
              const numCols = values.length > 0 ? (values[0]?.length ?? 0) : 0;

              if (numRows === 0 || numCols === 0) {
                return; // Nothing to update
              }

              const startCoords = parseCellRef(rangeRef);
              if (!startCoords) {
                throw new Error(`Invalid range reference: ${rangeRef}`);
              }

              // Use numeric getRange instead of manually building A1 string ranges
              const range = sheet.getRange(
                startCoords.row,
                startCoords.col,
                numRows,
                numCols
              );

              // PERFORMANCE OPTIMIZATION: Determine the best insertion strategy
              // Formula-only cells can use direct insertion, mixed cells need full conversion

              /**
               * Checks if a cell contains only a formula (no values or styles)
               * This allows for a performance optimization in bulk updates
               */
              const isFormulaOnlyCell = (cell: CellValue): boolean => {
                // Since cell is typed as CellValue, it's guaranteed to be an object
                // Core formula-only logic: has formula AND no conflicting data
                const hasFormula = 'f' in cell && Boolean(cell.f);
                const hasConflictingData =
                  ('v' in cell && cell.v !== null) ||
                  ('style' in cell && cell.style !== null);

                return hasFormula && !hasConflictingData;
              };

              // Check if ALL cells in the range are formula-only
              const allCellsAreFormulaOnly = values.every((row) =>
                row.every(isFormulaOnlyCell)
              );

              if (allCellsAreFormulaOnly) {
                // FAST PATH: Direct formula insertion (bypasses CellValue → ICellData conversion)
                const formulaGrid = values.map((row) =>
                  row.map((cell) => (cell as CellValue & { f: string }).f)
                );
                range.setFormulas(formulaGrid);
              } else {
                // SLOW PATH: Send data to Web Worker to avoid freezing the UI thread
                const univerValues =
                  await GlobalDataMapper.convertToUniverCellValueBatch(values);
                range.setValues(univerValues);
              }
            }, 'update range');
          });
        },

        getCellValue: (cellRef: string): Promise<string | number | null> => {
          return Promise.resolve(
            getCellValue({ current: univerAPIRef.current }, cellRef)
          );
        },

        getRange: (rangeRef: string): Promise<(string | number)[][]> => {
          return getRange({ current: univerAPIRef.current }, rangeRef);
        },

        getRangeFull: (rangeRef: string): Promise<CellValue[][]> => {
          return getRangeFull({ current: univerAPIRef.current }, rangeRef);
        },

        getSelection: (): Promise<string | null> => {
          return Promise.resolve(
            getSelection({ current: univerAPIRef.current })
          );
        },

        // ========================================================================
        // DIRECT FORMULA INSERTION (Performance Optimization)
        // ========================================================================

        /**
         * Directly insert formulas into spreadsheet cells without intermediate conversions.
         * Supports 1D (column/row) and 2D (rectangular range) formula insertion.
         *
         * @param rangeRef - Range reference: "A1" (single cell), "A:A" (column), "1:1" (row), or "C1:E5" (rectangle)
         * @param formulas - Array of formula strings (1D) or 2D array for rectangular ranges
         *
         * @example
         * // Insert single formula to cell A1
         * insertFormulas("A1", ["=B1+C1"])
         *
         * // Insert formulas to entire column A
         * insertFormulas("A:A", ["=B1+C1", "=B2+C2", "=B3+C3"])
         *
         * // Insert formulas to entire row 1
         * insertFormulas("1:1", ["=A2+A3", "=B2+B3", "=C2+C3"])
         *
         * // Insert 2x2 grid of formulas (C1:D2)
         * insertFormulas("C1:D2", [
         *   ["=A1+B1", "=A1*C1"],
         *   ["=A2+B2", "=A2*C2"]
         * ])
         */
        insertFormulas: (rangeRef: string, formulas: string[] | string[][]) => {
          if (!operationQueueRef.current) {
            throw new Error('Operation queue not initialized');
          }
          return operationQueueRef.current.enqueue(() => {
            if (!univerAPIRef.current) {
              throw new Error('Facade API not ready for formula insertion');
            }

            return safeSpreadsheetOperation(() => {
              const workbook = univerAPIRef.current?.getActiveWorkbook();
              if (!workbook) return;
              const sheet = workbook.getActiveSheet();

              if (Array.isArray(formulas) && formulas.length === 0) {
                return; // Nothing to insert
              }

              let rangeSpec: string;
              let formulaGrid: string[][];

              // Normalize the range reference
              if (rangeRef.includes(':')) {
                // Already a full range (e.g., "C1:C10", "A:A", "1:1", "C1:E5")
                rangeSpec = rangeRef;
              } else if (/^[A-Z]+\d+$/.test(rangeRef)) {
                // Single cell (e.g., "A1", "C7") - treat as 1x1 range
                rangeSpec = `${rangeRef}:${rangeRef}`;
              } else if (/^[A-Z]+$/.test(rangeRef)) {
                // Column reference (e.g., "A", "BC") - treat as entire column
                rangeSpec = `${rangeRef}:${rangeRef}`;
              } else if (/^\d+$/.test(rangeRef)) {
                // Row reference (e.g., "1", "42") - treat as entire row
                rangeSpec = `${rangeRef}:${rangeRef}`;
              } else {
                throw new Error(
                  `Invalid range reference: ${rangeRef}. Use A1 notation for cells, column letters for columns, or row numbers for rows.`
                );
              }

              // Handle formula array dimensions
              if (Array.isArray(formulas) && Array.isArray(formulas[0])) {
                // 2D array provided for rectangular range
                formulaGrid = formulas as string[][];
              } else {
                // 1D array provided - use vertical (column) orientation
                const formula1D = formulas as string[];
                formulaGrid = formula1D.map((formula) => [formula]);
              }

              // Get the range and set formulas directly
              const range = sheet.getRange(rangeSpec);
              range.setFormulas(formulaGrid);
            }, 'insert formulas');
          });
        },

        isReady: () => !!univerAPIRef.current,

        // ========================================================================
        // SHEET MANAGEMENT OPERATIONS
        // ========================================================================

        createSheet: (
          name: string,
          rows = DEFAULT_SHEET_ROWS,
          cols = DEFAULT_SHEET_COLS
        ) => {
          if (!univerAPIRef.current) {
            return Promise.reject(new Error('Univer API not ready'));
          }
          const workbook = univerAPIRef.current.getActiveWorkbook();
          if (!workbook) {
            return Promise.reject(new Error('No active workbook'));
          }

          // Generate unique sheet name to avoid duplicates
          const uniqueName = (() => {
            const existingSheets = workbook.getSheets();
            const existingNames = new Set(
              existingSheets.map((s: FWorksheet) =>
                getSheetNameSafely(s, existingSheets)
              )
            );

            let sheetName = name;
            let nameCounter = 1;

            while (existingNames.has(sheetName)) {
              sheetName = `${name} (${nameCounter++})`;
            }

            return sheetName;
          })();

          const newSheet = workbook.create(uniqueName, rows, cols);
          workbook.setActiveSheet(newSheet);

          return new Promise<string>((resolve) => {
            const checkReady = () => {
              const sheet = workbook
                .getSheets()
                .find(
                  (s: FWorksheet) => s.getSheetId() === newSheet.getSheetId()
                );
              if (sheet?.getSheetName() === uniqueName) {
                resolve(newSheet.getSheetId());
              } else {
                setTimeout(checkReady, 10);
              }
            };
            checkReady();
          });
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
          return Promise.resolve(
            sheets.map((sheet: FWorksheet) => ({
              id: sheet.getSheetId(),
              name: getSheetNameSafely(sheet, sheets),
            }))
          );
        },

        deleteSheet: (sheetId: string): Promise<void> => {
          if (!univerAPIRef.current) {
            return Promise.reject(new Error('Univer instance not initialized'));
          }

          try {
            const workbook = univerAPIRef.current.getActiveWorkbook();
            if (!workbook) {
              return Promise.reject(new Error('No active workbook found'));
            }

            const success = workbook.deleteSheet(sheetId);
            if (!success) {
              return Promise.reject(
                new Error(`Failed to delete sheet ${sheetId}`)
              );
            }

            return Promise.resolve();
          } catch (error) {
            console.error('Failed to delete sheet:', error);
            return Promise.reject(
              new Error(
                `Failed to delete sheet: ${error instanceof Error ? error.message : String(error)}`,
                { cause: error }
              )
            );
          }
        },

        // ========================================================================
        // WORKBOOK OPERATIONS
        // ========================================================================

        getWorkbookSnapshot: () => {
          if (!univerAPIRef.current) {
            throw new Error('Facade API not ready for snapshot export');
          }
          const workbook = univerAPIRef.current.getActiveWorkbook();
          if (!workbook) {
            throw new Error('No active workbook for snapshot export');
          }
          return Promise.resolve(workbook.save());
        },

        loadWorkbookSnapshot: async (snapshot: unknown) => {
          if (!operationQueueRef.current) {
            throw new Error('Operation queue not initialized');
          }
          return operationQueueRef.current.enqueue(async () => {
            if (!univerAPIRef.current) {
              throw new Error('Facade API not ready for snapshot import');
            }

            if (!snapshot || typeof snapshot !== 'object') {
              throw new Error('Invalid snapshot data');
            }

            const univerAPI = univerAPIRef.current;
            const currentWorkbook = univerAPI.getActiveWorkbook();

            if (currentWorkbook) {
              // Instead of disposing, let's keep the workbook and overwrite its sheets
              // if disposing the unit completely breaks the UI binding in v0.20
              if (typeof currentWorkbook.dispose === 'function') {
                currentWorkbook.dispose();
              } else {
                univerAPI.disposeUnit(currentWorkbook.getId());
              }
            }

            const snapshotObj = snapshot as Record<string, unknown>;
            const explicitFormat = (snapshotObj as { __format?: string })
              .__format;

            let isAbstractFormat = false;
            if (explicitFormat) {
              isAbstractFormat = explicitFormat === 'abstract';
            } else {
              const firstSheet = Object.values(snapshotObj.sheets ?? {})[0] as
                | Record<string, unknown>
                | undefined;
              if (
                firstSheet?.cellData &&
                typeof firstSheet.cellData === 'object'
              ) {
                const cellDataKeys = Object.keys(firstSheet.cellData);
                isAbstractFormat = cellDataKeys.some((key) =>
                  /^[A-Z]+\d+$/.test(key)
                );
              }
            }

            let snapshotToLoad = snapshot;
            if (isAbstractFormat) {
              snapshotToLoad =
                await GlobalDataMapper.convertToUniverDataMultiSheet(
                  snapshot as WorkbookData
                );
            }

            univerAPI.createWorkbook(snapshotToLoad as IWorkbookData);

            const snapshotWithResources = snapshotToLoad as {
              resources?: Array<{ name: string; data: string }>;
            };
            const univerInstanceForProtection =
              univerInstanceRef.current ??
              univerAPIRef.current.__univerInstance;

            if (
              snapshotWithResources.resources?.length &&
              univerInstanceForProtection
            ) {
              const abortController = new AbortController();

              import('../utils/protectionUtils')
                .then(({ applyProtectionRules }) => {
                  return applyProtectionRules(
                    snapshotWithResources.resources ?? [],
                    univerInstanceForProtection,
                    abortController.signal
                  )
                    .catch((error: unknown) => {
                      console.error('Failed to apply protection rules:', error);
                    })
                    .finally(() => {
                      if (!abortController.signal.aborted) {
                        abortController.abort();
                      }
                    });
                })
                .catch((error: unknown) => {
                  console.error(
                    'Failed to import protection utilities:',
                    error
                  );
                });
            }

            return Promise.resolve();
          });
        },

        // ========================================================================
        // UTILITY OPERATIONS
        // ========================================================================

        getUsedRange: (): Promise<string> => {
          if (!univerAPIRef.current) {
            return Promise.reject(new Error('Univer API not ready'));
          }
          return Promise.resolve(determineUsedRange(univerAPIRef.current));
        },

        getSheetBounds: (
          sheetId?: string
        ): Promise<{
          startCol: number;
          startRow: number;
          endCol: number;
          endRow: number;
        }> => {
          if (!univerAPIRef.current) {
            return Promise.reject(new Error('Univer API not ready'));
          }

          const workbook = univerAPIRef.current.getActiveWorkbook();
          if (!workbook) {
            return Promise.reject(new Error('No active workbook'));
          }

          const sheet = sheetId
            ? workbook
                .getSheets()
                .find((s: FWorksheet) => s.getSheetId() === sheetId)
            : workbook.getActiveSheet();
          if (!sheet) {
            return Promise.reject(new Error('Sheet not found'));
          }

          const lastRow = sheet.getLastRow();
          const lastCol = sheet.getLastColumn();

          return Promise.resolve({
            startCol: 0,
            startRow: 0,
            endCol: Math.max(0, lastCol),
            endRow: Math.max(0, lastRow),
          });
        },

        // ========================================================================
        // APPEND MODE OPERATIONS
        // ========================================================================

        getNewlyCreatedSheet: (sheetName: string) => {
          if (!univerAPIRef.current) {
            return Promise.reject(new Error('Univer API not initialized'));
          }
          const workbook = univerAPIRef.current.getActiveWorkbook();
          if (!workbook) {
            return Promise.reject(new Error('No active workbook'));
          }
          const sheets = workbook.getSheets();
          const sheet = sheets.find(
            (s: FWorksheet) => s.getSheetName() === sheetName
          );
          if (!sheet) {
            return Promise.reject(new Error(`Sheet "${sheetName}" not found`));
          }
          return Promise.resolve(sheet);
        },

        loadSheetDataBulk: async (
          sheetId: string,
          sheetData: unknown,
          options = {}
        ) => {
          if (!operationQueueRef.current) {
            throw new Error('Operation queue not initialized');
          }
          return operationQueueRef.current.enqueue(async () => {
            if (!univerInstanceRef.current || !univerAPIRef.current) {
              throw new Error('Univer instance not initialized');
            }
            const { bulkLoadSheetDataFromMatrix } = await import(
              './../../univer/operations/bulkImportOperations'
            );
            const workbook = univerAPIRef.current.getActiveWorkbook();
            if (!workbook) {
              throw new Error('No active workbook');
            }
            const sheet = workbook
              .getSheets()
              .find((s: FWorksheet) => s.getSheetId() === sheetId);
            if (!sheet) {
              throw new Error(`Sheet with ID "${sheetId}" not found`);
            }
            return bulkLoadSheetDataFromMatrix(
              univerInstanceRef.current,
              sheet,
              sheetData as Parameters<typeof bulkLoadSheetDataFromMatrix>[2],
              options
            );
          });
        },

        applySheetProtection: (
          sheetId: string,
          protectionData: Array<{ name: string; data: string }>,
          sheetIdMapping?: Map<string, string>
        ) => {
          if (!univerAPIRef.current) {
            return Promise.reject(new Error('Univer instance not initialized'));
          }

          try {
            const workbook = univerAPIRef.current.getActiveWorkbook();
            if (!workbook) {
              console.warn('No active workbook for applying protection');
              return Promise.resolve();
            }

            const sheet = workbook
              .getSheets()
              .find((s: FWorksheet) => s.getSheetId() === sheetId);
            if (!sheet) {
              console.warn(
                `Sheet with ID "${sheetId}" not found for protection`
              );
              return Promise.resolve();
            }

            return (async () => {
              try {
                for (const resource of protectionData) {
                  if (
                    !resource.name.includes('PROTECTION') &&
                    !resource.name.includes('PERMISSION')
                  ) {
                    continue;
                  }

                  try {
                    const parsedData: unknown = JSON.parse(
                      resource.data || '{}'
                    );

                    if (typeof parsedData !== 'object' || parsedData === null) {
                      continue;
                    }

                    for (const [, protections] of Object.entries(parsedData)) {
                      if (!Array.isArray(protections)) {
                        continue;
                      }

                      for (const protection of protections) {
                        if (
                          typeof protection !== 'object' ||
                          protection === null
                        ) {
                          continue;
                        }

                        // Facade mapping
                        const protectionObj = protection as {
                          unitType?: number;
                          subUnitId?: string;
                          name?: string;
                          ranges?: Array<{
                            startRow: number;
                            startColumn: number;
                            endRow: number;
                            endColumn: number;
                          }>;
                        };

                        let targetSubUnitId = protectionObj.subUnitId;
                        if (targetSubUnitId && sheetIdMapping) {
                          const mappedId = sheetIdMapping.get(targetSubUnitId);
                          if (mappedId) {
                            targetSubUnitId = mappedId;
                          }
                        }

                        if (targetSubUnitId !== sheetId) {
                          continue;
                        }

                        const isWorksheetProtection =
                          protectionObj.unitType === 2;
                        const isRangeProtection = protectionObj.unitType === 3;

                        // Use cleanly encapsulated Facade APIs instead of raw internal mutations
                        if (isWorksheetProtection) {
                          const options = protectionObj.name
                            ? { name: protectionObj.name }
                            : undefined;
                          await sheet.getWorksheetPermission().protect(options);
                        } else if (isRangeProtection && protectionObj.ranges) {
                          const fRanges = protectionObj.ranges.map((r) =>
                            sheet.getRange(r)
                          );
                          const options = protectionObj.name
                            ? { name: protectionObj.name }
                            : undefined;
                          await sheet.getWorksheetPermission().protectRanges([
                            {
                              ranges: fRanges,
                              ...(options ? { options } : {}),
                            },
                          ]);
                        }
                      }
                    }
                  } catch (parseError) {
                    console.debug(
                      'Failed to parse protection data:',
                      parseError
                    );
                  }
                }
              } catch (error) {
                console.warn('Failed to apply sheet protection:', error);
              }
            })();
          } catch (error) {
            console.warn('Failed to apply sheet protection:', error);
            return Promise.resolve();
          }
        },
      }),
      []
    );

    useImperativeHandle(ref, () => memoizedOperations, [memoizedOperations]);

    // Memoized the converted data to prevent unnecessary re-initialization
    const univerData = useMemo(
      () => convertToUniverData(initialData),
      [initialData]
    );

    return (
      <UniverSpreadsheet
        initialData={univerData}
        onSelectionChange={onSelectionChange ?? (() => {})}
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
