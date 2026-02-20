// CSV parsing utility for direct file import

import type {
  CsvImportSettings,
  ImportedColumn,
  ImportedData,
} from '../types/fittingTypes';

function detectSeparator(line: string): ',' | ';' | '\t' {
  const counts = {
    '\t': line.match(/\t/g)?.length ?? 0,
    ';': line.match(/;/g)?.length ?? 0,
    ',': line.match(/,/g)?.length ?? 0,
  };

  if (counts['\t'] >= counts[';'] && counts['\t'] >= counts[',']) {
    return '\t';
  }
  if (counts[';'] >= counts[',']) {
    return ';';
  }
  return ',';
}

function parseNumericField(
  raw: string,
  decimalFormat: '.' | ',',
  rowNumber: number,
  columnNumber: number
): number {
  const trimmed = raw.trim();
  if (trimmed.length === 0) {
    throw new Error(
      `Missing value at row ${rowNumber}, column ${columnNumber}`
    );
  }

  const normalized =
    decimalFormat === ',' ? trimmed.replace(',', '.') : trimmed;
  const value = Number(normalized);

  if (!Number.isFinite(value)) {
    throw new Error(
      `Invalid numeric value '${raw}' at row ${rowNumber}, column ${columnNumber}`
    );
  }

  return value;
}

export function parseCsvText(
  text: string,
  settings: CsvImportSettings,
  sourceName: string = 'CSV'
): ImportedData {
  const lines = text.split(/\r?\n/).filter((line) => line.trim().length > 0);

  if (lines.length <= settings.skipRows) {
    return { columns: [], sourceName, rowCount: 0 };
  }

  const dataLines = lines.slice(settings.skipRows);
  if (dataLines.length === 0) {
    return { columns: [], sourceName, rowCount: 0 };
  }

  const separator =
    settings.separator === 'auto'
      ? detectSeparator(dataLines[0] ?? '')
      : settings.separator;

  let headers: string[];
  let startIdx: number;

  if (settings.hasHeader) {
    headers = (dataLines[0] ?? '')
      .split(separator)
      .map((header) => header.trim());
    startIdx = 1;
  } else {
    const colCount = (dataLines[0] ?? '').split(separator).length;
    headers = Array.from({ length: colCount }, (_, idx) => `Column ${idx + 1}`);
    startIdx = 0;
  }

  const columnCount = headers.length;
  const columnData: number[][] = Array.from({ length: columnCount }, () => []);

  for (let rowIdx = startIdx; rowIdx < dataLines.length; rowIdx++) {
    const fields = (dataLines[rowIdx] ?? '').split(separator);

    if (fields.length !== columnCount) {
      throw new Error(
        `Row ${rowIdx + 1} has ${fields.length} columns but expected ${columnCount}`
      );
    }

    for (let colIdx = 0; colIdx < columnCount; colIdx++) {
      const raw = fields[colIdx] ?? '';
      const value = parseNumericField(
        raw,
        settings.decimalFormat,
        rowIdx + 1,
        colIdx + 1
      );
      columnData[colIdx]?.push(value);
    }
  }

  const columns: ImportedColumn[] = headers.map((name, idx) => ({
    name,
    data: columnData[idx] ?? [],
  }));

  const rowCount = dataLines.length - startIdx;
  return { columns, sourceName, rowCount };
}
