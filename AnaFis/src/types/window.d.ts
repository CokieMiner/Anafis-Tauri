/**
 * Global window augmentation for dynamic handler registration
 * Used by sidebar components to register selection change handlers
 */

declare global {
  interface Window {
    /**
     * Export sidebar selection handler
     */
    __exportSelectionHandler?: (cellRef: string) => void;
    
    /**
     * Import sidebar selection handler
     */
    __importSelectionHandler?: (cellRef: string) => void;
    
    /**
     * Uncertainty sidebar selection handler
     */
    __uncertaintySidebarSelectionHandler?: (cellRef: string) => void;
    
    /**
     * Quick plot sidebar selection handler
     */
    __quickPlotSelectionHandler?: (cellRef: string) => void;
    
    /**
     * Unit conversion sidebar selection handler
     */
    __unitConverterSelectionHandler?: (cellRef: string) => void;
    
    /**
     * Univer instance tracking (prevents multiple instances)
     */
    __UNIVER_INSTANCES__?: Set<string>;
  }
}

export {};
