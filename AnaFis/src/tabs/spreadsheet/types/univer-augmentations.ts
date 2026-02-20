// univer-augmentations.ts - Type augmentations for Univer.js types
import type { Univer } from '@univerjs/core';

// Augment FUniver type to include internal properties used in the adapter
declare module '@univerjs/core/facade' {
  interface FUniver {
    __univerInstance?: Univer;
  }
}

// Augment worksheet types to include merge data methods
declare module '@univerjs/sheets/facade' {
  // Empty augmentation - FWorksheet interface exists but may need extension points
}

// Augment internal Univer worksheet types for merge data operations
declare module '@univerjs/core' {
  interface Worksheet {
    getConfig(): { mergeData?: unknown } | null | undefined;
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
