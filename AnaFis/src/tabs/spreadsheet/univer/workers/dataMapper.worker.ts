import type { ICellData, IWorkbookData } from '@univerjs/core';
import type {
  CellValue,
  WorkbookData,
} from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import {
  convertToUniverCellValue,
  convertToUniverData,
  convertToUniverDataMultiSheet,
} from '@/tabs/spreadsheet/univer/utils/dataConversion';

export type MapperWorkerResult = IWorkbookData | ICellData[][];

export type MapperWorkerResponse =
  | { id: string; result: MapperWorkerResult; success: true }
  | { id: string; error: string; success: false };

export type MapperWorkerMessage =
  | { type: 'convertToUniverData'; payload: WorkbookData; id: string }
  | { type: 'convertToUniverDataMultiSheet'; payload: WorkbookData; id: string }
  | {
      type: 'convertToUniverCellValueBatch';
      payload: CellValue[][];
      id: string;
    };

self.onmessage = (e: MessageEvent<MapperWorkerMessage>) => {
  const { type, payload, id } = e.data;

  try {
    let result: MapperWorkerResult;
    if (type === 'convertToUniverData') {
      result = convertToUniverData(payload);
    } else if (type === 'convertToUniverDataMultiSheet') {
      result = convertToUniverDataMultiSheet(payload);
    } else if (type === 'convertToUniverCellValueBatch') {
      result = payload.map((row) => row.map(convertToUniverCellValue));
    } else {
      throw new Error(`Unknown message type: ${type}`);
    }

    self.postMessage({ id, result, success: true } as MapperWorkerResponse);
  } catch (error) {
    self.postMessage({
      id,
      error: error instanceof Error ? error.message : String(error),
      success: false,
    } as MapperWorkerResponse);
  }
};
