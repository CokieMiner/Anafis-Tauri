import { invoke } from '@tauri-apps/api/core';
import {
  Disposable,
  type ICellData,
  type ICommandInfo,
  ICommandService,
  type IUnitRange,
  IUniverInstanceService,
  isICellData,
  type Nullable,
  setDependencies,
  UniverInstanceType,
  type Workbook,
  type Worksheet,
} from '@univerjs/core';
import {
  type IAllRuntimeData,
  IDependencyManagerService,
  IFeatureCalculationManagerService,
  IFormulaCurrentConfigService,
  type IFormulaDirtyData,
} from '@univerjs/engine-formula';
import {
  type IInsertSheetCommandParams,
  InsertSheetCommand,
  type ISetRangeValuesCommandParams,
  SetRangeValuesCommand,
} from '@univerjs/sheets';
import type { UncertaintyMetadata } from '@/tabs/spreadsheet/univer/plugins/uncertainty/types';

interface PropagationVariable {
  name: string;
  value: number;
  uncertainty: number;
}

interface PropagationTask {
  unitId: string;
  subUnitId: string;
  dirtyData: IFormulaDirtyData;
}

export class UncertaintyPropagationController extends Disposable {
  private _dependencyRangesBySheet: Map<string, IUnitRange[]> = new Map();
  private _isPropagating = false;
  private _propagationQueue: PropagationTask[] = [];
  private _isProcessingQueue = false;

  constructor(
    private readonly _featureCalculationManagerService: IFeatureCalculationManagerService,
    private readonly _univerInstanceService: IUniverInstanceService,
    private readonly _commandService: ICommandService,
    private readonly _formulaCurrentConfigService: IFormulaCurrentConfigService,
    private readonly _dependencyManagerService: IDependencyManagerService
  ) {
    super();
    this._init();
  }

  private _init(): void {
    // 1. Initial Registration for existing units
    this._univerInstanceService
      .getAllUnitsForType<Workbook>(UniverInstanceType.UNIVER_SHEET)
      .forEach((workbook) => {
        const unitId = workbook.getUnitId();
        workbook.getSheets().forEach((sheet) => {
          this._registerSheet(unitId, sheet.getSheetId());
        });
      });

    // 2. Listen for new sheets/units
    this.disposeWithMe(
      this._univerInstanceService
        .getTypeOfUnitAdded$(UniverInstanceType.UNIVER_SHEET)
        .subscribe((unitModel) => {
          const workbook = unitModel as unknown as Workbook;
          const unitId = workbook.getUnitId();

          if (workbook) {
            workbook.getSheets().forEach((sheet) => {
              this._registerSheet(unitId, sheet.getSheetId());
            });
          }
        })
    );

    this.disposeWithMe(
      this._commandService.onCommandExecuted((command: ICommandInfo) => {
        if (command.id === InsertSheetCommand.id) {
          const params = command.params as IInsertSheetCommandParams;
          if (params?.unitId && params?.sheet?.id) {
            this._registerSheet(params.unitId, params.sheet.id);
          }
        }
      })
    );

    // 3. Dynamically update dependency ranges AND detect formula cell creation
    this.disposeWithMe(
      this._commandService.onCommandExecuted((command: ICommandInfo) => {
        if (this._isPropagating) return;

        if (command.id === SetRangeValuesCommand.id) {
          const params = command.params as ISetRangeValuesCommandParams;
          if (!params?.value) return;

          const unitId = params.unitId || '';
          const subUnitId = params.subUnitId || '';
          const ranges = this._dependencyRangesBySheet.get(subUnitId);

          if (!ranges) return;

          const value = params.value;
          const formulaCells: Array<{ r: number; c: number; f: string }> = [];

          if (isICellData(value)) {
            if (params.range) {
              this._updateDependency(
                unitId,
                subUnitId,
                params.range.startRow,
                params.range.startColumn,
                value,
                ranges
              );
              if (value.f) {
                formulaCells.push({
                  r: params.range.startRow,
                  c: params.range.startColumn,
                  f: value.f,
                });
              }
            }
          } else if (Array.isArray(value)) {
            const startRow = params.range?.startRow ?? 0;
            const startCol = params.range?.startColumn ?? 0;
            for (let r = 0; r < value.length; r++) {
              const row = value[r];
              if (!row) continue;
              for (let c = 0; c < row.length; c++) {
                const cell = row[c];
                if (cell) {
                  this._updateDependency(
                    unitId,
                    subUnitId,
                    startRow + r,
                    startCol + c,
                    cell,
                    ranges
                  );
                  if (cell.f) {
                    formulaCells.push({
                      r: startRow + r,
                      c: startCol + c,
                      f: cell.f,
                    });
                  }
                }
              }
            }
          } else {
            for (const rowStr in value) {
              const r = parseInt(rowStr, 10);
              const row = value[rowStr];
              if (!row) continue;
              for (const colStr in row) {
                const c = parseInt(colStr, 10);
                const cell = row[colStr];
                if (cell) {
                  this._updateDependency(unitId, subUnitId, r, c, cell, ranges);
                  if (cell.f) {
                    formulaCells.push({ r, c, f: cell.f });
                  }
                }
              }
            }
          }

          // Trigger propagation for newly created/modified formula cells.
          // getDirtyData only fires when SOURCE cells change, not when a new
          // formula referencing them is created — so we handle that here.
          if (formulaCells.length > 0) {
            void this._handleFormulaCreation(unitId, subUnitId, formulaCells);
          }
        }
      })
    );
  }

  private _updateDependency(
    unitId: string,
    sheetId: string,
    r: number,
    c: number,
    cell: ICellData,
    ranges: IUnitRange[]
  ) {
    const custom = cell?.custom as Record<string, unknown> | undefined;
    const hasUncertainty = !!custom?.uncertainty;

    const existingIdx = ranges.findIndex(
      (item) => item.range.startRow === r && item.range.startColumn === c
    );

    if (hasUncertainty && existingIdx === -1) {
      ranges.push({
        unitId,
        sheetId,
        range: {
          startRow: r,
          endRow: r,
          startColumn: c,
          endColumn: c,
        },
      });
    } else if (!hasUncertainty && existingIdx !== -1) {
      ranges.splice(existingIdx, 1);
    }
  }

  private _registerSheet(unitId: string, subUnitId: string) {
    if (this._dependencyRangesBySheet.has(subUnitId)) return;

    const dependencyRanges: IUnitRange[] = [];
    this._dependencyRangesBySheet.set(subUnitId, dependencyRanges);

    // Initial scan for uncertainty
    const workbook = this._univerInstanceService.getUnit<Workbook>(
      unitId,
      UniverInstanceType.UNIVER_SHEET
    );
    if (workbook) {
      const worksheet = workbook.getSheetBySheetId(subUnitId);
      if (worksheet) {
        worksheet
          .getCellMatrix()
          .forValue((r: number, c: number, cell: Nullable<ICellData>) => {
            if (cell?.custom?.uncertainty) {
              dependencyRanges.push({
                unitId,
                sheetId: subUnitId,
                range: {
                  startRow: r,
                  endRow: r,
                  startColumn: c,
                  endColumn: c,
                },
              });
            }
            return true;
          });
      }
    }

    this._featureCalculationManagerService.register(
      unitId,
      subUnitId,
      'uncertainty-propagation',
      {
        unitId,
        subUnitId,
        dependencyRanges,
        getDirtyData: (
          dirtyData: IFormulaDirtyData,
          _runtimeData: IAllRuntimeData
        ) => {
          this._enqueuePropagation(unitId, subUnitId, dirtyData);

          return {
            runtimeCellData: {},
            dirtyRanges: {},
          };
        },
      }
    );
  }

  private _enqueuePropagation(
    unitId: string,
    subUnitId: string,
    dirtyData: IFormulaDirtyData
  ) {
    // DEDUPLICATION: If a task for the same sheet is already in the queue,
    // it will be replaced because the newer task will process the most recent sheet state anyway.
    const existingIdx = this._propagationQueue.findIndex(
      (task) => task.unitId === unitId && task.subUnitId === subUnitId
    );

    if (existingIdx !== -1) {
      this._propagationQueue[existingIdx] = { unitId, subUnitId, dirtyData };
    } else {
      this._propagationQueue.push({ unitId, subUnitId, dirtyData });
    }

    void this._processQueue();
  }

  private async _processQueue() {
    if (this._isProcessingQueue) return;
    this._isProcessingQueue = true;

    try {
      while (this._propagationQueue.length > 0) {
        const task = this._propagationQueue.shift();
        if (task) {
          await this._schedulePropagation(
            task.unitId,
            task.subUnitId,
            task.dirtyData
          );
        }
      }
    } finally {
      this._isProcessingQueue = false;
    }
  }

  private async _schedulePropagation(
    unitId: string,
    subUnitId: string,
    dirtyData: IFormulaDirtyData
  ) {
    const workbook = this._univerInstanceService.getUnit<Workbook>(
      unitId,
      UniverInstanceType.UNIVER_SHEET
    );
    if (!workbook) return;

    const worksheet = workbook.getSheetBySheetId(subUnitId);
    if (!worksheet) return;

    const formulaData = this._formulaCurrentConfigService.getFormulaData();
    const sheetFormulaData = formulaData[unitId]?.[subUnitId];
    if (!sheetFormulaData) return;

    const mutations: Record<string, Record<string, ICellData>> = {};

    // 1. Identify affected formulas using DependencyManagerService
    const affectedTreeIds = this._dependencyManagerService.searchDependency(
      dirtyData.dirtyRanges
    );
    const affectedFormulas: Array<{ r: number; c: number; f: string }> = [];

    if (dirtyData.forceCalculation) {
      // If force, iterate all formulas in sheet
      for (const rStr in sheetFormulaData) {
        const r = parseInt(rStr, 10);
        const row = sheetFormulaData[r];
        if (!row) continue;
        for (const cStr in row) {
          const c = parseInt(cStr, 10);
          const formulaItem = row[c];
          if (formulaItem) {
            affectedFormulas.push({ r, c, f: formulaItem.f });
          }
        }
      }
    } else {
      for (const treeId of affectedTreeIds) {
        const tree = this._dependencyManagerService.getTreeById(treeId);
        if (
          tree &&
          tree.unitId === unitId &&
          tree.subUnitId === subUnitId &&
          !tree.isVirtual
        ) {
          const formulaItem = sheetFormulaData[tree.row]?.[tree.column];
          if (formulaItem) {
            affectedFormulas.push({
              r: tree.row,
              c: tree.column,
              f: formulaItem.f,
            });
          }
        }
      }
    }

    if (affectedFormulas.length === 0) return;

    // Yield to the microtask queue so the formula engine's web worker
    // can flush its results into the cell matrix before we read.
    await new Promise<void>((resolve) => setTimeout(resolve, 50));

    // 2. Process formulas
    for (const { r, c, f } of affectedFormulas) {
      // Re-read cell to ensure we have the latest evaluated nominal value.
      const freshWorksheet = workbook.getSheetBySheetId(subUnitId);
      const freshMatrix = freshWorksheet?.getCellMatrix();
      const cell = freshMatrix?.getValue(r, c);

      const custom = cell?.custom as Record<string, unknown> | undefined;
      const metadata = custom?.uncertainty as UncertaintyMetadata | undefined;
      const formulaClean = f.replace(/^=/, '').trim();

      // ── Handle =UNCERT(expression, uncertainty) ──────────────────────
      const uncertMeta = this._parseUncertFormula(formulaClean, worksheet);
      if (uncertMeta) {
        // If it's an UNCERT formula, we update the metadata from the formula string
        // (which might reference a cell like B2 that just changed).
        if (!mutations[r]) mutations[r] = {};
        mutations[r][c] = {
          v: cell?.v,
          f,
          custom: {
            ...custom,
            uncertainty: uncertMeta,
          },
        };
        continue;
      }

      // Skip manual overrides
      if (metadata?.upperSource === 'manual') continue;

      // Extract only used variables for this specific formula
      const variables = this._getVariablesForFormula(unitId, subUnitId, r, c);
      if (variables.length === 0) {
        if (metadata?.upperSource === 'propagated') {
          if (!mutations[r]) mutations[r] = {};
          mutations[r][c] = {
            v: cell?.v,
            f,
            custom: { ...custom, uncertainty: null },
          };
        }
        continue;
      }

      try {
        const result = (await invoke('calculate_uncertainty', {
          formula: formulaClean,
          variables,
        })) as { value: number; uncertainty: number };

        if (!mutations[r]) mutations[r] = {};
        mutations[r][c] = {
          v: cell?.v,
          f,
          custom: {
            ...custom,
            uncertainty: {
              upperBound: result.uncertainty,
              upperType: 'abs',
              upperSource: 'propagated',
            } as UncertaintyMetadata,
          },
        };
      } catch (error) {
        console.warn(
          `[Uncertainty] Calculation failed for ${this._numberToABC(c)}${r + 1} (${f}):`,
          error
        );
      }
    }

    if (Object.keys(mutations).length > 0) {
      this._isPropagating = true;
      try {
        const params: ISetRangeValuesCommandParams = {
          unitId,
          subUnitId,
          value: mutations,
        };
        await this._commandService.executeCommand(
          SetRangeValuesCommand.id,
          params
        );
      } finally {
        this._isPropagating = false;
      }
    }
  }

  /**
   * Extracts ONLY the variables used in a specific formula cell.
   */
  private _getVariablesForFormula(
    unitId: string,
    subUnitId: string,
    row: number,
    col: number
  ): PropagationVariable[] {
    const workbook = this._univerInstanceService.getUnit<Workbook>(
      unitId,
      UniverInstanceType.UNIVER_SHEET
    );
    if (!workbook) return [];

    const treeId = this._dependencyManagerService.getFormulaDependency(
      unitId,
      subUnitId,
      row,
      col
    );
    if (treeId == null) return [];

    const tree = this._dependencyManagerService.getTreeById(treeId);
    if (!tree) return [];

    const variables: PropagationVariable[] = [];
    const seen = new Set<string>();

    for (const range of tree.rangeList) {
      const depWorkbook = this._univerInstanceService.getUnit<Workbook>(
        range.unitId,
        UniverInstanceType.UNIVER_SHEET
      );
      if (!depWorkbook) continue;

      const worksheet = depWorkbook.getSheetBySheetId(range.sheetId);
      if (!worksheet) continue;

      const cellMatrix = worksheet.getCellMatrix();
      for (let r = range.range.startRow; r <= range.range.endRow; r++) {
        for (let c = range.range.startColumn; c <= range.range.endColumn; c++) {
          const cell = cellMatrix.getValue(r, c);
          if (!cell) continue;

          const varName = `${this._numberToABC(c)}${r + 1}`;
          if (seen.has(varName)) continue;

          const custom = cell?.custom as Record<string, unknown> | undefined;
          if (custom?.uncertainty) {
            const u = custom.uncertainty as UncertaintyMetadata;
            const value = Number(cell?.v) || 0;
            variables.push({
              name: varName,
              value,
              uncertainty: this._toAbsoluteBound(
                u.upperBound || 0,
                u.upperType,
                value
              ),
            });
            seen.add(varName);
          }
        }
      }
    }

    return variables;
  }

  /**
   * Handle formula cells that were just created or modified.
   * Extracts cell references directly from the formula string (bypassing
   * the dependency tree which may not exist yet for new formulas), checks
   * if referenced cells have uncertainty, and triggers propagation.
   */
  private async _handleFormulaCreation(
    unitId: string,
    subUnitId: string,
    formulaCells: Array<{ r: number; c: number; f: string }>
  ) {
    const workbook = this._univerInstanceService.getUnit<Workbook>(
      unitId,
      UniverInstanceType.UNIVER_SHEET
    );
    if (!workbook) return;

    const worksheet = workbook.getSheetBySheetId(subUnitId);
    if (!worksheet) return;

    const cellMatrix = worksheet.getCellMatrix();
    const mutations: Record<string, Record<string, ICellData>> = {};

    for (const { r, c, f } of formulaCells) {
      const cell = cellMatrix.getValue(r, c);
      const custom = cell?.custom as Record<string, unknown> | undefined;
      const metadata = custom?.uncertainty as UncertaintyMetadata | undefined;

      // Skip cells with manual uncertainty — the user explicitly set it
      if (metadata?.upperSource === 'manual') continue;

      const formulaClean = f.replace(/^=/, '').trim();

      // ── Handle =UNCERT(expression, uncertainty) ──────────────────────
      const uncertMeta = this._parseUncertFormula(formulaClean, worksheet);
      if (uncertMeta) {
        // Yield to the formula engine so custom metadata is up-to-date.
        await new Promise<void>((resolve) => setTimeout(resolve, 50));

        // Re-read cell to get the latest custom metadata.
        const freshWorksheet = workbook.getSheetBySheetId(subUnitId);
        const freshMatrix = freshWorksheet?.getCellMatrix();
        const currentCell = freshMatrix?.getValue(r, c);

        if (!mutations[r]) mutations[r] = {};
        mutations[r][c] = {
          v: currentCell?.v,
          f,
          custom: {
            ...(currentCell?.custom as Record<string, unknown> | undefined),
            uncertainty: uncertMeta,
          },
        };
        continue;
      }

      // ── Regular formula: propagate uncertainty via Rust CAS ──────────
      const refs = this._extractCellRefsFromFormula(formulaClean);
      if (refs.length === 0) {
        // No cell refs → can't propagate. Clear any stale manual UNCERT metadata.
        if (metadata) {
          if (!mutations[r]) mutations[r] = {};
          mutations[r][c] = {
            v: cell?.v,
            f,
            custom: { ...custom, uncertainty: null },
          };
        }
        continue;
      }

      const variables: PropagationVariable[] = [];
      const seen = new Set<string>();

      for (const ref of refs) {
        const refCell = cellMatrix.getValue(ref.row, ref.col);
        if (!refCell) continue;

        const varName = `${this._numberToABC(ref.col)}${ref.row + 1}`;
        if (seen.has(varName)) continue;

        const refCustom = refCell.custom as Record<string, unknown> | undefined;
        if (refCustom?.uncertainty) {
          const u = refCustom.uncertainty as UncertaintyMetadata;
          const value = Number(refCell.v) || 0;
          variables.push({
            name: varName,
            value,
            uncertainty: this._toAbsoluteBound(
              u.upperBound || 0,
              u.upperType,
              value
            ),
          });
          seen.add(varName);
        }
      }

      if (variables.length === 0) {
        // Refs exist but none have uncertainty → clear any stale metadata.
        if (metadata) {
          if (!mutations[r]) mutations[r] = {};
          mutations[r][c] = {
            v: cell?.v,
            f,
            custom: { ...custom, uncertainty: null },
          };
        }
        continue;
      }

      try {
        const result = (await invoke('calculate_uncertainty', {
          formula: formulaClean,
          variables,
        })) as { value: number; uncertainty: number };

        // Yield to the microtask queue so the formula engine's web worker
        // can flush its results into the cell matrix before we read.
        await new Promise<void>((resolve) => setTimeout(resolve, 50));

        // Re-read cell to get the latest custom metadata.
        const freshWorksheet = workbook.getSheetBySheetId(subUnitId);
        const freshMatrix = freshWorksheet?.getCellMatrix();
        const currentCell = freshMatrix?.getValue(r, c);

        if (!mutations[r]) mutations[r] = {};
        mutations[r][c] = {
          v: currentCell?.v,
          f,
          custom: {
            ...(currentCell?.custom as Record<string, unknown> | undefined),
            uncertainty: {
              upperBound: result.uncertainty,
              upperType: 'abs',
              upperSource: 'propagated',
            } as UncertaintyMetadata,
          },
        };
      } catch (error) {
        console.warn(
          `[Uncertainty] Propagation failed for ${this._numberToABC(c)}${r + 1} (${f}):`,
          error
        );
      }
    }

    if (Object.keys(mutations).length > 0) {
      this._isPropagating = true;
      try {
        await this._commandService.executeCommand(SetRangeValuesCommand.id, {
          unitId,
          subUnitId,
          value: mutations,
        } as ISetRangeValuesCommandParams);
      } finally {
        this._isPropagating = false;
      }
    }
  }

  /**
   * Extract cell references (e.g. A1, B2, AA10) from a formula string.
   * Handles simple references only — range expressions like A1:A10 will
   * yield the two endpoints but not the cells in between.
   */
  private _extractCellRefsFromFormula(
    formula: string
  ): Array<{ col: number; row: number }> {
    const refs: Array<{ col: number; row: number }> = [];
    const regex = /([A-Z]+)(\d+)/gi;
    let match = regex.exec(formula);
    while (match !== null) {
      const colStr = (match[1] as string).toUpperCase();
      const rowNum = parseInt(match[2] as string, 10);
      if (!Number.isNaN(rowNum) && rowNum > 0) {
        refs.push({ col: this._ABCToNumber(colStr), row: rowNum - 1 });
      }
      match = regex.exec(formula);
    }
    return refs;
  }

  private _ABCToNumber(abc: string): number {
    let result = 0;
    for (let i = 0; i < abc.length; i++) {
      result = result * 26 + (abc.charCodeAt(i) - 64);
    }
    return result - 1;
  }

  /**
   * Parse an UNCERT(expression, uncertainty) formula string.
   * Returns uncertainty metadata if successful, null otherwise.
   */
  private _parseUncertFormula(
    formula: string,
    worksheet: Worksheet
  ): UncertaintyMetadata | null {
    const match = formula.match(/^UNCERT\s*\(/i);
    if (!match) return null;

    // Strip "UNCERT(" prefix and trailing ")"
    const inner = formula.slice(match[0].length, -1).trim();

    const splitIdx = this._findTopLevelComma(inner);
    if (splitIdx === -1) return null;

    const uncPart = inner.slice(splitIdx + 1).trim();

    // Check if uncPart is a percentage literal
    const isPercent = uncPart.endsWith('%');
    const uncStr = isPercent ? uncPart.slice(0, -1).trim() : uncPart;

    // Resolve all cell references in the uncertainty part
    const refs = this._extractCellRefsFromFormula(uncStr);
    let resolvedExpr = uncStr;

    // Sort refs by length descending to avoid partial replacements (e.g., A10 before A1)
    const sortedRefs = [...refs].sort((a, b) => {
      const aStr = `${this._numberToABC(a.col)}${a.row + 1}`;
      const bStr = `${this._numberToABC(b.col)}${b.row + 1}`;
      return bStr.length - aStr.length;
    });

    for (const ref of sortedRefs) {
      const refStr = `${this._numberToABC(ref.col)}${ref.row + 1}`;
      const refCell = worksheet.getCell(ref.row, ref.col);
      const val =
        typeof refCell?.v === 'number'
          ? refCell.v
          : parseFloat(String(refCell?.v)) || 0;
      resolvedExpr = resolvedExpr.replace(
        new RegExp(refStr, 'gi'),
        String(val)
      );
    }

    // Now evaluate the expression (only simple arithmetic allowed for safety)
    let uncValue: number;
    try {
      // Remove all cell references already resolved to numbers
      // safeExpr should only contain numbers and operators now
      const safeExpr = resolvedExpr.replace(/[^0-9.\s+\-*/()]/g, '');

      // Simple recursive-descent or just use a basic eval-like logic
      // For simplicity and safety, we can use a basic regex-based evaluator
      // or just keep the Function constructor but with even stricter cleaning.
      // Given the environment, a simple Function constructor on a strictly sanitized
      // string is the most practical way to handle nested parentheses.
      uncValue = Number(new Function(`"use strict"; return (${safeExpr})`)());
    } catch {
      uncValue = NaN;
    }

    if (Number.isNaN(uncValue) || typeof uncValue !== 'number') return null;

    return {
      upperBound: uncValue,
      upperType: isPercent ? 'rel' : 'abs',
      upperSource: 'manual',
    };
  }

  /**
   * Find the index of the first comma at parenthesis depth 0.
   * Returns -1 if not found.
   */
  private _findTopLevelComma(s: string): number {
    let depth = 0;
    for (let i = 0; i < s.length; i++) {
      const ch = s[i];
      if (ch === '(') depth++;
      else if (ch === ')') depth--;
      else if (ch === ',' && depth === 0) return i;
    }
    return -1;
  }

  private _numberToABC(col: number): string {
    let abc = '';
    let c = col;
    while (c >= 0) {
      abc = String.fromCharCode((c % 26) + 65) + abc;
      c = Math.floor(c / 26) - 1;
    }
    return abc;
  }

  private _toAbsoluteBound(
    bound: number,
    type: UncertaintyMetadata['upperType'],
    nominal: number
  ): number {
    return type === 'rel' ? Math.abs(nominal) * bound * 0.01 : bound;
  }
}

setDependencies(UncertaintyPropagationController, [
  IFeatureCalculationManagerService,
  IUniverInstanceService,
  ICommandService,
  IFormulaCurrentConfigService,
  IDependencyManagerService,
]);
