import { columnToLetter } from './univerUtils';
import {
    ICellData,
    IStyleData,
    CellValueType,
    Nullable,
    type IWorkbookData
} from '@univerjs/core';
import { FUniver } from '@univerjs/core/facade';
import { FWorksheet } from '@univerjs/sheets/facade';

/**
 * Convert a column number (1-based) to Excel-style column letters (A, B, ..., Z, AA, AB, etc.)
 */
function columnNumberToLetters(columnNumber: number): string {
  if (columnNumber <= 0) {
    return 'A';
  }
  // columnNumber is 1-based (number of columns), convert to 0-based index
  return columnToLetter(columnNumber - 1);
}

/**
 * Build a default range string from maxRows and maxCols options
 */
function buildDefaultRange(options: ExtractionOptions): string {
    const maxCols = options.maxCols ?? 26; // Default to 26 columns (A-Z)
    const maxRows = options.maxRows ?? 100; // Default to 100 rows

    const endCol = columnNumberToLetters(maxCols);
    return `A1:${endCol}${maxRows}`;
}

/**
 * Comprehensive cell formatting information based on Univer's ICellData structure
 */
export interface CellFormatInfo {
    // Core cell data (from ICellData)
    v?: string | number | boolean | null | undefined;  // Cell original value
    f?: string;                     // Formula
    t?: CellValueType | undefined;  // Cell type (1=string, 2=number, 3=boolean, 4=force text)
    s?: string | IStyleData;        // Style id or style object
    p?: unknown;                        // Rich text (Univer Doc)
    si?: string | undefined;        // Formula id
    custom?: Record<string, unknown> | undefined;   // Custom field

    // Extracted formatting information for easier access
    formatting?: {
        // Font properties
        fontFamily?: string;
        fontSize?: number;
        fontColor?: string;
        bold?: boolean;
        italic?: boolean;
        underline?: boolean;
        strikethrough?: boolean;

        // Fill/background
        backgroundColor?: string;

        // Borders
        borderTop?: BorderInfo;
        borderBottom?: BorderInfo;
        borderLeft?: BorderInfo;
        borderRight?: BorderInfo;

        // Alignment
        horizontalAlign?: 'left' | 'center' | 'right' | 'justify';
        verticalAlign?: 'top' | 'middle' | 'bottom';

        // Number formatting
        numberFormat?: string;

        // Protection
        locked?: boolean;
        hidden?: boolean;
    } | undefined;
}

export interface BorderInfo {
    color?: string;
    style?: number;
    width?: number;
}

/**
 * Extraction options for table formatting
 */
export interface ExtractionOptions {
    includeFormulas?: boolean;
    includeFormatting?: boolean;
    includeMetadata?: boolean;
    range?: string;
    maxRows?: number;
    maxCols?: number;
}

/**
 * Formatted table data with comprehensive formatting information
 */
export interface FormattedTable {
    data: CellFormatInfo[][];
    range: string;
    sheetName?: string;
    metadata?: {
        extractedAt: string;
        options: ExtractionOptions;
        totalRows: number;
        totalCols: number;
    };
}

/**
 * Table data transformer for converting between formats
 */
export class TableDataTransformer {
    private univerAPI: ReturnType<typeof FUniver.newAPI>;
    private workbookSnapshot: IWorkbookData | null = null;

    constructor(univerAPI: ReturnType<typeof FUniver.newAPI>) {
        this.univerAPI = univerAPI;
        // Get workbook snapshot for style resolution using official Facade API
        try {
            const workbook = univerAPI.getActiveWorkbook();
            if (workbook) {
                this.workbookSnapshot = workbook.getSnapshot();
            }
        } catch (error) {
            console.warn('Failed to get workbook snapshot for style resolution:', error);
            this.workbookSnapshot = null;
        }
    }

    /**
     * Extract formatted table data from the active sheet
     */
    extractFormattedTable(options: ExtractionOptions = {}): FormattedTable {
        const workbook = this.univerAPI.getActiveWorkbook();
        if (!workbook) {
            throw new Error('No active workbook');
        }

        const sheet = workbook.getActiveSheet();

        const rangeRef = options.range ?? buildDefaultRange(options);
        const range = sheet.getRange(rangeRef);
        const cellDatas = range.getCellDatas();

        // Convert cell data to formatted format
        const formattedData: CellFormatInfo[][] = cellDatas.map((row: Nullable<ICellData>[]) =>
            row.map((cell) => this.convertCellToFormatInfo(cell, options))
        );

        return {
            data: formattedData,
            range: rangeRef,
            sheetName: this.getSheetName(sheet),
            metadata: {
                extractedAt: new Date().toISOString(),
                options,
                totalRows: formattedData.length,
                totalCols: formattedData[0]?.length ?? 0
            }
        };
    }

    /**
     * Get the name of a sheet
     */
    private getSheetName(sheet: FWorksheet): string {
        try {
            // Use official Facade API to get sheet name
            const sheetName = sheet.getSheetName();
            if (typeof sheetName === 'string') {
                return sheetName;
            }

            return 'Unknown Sheet';
        } catch {
            return 'Unknown Sheet';
        }
    }

    /**
     * Convert ICellData to CellFormatInfo
     */
    private convertCellToFormatInfo(cellData: Nullable<ICellData>, options: ExtractionOptions): CellFormatInfo {
        // Handle null/undefined cells by creating an empty cell data object
        const safeCellData: ICellData = cellData ?? {};

        const formatInfo: CellFormatInfo = {
            v: safeCellData.v ?? undefined,
            t: safeCellData.t ?? undefined,
            custom: safeCellData.custom ?? undefined
        };

        // Include formulas if requested
        if (options.includeFormulas && safeCellData.f) {
            formatInfo.f = safeCellData.f;
            formatInfo.si = safeCellData.si ?? undefined;
        }

        // Include metadata if requested
        if (options.includeMetadata && safeCellData.p) {
            formatInfo.p = safeCellData.p;
        }

        // Extract formatting information if requested
        if (options.includeFormatting && safeCellData.s) {
            formatInfo.s = safeCellData.s;
            const extractedFormatting = this.extractFormatting(safeCellData.s);
            if (extractedFormatting) {
                formatInfo.formatting = extractedFormatting;
            }
        }

        return formatInfo;
    }

    /**
     * Extract formatting information from style data
     */
    private extractFormatting(styleData: string | IStyleData): CellFormatInfo['formatting'] | undefined {
        // Simplified formatting extraction - in a real implementation,
        // this would parse the style data comprehensively
        const formatting: CellFormatInfo['formatting'] = {};

        if (typeof styleData === 'object') {
            // Extract basic formatting properties
            if (styleData.ff) {formatting.fontFamily = styleData.ff;}
            if (styleData.fs) {formatting.fontSize = styleData.fs;}
            if (styleData.cl && typeof styleData.cl === 'object' && 'rgb' in styleData.cl && styleData.cl.rgb) {
                formatting.fontColor = styleData.cl.rgb;
            }
            if (styleData.bl) {formatting.bold = true;}
            if (styleData.it) {formatting.italic = true;}
            if (styleData.ul) {formatting.underline = true;}
            if (styleData.st) {formatting.strikethrough = true;}
            if (styleData.bg && typeof styleData.bg === 'object' && 'rgb' in styleData.bg && styleData.bg.rgb) {
                formatting.backgroundColor = styleData.bg.rgb;
            }

            // Number format
            if (styleData.n && typeof styleData.n === 'object' && 'pattern' in styleData.n) {
                formatting.numberFormat = styleData.n.pattern;
            }
        } else if (typeof styleData === 'string') {
            // Resolve style ID to style data using the underlying workbook's style manager
            const resolvedStyleData = this.resolveStyleId(styleData);
            if (resolvedStyleData) {
                // Recursively extract formatting from the resolved style data
                return this.extractFormatting(resolvedStyleData);
            } else {
                console.warn(`Could not resolve style ID: ${styleData}`);
                return undefined;
            }
        }

        // Return undefined if no formatting was extracted
        return Object.keys(formatting).length > 0 ? formatting : undefined;
    }

    /**
     * Resolve a style ID to IStyleData using the workbook snapshot from Facade API
     */
    private resolveStyleId(styleId: string): IStyleData | null {
        try {
            if (!this.workbookSnapshot?.styles) {
                console.warn('Workbook snapshot not available for style resolution');
                return null;
            }

            // Access styles from the workbook snapshot
            const styleData = this.workbookSnapshot.styles[styleId];

            if (!styleData) {
                console.warn(`Style ID '${styleId}' not found in workbook styles`);
                return null;
            }

            return styleData;
        } catch (error) {
            console.warn(`Error resolving style ID '${styleId}':`, error);
            return null;
        }
    }
}

/**
 * Extract formatted table data using Facade API
 */
export function extractFormattedTable(
    univerAPI: ReturnType<typeof FUniver.newAPI>,
    options: ExtractionOptions = {}
): FormattedTable {
    const transformer = new TableDataTransformer(univerAPI);
    return transformer.extractFormattedTable(options);
}

/**
 * Create a table data transformer instance
 */
export function createTableTransformer(univerAPI: ReturnType<typeof FUniver.newAPI>): TableDataTransformer {
    return new TableDataTransformer(univerAPI);
}