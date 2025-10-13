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

  // Listen to selection changes and update focused input
  useEffect(() => {
    if (!onSelectionChange) return;

    const handleSelection = (selection: string) => {
      // Skip if no input is focused or not in selection mode
      if (!focusedInput || !isSelectionMode) {
        return;
      }

      // If this is the first selection, store it as anchor
      if (!anchorCellRef.current) {
        anchorCellRef.current = selection.split(':')[0]; // Store just the first cell
      }

      // Check if this is a single cell selection (no colon means single cell)
      const isSingleCell = !selection.includes(':');
      const currentCell = selection.split(':')[0];

      // If clicking a different single cell (not the anchor), exit selection mode
      if (isSingleCell && currentCell !== anchorCellRef.current) {
        setIsSelectionMode(false);
        setFocusedInput(null);
        anchorCellRef.current = '';
        lastSelectionRef.current = '';
        return;
      }

      // Update the field with the selection
      lastSelectionRef.current = selection;
      updateField(focusedInput, selection);
    };

    // Register the handler on window
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any)[handlerName] = handleSelection;

    return () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      delete (window as any)[handlerName];
    };
  }, [focusedInput, isSelectionMode, onSelectionChange, updateField, handlerName]);

  // Exit selection mode when clicking on sidebar elements (buttons, etc.)
  useEffect(() => {
    const handleSidebarClick = (e: MouseEvent) => {
      const sidebar = document.querySelector(`[${sidebarDataAttribute}]`);
      const target = e.target as HTMLElement;
      
      // Check if clicking on a button, select, or other interactive element in the sidebar
      if (sidebar && sidebar.contains(target)) {
        const isInteractiveElement = target.closest('button, select, .MuiAutocomplete-root');
        if (isInteractiveElement) {
          setIsSelectionMode(false);
          setFocusedInput(null);
          lastSelectionRef.current = '';
          anchorCellRef.current = '';
        }
      }
    };

    document.addEventListener('click', handleSidebarClick);
    return () => {
      document.removeEventListener('click', handleSidebarClick);
    };
  }, [sidebarDataAttribute]);

  // Input focus handler - enter selection mode
  const handleInputFocus = useCallback((inputType: T) => {
    setFocusedInput(inputType);
    setIsSelectionMode(true);
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
