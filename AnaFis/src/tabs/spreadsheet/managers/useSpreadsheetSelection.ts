import { useRef, useEffect, useCallback, useReducer } from 'react';
import { useSelectionContext } from './useSelectionContext';
import { CellReference } from '@/tabs/spreadsheet/utils/CellReference';

// Simplified cache interfaces
interface ParsedCell {
  col: string;
  row: number;
  colNum: number;
}

interface ParsedRange {
  start: ParsedCell;
  end: ParsedCell;
  minCol: number;
  maxCol: number;
  minRow: number;
  maxRow: number;
}

// Simplified UI state (only what triggers re-renders)
interface UIState<T> {
  focusedInput: T | null;
  isSelectionMode: boolean;
}

// Internal state (mutable, doesn't trigger re-renders)
interface InternalState {
  lastSelection: string;
  anchorCell: string;
  isActive: boolean;
}

type UIAction<T> =
  | { type: 'ENTER_SELECTION_MODE'; inputType: T }
  | { type: 'EXIT_SELECTION_MODE' };

function uiReducer<T>(
  state: UIState<T>,
  action: UIAction<T>
): UIState<T> {
  switch (action.type) {
    case 'ENTER_SELECTION_MODE':
      return {
        focusedInput: action.inputType,
        isSelectionMode: true,
      };
    case 'EXIT_SELECTION_MODE':
      return {
        focusedInput: null,
        isSelectionMode: false,
      };
    default:
      return state;
  }
}

// Simplified cache - just Maps with basic size limits
class SimpleRangeCache {
  private cellCache = new Map<string, ParsedCell>();
  private rangeCache = new Map<string, ParsedRange>();
  private columnCache = new Map<string, number>();
  private readonly maxSize = 1000;

  parseCell(cell: string): ParsedCell | null {
    const cached = this.cellCache.get(cell);
    if (cached) {
      return cached;
    }

    const match = cell.match(/^([A-Z]+)(\d+)$/);
    if (!match?.[1] || !match[2]) {
      return null;
    }

    const col = match[1];
    const row = parseInt(match[2], 10);
    const colNum = this.colToNum(col);

    const parsedCell: ParsedCell = { col, row, colNum };
    this.setCache('cell', cell, parsedCell);
    return parsedCell;
  }

  colToNum(col: string): number {
    const cached = this.columnCache.get(col);
    if (cached !== undefined) {
      return cached;
    }

    const num = CellReference.letterToColumn(col);
    this.setCache('column', col, num);
    return num;
  }

  parseRange(range: string): ParsedRange | null {
    const cached = this.rangeCache.get(range);
    if (cached) {
      return cached;
    }

    // Single cell range
    if (!range.includes(':')) {
      const cell = this.parseCell(range);
      if (!cell) {
        return null;
      }

      const parsedRange: ParsedRange = {
        start: cell,
        end: cell,
        minCol: cell.colNum,
        maxCol: cell.colNum,
        minRow: cell.row,
        maxRow: cell.row
      };
      this.setCache('range', range, parsedRange);
      return parsedRange;
    }

    // Multi-cell range
    const rangeParts = range.split(':');
    if (rangeParts.length !== 2) {
      return null;
    }

    const [startStr, endStr] = rangeParts;
    if (!startStr || !endStr) {
      return null;
    }

    const start = this.parseCell(startStr);
    const end = this.parseCell(endStr);
    if (!start || !end) {
      return null;
    }

    const parsedRange: ParsedRange = {
      start,
      end,
      minCol: Math.min(start.colNum, end.colNum),
      maxCol: Math.max(start.colNum, end.colNum),
      minRow: Math.min(start.row, end.row),
      maxRow: Math.max(start.row, end.row)
    };

    this.setCache('range', range, parsedRange);
    return parsedRange;
  }

  private setCache(type: 'cell' | 'range' | 'column', key: string, value: ParsedCell | ParsedRange | number): void {
    if (type === 'cell') {
      if (this.cellCache.size >= this.maxSize) {
        this.cellCache.clear();
      }
      this.cellCache.set(key, value as ParsedCell);
    } else if (type === 'range') {
      if (this.rangeCache.size >= this.maxSize) {
        this.rangeCache.clear();
      }
      this.rangeCache.set(key, value as ParsedRange);
    } else {
      if (this.columnCache.size >= this.maxSize) {
        this.columnCache.clear();
      }
      this.columnCache.set(key, value as number);
    }
  }

  clear(): void {
    this.cellCache.clear();
    this.rangeCache.clear();
    this.columnCache.clear();
  }

  get stats() {
    return {
      cellCache: { size: this.cellCache.size, maxSize: this.maxSize },
      rangeCache: { size: this.rangeCache.size, maxSize: this.maxSize },
      columnCache: { size: this.columnCache.size, maxSize: this.maxSize },
      totalEntries: this.cellCache.size + this.rangeCache.size + this.columnCache.size
    };
  }
}

interface UseSpreadsheetSelectionOptions<T> {
  onSelectionChange?: (selection: string) => void;
  updateField: (inputType: T, selection: string) => void;
  sidebarDataAttribute: string;
}

interface UseSpreadsheetSelectionReturn<T> {
  focusedInput: T | null;
  handleInputFocus: (inputType: T) => void;
  handleInputBlur: () => void;
  isSelectionMode: boolean;
  getCacheStats: () => {
    cellCache: { size: number; maxSize: number; };
    rangeCache: { size: number; maxSize: number; };
    columnCache: { size: number; maxSize: number; };
    totalEntries: number;
  };
}

export function useSpreadsheetSelection<T>({
  onSelectionChange,
  updateField,
  sidebarDataAttribute,
}: UseSpreadsheetSelectionOptions<T>): UseSpreadsheetSelectionReturn<T> {
  const { registerHandler } = useSelectionContext();

  // UI state - only what needs to trigger re-renders
  const [uiState, dispatch] = useReducer(uiReducer<T>, {
    focusedInput: null,
    isSelectionMode: false,
  });

  // Internal state - all mutable state in one place
  const internalStateRef = useRef<InternalState>({
    lastSelection: '',
    anchorCell: '',
    isActive: false,
  });

  // Simplified cache
  const cacheRef = useRef<SimpleRangeCache>(new SimpleRangeCache());

  // Helper to update internal state
  const updateInternalState = useCallback((updates: Partial<InternalState>) => {
    Object.assign(internalStateRef.current, updates);
  }, []);

  // Check if anchor cell is within range
  const isAnchorInRange = useCallback((range: string): boolean => {
    const state = internalStateRef.current;
    if (!state.anchorCell) {
      return false;
    }

    // Quick check for exact match
    if (range === state.anchorCell) {
      return true;
    }

    // Parse anchor cell
    const anchorBounds = cacheRef.current.parseCell(state.anchorCell);
    if (!anchorBounds) {
      return false;
    }

    // Parse range
    const parsedRange = cacheRef.current.parseRange(range);
    if (!parsedRange) {
      return false;
    }

    // Check bounds
    return (
      anchorBounds.colNum >= parsedRange.minCol &&
      anchorBounds.colNum <= parsedRange.maxCol &&
      anchorBounds.row >= parsedRange.minRow &&
      anchorBounds.row <= parsedRange.maxRow
    );
  }, []);

  // Handle selection changes
  useEffect(() => {
    if (!onSelectionChange) {
      return;
    }

    const handleSelection = (selection: string) => {
      const state = internalStateRef.current;
      if (!state.isActive || !uiState.isSelectionMode || !uiState.focusedInput) {
        return;
      }

      if (selection === state.lastSelection) {
        return;
      }

      // Set anchor cell on first selection
      if (!state.anchorCell) {
        const firstCell = selection.split(':')[0];
        if (!firstCell) {
          console.warn('Invalid selection format:', selection);
          return;
        }
        updateInternalState({ anchorCell: firstCell, lastSelection: selection });
        updateField(uiState.focusedInput, selection);
        return;
      }

      // Check if selection still contains anchor
      if (!isAnchorInRange(selection)) {
        updateInternalState({
          isActive: false,
          lastSelection: '',
          anchorCell: ''
        });
        dispatch({ type: 'EXIT_SELECTION_MODE' });
        return;
      }

      updateInternalState({ lastSelection: selection });
      updateField(uiState.focusedInput, selection);
    };

    const unregister = registerHandler(sidebarDataAttribute, handleSelection);
    return unregister;
  }, [uiState.focusedInput, uiState.isSelectionMode, onSelectionChange, updateField, isAnchorInRange, dispatch, registerHandler, sidebarDataAttribute, updateInternalState]);

  // Exit selection mode on sidebar clicks
  useEffect(() => {
    const handleSidebarClick = (e: MouseEvent) => {
      if (!uiState.isSelectionMode) {
        return;
      }

      const sidebar = document.querySelector(`[${sidebarDataAttribute}]`);
      const target = e.target as HTMLElement;

      if (sidebar?.contains(target)) {
        const isInteractiveElement = target.closest('button, select, .MuiAutocomplete-root');
        if (isInteractiveElement) {
          updateInternalState({
            isActive: false,
            lastSelection: '',
            anchorCell: ''
          });
          dispatch({ type: 'EXIT_SELECTION_MODE' });
        }
      }
    };

    if (uiState.isSelectionMode) {
      document.addEventListener('click', handleSidebarClick, { passive: true });
    }

    return () => {
      document.removeEventListener('click', handleSidebarClick);
    };
  }, [sidebarDataAttribute, uiState.isSelectionMode, dispatch, updateInternalState]);

  const handleInputFocus = useCallback((inputType: T) => {
    updateInternalState({
      lastSelection: '',
      anchorCell: '',
      isActive: true
    });
    dispatch({ type: 'ENTER_SELECTION_MODE', inputType });
  }, [dispatch, updateInternalState]);

  const handleInputBlur = useCallback(() => {
    // Keep selection mode active on blur
  }, []);

  return {
    focusedInput: uiState.focusedInput,
    handleInputFocus,
    handleInputBlur,
    isSelectionMode: uiState.isSelectionMode,
    getCacheStats: () => cacheRef.current.stats,
  };
}