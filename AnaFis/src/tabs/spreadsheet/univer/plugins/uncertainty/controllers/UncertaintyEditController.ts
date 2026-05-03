import { Disposable, setDependencies } from '@univerjs/core';
import { BEFORE_CELL_EDIT, SheetInterceptorService } from '@univerjs/sheets';
import type { UncertaintyMetadata } from '@/tabs/spreadsheet/univer/plugins/uncertainty/types';

export class UncertaintyEditController extends Disposable {
  constructor(
    private readonly _sheetInterceptorService: SheetInterceptorService
  ) {
    super();
    this._init();
  }

  private _init(): void {
    // Intercept edit initialization to reconstruct the string
    this.disposeWithMe(
      this._sheetInterceptorService.writeCellInterceptor.intercept(
        BEFORE_CELL_EDIT,
        {
          priority: 1000, // High priority to run before formula editor
          handler: (cell, _context, next) => {
            if (cell?.custom?.uncertainty) {
              const u = cell.custom.uncertainty as UncertaintyMetadata;
              const nominal = Number(cell.v) || 0;
              let editString = `${cell.v}`;

              // Prevent appending the uncertainty multiple times if the interceptor is called more than once
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

              // Return directly WITHOUT calling next() — this is intentional.
              // The reconstructed uncertainty string (e.g. "5 ± 0.1") must be
              // presented as-is in the editor. Calling next() would pass it to
              // downstream interceptors (formula editor, rich text) which could
              // misinterpret the ± delimiters or re-parse the string.
              return {
                ...cell,
                v: editString,
              };
            }

            return next(cell);
          },
        }
      )
    );
  }

  private _toAbsoluteBound(
    bound: number,
    type: UncertaintyMetadata['upperType'],
    nominal: number
  ): number {
    return type === 'rel' ? Math.abs(nominal) * bound * 0.01 : bound;
  }
}

setDependencies(UncertaintyEditController, [SheetInterceptorService]);
