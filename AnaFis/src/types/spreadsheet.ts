// AnaFis/src/types/spreadsheet.ts

export type CellType = 'text' | 'number' | 'uncertainty' | 'boolean';

export type Uncertainty = {
  value: number | null;
  uncertainty: number | null;
};

export type TextCell = {
  type: 'text';
  value: string | null;
};

export type NumberCell = {
  type: 'number';
  value: number | null;
};

export type UncertaintyCell = {
  type: 'uncertainty';
  value: Uncertainty | null;
};

export type BooleanCell = {
  type: 'boolean';
  value: boolean | null;
};

export type Cell = TextCell | NumberCell | UncertaintyCell | BooleanCell;

export type SpreadsheetRow = Cell[];

export type SpreadsheetData = SpreadsheetRow[];
