import { useState, useRef, useEffect, useCallback } from 'react';

interface UseSpreadsheetSelectionOptions<T> {
  /**
   * Callback to handle selection changes from the spreadsheet
   */
  onSelectionChange?: (selection: string) => void;

  /**
   * Function to update the appropriate field with the selected range
   */
  updateField: (inputType: T, selection: string) => void;

  /**
   * Data attribute for the sidebar container (e.g., 'data-unit-converter-sidebar')
   */
  sidebarDataAttribute: string;

  /**
   * Name of the global window handler (e.g., '__unitConverterSelectionHandler')
   */
  handlerName: string;
}

interface UseSpreadsheetSelectionReturn<T> {
  /**
   * The currently focused input field
   */
  focusedInput: T | null;

  /**
   * Call when an input field receives focus to enter selection mode
   */
  handleInputFocus: (inputType: T) => void;

  /**
   * Call when an input field loses focus
   */
  handleInputBlur: () => void;

  /**
   * Whether selection mode is currently active
   */
  isSelectionMode: boolean;
}

/**
 * Custom hook for managing spreadsheet range selection in sidebar inputs
 * 
 * This hook provides a smart selection mode that:
 * - Activates when an input is focused
 * - Stays active even when the input blurs (so users can click on spreadsheet)
 * - Updates the input as the user drags to expand ranges
 * - Automatically exits when clicking a different single cell (not part of the current range)
 * - Exits when clicking sidebar interactive elements (buttons, selects, etc.)
 * 
 * @example
 * ```tsx
 * const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection({
 *   onSelectionChange,
 *   updateField: (inputType, selection) => {
 *     if (inputType === 'value') setValue(selection);
 *   },
 *   sidebarDataAttribute: 'data-unit-converter-sidebar',
 *   handlerName: '__unitConverterSelectionHandler'
 * });
 * 
 * <TextField
 *   onFocus={() => handleInputFocus('value')}
 *   onBlur={handleInputBlur}
 * />
 * ```
 */
export function useSpreadsheetSelection<T>({
  onSelectionChange,
  updateField,
  sidebarDataAttribute,
  handlerName,
}: UseSpreadsheetSelectionOptions<T>): UseSpreadsheetSelectionReturn<T> {
  const [focusedInput, setFocusedInput] = useState<T | null>(null);
  const [isSelectionMode, setIsSelectionMode] = useState<boolean>(false);
  const lastSelectionRef = useRef<string>('');
  const anchorCellRef = useRef<string>(''); // Store the first cell clicked
  const isActiveRef = useRef<boolean>(false); // Immediate flag to prevent race conditions

  // Listen to selection changes and update focused input
  useEffect(() => {
    if (!onSelectionChange) {return;}

    const handleSelection = (selection: string) => {
      // CRITICAL: Check the ref first to prevent race conditions
      if (!isActiveRef.current) {
        return;
      }

      // Skip if no input is focused or not in selection mode
      if (!focusedInput || !isSelectionMode) {
        return;
      }

      // Skip if selection hasn't changed (reduces unnecessary updates during drag)
      if (selection === lastSelectionRef.current) {
        return;
      }

      // Helper function to parse cell reference (e.g., "B12" -> { col: "B", row: 12 })
      const parseCell = (cell: string): { col: string; row: number } | null => {
        const match = cell.match(/^([A-Z]+)(\d+)$/);
        if (!match?.[1] || !match[2]) {return null;}
        return { col: match[1], row: parseInt(match[2], 10) };
      };

      // Helper function to convert column letter to number (A=1, B=2, ..., Z=26, AA=27, etc.)
      const colToNum = (col: string): number => {
        let num = 0;
        for (let i = 0; i < col.length; i++) {
          num = num * 26 + (col.charCodeAt(i) - 64);
        }
        return num;
      };

      // Helper function to check if anchor cell is in the range
      const isAnchorInRange = (range: string, anchor: string): boolean => {
        if (range === anchor) {return true;} // Single cell match

        if (!range.includes(':')) {return range === anchor;} // Single cell range

        const rangeParts = range.split(':');
        if (rangeParts.length !== 2) {return false;}
        
        const [start, end] = rangeParts;
        if (!start || !end) {return false;}
        
        const anchorParsed = parseCell(anchor);
        const startParsed = parseCell(start);
        const endParsed = parseCell(end);

        if (!anchorParsed || !startParsed || !endParsed) {return false;}

        // Convert columns to numbers for comparison
        const anchorCol = colToNum(anchorParsed.col);
        const startCol = colToNum(startParsed.col);
        const endCol = colToNum(endParsed.col);

        // Determine the actual bounds (start might be > end depending on drag direction)
        const minCol = Math.min(startCol, endCol);
        const maxCol = Math.max(startCol, endCol);
        const minRow = Math.min(startParsed.row, endParsed.row);
        const maxRow = Math.max(startParsed.row, endParsed.row);

        // Check if anchor is within the rectangular bounds
        return (
          anchorCol >= minCol &&
          anchorCol <= maxCol &&
          anchorParsed.row >= minRow &&
          anchorParsed.row <= maxRow
        );
      };

      // If this is the first selection, store it as anchor
      if (!anchorCellRef.current) {
        // Store the first cell of the selection as anchor
        const firstCell = selection.split(':')[0];
        if (!firstCell) {
          console.warn('Invalid selection format:', selection);
          return;
        }
        anchorCellRef.current = firstCell;
        lastSelectionRef.current = selection;
        updateField(focusedInput, selection);
        return;
      }

      // CRITICAL: Check if anchor cell is in the new selection BEFORE updating
      if (!isAnchorInRange(selection, anchorCellRef.current)) {
        // Anchor cell is not in the new range - exit selection mode immediately
        // Set ref FIRST to immediately block subsequent events
        isActiveRef.current = false;
        setIsSelectionMode(false);
        setFocusedInput(null);
        anchorCellRef.current = '';
        lastSelectionRef.current = '';
        return;
      }

      // Anchor is in the range - update the field
      lastSelectionRef.current = selection;
      updateField(focusedInput, selection);
    };

    // Register the handler on window
    (window as unknown as Record<string, unknown>)[handlerName] = handleSelection;

    return () => {
      delete (window as unknown as Record<string, unknown>)[handlerName];
    };
  }, [focusedInput, isSelectionMode, onSelectionChange, updateField, handlerName]);

  // Exit selection mode when clicking on sidebar elements (buttons, etc.)
  useEffect(() => {
    const handleSidebarClick = (e: MouseEvent) => {
      if (!isSelectionMode) {return;} // Early exit if not in selection mode
      
      const sidebar = document.querySelector(`[${sidebarDataAttribute}]`);
      const target = e.target as HTMLElement;

      // Check if clicking on a button, select, or other interactive element in the sidebar
      if (sidebar?.contains(target)) {
        const isInteractiveElement = target.closest('button, select, .MuiAutocomplete-root');
        if (isInteractiveElement) {
          // Set ref FIRST to immediately block subsequent events
          isActiveRef.current = false;
          setIsSelectionMode(false);
          setFocusedInput(null);
          lastSelectionRef.current = '';
          anchorCellRef.current = '';
        }
      }
    };

    if (isSelectionMode) {
      document.addEventListener('click', handleSidebarClick, { passive: true });
    }
    
    return () => {
      document.removeEventListener('click', handleSidebarClick);
    };
  }, [sidebarDataAttribute, isSelectionMode]);

  // Input focus handler - enter selection mode
  const handleInputFocus = useCallback((inputType: T) => {
    setFocusedInput(inputType);
    setIsSelectionMode(true);
    // Set ref FIRST to immediately allow events
    isActiveRef.current = true;
    // Reset tracking refs when entering selection mode
    lastSelectionRef.current = '';
    anchorCellRef.current = '';
  }, []);

  // Input blur handler - keep selection mode active
  const handleInputBlur = useCallback(() => {
    // DON'T exit selection mode on blur!
    // The user needs to be able to click on spreadsheet without losing selection mode
  }, []);

  return {
    focusedInput,
    handleInputFocus,
    handleInputBlur,
    isSelectionMode,
  };
}
