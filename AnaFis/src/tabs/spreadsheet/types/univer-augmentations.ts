import type { IRange, IWorkbookData, Univer } from '@univerjs/core';
import type { FWorkbook } from '@univerjs/sheets/facade';

// Augment FUniver type to include internal properties used in the adapter
// We explicitly declare the Facade methods here so TypeScript knows they exist
// everywhere FUniver is used, even if the side-effect import is missing.
declare module '@univerjs/core/facade' {
  interface FUniver {
    __univerInstance?: Univer;
    getActiveWorkbook(): FWorkbook | null;
    createWorkbook(snapshot: IWorkbookData): FWorkbook;
    disposeUnit(unitId: string): boolean;
  }
}

// Augment internal Univer worksheet types for merge data operations
declare module '@univerjs/core' {
  interface Worksheet {
    getConfig():
      | {
          mergeData?: IRange[];
        }
      | null
      | undefined;
  }
}

// Type declarations for Univer permission constructors
declare module '@univerjs/sheets' {
  export interface WorksheetEditPermissionConstructor {
    new (unitId: string, sheetId: string): { id: string };
  }

  export interface RangeProtectionPermissionEditPointConstructor {
    new (unitId: string, sheetId: string, permissionId: string): { id: string };
  }
}
