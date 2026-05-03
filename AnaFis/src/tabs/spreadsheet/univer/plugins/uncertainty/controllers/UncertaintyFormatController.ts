import {
  type CellValue,
  Disposable,
  type ICellData,
  type ICommandInfo,
  ICommandService,
  IUniverInstanceService,
  isICellData,
  type Nullable,
  setDependencies,
  UniverInstanceType,
  type Workbook,
} from '@univerjs/core';
import {
  type ISetRangeValuesCommandParams,
  SetRangeValuesCommand,
} from '@univerjs/sheets';
import {
  type ISetNumfmtCommandParams,
  SetNumfmtCommand,
} from '@univerjs/sheets-numfmt';
import type { UncertaintyMetadata } from '@/tabs/spreadsheet/univer/plugins/uncertainty/types';

interface INumfmtValue {
  row: number;
  col: number;
  pattern: string;
}

export class UncertaintyFormatController extends Disposable {
  private _isApplyingFormat = false;

  constructor(
    private readonly _commandService: ICommandService,
    private readonly _univerInstanceService: IUniverInstanceService
  ) {
    super();
    this._init();
  }

  private _init(): void {
    // Listen for SetRangeValuesCommand completion to apply formatting
    this.disposeWithMe(
      this._commandService.onCommandExecuted((command: ICommandInfo) => {
        if (this._isApplyingFormat) return;

        if (command.id === SetRangeValuesCommand.id) {
          const params = command.params as ISetRangeValuesCommandParams;
          if (!params?.value) return;

          const numfmtValues: INumfmtValue[] = [];
          const value = params.value;

          if (isICellData(value)) {
            if (params.range) {
              this._processCell(
                value,
                params.range.startRow,
                params.range.startColumn,
                params.unitId || '',
                params.subUnitId || '',
                numfmtValues
              );
            }
          } else if (Array.isArray(value)) {
            const startRow = params.range?.startRow ?? 0;
            const startCol = params.range?.startColumn ?? 0;
            for (let r = 0; r < value.length; r++) {
              const row = value[r];
              if (!row) continue;
              for (let c = 0; c < row.length; c++) {
                const cell = row[c];
                if (cell)
                  this._processCell(
                    cell,
                    startRow + r,
                    startCol + c,
                    params.unitId || '',
                    params.subUnitId || '',
                    numfmtValues
                  );
              }
            }
          } else {
            // Object matrix
            for (const rowKey in value) {
              const row = value[rowKey];
              if (!row) continue;
              for (const colKey in row) {
                const cell = row[colKey];
                if (cell) {
                  this._processCell(
                    cell,
                    parseInt(rowKey, 10),
                    parseInt(colKey, 10),
                    params.unitId || '',
                    params.subUnitId || '',
                    numfmtValues
                  );
                }
              }
            }
          }

          if (numfmtValues.length > 0) {
            const numfmtParams: ISetNumfmtCommandParams = {
              unitId: params.unitId || '',
              subUnitId: params.subUnitId || '',
              values: numfmtValues,
            };

            this._isApplyingFormat = true;
            try {
              this._commandService.executeCommand(
                SetNumfmtCommand.id,
                numfmtParams
              );
            } finally {
              this._isApplyingFormat = false;
            }
          }
        }
      })
    );
  }

  private _processCell(
    cell: ICellData,
    row: number,
    col: number,
    unitId: string,
    subUnitId: string,
    results: INumfmtValue[]
  ): void {
    if (cell?.custom && 'uncertainty' in cell.custom) {
      const uncertainty = cell.custom.uncertainty as UncertaintyMetadata | null;
      if (uncertainty) {
        // Use cell.v from command params if available, otherwise read from worksheet
        let nominal: Nullable<CellValue> = cell.v;
        if (typeof nominal !== 'number') {
          const workbook = this._univerInstanceService.getUnit<Workbook>(
            unitId,
            UniverInstanceType.UNIVER_SHEET
          );
          const wsCell = workbook
            ?.getSheetBySheetId(subUnitId)
            ?.getCellMatrix()
            ?.getValue(row, col);
          nominal = wsCell?.v;
        }
        const formatString = this._generateFormatString(nominal, uncertainty);
        results.push({
          row,
          col,
          pattern: formatString,
        });
      } else {
        // Clear the format if uncertainty is explicitly set to null
        results.push({
          row,
          col,
          pattern: '',
        });
      }
    }
  }

  private _generateFormatString(
    nominal: Nullable<CellValue>,
    uncertainty: UncertaintyMetadata
  ): string {
    if (typeof nominal !== 'number') return '0.00 "± 0.00"';

    const upperAbs = this._toAbsoluteBound(
      uncertainty.upperBound,
      uncertainty.upperType,
      nominal
    );
    const lowerAbs =
      uncertainty.lowerBound !== undefined
        ? this._toAbsoluteBound(
            uncertainty.lowerBound,
            uncertainty.lowerType ?? uncertainty.upperType,
            nominal
          )
        : undefined;
    const roundingBound =
      lowerAbs === undefined ? upperAbs : Math.max(upperAbs, lowerAbs);

    // GUM precision matching: determine the number of decimal places to display
    // based on the uncertainty's magnitude. If the leading significant digit
    // is 1 or 2, show one extra decimal (2 sig figs of uncertainty); otherwise
    // show just enough decimals for 1 sig fig. The nominal value is displayed
    // with the same number of decimal places to match precision.
    // NOTE: We do NOT round the uncertainty value itself — toFixed handles display.
    let decimals = 2;
    if (roundingBound > 0) {
      const order = Math.floor(Math.log10(roundingBound));
      const leadingDigit = roundingBound / 10 ** order; // value in [1, 10)
      const sigFigs = leadingDigit < 2.95 ? 2 : 1;
      decimals = Math.max(0, sigFigs - 1 - order);
    }

    const zeroStr = decimals > 0 ? `0.${'0'.repeat(decimals)}` : '0';

    if (uncertainty.lowerBound !== undefined) {
      return `${zeroStr} "+${upperAbs.toFixed(decimals)}/-${(lowerAbs ?? 0).toFixed(decimals)}"`;
    }

    const formattedErr = upperAbs.toFixed(decimals);
    return `${zeroStr} "± ${formattedErr}"`;
  }

  private _toAbsoluteBound(
    bound: number,
    type: UncertaintyMetadata['upperType'],
    nominal: number
  ): number {
    return type === 'rel' ? Math.abs(nominal) * bound * 0.01 : bound;
  }
}

setDependencies(UncertaintyFormatController, [
  ICommandService,
  IUniverInstanceService,
]);
