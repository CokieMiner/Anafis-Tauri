// UniverAdapter.tsx - Improved Facade API approach with proper plugin mode integration

import type { ICellData, Univer } from '@univerjs/core';
import { FUniver } from '@univerjs/core/facade';
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
import UniverSpreadsheet from '@/tabs/spreadsheet/univer/core//UniverSpreadsheet';
import {
  // Import utilities
  columnToLetter,
  convertFromUniverCellData,
  convertToUniverCellValue,
  convertToUniverData,
  convertToUniverDataMultiSheet,
  determineUsedRange,
  // Import services
  ExportService,
  getCellValue,
  getRange,
  getRangeFull,
  getSelection,
  ImportService,
  letterToColumn,
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
  (
    {
      initialData,
      onCellChange,
      onFormulaIntercept,
      onSelectionChange,
      onReady,
      tabId,
    },
    ref
  ) => {
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

            return safeSpreadsheetOperation(() => {
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
              const startCol = columnToLetter(startCoords.col);
              const startRow = startCoords.row + 1; // Convert to 1-based
              const endColIndex = columnToLetter(
                letterToColumn(startCol) + numCols - 1
              );
              const endRow = startRow + numRows - 1;
              const endCell = `${endColIndex}${endRow}`;

              const range = sheet.getRange(`${rangeRef}:${endCell}`);

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
                // SLOW PATH: Full conversion (handles mixed data types)
                const univerValues = values.map((row) =>
                  row.map((cell) => convertToUniverCellValue(cell))
                );
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
              existingSheets.map((s) => getSheetNameSafely(s, existingSheets))
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
                .find((s) => s.getSheetId() === newSheet.getSheetId());
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
            sheets.map((sheet) => ({
              id: sheet.getSheetId(),
              name: getSheetNameSafely(sheet, sheets),
            }))
          );
        },

        deleteSheet: async (sheetId: string) => {
          if (!univerInstanceRef.current || !univerAPIRef.current) {
            throw new Error('Univer instance not initialized');
          }

          try {
            const { ICommandService, IUniverInstanceService } = await import(
              '@univerjs/core'
            );
            const { RemoveSheetMutation } = await import('@univerjs/sheets');

            const univer = univerInstanceRef.current as {
              __getInjector: () => { get: (token: unknown) => unknown };
            };
            const injector = univer.__getInjector();
            const commandService = injector.get(ICommandService) as {
              executeCommand: (
                commandId: string,
                params: unknown
              ) => Promise<boolean>;
            };
            const instanceService = injector.get(IUniverInstanceService) as {
              getFocusedUnit: () => { getUnitId: () => string } | null;
            };

            const focusedUnit = instanceService.getFocusedUnit();
            if (!focusedUnit) {
              throw new Error('No focused unit found');
            }

            const unitId = focusedUnit.getUnitId();
            const success = await commandService.executeCommand(
              RemoveSheetMutation.id,
              {
                unitId,
                subUnitId: sheetId,
              }
            );

            if (!success) {
              throw new Error(`Failed to delete sheet ${sheetId}`);
            }
          } catch (error) {
            console.error('Failed to delete sheet:', error);
            throw new Error(
              `Failed to delete sheet: ${error instanceof Error ? error.message : String(error)}`,
              { cause: error }
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

        loadWorkbookSnapshot: (snapshot: unknown) => {
          if (!operationQueueRef.current) {
            throw new Error('Operation queue not initialized');
          }
          return operationQueueRef.current.enqueue(() => {
            if (!univerAPIRef.current) {
              throw new Error('Facade API not ready for snapshot import');
            }

            if (!snapshot || typeof snapshot !== 'object') {
              throw new Error('Invalid snapshot data');
            }

            const univerAPI = univerAPIRef.current;
            const currentWorkbook = univerAPI.getActiveWorkbook();

            if (currentWorkbook) {
              univerAPI.disposeUnit(currentWorkbook.getId());
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
              snapshotToLoad = convertToUniverDataMultiSheet(
                snapshot as WorkbookData
              );
            }

            univerAPI.createWorkbook(snapshotToLoad);

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
            ? workbook.getSheets().find((s) => s.getSheetId() === sheetId)
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
          const sheet = sheets.find((s) => s.getSheetName() === sheetName);
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
              .find((s) => s.getSheetId() === sheetId);
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
          if (!univerInstanceRef.current || !univerAPIRef.current) {
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
              .find((s) => s.getSheetId() === sheetId);
            if (!sheet) {
              console.warn(
                `Sheet with ID "${sheetId}" not found for protection`
              );
              return Promise.resolve();
            }

            return (async () => {
              try {
                const {
                  ICommandService,
                  IUniverInstanceService,
                  IPermissionService,
                } = await import('@univerjs/core');
                const {
                  AddWorksheetProtectionMutation,
                  AddRangeProtectionMutation,
                  WorksheetEditPermission,
                  RangeProtectionPermissionEditPoint,
                } = await import('@univerjs/sheets');

                const univer = univerInstanceRef.current as {
                  __getInjector: () => { get: (token: unknown) => unknown };
                };
                const injector = univer.__getInjector();
                const commandService = injector.get(ICommandService) as {
                  executeCommand: (
                    commandId: string,
                    params: unknown
                  ) => Promise<boolean>;
                };
                const instanceService = injector.get(
                  IUniverInstanceService
                ) as {
                  getFocusedUnit: () => { getUnitId: () => string } | null;
                };
                const permissionService = injector.get(IPermissionService) as {
                  updatePermissionPoint: (
                    pointId: string,
                    value: boolean
                  ) => void;
                };

                const focusedUnit = instanceService.getFocusedUnit();
                if (!focusedUnit) {
                  return;
                }

                const unitId = focusedUnit.getUnitId();

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

                        const protectionObj = protection as {
                          unitType?: number;
                          subUnitId?: string;
                          permissionId?: string;
                          name?: string;
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

                        const protectionToApply = {
                          ...(protection as Record<string, unknown>),
                          subUnitId: sheetId,
                          unitId,
                        };

                        const isWorksheetProtection =
                          protectionObj.unitType === 2;
                        const isRangeProtection = protectionObj.unitType === 3;

                        if (isWorksheetProtection) {
                          const success = await commandService.executeCommand(
                            AddWorksheetProtectionMutation.id,
                            {
                              unitId,
                              subUnitId: sheetId,
                              rule: protectionToApply,
                            }
                          );

                          if (success) {
                            const editPermission = new WorksheetEditPermission(
                              unitId,
                              sheetId
                            );
                            permissionService.updatePermissionPoint(
                              editPermission.id,
                              false
                            );
                          }
                        } else if (isRangeProtection) {
                          const success = await commandService.executeCommand(
                            AddRangeProtectionMutation.id,
                            {
                              unitId,
                              subUnitId: sheetId,
                              rules: [protectionToApply],
                            }
                          );

                          if (
                            success &&
                            protectionObj.permissionId !== undefined
                          ) {
                            const editPermission =
                              new RangeProtectionPermissionEditPoint(
                                unitId,
                                sheetId,
                                protectionObj.permissionId
                              );
                            permissionService.updatePermissionPoint(
                              editPermission.id,
                              false
                            );
                          }
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

        // ========================================================================
        // SERVICE ACCESS (Singleton Pattern)
        // ========================================================================

        getExportService: (() => {
          let instance: ExportService | null = null;
          return () => (instance ??= new ExportService());
        })(),

        getImportService: (() => {
          let instance: ImportService | null = null;
          return () => (instance ??= new ImportService());
        })(),
      }),
      []
    );

    useImperativeHandle(ref, () => memoizedOperations, [memoizedOperations]);

    // Memoized callback to handle type conversion
    const handleCellChange = useCallback(
      (cellRef: string, univerCellData: ICellData) => {
        const abstractCellData = convertFromUniverCellData(univerCellData);
        onCellChange(cellRef, abstractCellData);
      },
      [onCellChange]
    );

    // Memoized the converted data to prevent unnecessary re-initialization
    const univerData = useMemo(
      () => convertToUniverData(initialData),
      [initialData]
    );

    return (
      <UniverSpreadsheet
        initialData={univerData}
        onCellChange={handleCellChange}
        onFormulaIntercept={onFormulaIntercept}
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
