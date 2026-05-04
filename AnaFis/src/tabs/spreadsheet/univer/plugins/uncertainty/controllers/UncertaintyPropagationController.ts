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
  type ISetRangeValuesMutationParams,
  SetRangeValuesCommand,
  SetRangeValuesMutation,
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

const KNOWN_FUNCTION_NAMES = new Set([
  'ABS',
  'ACCRINT',
  'ACCRINTM',
  'ACOS',
  'ACOSH',
  'ACOT',
  'ACOTH',
  'ACSC',
  'ACSCH',
  'ADDRESS',
  'AGGREGATE',
  'AMORDEGRC',
  'AMORLINC',
  'AND',
  'ARABIC',
  'AREAS',
  'ARRAYTOTEXT',
  'ASC',
  'ASEC',
  'ASECH',
  'ASIN',
  'ASINH',
  'ASSOC_LEGENDRE',
  'ATAN',
  'ATAN2',
  'ATANH',
  'AVEDEV',
  'AVERAGE',
  'AVERAGEA',
  'AVERAGEIF',
  'AVERAGEIFS',
  'BAHTTEXT',
  'BASE',
  'BESSELI',
  'BESSELJ',
  'BESSELK',
  'BESSELY',
  'BETA',
  'BETADIST',
  'BETAINV',
  'BIN2DEC',
  'BIN2HEX',
  'BIN2OCT',
  'BINOMDIST',
  'BITAND',
  'BITLSHIFT',
  'BITOR',
  'BITRSHIFT',
  'BITXOR',
  'BYCOL',
  'BYROW',
  'CBRT',
  'CEILING',
  'CELL',
  'CHAR',
  'CHIDIST',
  'CHIINV',
  'CHITEST',
  'CHOOSE',
  'CHOOSECOLS',
  'CHOOSEROWS',
  'CLEAN',
  'CODE',
  'COLUMN',
  'COLUMNS',
  'COMBIN',
  'COMBINA',
  'COMPLEX',
  'CONCAT',
  'CONCATENATE',
  'CONFIDENCE',
  'CONVERT',
  'CORREL',
  'COS',
  'COSH',
  'COT',
  'COTH',
  'COUNT',
  'COUNTA',
  'COUNTBLANK',
  'COUNTIF',
  'COUNTIFS',
  'COUPDAYBS',
  'COUPDAYS',
  'COUPDAYSNC',
  'COUPNCD',
  'COUPNUM',
  'COUPPCD',
  'COVAR',
  'CRITBINOM',
  'CSC',
  'CSCH',
  'CUBEKPIMEMBER',
  'CUBEMEMBER',
  'CUBEMEMBERPROPERTY',
  'CUBERANKEDMEMBER',
  'CUBESET',
  'CUBESETCOUNT',
  'CUBEVALUE',
  'CUMIPMT',
  'CUMPRINC',
  'DATE',
  'DATEDIF',
  'DATEVALUE',
  'DAVERAGE',
  'DAY',
  'DAYS',
  'DAYS360',
  'DB',
  'DBCS',
  'DCOUNT',
  'DCOUNTA',
  'DDB',
  'DIGAMMA',
  'DEC2BIN',
  'DEC2HEX',
  'DEC2OCT',
  'DECIMAL',
  'DEGREES',
  'DELTA',
  'DEVSQ',
  'DGET',
  'DISC',
  'DMAX',
  'DMIN',
  'DOLLAR',
  'DOLLARDE',
  'DOLLARFR',
  'DPRODUCT',
  'DROP',
  'DSTDEV',
  'DSTDEVP',
  'DSUM',
  'DURATION',
  'DVAR',
  'DVARP',
  'EDATE',
  'EFFECT',
  'ENCODEURL',
  'EOMONTH',
  'ELLIPTIC_E',
  'ELLIPTIC_K',
  'EPOCHTODATE',
  'ERF',
  'ERFC',
  'EUROCONVERT',
  'EVEN',
  'EXACT',
  'EXP',
  'EXPAND',
  'EXPONDIST',
  'FACT',
  'FACTDOUBLE',
  'FALSE',
  'FDIST',
  'FILTER',
  'FILTERXML',
  'FIND',
  'FINDB',
  'FINV',
  'FISHER',
  'FISHERINV',
  'FIXED',
  'FLOOR',
  'FORECAST',
  'FORMULATEXT',
  'FREQUENCY',
  'FTEST',
  'FV',
  'FVSCHEDULE',
  'GAMMA',
  'GAMMADIST',
  'GAMMAINV',
  'GAMMALN',
  'GAUSS',
  'GCD',
  'GEOMEAN',
  'GESTEP',
  'GETPIVOTDATA',
  'GROWTH',
  'HARMEAN',
  'HEX2BIN',
  'HEX2DEC',
  'HEX2OCT',
  'HERMITE',
  'HLOOKUP',
  'HOUR',
  'HSTACK',
  'HYPERLINK',
  'HYPGEOMDIST',
  'IF',
  'IFERROR',
  'IFNA',
  'IFS',
  'IMABS',
  'IMAGE',
  'IMAGINARY',
  'IMARGUMENT',
  'IMCONJUGATE',
  'IMCOS',
  'IMCOSH',
  'IMCOT',
  'IMCOTH',
  'IMCSC',
  'IMCSCH',
  'IMDIV',
  'IMEXP',
  'IMLN',
  'IMLOG',
  'IMLOG10',
  'IMLOG2',
  'IMPOWER',
  'IMPRODUCT',
  'IMREAL',
  'IMSEC',
  'IMSECH',
  'IMSIN',
  'IMSINH',
  'IMSQRT',
  'IMSUB',
  'IMSUM',
  'IMTAN',
  'IMTANH',
  'INDEX',
  'INDIRECT',
  'INFO',
  'INT',
  'INTERCEPT',
  'INTRATE',
  'IPMT',
  'IRR',
  'ISBETWEEN',
  'ISBLANK',
  'ISDATE',
  'ISEMAIL',
  'ISERR',
  'ISERROR',
  'ISEVEN',
  'ISFORMULA',
  'ISLOGICAL',
  'ISNA',
  'ISNONTEXT',
  'ISNUMBER',
  'ISO',
  'ISODD',
  'ISOMITTED',
  'ISOWEEKNUM',
  'ISPMT',
  'ISREF',
  'ISTEXT',
  'ISURL',
  'KURT',
  'LAMBDA',
  'LAMBERTW',
  'LARGE',
  'LCM',
  'LEFT',
  'LEFTB',
  'LEN',
  'LENB',
  'LET',
  'LINEST',
  'LN',
  'LOG',
  'LOG10',
  'LOGEST',
  'LOGINV',
  'LOGNORMDIST',
  'LOOKUP',
  'LOWER',
  'MAKEARRAY',
  'MAP',
  'MARGINOFERROR',
  'MATCH',
  'MAX',
  'MAXA',
  'MAXIFS',
  'MDETERM',
  'MDURATION',
  'MEDIAN',
  'MID',
  'MIDB',
  'MIN',
  'MINA',
  'MINIFS',
  'MINUTE',
  'MINVERSE',
  'MIRR',
  'MMULT',
  'MOD',
  'MODE',
  'MONTH',
  'MROUND',
  'MULTINOMIAL',
  'MUNIT',
  'N',
  'NA',
  'NEGBINOMDIST',
  'NETWORKDAYS',
  'NOMINAL',
  'NORMDIST',
  'NORMINV',
  'NORMSDIST',
  'NORMSINV',
  'NOT',
  'NOW',
  'NPER',
  'NPV',
  'NUMBERSTRING',
  'NUMBERVALUE',
  'OCT2BIN',
  'OCT2DEC',
  'OCT2HEX',
  'ODD',
  'ODDFPRICE',
  'ODDFYIELD',
  'ODDLPRICE',
  'ODDLYIELD',
  'OFFSET',
  'OR',
  'PDURATION',
  'PEARSON',
  'PERCENTILE',
  'PERCENTRANK',
  'PERMUT',
  'PERMUTATIONA',
  'PHI',
  'PHONETIC',
  'PI',
  'POLYGAMMA',
  'PMT',
  'POISSON',
  'POWER',
  'PPMT',
  'PRICE',
  'PRICEDISC',
  'PRICEMAT',
  'PROB',
  'PRODUCT',
  'PROPER',
  'PV',
  'QUARTILE',
  'QUOTIENT',
  'RADIANS',
  'RAND',
  'RANDARRAY',
  'RANDBETWEEN',
  'RANK',
  'RATE',
  'RECEIVED',
  'REDUCE',
  'REGEXEXTRACT',
  'REGEXMATCH',
  'REGEXREPLACE',
  'REGISTER',
  'REPLACE',
  'REPLACEB',
  'REPT',
  'RIGHT',
  'RIGHTB',
  'ROMAN',
  'ROUND',
  'ROUNDBANK',
  'ROUNDDOWN',
  'ROUNDUP',
  'ROW',
  'ROWS',
  'RRI',
  'RSQ',
  'RTD',
  'SCAN',
  'SEARCH',
  'SEARCHB',
  'SEC',
  'SECH',
  'SECOND',
  'SEQUENCE',
  'SERIESSUM',
  'SHEET',
  'SHEETS',
  'SIGN',
  'SINC',
  'SIN',
  'SINH',
  'SKEW',
  'SLN',
  'SLOPE',
  'SMALL',
  'SORT',
  'SORTBY',
  'SQRT',
  'SQRTPI',
  'SPHERICAL_HARMONIC',
  'STANDARDIZE',
  'STDEV',
  'STDEVA',
  'STDEVP',
  'STDEVPA',
  'STEYX',
  'SUBSTITUTE',
  'SUBTOTAL',
  'SUM',
  'SUMIF',
  'SUMIFS',
  'SUMPRODUCT',
  'SUMSQ',
  'SUMX2MY2',
  'SUMX2PY2',
  'SUMXMY2',
  'SWITCH',
  'SYD',
  'T',
  'TAKE',
  'TAN',
  'TANH',
  'TBILLEQ',
  'TBILLPRICE',
  'TBILLYIELD',
  'TDIST',
  'TEXT',
  'TEXTAFTER',
  'TEXTBEFORE',
  'TEXTJOIN',
  'TEXTSPLIT',
  'TETRAGAMMA',
  'TIME',
  'TIMEVALUE',
  'TINV',
  'TODAY',
  'TOCOL',
  'TOROW',
  'TRANSPOSE',
  'TREND',
  'TRIM',
  'TRIGAMMA',
  'TRIMMEAN',
  'TRUE',
  'TRUNC',
  'TTEST',
  'TYPE',
  'UNCERT',
  'UNICHAR',
  'UNICODE',
  'UNIQUE',
  'UPPER',
  'VALUE',
  'VALUETOTEXT',
  'VAR',
  'VARA',
  'VARP',
  'VARPA',
  'VDB',
  'VLOOKUP',
  'VSTACK',
  'WEBSERVICE',
  'WEEKDAY',
  'WEEKNUM',
  'WEIBULL',
  'WORKDAY',
  'WRAPCOLS',
  'WRAPROWS',
  'XIRR',
  'XLOOKUP',
  'XMATCH',
  'XNPV',
  'XOR',
  'YEAR',
  'YEARFRAC',
  'YIELD',
  'YIELDDISC',
  'YIELDMAT',
  'ZTEST',
  'ZETA',
  'ZETA_DERIV',
]);

const MAX_FORMULA_EVALUATION_WAIT_MS = 3000;
const FORMULA_POLL_INTERVAL_MS = 15;

export class UncertaintyPropagationController extends Disposable {
  private _dependencyRangesBySheet: Map<string, IUnitRange[]> = new Map();
  private _isPropagating = false;
  private _propagationQueue: PropagationTask[] = [];
  private _isProcessingQueue = false;
  private _activeProcessingSheets = new Set<string>();

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
          const staleMetadataCells: Array<{ r: number; c: number }> = [];

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
              } else if (
                value.custom &&
                (value.custom as Record<string, unknown>).uncertainty &&
                (
                  (value.custom as Record<string, unknown>)
                    .uncertainty as UncertaintyMetadata
                ).upperSource === 'propagated'
              ) {
                // Bug 3: Formula was replaced with a non-formula value but
                // propagated uncertainty metadata hasn't been cleared yet.
                staleMetadataCells.push({
                  r: params.range.startRow,
                  c: params.range.startColumn,
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
                  } else if (
                    cell.custom &&
                    (cell.custom as Record<string, unknown>).uncertainty &&
                    (
                      (cell.custom as Record<string, unknown>)
                        .uncertainty as UncertaintyMetadata
                    ).upperSource === 'propagated'
                  ) {
                    staleMetadataCells.push({
                      r: startRow + r,
                      c: startCol + c,
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
                  } else if (
                    cell.custom &&
                    (cell.custom as Record<string, unknown>).uncertainty &&
                    (
                      (cell.custom as Record<string, unknown>)
                        .uncertainty as UncertaintyMetadata
                    ).upperSource === 'propagated'
                  ) {
                    staleMetadataCells.push({ r, c });
                  }
                }
              }
            }
          }

          // Bug 3: Clean up stale propagated uncertainty metadata on cells
          // that no longer have a formula (e.g. formula replaced by plain value).
          if (staleMetadataCells.length > 0) {
            void (async () => {
              this._isPropagating = true;
              try {
                const cleanupMutations: Record<
                  string,
                  Record<string, ICellData>
                > = {};
                for (const { r, c } of staleMetadataCells) {
                  if (!cleanupMutations[r]) cleanupMutations[r] = {};
                  cleanupMutations[r][c] = {
                    custom: { uncertainty: null },
                  };
                }
                await this._commandService.executeCommand(
                  SetRangeValuesCommand.id,
                  {
                    unitId,
                    subUnitId,
                    value: cleanupMutations,
                  } as ISetRangeValuesCommandParams
                );
              } finally {
                this._isPropagating = false;
              }
            })();
          }

          // Trigger propagation for newly created/modified formula cells.
          // getDirtyData only fires when SOURCE cells change, not when a new
          // formula referencing them is created — so we handle that here.
          if (formulaCells.length > 0) {
            void this._handleFormulaCreation(unitId, subUnitId, formulaCells);
          }

          // Enqueue propagation for affected formulas on this sheet so that
          // UNCERT metadata is re-parsed when the uncertainty arguments' cells
          // change.  The formula engine tracks dependencies on ALL args, but
          // our feature's getDirtyData only fires for cells in dependencyRanges.
          // UNCERT's uncertainty args may reference plain-number cells not in
          // those ranges, yet we must re-parse the UNCERT metadata when they
          // change.  Enqueuing here bridges that gap.
          const dirtyRanges = this._buildDirtyRanges(
            value,
            params.range,
            unitId,
            subUnitId
          );
          if (dirtyRanges.length > 0) {
            this._enqueuePropagation(unitId, subUnitId, {
              forceCalculation: false,
              dirtyRanges,
              dirtyNameMap: {},
              dirtyDefinedNameMap: {},
              dirtyUnitFeatureMap: {},
              dirtyUnitOtherFormulaMap: {},
              clearDependencyTreeCache: {},
            });
          }
        }
      })
    );

    // 4. Auto-fill / formula engine result apply — these dispatch
    // SetRangeValuesMutation directly (bypassing SetRangeValuesCommand),
    // so our command listener never fires.  We catch formula cells from
    // the mutation and trigger propagation + dependency updates here.
    this.disposeWithMe(
      this._commandService.onCommandExecuted((command: ICommandInfo) => {
        if (this._isPropagating) return;

        if (command.id === SetRangeValuesMutation.id) {
          const params = command.params as ISetRangeValuesMutationParams;
          const cellValue = params.cellValue;
          if (!cellValue) return;

          const unitId = params.unitId || '';
          const subUnitId = params.subUnitId || '';
          const ranges = this._dependencyRangesBySheet.get(subUnitId);
          if (!ranges) return;

          const formulaCells: Array<{ r: number; c: number; f: string }> = [];
          const dirtyRanges: IUnitRange[] = [];

          for (const rStr in cellValue) {
            const r = parseInt(rStr, 10);
            if (Number.isNaN(r)) continue;
            const row = cellValue[rStr];
            if (!row) continue;
            for (const cStr in row) {
              const c = parseInt(cStr, 10);
              if (Number.isNaN(c)) continue;
              const cell = row[cStr];
              if (!cell) continue;

              // Keep dependency ranges in sync when cells are modified
              // via mutations (auto-fill, formula engine results, etc.)
              this._updateDependency(unitId, subUnitId, r, c, cell, ranges);

              // Formula engine writes results (v only) through mutations —
              // those have no f, so they're naturally skipped here.
              if (cell.f) {
                formulaCells.push({ r, c, f: cell.f });
              }

              dirtyRanges.push({
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
          }

          if (formulaCells.length > 0) {
            void this._handleFormulaCreation(unitId, subUnitId, formulaCells);
          }

          if (dirtyRanges.length > 0) {
            this._enqueuePropagation(unitId, subUnitId, {
              forceCalculation: false,
              dirtyRanges,
              dirtyNameMap: {},
              dirtyDefinedNameMap: {},
              dirtyUnitFeatureMap: {},
              dirtyUnitOtherFormulaMap: {},
              clearDependencyTreeCache: {},
            });
          }
        }
      })
    );
  }

  /** Build IUnitRange[] from a SetRangeValuesCommand value shape. */
  private _buildDirtyRanges(
    value: ISetRangeValuesCommandParams['value'],
    baseRange: ISetRangeValuesCommandParams['range'],
    unitId: string,
    subUnitId: string
  ): IUnitRange[] {
    if (isICellData(value)) {
      return baseRange
        ? [{ unitId, sheetId: subUnitId, range: { ...baseRange } }]
        : [];
    }

    if (Array.isArray(value) && baseRange) {
      const maxR = baseRange.startRow + Math.max(0, value.length - 1);
      const maxC =
        baseRange.startColumn +
        Math.max(0, ...value.filter(Boolean).map((r) => r.length)) -
        1;
      return [
        {
          unitId,
          sheetId: subUnitId,
          range: {
            startRow: baseRange.startRow,
            endRow: maxR,
            startColumn: baseRange.startColumn,
            endColumn: Math.max(baseRange.startColumn, maxC),
          },
        },
      ];
    }

    // Object matrix — compute bounding box from keys
    let minR = Number.POSITIVE_INFINITY;
    let maxR = Number.NEGATIVE_INFINITY;
    let minC = Number.POSITIVE_INFINITY;
    let maxC = Number.NEGATIVE_INFINITY;

    for (const rStr in value) {
      const r = parseInt(rStr, 10);
      if (Number.isNaN(r)) continue;
      minR = Math.min(minR, r);
      maxR = Math.max(maxR, r);
      const row = value[rStr];
      if (!row) continue;
      for (const cStr in row) {
        const c = parseInt(cStr, 10);
        if (Number.isNaN(c)) continue;
        minC = Math.min(minC, c);
        maxC = Math.max(maxC, c);
      }
    }

    if (Number.isFinite(minR)) {
      return [
        {
          unitId,
          sheetId: subUnitId,
          range: {
            startRow: minR,
            endRow: maxR,
            startColumn: minC,
            endColumn: maxC,
          },
        },
      ];
    }

    return [];
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
    // DEDUPLICATION: If a task for the same sheet is already queued OR currently
    // being processed, replace it — the newer task carries more up-to-date state.
    const sheetKey = `${unitId}|${subUnitId}`;

    if (this._activeProcessingSheets.has(sheetKey)) {
      // A task for this sheet is being processed right now; push a replacement
      // that will be picked up on the next loop iteration.
      this._propagationQueue.push({ unitId, subUnitId, dirtyData });
      return;
    }

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
          const sheetKey = `${task.unitId}|${task.subUnitId}`;
          this._activeProcessingSheets.add(sheetKey);
          try {
            await this._schedulePropagation(
              task.unitId,
              task.subUnitId,
              task.dirtyData
            );
          } finally {
            this._activeProcessingSheets.delete(sheetKey);
          }
        }
      }
    } finally {
      this._isProcessingQueue = false;
    }
  }

  /**
   * Poll the cell matrix until the formula engine has written the nominal
   * value for the given cell (v becomes a number), or until the timeout.
   * Returns the cell value or null on timeout.
   */
  private async _waitForCellValue(
    worksheet: Worksheet,
    row: number,
    col: number
  ): Promise<Nullable<ICellData>> {
    const started = Date.now();
    let cell: Nullable<ICellData> = null;
    while (Date.now() - started < MAX_FORMULA_EVALUATION_WAIT_MS) {
      cell = worksheet.getCellMatrix().getValue(row, col);
      if (typeof cell?.v === 'number') return cell;
      await new Promise<void>((r) => setTimeout(r, FORMULA_POLL_INTERVAL_MS));
    }
    return cell;
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

    // 2. Process formulas
    for (const { r, c, f } of affectedFormulas) {
      const formulaClean = f.replace(/^=/, '').trim();

      // ── Handle =UNCERT(expression, uncertainty) ──────────────────────
      const uncertMeta = this._parseUncertFormula(formulaClean, worksheet);
      if (uncertMeta) {
        // Wait for the formula engine to write the nominal value before
        // applying metadata and triggering the format controller.
        const cell = await this._waitForCellValue(worksheet, r, c);
        const custom = cell?.custom as Record<string, unknown> | undefined;

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

      // Read cell to check for manual overrides
      const cell = worksheet.getCellMatrix().getValue(r, c);
      const custom = cell?.custom as Record<string, unknown> | undefined;
      const metadata = custom?.uncertainty as UncertaintyMetadata | undefined;

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
        // Substitute cell references: cells WITH uncertainty get safe
        // variable names (__v0__, __v1__, ...); cells WITHOUT uncertainty
        // get substituted with their numeric value so symb_anafis never
        // sees raw column letters that could be misinterpreted (E12 → e12
        // as scientific notation, E → e as Euler's number).
        let subFormula = formulaClean;
        const allRefs = this._extractCellRefsFromFormula(formulaClean);

        for (let vi = 0; vi < variables.length; vi++) {
          const v = variables[vi] as PropagationVariable;
          const safe = `__v${vi}__`;
          subFormula = subFormula.replace(new RegExp(v.name, 'gi'), safe);
          (variables[vi] as { name: string }).name = safe;
        }

        for (const ref of allRefs) {
          const cellRef = `${this._numberToABC(ref.col)}${ref.row + 1}`;
          const refCell = worksheet.getCellMatrix().getValue(ref.row, ref.col);
          const val = typeof refCell?.v === 'number' ? refCell.v : 0;
          subFormula = subFormula.replace(
            new RegExp(cellRef, 'gi'),
            String(val)
          );
        }

        const result = (await invoke('calculate_uncertainty', {
          formula: subFormula,
          variables,
        })) as { value: number; uncertainty: number };

        // Use the nominal value computed by the Rust CAS directly, rather
        // than waiting for the formula engine's Web Worker to flush results
        // into the cell matrix.  This avoids the unreliable setTimeout(50)
        // race and guarantees we always have the correct nominal.
        if (!mutations[r]) mutations[r] = {};
        mutations[r][c] = {
          v: result.value,
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
          _skipCustomCleanup: true,
        } as ISetRangeValuesCommandParams;
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
    if (this._isPropagating) return;
    this._isPropagating = true;

    try {
      const workbook = this._univerInstanceService.getUnit<Workbook>(
        unitId,
        UniverInstanceType.UNIVER_SHEET
      );
      if (!workbook) return;

      const worksheet = workbook.getSheetBySheetId(subUnitId);
      if (!worksheet) return;

      const mutations: Record<string, Record<string, ICellData>> = {};

      for (const { r, c, f } of formulaCells) {
        const formulaClean = f.replace(/^=/, '').trim();

        // ── Handle =UNCERT(expression, uncertainty) ──────────────────────
        // Process UNCERT formulas BEFORE the manual-source guard because the
        // formula string itself is the authority for the uncertainty value.
        // Skipping would cause stale metadata to persist when the user edits
        // an UNCERT formula or drag-fills it to adjacent cells.
        const isUncertFormula = /^UNCERT\s*\(/i.test(formulaClean);
        if (isUncertFormula) {
          const cell = await this._waitForCellValue(worksheet, r, c);
          const freshWorksheet = workbook.getSheetBySheetId(subUnitId);
          const uncertMeta = freshWorksheet
            ? this._parseUncertFormula(formulaClean, freshWorksheet)
            : this._parseUncertFormula(formulaClean, worksheet);

          if (uncertMeta) {
            if (!mutations[r]) mutations[r] = {};
            mutations[r][c] = {
              v: cell?.v,
              f,
              custom: {
                ...(cell?.custom as Record<string, unknown> | undefined),
                uncertainty: uncertMeta,
              },
            };
          }
          continue;
        }

        const cell = await this._waitForCellValue(worksheet, r, c);
        const custom = cell?.custom as Record<string, unknown> | undefined;
        const metadata = custom?.uncertainty as UncertaintyMetadata | undefined;

        // Skip non-UNCERT cells with manual uncertainty — the user explicitly set it
        if (metadata?.upperSource === 'manual') continue;

        // ── Regular formula: propagate uncertainty via Rust CAS ──────────
        const refs = this._extractCellRefsFromFormula(formulaClean);
        if (refs.length === 0) {
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
        const varSubs: Map<string, string> = new Map();
        let varIdx = 0;

        for (const ref of refs) {
          const refCell = worksheet.getCellMatrix().getValue(ref.row, ref.col);
          const cellRef = `${this._numberToABC(ref.col)}${ref.row + 1}`;
          if (seen.has(cellRef)) continue;
          seen.add(cellRef);

          const value = typeof refCell?.v === 'number' ? refCell.v : 0;
          const refCustom = refCell?.custom as
            | Record<string, unknown>
            | undefined;

          if (refCustom?.uncertainty) {
            const u = refCustom.uncertainty as UncertaintyMetadata;
            const safeName = `__v${varIdx}__`;
            varSubs.set(cellRef, safeName);
            variables.push({
              name: safeName,
              value,
              uncertainty: this._toAbsoluteBound(
                u.upperBound || 0,
                u.upperType,
                value
              ),
            });
            varIdx++;
          } else {
            // Cell has no uncertainty — substitute with its numeric value.
            varSubs.set(cellRef, String(value));
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
          // Substitute cell references with safe variable names so
          // symb_anafis doesn't misinterpret column letters (e.g. E12
          // lowercased to e12 is parsed as scientific notation).
          let subFormula = formulaClean;
          for (const [ref, safe] of varSubs) {
            subFormula = subFormula.replace(new RegExp(ref, 'gi'), safe);
          }
          const result = (await invoke('calculate_uncertainty', {
            formula: subFormula,
            variables,
          })) as { value: number; uncertainty: number };

          // Use the nominal value computed by the Rust CAS directly.
          // This avoids waiting for the formula engine's Web Worker to flush
          // results and removes the unreliable setTimeout(50) race.
          if (!mutations[r]) mutations[r] = {};
          mutations[r][c] = {
            v: result.value,
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
            `[Uncertainty] Propagation failed for ${this._numberToABC(c)}${r + 1} (${f}):`,
            error
          );
        }
      }

      if (Object.keys(mutations).length > 0) {
        await this._commandService.executeCommand(SetRangeValuesCommand.id, {
          unitId,
          subUnitId,
          value: mutations,
          _skipCustomCleanup: true,
        } as ISetRangeValuesCommandParams);
      }
    } finally {
      this._isPropagating = false;
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
      const charAfter = formula[regex.lastIndex] ?? '';

      // Skip if followed by '(' — it's a function call (e.g. T(A1),
      // N(A1), SIN(0.5)), not a cell reference.
      if (charAfter === '(') {
        match = regex.exec(formula);
        continue;
      }

      // Skip matches embedded in longer alphanumeric identifiers
      // (e.g. SUMX2PY2 would yield phantom SUMX row 2 + PY row 2).
      const charBefore = formula[match.index - 1] ?? '';
      if (/[A-Z]/i.test(charAfter) || /[A-Z0-9]/i.test(charBefore)) {
        match = regex.exec(formula);
        continue;
      }

      // Skip known function names that the regex would misinterpret
      // as cell references (SIN, COS, LOG, etc.).
      if (KNOWN_FUNCTION_NAMES.has(colStr)) {
        match = regex.exec(formula);
        continue;
      }

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
