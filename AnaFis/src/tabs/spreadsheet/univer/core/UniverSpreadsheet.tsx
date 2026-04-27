// UniverSpreadsheet.tsx - High-stability Univer integration with Facade API
import { forwardRef, useEffect, useImperativeHandle, useRef } from 'react';

// Extend window interface for instance tracking and HMR recovery
declare global {
  interface Window {
    __UNIVER_INSTANCES__?: Set<string>;
    _lastUniverDisposable?: { dispose: () => void };
  }
}

import { useTheme } from '@mui/material';
import {
  type ICellData,
  type ICommandInfo,
  ICommandService,
  type IRange,
  IUniverInstanceService,
  type IWorkbookData,
  LocaleType,
  mergeLocales,
  Univer,
  UniverInstanceType,
  type Workbook,
} from '@univerjs/core';
import { UniverDocsPlugin } from '@univerjs/docs';
import { UniverDocsUIPlugin } from '@univerjs/docs-ui';
import docsUIEnUS from '@univerjs/docs-ui/locale/en-US';
import { UniverFormulaEnginePlugin } from '@univerjs/engine-formula';
import { UniverRenderEnginePlugin } from '@univerjs/engine-render';
import { UniverFindReplacePlugin } from '@univerjs/find-replace';
import findReplaceEnUS from '@univerjs/find-replace/locale/en-US';
import { UniverNetworkPlugin } from '@univerjs/network';
import { UniverSheetsPlugin } from '@univerjs/sheets';
import sheetsEnUS from '@univerjs/sheets/locale/en-US';
import { UniverSheetsFilterPlugin } from '@univerjs/sheets-filter';
import { UniverSheetsFilterUIPlugin } from '@univerjs/sheets-filter-ui';
import sheetsFilterUIEnUS from '@univerjs/sheets-filter-ui/locale/en-US';
import { UniverSheetsFindReplacePlugin } from '@univerjs/sheets-find-replace';
import {
  IRegisterFunctionService,
  UniverSheetsFormulaPlugin,
} from '@univerjs/sheets-formula';
import sheetsFormulaEnUS from '@univerjs/sheets-formula/locale/en-US';
import { UniverSheetsFormulaUIPlugin } from '@univerjs/sheets-formula-ui';
import sheetsFormulaUIEnUS from '@univerjs/sheets-formula-ui/locale/en-US';
import { UniverSheetsNumfmtPlugin } from '@univerjs/sheets-numfmt';
import { UniverSheetsNumfmtUIPlugin } from '@univerjs/sheets-numfmt-ui';
import sheetsNumfmtUIEnUS from '@univerjs/sheets-numfmt-ui/locale/en-US';
import { UniverSheetsUIPlugin } from '@univerjs/sheets-ui';
import sheetsUIEnUS from '@univerjs/sheets-ui/locale/en-US';
import { UniverUIPlugin } from '@univerjs/ui';
import uiEnUS from '@univerjs/ui/locale/en-US';

// Styles
import '@univerjs/design/lib/index.css';
import '@univerjs/ui/lib/index.css';
import '@univerjs/docs-ui/lib/index.css';
import '@univerjs/sheets-ui/lib/index.css';
import '@univerjs/sheets-formula-ui/lib/index.css';
import '@univerjs/sheets-numfmt-ui/lib/index.css';

// Facade APIs
import '@univerjs/engine-formula/facade';
import '@univerjs/ui/facade';
import '@univerjs/docs-ui/facade';
import '@univerjs/sheets/facade';
import '@univerjs/sheets-ui/facade';
import '@univerjs/sheets-formula/facade';
import '@univerjs/sheets-numfmt/facade';

import {
  parseCellRef,
  parseRange,
  rangeToA1,
  registerCustomFunctions,
} from '@/tabs/spreadsheet/univer/index';

interface Props {
  initialData: IWorkbookData;
  onCellChange: (cellRef: string, value: ICellData) => void;
  onFormulaIntercept: (cellRef: string, formula: string) => void;
  onSelectionChange?: (cellRef: string) => void;
  onUniverReady?: (univerInstance: Univer) => void;
  tabId?: string;
}

interface UniverSpreadsheetRef {
  updateCell: (cellRef: string, value: ICellData) => void;
  getCellValue: (cellRef: string) => string | number | null;
  getRange: (rangeRef: string) => (string | number)[][];
  univer: Univer | null;
}

const UniverSpreadsheet = forwardRef<UniverSpreadsheetRef, Props>(
  (
    {
      initialData,
      onCellChange,
      onFormulaIntercept,
      onSelectionChange,
      onUniverReady,
      tabId,
    },
    ref
  ) => {
    const theme = useTheme();
    const containerRef = useRef<HTMLDivElement>(null);
    const univerRef = useRef<Univer | null>(null);
    const isInitializedRef = useRef(false);

    // Stable references for callbacks to prevent re-registration loops
    const callbacks = useRef({
      onCellChange,
      onFormulaIntercept,
      onSelectionChange,
      onUniverReady,
    });
    useEffect(() => {
      callbacks.current = {
        onCellChange,
        onFormulaIntercept,
        onSelectionChange,
        onUniverReady,
      };
    }, [onCellChange, onFormulaIntercept, onSelectionChange, onUniverReady]);

    const containerId = useRef(
      tabId
        ? `univer-container-${tabId}`
        : `univer-container-${Math.random().toString(36).substring(2, 11)}`
    ).current;

    useImperativeHandle(ref, () => ({
      updateCell: (cellRef, value) => {
        const univer = univerRef.current;
        if (!univer) return;
        try {
          const injector = univer.__getInjector();
          const commandService = injector.get(ICommandService);
          const workbook = injector
            .get(IUniverInstanceService)
            .getFocusedUnit() as Workbook;
          const indices = parseCellRef(cellRef);
          if (!indices) return;

          void commandService.executeCommand('sheet.command.set-range-values', {
            unitId: workbook.getUnitId(),
            subUnitId: workbook.getActiveSheet().getSheetId(),
            range: {
              startRow: indices.row,
              startColumn: indices.col,
              endRow: indices.row,
              endColumn: indices.col,
            },
            value: [[value]],
          });
        } catch (e) {
          console.error('updateCell failed', e);
        }
      },
      getCellValue: (cellRef) => {
        const univer = univerRef.current;
        if (!univer) return null;
        try {
          const workbook = univer
            .__getInjector()
            .get(IUniverInstanceService)
            .getFocusedUnit() as Workbook;
          const indices = parseCellRef(cellRef);
          if (!indices) return null;
          const data = workbook
            .getActiveSheet()
            .getCellRaw(indices.row, indices.col);
          return data?.v !== undefined ? (data.v as string | number) : null;
        } catch {
          return null;
        }
      },
      getRange: (rangeRef) => {
        const univer = univerRef.current;
        if (!univer) return [];
        try {
          const workbook = univer
            .__getInjector()
            .get(IUniverInstanceService)
            .getFocusedUnit() as Workbook;
          const range = parseRange(rangeRef);
          if (!range) return [];
          const sheet = workbook.getActiveSheet();
          const result: (string | number)[][] = [];
          for (let r = range.startRow; r <= range.endRow; r++) {
            const row: (string | number)[] = [];
            for (let c = range.startCol; c <= range.endCol; c++) {
              const cell = sheet.getCellRaw(r, c);
              row.push(
                cell?.v !== undefined ? (cell.v as string | number) : ''
              );
            }
            result.push(row);
          }
          return result;
        } catch {
          return [];
        }
      },
      get univer() {
        return univerRef.current;
      },
    }));

    useEffect(() => {
      if (!containerRef.current || isInitializedRef.current) return;

      let isDisposed = false;
      let initTimer: ReturnType<typeof setTimeout> | null = null;
      let commandDisposable: { dispose: () => void } | null = null;

      window.__UNIVER_INSTANCES__ ??= new Set();

      // SAFETY INIT: 50ms buffer prevents HMR race conditions and RangeErrors
      initTimer = setTimeout(() => {
        if (isDisposed || isInitializedRef.current) return;

        isInitializedRef.current = true;
        window.__UNIVER_INSTANCES__?.add(containerId);

        try {
          const univer = new Univer({
            darkMode: true,
            locale: LocaleType.EN_US,
            locales: {
              [LocaleType.EN_US]: mergeLocales(
                docsUIEnUS,
                sheetsEnUS,
                sheetsFormulaEnUS,
                sheetsFormulaUIEnUS,
                sheetsUIEnUS,
                sheetsNumfmtUIEnUS,
                uiEnUS,
                sheetsFilterUIEnUS,
                findReplaceEnUS
              ),
            },
          });

          // Helper to register plugins without 'any' in the main flow
          const register = <T, O>(plugin: T, options?: O) => {
            univer.registerPlugin(plugin as never, options as never);
          };

          // Core Plugins
          register(UniverRenderEnginePlugin);
          register(UniverFormulaEnginePlugin);
          register(UniverNetworkPlugin);
          register(UniverUIPlugin, {
            container: containerId,
            ribbonType: 'collapsed',
          });
          register(UniverDocsPlugin, { hasScroll: false });
          register(UniverDocsUIPlugin);

          // Sheet Plugins
          register(UniverSheetsPlugin);
          register(UniverSheetsUIPlugin);
          register(UniverSheetsFormulaPlugin);
          register(UniverSheetsFormulaUIPlugin);
          register(UniverSheetsNumfmtPlugin);
          register(UniverSheetsNumfmtUIPlugin);
          register(UniverSheetsFilterPlugin);
          register(UniverSheetsFilterUIPlugin);
          register(UniverFindReplacePlugin);
          register(UniverSheetsFindReplacePlugin);

          univerRef.current = univer;
          univer.createUnit(UniverInstanceType.UNIVER_SHEET, initialData);

          // Formula Engine Setup
          const injector = univer.__getInjector();
          registerCustomFunctions(injector.get(IRegisterFunctionService));

          // Command Listener (Selection & Changes)
          const commandService = injector.get(ICommandService);
          commandDisposable = commandService.onCommandExecuted(
            (command: ICommandInfo) => {
              if (isDisposed) return;

              // Handle Selection
              if (command.id === 'sheet.operation.set-selections') {
                const params = command.params as {
                  selections?: Array<{ range: IRange }>;
                };
                const range = params?.selections?.[0]?.range;
                if (range)
                  callbacks.current.onSelectionChange?.(rangeToA1(range));
              }
              // Handle Changes
              else if (
                command.id === 'sheet.mutation.set-range-values' ||
                command.id === 'sheet.command.set-range-values'
              ) {
                const params = command.params as {
                  range: IRange;
                  value?: ICellData[][];
                };
                if (params?.range) {
                  const cellRef = rangeToA1(params.range);
                  const cellValue = params.value?.[0]?.[0];

                  // Formula Interceptor (Original Logic)
                  if (cellValue?.v !== undefined && cellValue.v !== null) {
                    const valStr = cellValue.v.toString();
                    if (valStr.startsWith('='))
                      callbacks.current.onFormulaIntercept(cellRef, valStr);
                  }

                  // Mutation Handler (Original Logic)
                  if (
                    command.id === 'sheet.mutation.set-range-values' &&
                    cellValue
                  ) {
                    callbacks.current.onCellChange(cellRef, cellValue);
                  }
                }
              }
            }
          );

          if (callbacks.current.onUniverReady)
            callbacks.current.onUniverReady(univer);
          window._lastUniverDisposable = commandDisposable;
        } catch (error) {
          console.error('[Univer] Init failed:', error);
          isInitializedRef.current = false;
        }
      }, 50);

      return () => {
        isDisposed = true;
        if (initTimer) clearTimeout(initTimer);

        if (univerRef.current) {
          univerRef.current.dispose();
          univerRef.current = null;
        }

        window._lastUniverDisposable?.dispose();
        window.__UNIVER_INSTANCES__?.delete(containerId);
        isInitializedRef.current = false;
      };
    }, [initialData, containerId]);

    return (
      <div
        ref={containerRef}
        id={containerId}
        className="univer-spreadsheet-container"
        style={{
          width: '100%',
          height: '100%',
          minHeight: '400px',
          backgroundColor: theme.palette.background.paper,
        }}
      />
    );
  }
);

UniverSpreadsheet.displayName = 'UniverSpreadsheet';
export default UniverSpreadsheet;
