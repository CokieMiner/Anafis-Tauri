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
    Univer,
} from '@univerjs/core';

// LAYER 1: Core infrastructure plugins (no dependencies)
import { UniverRenderEnginePlugin } from '@univerjs/engine-render';
import { UniverFormulaEnginePlugin } from '@univerjs/engine-formula';
import { UniverNetworkPlugin } from '@univerjs/network';

// LAYER 2: UI and Document foundation
import { UniverUIPlugin } from '@univerjs/ui';
import { UniverDocsPlugin } from '@univerjs/docs';
import { UniverDocsUIPlugin } from '@univerjs/docs-ui';

// LAYER 3: Base Sheets (depends on Docs, UI, Engines)
import { UniverSheetsPlugin } from '@univerjs/sheets';
import { UniverSheetsUIPlugin } from '@univerjs/sheets-ui';

// LAYER 4: Formula extensions (depends on Sheets UI)
import { IRegisterFunctionService, UniverSheetsFormulaPlugin } from '@univerjs/sheets-formula';
import { UniverSheetsFormulaUIPlugin } from '@univerjs/sheets-formula-ui';

// LAYER 5: Number formatting (depends on Sheets UI)
import { UniverSheetsNumfmtPlugin } from '@univerjs/sheets-numfmt';
import { UniverSheetsNumfmtUIPlugin } from '@univerjs/sheets-numfmt-ui';

// LAYER 6: Data validation (depends on Sheets UI, Formula UI, Numfmt)
import { UniverSheetsDataValidationPlugin } from '@univerjs/sheets-data-validation';
import { UniverSheetsDataValidationUIPlugin } from '@univerjs/sheets-data-validation-ui';

// LAYER 7: Conditional formatting (depends on Sheets UI)
import { UniverSheetsConditionalFormattingPlugin } from '@univerjs/sheets-conditional-formatting';
import { UniverSheetsConditionalFormattingUIPlugin } from '@univerjs/sheets-conditional-formatting-ui';

// LAYER 8: Filter functionality (depends on Sheets UI)
import { UniverSheetsFilterPlugin } from '@univerjs/sheets-filter';
import { UniverSheetsFilterUIPlugin } from '@univerjs/sheets-filter-ui';

// LAYER 9: Find & Replace (depends on UI)
import { UniverFindReplacePlugin } from '@univerjs/find-replace';
import { UniverSheetsFindReplacePlugin } from '@univerjs/sheets-find-replace';

// LAYER 10: Hyperlinks (depends on Sheets UI)
import { UniverSheetsHyperLinkPlugin } from '@univerjs/sheets-hyper-link';
import { UniverSheetsHyperLinkUIPlugin } from '@univerjs/sheets-hyper-link-ui';

// LAYER 11: Notes/Comments (depends on Sheets UI)
import { UniverSheetsNotePlugin } from '@univerjs/sheets-note';
import { UniverSheetsNoteUIPlugin } from '@univerjs/sheets-note-ui';

// LAYER 12: Drawing (depends on Docs, Sheets)
import { UniverDrawingPlugin } from '@univerjs/drawing';
import { UniverDocsDrawingPlugin } from '@univerjs/docs-drawing';
import { UniverDrawingUIPlugin } from '@univerjs/drawing-ui';
import { UniverSheetsDrawingPlugin } from '@univerjs/sheets-drawing';
import { UniverSheetsDrawingUIPlugin } from '@univerjs/sheets-drawing-ui';

// LAYER 13: Thread Comments (highest dependencies - must be last)
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
import sheetsFilterUIEnUS from '@univerjs/sheets-filter-ui/locale/en-US';
import findReplaceEnUS from '@univerjs/find-replace/locale/en-US';
import sheetsHyperLinkUIEnUS from '@univerjs/sheets-hyper-link-ui/locale/en-US';
import sheetsDrawingUIEnUS from '@univerjs/sheets-drawing-ui/locale/en-US';
import sheetsThreadCommentUIEnUS from '@univerjs/sheets-thread-comment-ui/locale/en-US';
import sheetsDataValidationUIEnUS from '@univerjs/sheets-data-validation-ui/locale/en-US';

// Import styles FIRST - before Facade APIs
// Styles must be loaded before Facade initialization
import '@univerjs/design/lib/index.css';
import '@univerjs/ui/lib/index.css';
import '@univerjs/docs-ui/lib/index.css';
import '@univerjs/sheets-ui/lib/index.css';
import '@univerjs/sheets-formula-ui/lib/index.css';
import '@univerjs/sheets-numfmt-ui/lib/index.css';

// Import Facade APIs LAST - after all plugins and styles are loaded
// These are side-effect imports that initialize global APIs
// They must come after CSS to avoid DOM/style access issues
import '@univerjs/engine-formula/facade';
import '@univerjs/ui/facade';
import '@univerjs/docs-ui/facade';
import '@univerjs/sheets/facade';
import '@univerjs/sheets-ui/facade';
import '@univerjs/sheets-formula/facade';
import '@univerjs/sheets-numfmt/facade';

import { registerCustomFunctions } from './customFormulas';

// Import optimized utilities
import { rangeToA1, cellRefToIndices, parseRange, startPeriodicCacheCleanup, stopPeriodicCacheCleanup } from './univerUtils';

interface Props {
    initialData: IWorkbookData;
    onCellChange: (cellRef: string, value: ICellData) => void;
    onFormulaIntercept: (cellRef: string, formula: string) => void;
    onSelectionChange?: (cellRef: string) => void;
    onUniverReady?: (univerInstance: Univer) => void;
    tabId?: string; // Optional tab ID for better instance tracking
}

export interface UniverSpreadsheetRef {
    updateCell: (cellRef: string, value: { v?: string | number; f?: string }) => void;
    getCellValue: (cellRef: string) => string | number | null;
    getRange: (rangeRef: string) => (string | number)[][];
    univer: Univer | null;
}

const UniverSpreadsheet = forwardRef<UniverSpreadsheetRef, Props>(
    ({ initialData, onCellChange, onFormulaIntercept, onSelectionChange, onUniverReady, tabId }, ref) => {
        const theme = useTheme();
        const containerRef = useRef<HTMLDivElement>(null);
        const univerRef = useRef<Univer | null>(null);
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
                    const injector = univerRef.current.__getInjector();
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
                    const injector = univerRef.current.__getInjector();
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
                    const injector = univerRef.current.__getInjector();
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

            // Create Univer instance with direct plugin registration
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
                        sheetsConditionalFormattingUIEnUS,
                        sheetsFilterUIEnUS,
                        findReplaceEnUS,
                        sheetsHyperLinkUIEnUS,
                        sheetsDrawingUIEnUS,
                        sheetsThreadCommentUIEnUS,
                        sheetsDataValidationUIEnUS
                    ),
                },
            });

            // Register plugins in the correct dependency order
            // LAYER 1: Core infrastructure (no dependencies on other plugins)
            univer.registerPlugin(UniverRenderEnginePlugin);
            univer.registerPlugin(UniverFormulaEnginePlugin);
            univer.registerPlugin(UniverNetworkPlugin); // Only depends on @univerjs/core

            // LAYER 2: UI and Document foundation
            univer.registerPlugin(UniverUIPlugin, { container: containerIdRef.current });
            univer.registerPlugin(UniverDocsPlugin, { hasScroll: false });
            univer.registerPlugin(UniverDocsUIPlugin);

            // LAYER 3: Base Sheets (depends on Docs, UI, Engines)
            univer.registerPlugin(UniverSheetsPlugin);
            univer.registerPlugin(UniverSheetsUIPlugin); // Depends on: sheets, docs-ui, ui, telemetry

            // LAYER 4: Formula extensions (depends on Sheets UI)
            univer.registerPlugin(UniverSheetsFormulaPlugin);
            univer.registerPlugin(UniverSheetsFormulaUIPlugin);

            // LAYER 5: Number formatting (depends on Sheets UI)
            univer.registerPlugin(UniverSheetsNumfmtPlugin);
            univer.registerPlugin(UniverSheetsNumfmtUIPlugin);

            // LAYER 6: Data validation (depends on Sheets UI, Formula UI, Numfmt)
            univer.registerPlugin(UniverSheetsDataValidationPlugin);
            univer.registerPlugin(UniverSheetsDataValidationUIPlugin);

            // LAYER 7: Conditional formatting (depends on Sheets UI)
            univer.registerPlugin(UniverSheetsConditionalFormattingPlugin);
            univer.registerPlugin(UniverSheetsConditionalFormattingUIPlugin);

            // LAYER 8: Filter functionality (depends on Sheets UI)
            univer.registerPlugin(UniverSheetsFilterPlugin);
            univer.registerPlugin(UniverSheetsFilterUIPlugin);

            // LAYER 9: Find & Replace (depends on UI)
            univer.registerPlugin(UniverFindReplacePlugin);
            univer.registerPlugin(UniverSheetsFindReplacePlugin);

            // LAYER 10: Hyperlinks (depends on Sheets UI)
            univer.registerPlugin(UniverSheetsHyperLinkPlugin);
            univer.registerPlugin(UniverSheetsHyperLinkUIPlugin);

            // LAYER 11: Notes/Comments (depends on Sheets UI)
            univer.registerPlugin(UniverSheetsNotePlugin);
            univer.registerPlugin(UniverSheetsNoteUIPlugin);

            // LAYER 12: Drawing (depends on Docs, Sheets)
            univer.registerPlugin(UniverDrawingPlugin, { override: [] });
            univer.registerPlugin(UniverDocsDrawingPlugin);
            univer.registerPlugin(UniverDrawingUIPlugin);
            univer.registerPlugin(UniverSheetsDrawingPlugin);
            univer.registerPlugin(UniverSheetsDrawingUIPlugin);

            // LAYER 13: Thread Comments (depends on Sheets UI and thread-comment base)
            // Must be last - depends on almost everything including sheets-ui
            univer.registerPlugin(UniverThreadCommentUIPlugin);
            univer.registerPlugin(UniverSheetsThreadCommentPlugin);
            univer.registerPlugin(UniverSheetsThreadCommentUIPlugin);

            univerRef.current = univer;

            univer.createUnit(UniverInstanceType.UNIVER_SHEET, initialData);

            // Register custom mathematical functions with high precision
            const injector = univer.__getInjector();
            const formulaEngine = injector.get(IRegisterFunctionService);

            // Register custom mathematical functions with the formula engine
            registerCustomFunctions(formulaEngine);
            const commandService = injector.get(ICommandService);

            // Start periodic cache cleanup for long-running sessions  
            startPeriodicCacheCleanup();

            // Disposal flag to prevent handlers from running after cleanup
            let isDisposed = false;

            // Track selection changes
            const selectionDisposable = commandService.onCommandExecuted((command: ICommandInfo) => {
                if (isDisposed) {return;} // Guard against post-cleanup execution
                
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
                if (isDisposed) {return;} // Guard against post-cleanup execution
                
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
                if (isDisposed) {return;} // Guard against post-cleanup execution
                
                if (command.id === 'sheet.mutation.set-range-values') {
                    handleCellChangeCommand(command, 'mutation');
                } else if (command.id === 'sheet.command.set-range-values') {
                    handleCellChangeCommand(command, 'command');
                }
            });

            // Separate handler for formula interception to avoid conflicts with cell change handling
            const editingDisposable = commandService.onCommandExecuted((command: ICommandInfo) => {
                if (isDisposed) {return;} // Guard against post-cleanup execution
                
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
                // Set disposal flag FIRST to stop all handlers immediately
                isDisposed = true;
                
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
                    univerRef.current.dispose();
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