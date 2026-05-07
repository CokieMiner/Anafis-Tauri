import { Disposable, setDependencies } from '@univerjs/core';
import {
  AFTER_CELL_EDIT,
  BEFORE_CELL_EDIT,
  SheetInterceptorService,
} from '@univerjs/sheets';
import type { UncertaintyMetadata } from '@/tabs/spreadsheet/univer/plugins/uncertainty/types';

export class UncertaintyEditController extends Disposable {
  constructor(
    private readonly _sheetInterceptorService: SheetInterceptorService
  ) {
    super();
    this._init();
  }

  private _init(): void {
    this.disposeWithMe(
      this._sheetInterceptorService.writeCellInterceptor.intercept(
        BEFORE_CELL_EDIT,
        {
          priority: 1000,
          handler: (cell, _context, next) => {
            if (cell?.custom?.uncertainty) {
              const u = cell.custom.uncertainty as UncertaintyMetadata;

              if (cell.f && /^=UNCERT\s*\(/i.test(cell.f)) {
                return { ...cell, v: cell.f };
              }

              const nominal = Number(cell.v) || 0;
              let editString = `${cell.v}`;

              const isAlreadyFormatted =
                typeof cell.v === 'string' &&
                (cell.v.includes('±') ||
                  cell.v.includes('+-') ||
                  cell.v.includes('/-') ||
                  cell.v.includes('/+'));

              if (!isAlreadyFormatted) {
                const upperAbs = this._toAbsoluteBound(
                  u.upperBound,
                  u.upperType,
                  nominal
                );

                if (u.lowerBound !== undefined) {
                  const lowerAbs = this._toAbsoluteBound(
                    u.lowerBound,
                    u.lowerType ?? u.upperType,
                    nominal
                  );
                  editString += ` +${upperAbs}/-${lowerAbs}`;
                } else {
                  editString += ` ± ${upperAbs}`;
                }
              }

              // Strip `custom` so the InputController treats the commit as
              // fresh user input rather than an import write.  Known
              // limitation: editing "5 ± 0.4" to just "5" keeps the
              // uncertainty — Univer skips SetRangeValuesCommand when the
              // nominal value does not change.
              const { custom: _, ...rest } = cell;
              return { ...rest, v: editString };
            }

            return next(cell);
          },
        }
      )
    );

    // Add AFTER_CELL_EDIT interceptor to handle cleared uncertainty
    this.disposeWithMe(
      this._sheetInterceptorService.writeCellInterceptor.intercept(
        AFTER_CELL_EDIT,
        {
          priority: 1000,
          handler: (cell, context, next) => {
            // Check if the cell has uncertainty that was manually cleared in the editor
            const rawEditorCellData = context.rawEditorCellData;
            const origin = context.origin;

            if (origin?.custom?.uncertainty && !rawEditorCellData?.custom) {
              const { custom: _, ...rest } = cell || {};
              return next({
                ...rest,
                custom: {
                  ...(cell?.custom || {}),
                  uncertainty: null,
                },
              });
            }

            return next(cell);
          },
        }
      )
    );
  }

  private _toAbsoluteBound(
    bound: number,
    type: 'abs' | 'rel',
    nominal: number
  ): number {
    return type === 'rel' ? Math.abs(nominal) * bound * 0.01 : bound;
  }
}

setDependencies(UncertaintyEditController, [SheetInterceptorService]);
