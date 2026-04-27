import type { ICellData, IWorkbookData } from '@univerjs/core';
import type {
  CellValue,
  WorkbookData,
} from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import type {
  MapperWorkerMessage,
  MapperWorkerResponse,
  MapperWorkerResult,
} from '@/tabs/spreadsheet/univer/workers/dataMapper.worker';
import DataMapperWorker from '@/tabs/spreadsheet/univer/workers/dataMapper.worker?worker';

class DataMapperClient {
  private worker: Worker;
  private pendingResolvers: Map<
    string,
    {
      resolve: (val: MapperWorkerResult) => void;
      reject: (err: Error) => void;
    }
  >;
  private msgIdCounter = 0;

  constructor() {
    this.worker = new DataMapperWorker();
    this.pendingResolvers = new Map();

    this.worker.onmessage = (e: MessageEvent<MapperWorkerResponse>) => {
      const data = e.data;
      const id = data.id;
      const resolver = this.pendingResolvers.get(id);
      if (resolver) {
        this.pendingResolvers.delete(id);
        if (data.success) {
          resolver.resolve(data.result);
        } else {
          resolver.reject(new Error(data.error));
        }
      }
    };
  }

  private dispatch<T>(
    type: MapperWorkerMessage['type'],
    payload: unknown
  ): Promise<T> {
    const id = `msg_${this.msgIdCounter++}`;
    return new Promise((resolve, reject) => {
      this.pendingResolvers.set(id, {
        resolve: (result: MapperWorkerResult) => {
          resolve(result as T);
        },
        reject,
      });
      this.worker.postMessage({ type, payload, id });
    });
  }

  public convertToUniverData(data: WorkbookData): Promise<IWorkbookData> {
    return this.dispatch<IWorkbookData>('convertToUniverData', data);
  }

  public convertToUniverDataMultiSheet(
    data: WorkbookData
  ): Promise<IWorkbookData> {
    return this.dispatch<IWorkbookData>('convertToUniverDataMultiSheet', data);
  }

  public convertToUniverCellValueBatch(
    data: CellValue[][]
  ): Promise<ICellData[][]> {
    return this.dispatch<ICellData[][]>('convertToUniverCellValueBatch', data);
  }

  public terminate() {
    this.worker.terminate();
  }
}

export const GlobalDataMapper = new DataMapperClient();
