import {
  CellValueType,
  Disposable,
  type ICellData,
  ICommandService,
  IUniverInstanceService,
  isICellData,
  setDependencies,
  UniverInstanceType,
  type Workbook,
} from '@univerjs/core';
import {
  type ISetRangeValuesCommandParams,
  SetRangeValuesCommand,
} from '@univerjs/sheets';
import { parseUncertaintyInput } from '@/tabs/spreadsheet/univer/plugins/uncertainty/utils/parser';

export class UncertaintyInputController extends Disposable {
  constructor(
    private readonly _commandService: ICommandService,
    private readonly _univerInstanceService: IUniverInstanceService
  ) {
    super();
    this._init();
  }

  private _init(): void {
    // Intercept SetRangeValuesCommand BEFORE execution to mutate the params
    // inline.  Although the CommandListener type declares commandInfo as
    // Readonly<ICommandInfo>, at runtime the same object reference is passed
    // to the command handler — nested mutations to params.value therefore
    // take effect.  This is the only hook that allows injecting uncertainty
    // metadata into the same Undo/Redo mutation as the user's data entry.
    this.disposeWithMe(
      this._commandService.beforeCommandExecuted((command, _options) => {
        if (command.id !== SetRangeValuesCommand.id) {
          return;
        }

        const params = command.params as ISetRangeValuesCommandParams & {
          _skipCustomCleanup?: boolean;
        };
        if (!params?.value) return;

        // Imports, propagation writes, and undo/redo restores carry their own
        // custom metadata alongside the value.  Skip the parser and clearing
        // logic entirely — the incoming data is authoritative.
        if (params._skipCustomCleanup) return;

        const value = params.value;
        const unitId = params.unitId ?? '';
        const subUnitId = params.subUnitId ?? '';

        if (isICellData(value)) {
          this._processCell(
            value,
            this._getExistingNominal(
              unitId,
              subUnitId,
              params.range?.startRow,
              params.range?.startColumn
            )
          );
        } else if (Array.isArray(value)) {
          const startRow = params.range?.startRow ?? 0;
          const startCol = params.range?.startColumn ?? 0;
          for (let r = 0; r < value.length; r++) {
            const row = value[r];
            if (!row) continue;
            for (let c = 0; c < row.length; c++) {
              const cell = row[c];
              if (cell) {
                this._processCell(
                  cell,
                  this._getExistingNominal(
                    unitId,
                    subUnitId,
                    startRow + r,
                    startCol + c
                  )
                );
              }
            }
          }
        } else {
          // Object matrix structure
          for (const rowKey in value) {
            const row = value[rowKey];
            if (!row) continue;
            for (const colKey in row) {
              const cell = row[colKey];
              if (cell) {
                this._processCell(
                  cell,
                  this._getExistingNominal(
                    unitId,
                    subUnitId,
                    parseInt(rowKey, 10),
                    parseInt(colKey, 10)
                  )
                );
              }
            }
          }
        }
      })
    );
  }

  private _processCell(cell: ICellData, existingNominal?: number): void {
    if (!cell || typeof cell !== 'object') return;

    // Skip metadata-only updates (from the propagation controller).
    // User edits always include 'v' or 'f' in the command payload.
    if (!('v' in cell) && !('f' in cell)) return;

    // Formula cells are handled entirely by the propagation controller —
    // both UNCERT formulas and regular formulas with uncertainty propagation.
    // The propagation controller's _handleFormulaCreation clears stale
    // metadata when a formula has no uncertainty to propagate.
    if (cell.f) return;

    // ── Data entry (no formula) ──────────────────────────────────────────
    let isUncertaintyParsed = false;

    // Only try to parse if it's a string input
    if (typeof cell.v === 'string') {
      const parsed = parseUncertaintyInput(cell.v, existingNominal);

      if (parsed) {
        isUncertaintyParsed = true;
        // Mutate the cell directly to split into nominal and metadata
        cell.v = parsed.nominal;
        cell.t = CellValueType.NUMBER;
        cell.p = null; // Clear rich text document to ensure nominal is rendered with numfmt

        // Add uncertainty metadata to the custom property
        cell.custom = {
          ...cell.custom,
          uncertainty: parsed.metadata,
        };
      }
    }

    // If parsing fails (e.g. user typed a plain string / number) or the cell
    // was cleared entirely, ensure we clear any existing uncertainty.
    // (Import/propagation writes are already skipped at the command level
    // via _skipCustomCleanup — this path only runs for genuine user input.)
    if (!isUncertaintyParsed) {
      if (!cell.custom) {
        cell.custom = { uncertainty: null };
      } else {
        cell.custom = { ...cell.custom, uncertainty: null };
      }
    }
  }

  private _getExistingNominal(
    unitId: string,
    subUnitId: string,
    row?: number,
    col?: number
  ): number | undefined {
    if (!unitId || !subUnitId || row === undefined || col === undefined) {
      return undefined;
    }

    const workbook = this._univerInstanceService.getUnit<Workbook>(
      unitId,
      UniverInstanceType.UNIVER_SHEET
    );
    const cell = workbook?.getSheetBySheetId(subUnitId)?.getCellRaw(row, col);
    return typeof cell?.v === 'number' ? cell.v : undefined;
  }
}

setDependencies(UncertaintyInputController, [
  ICommandService,
  IUniverInstanceService,
]);
