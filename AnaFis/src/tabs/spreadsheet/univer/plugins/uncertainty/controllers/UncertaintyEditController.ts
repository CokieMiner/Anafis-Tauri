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

              // Strip `custom` so the editor UI doesn't try to parse or display
              // raw metadata objects. The final cleanup of uncertainty metadata
              // when a user clears a cell is handled by the AFTER_CELL_EDIT
              // interceptor, which forces Univer to detect the change.
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
            const origin = context.origin;

            // We aggressively strip the uncertainty metadata
            // during AFTER_CELL_EDIT. If we don't, Univer's `diffValue` check
            // will silently abort the edit when a user overwrites `5 ± 0.1` with `5`,
            // because the merged cell data and the original cell data will be
            // identical. By forcing a difference here, we guarantee that 
            // SetRangeValuesCommand is dispatched, allowing UncertaintyInputController
            // to correctly parse and reconstruct the final metadata.
            if (origin?.custom?.uncertainty) {
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
