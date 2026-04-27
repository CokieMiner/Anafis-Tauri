import type React from 'react';

export interface Tab<TState = Record<string, unknown>> {
  id: string;
  title: string;
  content: React.ReactNode;
  type: string; // Tab type (spreadsheet, fitting, etc.)
  state?: TState; // State for detached tabs
  isDetaching?: boolean; // Flag to prevent concurrent detachments
}
