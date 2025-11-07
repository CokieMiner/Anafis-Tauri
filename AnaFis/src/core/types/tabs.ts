import React from 'react';

export interface Tab<TState = Record<string, unknown>> {
  id: string;
  title: string;
  content: React.ReactNode;
  type: string; // Tab type (spreadsheet, fitting, etc.)
  state?: TState; // State for detached tabs
  isDetaching?: boolean; // Flag to prevent concurrent detachments
}

// Enhanced tab state for backend synchronization
// Field names use snake_case to match the backend API schema
export interface TabStateData<TData = unknown, TUIState = unknown> {
  id: string;
  title: string;
  tab_type: string;
  data: TData;
  ui_state: TUIState;
  last_modified: string; // ISO 8601 timestamp string
  window_id?: string;
  is_detached: boolean;
  version: number;
}

// Window geometry for detached windows
export interface WindowGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
}
