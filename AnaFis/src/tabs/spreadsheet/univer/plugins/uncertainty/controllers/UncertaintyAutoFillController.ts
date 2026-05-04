import {
  Direction,
  Disposable,
  type ICellData,
  IUniverInstanceService,
  type Nullable,
  setDependencies,
  UniverInstanceType,
  type Workbook,
  type Worksheet,
} from '@univerjs/core';
import {
  AUTO_FILL_APPLY_TYPE,
  type IAutoFillLocation,
  IAutoFillService,
  type ISheetAutoFillHook,
  SetRangeValuesMutation,
} from '@univerjs/sheets';
import type { UncertaintyMetadata } from '@/tabs/spreadsheet/univer/plugins/uncertainty/types';

export class UncertaintyAutoFillController extends Disposable {
  constructor(
    private readonly _autoFillService: IAutoFillService,
    private readonly _univerInstanceService: IUniverInstanceService
  ) {
    super();
    this._initAutoFill();
  }

  private _initAutoFill(): void {
    const hook: ISheetAutoFillHook = {
      id: 'uncertainty-plugin',
      onFillData: (
        location: IAutoFillLocation,
        direction: Direction,
        applyType: AUTO_FILL_APPLY_TYPE
      ) => {
        if (applyType === AUTO_FILL_APPLY_TYPE.ONLY_FORMAT) {
          return { undos: [], redos: [] };
        }

        const { source, target, unitId, subUnitId } = location;
        const sourceRows = source.rows;
        const sourceCols = source.cols;
        const targetRows = target.rows;
        const targetCols = target.cols;

        if (
          sourceRows.length === 0 ||
          sourceCols.length === 0 ||
          targetRows.length === 0 ||
          targetCols.length === 0
        ) {
          return { undos: [], redos: [] };
        }

        const workbook = this._univerInstanceService.getUnit<Workbook>(
          unitId,
          UniverInstanceType.UNIVER_SHEET
        );
        if (!workbook) return { undos: [], redos: [] };

        const worksheet = workbook.getSheetBySheetId(subUnitId);
        if (!worksheet) return { undos: [], redos: [] };

        const mutations: Record<string, Record<string, ICellData>> = {};

        if (direction === Direction.DOWN || direction === Direction.UP) {
          this._fillVertical(
            worksheet,
            sourceRows,
            sourceCols,
            targetRows,
            targetCols,
            mutations
          );
        } else {
          this._fillHorizontal(
            worksheet,
            sourceRows,
            sourceCols,
            targetRows,
            targetCols,
            mutations
          );
        }

        if (Object.keys(mutations).length === 0) {
          return { undos: [], redos: [] };
        }

        return {
          undos: [],
          redos: [
            {
              id: SetRangeValuesMutation.id,
              params: { unitId, subUnitId, cellValue: mutations },
            },
          ],
        };
      },
    };

    if (this._autoFillService) {
      this.disposeWithMe(this._autoFillService.addHook(hook));
    }
  }

  private _fillVertical(
    worksheet: Worksheet,
    sourceRows: number[],
    sourceCols: number[],
    targetRows: number[],
    targetCols: number[],
    mutations: Record<string, Record<string, ICellData>>
  ): void {
    const cellMatrix = worksheet.getCellMatrix();

    for (let ci = 0; ci < sourceCols.length; ci++) {
      const sourceCol = sourceCols[ci];
      if (sourceCol === undefined) continue;
      const targetCol = ci < targetCols.length ? targetCols[ci] : sourceCol;
      if (targetCol === undefined) continue;

      for (let ti = 0; ti < targetRows.length; ti++) {
        const targetRow = targetRows[ti];
        if (targetRow === undefined) continue;

        const si = ti % sourceRows.length;
        const sourceRow = sourceRows[si];
        if (sourceRow === undefined) continue;

        const srcCell = cellMatrix.getValue(sourceRow, sourceCol);
        this._copyUncertainty(mutations, srcCell, targetRow, targetCol);
      }
    }
  }

  private _fillHorizontal(
    worksheet: Worksheet,
    sourceRows: number[],
    sourceCols: number[],
    targetRows: number[],
    targetCols: number[],
    mutations: Record<string, Record<string, ICellData>>
  ): void {
    const cellMatrix = worksheet.getCellMatrix();

    for (let ri = 0; ri < sourceRows.length; ri++) {
      const sourceRow = sourceRows[ri];
      if (sourceRow === undefined) continue;
      const targetRow = ri < targetRows.length ? targetRows[ri] : sourceRow;
      if (targetRow === undefined) continue;

      for (let ti = 0; ti < targetCols.length; ti++) {
        const targetCol = targetCols[ti];
        if (targetCol === undefined) continue;

        const si = ti % sourceCols.length;
        const sourceCol = sourceCols[si];
        if (sourceCol === undefined) continue;

        const srcCell = cellMatrix.getValue(sourceRow, sourceCol);
        this._copyUncertainty(mutations, srcCell, targetRow, targetCol);
      }
    }
  }

  private _copyUncertainty(
    mutations: Record<string, Record<string, ICellData>>,
    srcCell: Nullable<ICellData>,
    targetRow: number,
    targetCol: number
  ): void {
    if (!srcCell) return;

    if (srcCell.f || srcCell.si) return;

    const custom = srcCell.custom as Record<string, unknown> | undefined;
    const uncertainty = custom?.uncertainty as UncertaintyMetadata | undefined;
    if (!uncertainty) return;

    if (!mutations[targetRow]) {
      mutations[targetRow] = {};
    }
    mutations[targetRow][targetCol] = {
      custom: { uncertainty },
    };
  }
}

setDependencies(UncertaintyAutoFillController, [
  IAutoFillService,
  IUniverInstanceService,
]);
