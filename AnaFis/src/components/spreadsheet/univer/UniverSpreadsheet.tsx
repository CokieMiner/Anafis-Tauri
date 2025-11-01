// UniverSpreadsheet.tsx - Improved plugin mode with proper Facade API integration
import { useRef, useEffect, forwardRef, useImperativeHandle } from 'react';

// Extend window interface for instance tracking
declare global {
    interface Window {
        __UNIVER_INSTANCES__?: Set<string>;
    }
}

import { useTheme } from '@mui/material';
import {
    IWorkbookData,
    ICommandInfo,
    IRange,
    ICellData,
    ICommandService,
    IUniverInstanceService,
    Workbook,
    UniverInstanceType,
    LocaleType,
    mergeLocales,
} from '@univerjs/core';
import { IRegisterFunctionService, UniverSheetsFormulaPlugin } from '@univerjs/sheets-formula';
import { createUniver } from '@univerjs/presets';
import { UniverSheetsDataValidationPlugin } from '@univerjs/sheets-data-validation';
// Import all individual plugins
import { UniverNetworkPlugin } from '@univerjs/network';
import { UniverDocsPlugin } from '@univerjs/docs';
import { UniverRenderEnginePlugin } from '@univerjs/engine-render';
import { UniverUIPlugin } from '@univerjs/ui';
import { UniverDocsUIPlugin } from '@univerjs/docs-ui';
import { UniverFormulaEnginePlugin } from '@univerjs/engine-formula';
import { UniverSheetsPlugin } from '@univerjs/sheets';
import { UniverSheetsUIPlugin } from '@univerjs/sheets-ui';
import { UniverSheetsNumfmtPlugin } from '@univerjs/sheets-numfmt';
import { UniverSheetsNumfmtUIPlugin } from '@univerjs/sheets-numfmt-ui';
import { UniverSheetsFormulaUIPlugin } from '@univerjs/sheets-formula-ui';
// Import additional feature plugins
import { UniverSheetsConditionalFormattingPlugin } from '@univerjs/sheets-conditional-formatting';
import { UniverSheetsConditionalFormattingUIPlugin } from '@univerjs/sheets-conditional-formatting-ui';
import { UniverSheetsFilterPlugin } from '@univerjs/sheets-filter';
import { UniverSheetsFilterUIPlugin } from '@univerjs/sheets-filter-ui';
import { UniverFindReplacePlugin } from '@univerjs/find-replace';
import { UniverSheetsFindReplacePlugin } from '@univerjs/sheets-find-replace';
import { UniverSheetsHyperLinkPlugin } from '@univerjs/sheets-hyper-link';
import { UniverSheetsHyperLinkUIPlugin } from '@univerjs/sheets-hyper-link-ui';
import { UniverSheetsNotePlugin } from '@univerjs/sheets-note';
import { UniverSheetsNoteUIPlugin } from '@univerjs/sheets-note-ui';
import { UniverDrawingPlugin } from '@univerjs/drawing';
import { UniverDocsDrawingPlugin } from '@univerjs/docs-drawing';
import { UniverDrawingUIPlugin } from '@univerjs/drawing-ui';
import { UniverSheetsDrawingPlugin } from '@univerjs/sheets-drawing';
import { UniverSheetsDrawingUIPlugin } from '@univerjs/sheets-drawing-ui';
import { UniverThreadCommentUIPlugin } from '@univerjs/thread-comment-ui';
import { UniverSheetsThreadCommentPlugin } from '@univerjs/sheets-thread-comment';
import { UniverSheetsThreadCommentUIPlugin } from '@univerjs/sheets-thread-comment-ui';

// Import locales
import docsUIEnUS from '@univerjs/docs-ui/locale/en-US';
import sheetsEnUS from '@univerjs/sheets/locale/en-US';
import sheetsFormulaEnUS from '@univerjs/sheets-formula/locale/en-US';
import sheetsFormulaUIEnUS from '@univerjs/sheets-formula-ui/locale/en-US';
import sheetsUIEnUS from '@univerjs/sheets-ui/locale/en-US';
import sheetsNumfmtUIEnUS from '@univerjs/sheets-numfmt-ui/locale/en-US';
import uiEnUS from '@univerjs/ui/locale/en-US';
import sheetsConditionalFormattingUIEnUS from '@univerjs/sheets-conditional-formatting-ui/locale/en-US';
import sheetsFilterEnUS from '@univerjs/preset-sheets-filter/locales/en-US';
import sheetsFindReplaceEnUS from '@univerjs/preset-sheets-find-replace/locales/en-US';
import sheetsHyperLinkEnUS from '@univerjs/preset-sheets-hyper-link/locales/en-US';
import sheetsDrawingEnUS from '@univerjs/preset-sheets-drawing/locales/en-US';
import sheetsThreadCommentEnUS from '@univerjs/preset-sheets-thread-comment/locales/en-US';
import sheetsDataValidationUIEnUS from '@univerjs/sheets-data-validation-ui/locale/en-US';

// Import Facade APIs
import '@univerjs/engine-formula/facade';
import '@univerjs/ui/facade';
import '@univerjs/docs-ui/facade';
import '@univerjs/sheets/facade';
import '@univerjs/sheets-ui/facade';
import '@univerjs/sheets-formula/facade';
import '@univerjs/sheets-numfmt/facade';

// Import styles in correct order
import '@univerjs/design/lib/index.css';
import '@univerjs/ui/lib/index.css';
import '@univerjs/docs-ui/lib/index.css';
import '@univerjs/sheets-ui/lib/index.css';
import '@univerjs/sheets-formula-ui/lib/index.css';
import '@univerjs/sheets-numfmt-ui/lib/index.css';

import { registerCustomFunctions } from './customFormulas';

// Import optimized utilities
import { rangeToA1, cellRefToIndices, parseRange, startPeriodicCacheCleanup, stopPeriodicCacheCleanup } from './univerUtils';

interface Props {
    initialData: IWorkbookData;
    onCellChange: (cellRef: string, value: ICellData) => void;
    onFormulaIntercept: (cellRef: string, formula: string) => void;
    onSelectionChange?: (cellRef: string) => void;
    onUniverReady?: (univerInstance: ReturnType<typeof createUniver>['univer']) => void;
    tabId?: string; // Optional tab ID for better instance tracking
}

export interface UniverSpreadsheetRef {
    updateCell: (cellRef: string, value: { v?: string | number; f?: string }) => void;
    getCellValue: (cellRef: string) => string | number | null;
    getRange: (rangeRef: string) => (string | number)[][];
    univer: ReturnType<typeof createUniver> | null;
}

const UniverSpreadsheet = forwardRef<UniverSpreadsheetRef, Props>(
    ({ initialData, onCellChange, onFormulaIntercept, onSelectionChange, onUniverReady, tabId }, ref) => {
        const theme = useTheme();
        const containerRef = useRef<HTMLDivElement>(null);
        const univerRef = useRef<ReturnType<typeof createUniver> | null>(null);
        const isInitializedRef = useRef(false);
        const onCellChangeRef = useRef(onCellChange);
        const onFormulaInterceptRef = useRef(onFormulaIntercept);
        const onSelectionChangeRef = useRef(onSelectionChange);
        const onUniverReadyRef = useRef(onUniverReady);
        // Generate unique container ID for this instance
        const containerIdRef = useRef(
            tabId && tabId.length > 0
                ? `univer-container-${tabId}`
                : `univer-container-${Math.random().toString(36).substring(2, 11)}`
        );

        // Keep refs updated
        useEffect(() => {
            onCellChangeRef.current = onCellChange;
            onFormulaInterceptRef.current = onFormulaIntercept;
            onSelectionChangeRef.current = onSelectionChange;
            onUniverReadyRef.current = onUniverReady;
        }, [onCellChange, onFormulaIntercept, onSelectionChange, onUniverReady]);

        useImperativeHandle(ref, () => ({
            updateCell: (cellRef: string, value: { v?: string | number; f?: string }) => {
                if (!univerRef.current) { return; }

                try {
                    const injector = univerRef.current.univer.__getInjector();
                    const commandService = injector.get(ICommandService);
                    const instanceService = injector.get(IUniverInstanceService);
                    const workbook = instanceService.getFocusedUnit() as Workbook;
                    const activeSheet = workbook.getActiveSheet();
                    const indices = cellRefToIndices(cellRef);
                    if (!indices) { return; }

                    const { row, col: colIndex } = indices;

                    const cellValue = typeof value === 'object' ? value : { v: value };

                    void commandService.executeCommand('sheet.command.set-range-values', {
                        unitId: workbook.getUnitId(),
                        subUnitId: activeSheet.getSheetId(),
                        range: {
                            startRow: row,
                            startColumn: colIndex,
                            endRow: row,
                            endColumn: colIndex
                        },
                        value: [[cellValue]]
                    });
                } catch (error) {
                    console.error('Failed to update cell:', error);
                }
            },
            getCellValue: (cellRef: string): string | number | null => {
                if (!univerRef.current) { return null; }

                try {
                    const injector = univerRef.current.univer.__getInjector();
                    const instanceService = injector.get(IUniverInstanceService);
                    const workbook = instanceService.getFocusedUnit() as Workbook;
                    const activeSheet = workbook.getActiveSheet();
                    const indices = cellRefToIndices(cellRef);
                    if (!indices) { return null; }

                    const { row, col: colIndex } = indices;

                    const cellData = activeSheet.getCellRaw(row, colIndex);
                    if (!cellData) { return null; }

                    return cellData.v !== undefined ? cellData.v as string | number : null;
                } catch (error) {
                    console.error('Failed to get cell value:', error);
                    return null;
                }
            },
            getRange: (rangeRef: string): (string | number)[][] => {
                if (!univerRef.current) { return []; }

                try {
                    const injector = univerRef.current.univer.__getInjector();
                    const instanceService = injector.get(IUniverInstanceService);
                    const workbook = instanceService.getFocusedUnit() as Workbook;
                    const activeSheet = workbook.getActiveSheet();
                    const parsedRange = parseRange(rangeRef);
                    if (!parsedRange) { return []; }

                    const { startCol: startColIndex, startRow, endCol: endColIndex, endRow } = parsedRange;

                    const result: (string | number)[][] = [];
                    for (let row = startRow; row <= endRow; row++) {
                        const rowValues: (string | number)[] = [];
                        for (let col = startColIndex; col <= endColIndex; col++) {
                            const cellData = activeSheet.getCellRaw(row, col);
                            const value = cellData && cellData.v !== undefined ? cellData.v as string | number : '';
                            rowValues.push(value);
                        }
                        result.push(rowValues);
                    }

                    return result;
                } catch (error) {
                    console.error('Failed to get range:', error);
                    return [];
                }
            },
            get univer() {
                return univerRef.current;
            }
        }));

        useEffect(() => {
            if (!containerRef.current || isInitializedRef.current) {
                return;
            }

            const containerId = containerIdRef.current;

            // Track active instances to help with debugging
            window.__UNIVER_INSTANCES__ ??= new Set();

            isInitializedRef.current = true;
            window.__UNIVER_INSTANCES__.add(containerId);

            if (process.env.NODE_ENV === 'development') {
                console.log(`Initializing Univer instance for container: ${containerIdRef.current}`);
                console.log(`Active instances: ${window.__UNIVER_INSTANCES__.size}`);
            }

            const { univer, univerAPI } = createUniver({
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
                        sheetsConditionalFormattingUIEnUS,
                        sheetsFilterEnUS,
                        sheetsFindReplaceEnUS,
                        sheetsHyperLinkEnUS,
                        sheetsDrawingEnUS,
                        sheetsThreadCommentEnUS,
                        sheetsDataValidationUIEnUS
                    ),
                },
                presets: [],
                plugins: [
                    // Core plugins - essential services first
                    UniverRenderEnginePlugin,
                    UniverFormulaEnginePlugin,
                    [UniverUIPlugin, { container: containerIdRef.current }],

                    // Essential sheet plugins
                    UniverSheetsPlugin,
                    [UniverSheetsUIPlugin, {
                        formulaBar: true,
                        footer: true,
                        clipboardConfig: {
                            enableCopyPasteShortcut: true,
                            enableCopyPasteMenu: true
                        },
                        scrollConfig: {
                            enableCache: true,
                            cacheSize: 100,
                            enableVirtualScrolling: true
                        },
                        protectedRangeShadow: true,
                        protectedRangeUserSelector: true,
                        disableForceStringAlert: true,
                        disableForceStringMark: true
                    }],

                    // Formula plugins
                    UniverSheetsFormulaPlugin,
                    UniverSheetsFormulaUIPlugin,

                    // Number formatting
                    UniverSheetsNumfmtPlugin,
                    UniverSheetsNumfmtUIPlugin,

                    // Conditional plugins
                    UniverSheetsConditionalFormattingPlugin,
                    UniverSheetsConditionalFormattingUIPlugin,

                    // Filter functionality
                    [UniverSheetsFilterPlugin, { enableSyncSwitch: true }],
                    UniverSheetsFilterUIPlugin,

                    // Search functionality
                    UniverFindReplacePlugin,
                    UniverSheetsFindReplacePlugin,

                    // Optional features
                    UniverSheetsHyperLinkPlugin,
                    UniverSheetsHyperLinkUIPlugin,
                    UniverSheetsNotePlugin,
                    UniverSheetsNoteUIPlugin,

                    // Data validation
                    UniverSheetsDataValidationPlugin,

                    // Document and drawing plugins
                    UniverNetworkPlugin,
                    [UniverDocsPlugin, { hasScroll: true }],
                    UniverDocsUIPlugin,
                    [UniverDrawingPlugin, { override: [] }],
                    UniverDocsDrawingPlugin,
                    UniverDrawingUIPlugin,
                    UniverSheetsDrawingPlugin,
                    UniverSheetsDrawingUIPlugin,

                    // Comment plugins
                    UniverThreadCommentUIPlugin,
                    UniverSheetsThreadCommentPlugin,
                    UniverSheetsThreadCommentUIPlugin,
                ],
            });
            univerRef.current = { univer, univerAPI };

            univer.createUnit(UniverInstanceType.UNIVER_SHEET, initialData);

            // Register custom mathematical functions with high precision
            const injector = univer.__getInjector();
            const formulaEngine = injector.get(IRegisterFunctionService);

            // Register custom mathematical functions with the formula engine
            registerCustomFunctions(formulaEngine);
            const commandService = injector.get(ICommandService);

            // Start periodic cache cleanup for long-running sessions
            startPeriodicCacheCleanup();

            // Track selection changes
            const selectionDisposable = commandService.onCommandExecuted((command: ICommandInfo) => {
                if (command.id === 'sheet.operation.set-selections') {
                    const params = command.params as { selections?: Array<{ range?: IRange }> };
                    if (params.selections && params.selections.length > 0) {
                        const selection = params.selections[0];
                        if (selection?.range) {
                            try {
                                const cellRef = rangeToA1(selection.range);
                                onSelectionChangeRef.current!(cellRef);
                            } catch (error) {
                                console.warn('[UniverSpreadsheet] Error converting selection range to A1:', error);
                            }
                        }
                    }
                }
            });

            // Helper function to handle cell change events from both command types
            const handleCellChangeCommand = (command: ICommandInfo, commandType: 'mutation' | 'command') => {
                const params = command.params as { range?: IRange; value?: ICellData[][] };
                if (!params.range) {
                    // Some internal Univer operations don't provide range - this is normal
                    return;
                }

                try {
                    const cellRef = rangeToA1(params.range);

                    // For mutation commands, we have the actual cell data
                    if (commandType === 'mutation' && params.value) {
                        // Defensive validation: ensure params.value is a valid 2D array with at least one cell
                        if (!Array.isArray(params.value) || params.value.length === 0) {
                            if (process.env.NODE_ENV === 'development') {
                                console.warn('[UniverSpreadsheet] set-range-values: params.value is not a valid array or is empty');
                            }
                            return;
                        }

                        const firstRow = params.value[0];
                        if (!Array.isArray(firstRow) || firstRow.length === 0) {
                            if (process.env.NODE_ENV === 'development') {
                                console.warn('[UniverSpreadsheet] set-range-values: first row is not a valid array or is empty');
                            }
                            return;
                        }

                        const firstCell = firstRow[0];
                        if (firstCell === undefined) {
                            if (process.env.NODE_ENV === 'development') {
                                console.warn('[UniverSpreadsheet] set-range-values: first cell is undefined or null');
                            }
                            return;
                        }

                        const value = firstCell;

                        if (value.v !== undefined && value.v !== null) {
                            if (typeof value.v !== 'string') {
                                onCellChangeRef.current(cellRef, value);
                            } else if (!value.v.startsWith('=')) {
                                onCellChangeRef.current(cellRef, value);
                            }
                        }
                    } else if (commandType === 'command') {
                        // For command events, we don't have the cell data but we still need to trigger bounds update
                        // Create a minimal cell data object to trigger bounds tracking
                        const minimalCellData: ICellData = { v: null };
                        onCellChangeRef.current(cellRef, minimalCellData);
                    }
                } catch (error) {
                    console.warn(`[UniverSpreadsheet] Error converting range to A1 in ${commandType} set-range-values:`, error);
                }
            };

            // Listen to both mutation and command events for comprehensive bounds tracking
            const afterCommandDisposable = commandService.onCommandExecuted((command: ICommandInfo) => {
                if (command.id === 'sheet.mutation.set-range-values') {
                    handleCellChangeCommand(command, 'mutation');
                } else if (command.id === 'sheet.command.set-range-values') {
                    handleCellChangeCommand(command, 'command');
                }
            });

            // Separate handler for formula interception to avoid conflicts with cell change handling
            const editingDisposable = commandService.onCommandExecuted((command: ICommandInfo) => {
                // Handle formula interception for both command types
                if (command.id === 'sheet.command.set-range-values' || command.id === 'sheet.mutation.set-range-values') {
                    const params = command.params as { range?: IRange; value?: ICellData[][] };
                    const cellValue = params.value?.[0]?.[0];

                    // Only intercept formulas (strings starting with '=')
                    if (cellValue?.v && typeof cellValue.v === 'string' && cellValue.v.startsWith('=') && params.range) {
                        try {
                            const cellRef = rangeToA1(params.range);
                            onFormulaInterceptRef.current(cellRef, cellValue.v);
                        } catch (error) {
                            console.warn('[UniverSpreadsheet] Error converting range to A1 in formula intercept:', error);
                        }
                    }
                }
            });

            // Notify parent that Univer is ready
            if (onUniverReadyRef.current) {
                onUniverReadyRef.current(univer);
            }

            return () => {
                if (process.env.NODE_ENV === 'development') {
                    console.log('Cleaning up Univer...');
                }

                // Stop periodic cache cleanup
                stopPeriodicCacheCleanup();

                selectionDisposable.dispose();
                afterCommandDisposable.dispose();
                editingDisposable.dispose();

                if (univerRef.current) {
                    if (process.env.NODE_ENV === 'development') {
                        console.log(`Disposing Univer instance for container: ${containerId}`);
                    }
                    univerRef.current.univer.dispose();
                    univerRef.current = null;
                }

                // Remove from instance tracking
                if (window.__UNIVER_INSTANCES__) {
                    window.__UNIVER_INSTANCES__.delete(containerId);
                    if (process.env.NODE_ENV === 'development') {
                        console.log(`Active instances after cleanup: ${window.__UNIVER_INSTANCES__.size}`);
                    }
                }

                isInitializedRef.current = false;
            };
        }, [initialData]);

        return (
            <div
                ref={containerRef}
                id={containerIdRef.current}
                className="univer-spreadsheet-container"
                style={{
                    width: '100%',
                    height: '100%',
                    minHeight: '400px',
                    backgroundColor: theme.palette.background.paper
                }}
            />
        );
    }
);

export default UniverSpreadsheet;