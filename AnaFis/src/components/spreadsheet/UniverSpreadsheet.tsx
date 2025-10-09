// UniverSpreadsheet.tsx - USING PRESETS APPROACH
import { useRef, useEffect, forwardRef, useImperativeHandle } from 'react';
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
} from '@univerjs/core';
import { createUniver } from '@univerjs/presets';
import { UniverSheetsCorePreset } from '@univerjs/preset-sheets-core';
import UniverPresetSheetsCoreEnUS from '@univerjs/preset-sheets-core/locales/en-US';

import '@univerjs/preset-sheets-core/lib/index.css';
import { defaultTheme } from '@univerjs/design';


function columnToLetter(column: number): string {
    let temp, letter = '';
    while (column >= 0) {
        temp = column % 26;
        letter = String.fromCharCode(temp + 65) + letter;
        column = Math.floor(column / 26) - 1;
    }
    return letter;
}

function rangeToA1(range: IRange): string {
    const startCol = columnToLetter(range.startColumn);
    const startRow = range.startRow + 1;
    const endCol = columnToLetter(range.endColumn);
    const endRow = range.endRow + 1;
    
    // If it's a single cell, return just that cell
    if (range.startColumn === range.endColumn && range.startRow === range.endRow) {
        return `${startCol}${startRow}`;
    }
    
    // Otherwise return full range notation
    return `${startCol}${startRow}:${endCol}${endRow}`;
}

interface Props {
    initialData: IWorkbookData;
    onCellChange: (cellRef: string, value: ICellData) => void;
    onFormulaIntercept: (cellRef: string, formula: string) => void;
    onSelectionChange?: (cellRef: string) => void;
}

export interface UniverSpreadsheetRef {
    updateCell: (cellRef: string, value: { v?: string | number; f?: string }) => void;
    getCellValue: (cellRef: string) => string | number | null;
    getRange: (rangeRef: string) => Promise<(string | number)[][]>;
}

const UniverSpreadsheet = forwardRef<UniverSpreadsheetRef, Props>(
    ({ initialData, onCellChange, onFormulaIntercept, onSelectionChange }, ref) => {
        const theme = useTheme();
        const containerRef = useRef<HTMLDivElement>(null);
        const univerRef = useRef<ReturnType<typeof createUniver> | null>(null);
        const isInitializedRef = useRef(false);
        const onCellChangeRef = useRef(onCellChange);
        const onFormulaInterceptRef = useRef(onFormulaIntercept);
        const onSelectionChangeRef = useRef(onSelectionChange);

        // Keep refs updated
        useEffect(() => {
            onCellChangeRef.current = onCellChange;
            onFormulaInterceptRef.current = onFormulaIntercept;
            onSelectionChangeRef.current = onSelectionChange;
        }, [onCellChange, onFormulaIntercept, onSelectionChange]);

        useImperativeHandle(ref, () => ({
            updateCell: (cellRef: string, value: { v?: string | number; f?: string }) => {
                if (!univerRef.current) return;

                try {
                    const injector = univerRef.current.univer.__getInjector();
                    const commandService = injector.get(ICommandService);
                    const instanceService = injector.get(IUniverInstanceService);
                    const workbook = instanceService.getFocusedUnit() as Workbook;
                    if (!workbook) return;

                    const activeSheet = workbook.getActiveSheet();
                    if (!activeSheet) return;

                    const match = cellRef.match(/^([A-Z]+)(\d+)$/);
                    if (!match) return;

                    const col = match[1];
                    const row = parseInt(match[2]) - 1;
                    let colIndex = 0;
                    for (let i = 0; i < col.length; i++) {
                        colIndex = colIndex * 26 + (col.charCodeAt(i) - 65 + 1);
                    }
                    colIndex -= 1;

                    const cellValue = typeof value === 'object' ? value : { v: value };

                    commandService.executeCommand('sheet.command.set-range-values', {
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
                if (!univerRef.current) return null;

                try {
                    const injector = univerRef.current.univer.__getInjector();
                    const instanceService = injector.get(IUniverInstanceService);
                    const workbook = instanceService.getFocusedUnit() as Workbook;
                    if (!workbook) return null;

                    const activeSheet = workbook.getActiveSheet();
                    if (!activeSheet) return null;

                    const match = cellRef.match(/^([A-Z]+)(\d+)$/);
                    if (!match) return null;

                    const col = match[1];
                    const row = parseInt(match[2]) - 1;
                    let colIndex = 0;
                    for (let i = 0; i < col.length; i++) {
                        colIndex = colIndex * 26 + (col.charCodeAt(i) - 65 + 1);
                    }
                    colIndex -= 1;

                    const cellData = activeSheet.getCellRaw(row, colIndex);
                    if (!cellData) return null;

                    // Return the calculated value (v) if available, otherwise null
                    return cellData.v !== undefined ? cellData.v as string | number : null;
                } catch (error) {
                    console.error('Failed to get cell value:', error);
                    return null;
                }
            },
            getRange: async (rangeRef: string): Promise<(string | number)[][]> => {
                if (!univerRef.current) return [];

                try {
                    const injector = univerRef.current.univer.__getInjector();
                    const instanceService = injector.get(IUniverInstanceService);
                    const workbook = instanceService.getFocusedUnit() as Workbook;
                    if (!workbook) return [];

                    const activeSheet = workbook.getActiveSheet();
                    if (!activeSheet) return [];

                    // Parse range: A1:B10 or A1 (single cell)
                    const rangeMatch = rangeRef.match(/^([A-Z]+)(\d+):([A-Z]+)(\d+)$/);
                    const singleMatch = rangeRef.match(/^([A-Z]+)(\d+)$/);

                    let startCol: string, startRow: number, endCol: string, endRow: number;

                    if (rangeMatch) {
                        [, startCol, startRow, endCol, endRow] = rangeMatch.map((v, i) => i === 2 || i === 4 ? parseInt(v) - 1 : v) as [string, string, number, string, number];
                    } else if (singleMatch) {
                        startCol = endCol = singleMatch[1];
                        startRow = endRow = parseInt(singleMatch[2]) - 1;
                    } else {
                        return [];
                    }

                    // Convert column letters to indices
                    const colToIndex = (col: string): number => {
                        let index = 0;
                        for (let i = 0; i < col.length; i++) {
                            index = index * 26 + (col.charCodeAt(i) - 65 + 1);
                        }
                        return index - 1;
                    };

                    const startColIndex = colToIndex(startCol);
                    const endColIndex = colToIndex(endCol);

                    // Extract values row by row
                    const result: (string | number)[][] = [];
                    for (let row = startRow; row <= endRow; row++) {
                        const rowValues: (string | number)[] = [];
                        for (let col = startColIndex; col <= endColIndex; col++) {
                            const cellData = activeSheet.getCellRaw(row, col);
                            const value = cellData?.v !== undefined ? cellData.v as string | number : '';
                            rowValues.push(value);
                        }
                        result.push(rowValues);
                    }

                    return result;
                } catch (error) {
                    console.error('Failed to get range:', error);
                    return [];
                }
            }
        }));

        useEffect(() => {
            if (!containerRef.current || isInitializedRef.current) {
                return;
            }

            isInitializedRef.current = true;
            console.log('Initializing Univer with Presets...');

            const { univer, univerAPI } = createUniver({
                theme: defaultTheme,
                darkMode: true,
                locale: LocaleType.EN_US,
                locales: {
                    [LocaleType.EN_US]: UniverPresetSheetsCoreEnUS,
                },
                presets: [
                    UniverSheetsCorePreset({
                        container: containerRef.current,
                    })
                ],
            });

            univerRef.current = { univer, univerAPI };

            univer.createUnit(UniverInstanceType.UNIVER_SHEET, initialData);

            const injector = univer.__getInjector();
            const commandService = injector.get(ICommandService);

            // Track selection changes
            const selectionDisposable = commandService.onCommandExecuted((command: ICommandInfo) => {
                if (command.id === 'sheet.operation.set-selections') {
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    const params = command.params as any;
                    if (params?.selections && params.selections.length > 0) {
                        const selection = params.selections[0];
                        const range = selection.range;
                        if (range) {
                            const cellRef = rangeToA1(range);
                            onSelectionChangeRef.current?.(cellRef);
                        }
                    }
                }
            });

            const afterCommandDisposable = commandService.onCommandExecuted((command: ICommandInfo) => {
                if (command.id === 'sheet.mutation.set-range-values') {
                    const params = command.params as { range: IRange; value: ICellData[][] };
                    if (params?.range && params?.value) {
                        const cellRef = rangeToA1(params.range);
                        const value = params.value[0][0];
                        
                        if (!value?.v?.toString().startsWith('=')) {
                            onCellChangeRef.current(cellRef, value);
                        }
                    }
                }
            });

            const editingDisposable = commandService.onCommandExecuted((command: ICommandInfo) => {
                if (command.id === 'sheet.command.set-range-values') {
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    const params = command.params as any;
                    const value = params?.value?.[0]?.[0]?.v;

                    if (typeof value === 'string' && value.startsWith('=')) {
                        const cellRef = rangeToA1(params.range);
                        onFormulaInterceptRef.current(cellRef, value);
                    }
                }
            });

            return () => {
                console.log('Cleaning up Univer...');
                selectionDisposable?.dispose();
                afterCommandDisposable?.dispose();
                editingDisposable?.dispose();
                
                if (univerRef.current) {
                    univerRef.current.univer.dispose();
                    univerRef.current = null;
                }
                isInitializedRef.current = false;
            };
        }, [initialData]);

        return (
            <div
                ref={containerRef}
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