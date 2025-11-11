import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
  Box,
  Paper
} from '@mui/material';

import { UniverAdapter as SpreadsheetAdapter } from '@/tabs/spreadsheet/univer';
import { SpreadsheetRef, WorkbookData, CellValue } from '@/tabs/spreadsheet/types/SpreadsheetInterface';

import UncertaintySidebar from '@/tabs/spreadsheet/components/sidebar/UncertaintySidebar';
import UnitConversionSidebar from '@/tabs/spreadsheet/components/sidebar/UnitConversionSidebar';
import QuickPlotSidebar from '@/tabs/spreadsheet/components/sidebar/QuickPlotSidebar';
import ExportSidebar from '@/tabs/spreadsheet/components/sidebar/ExportSidebar';
import ImportSidebar from '@/tabs/spreadsheet/components/sidebar/ImportSidebar';
import SpreadsheetSidebarToolbar from '@/tabs/spreadsheet/components/SpreadsheetSidebarToolbar';
import { SidebarErrorBoundary, SpreadsheetErrorBoundary } from '@/shared/components/error-boundaries';
import { useWorkbookData } from '@/core/managers/WorkbookDataProvider';
import { SelectionProvider } from '@/tabs/spreadsheet/managers/SelectionContext';
import { useSelectionContext } from '@/tabs/spreadsheet/managers/useSelectionContext';
import { DEFAULT_SHEET_ROWS, DEFAULT_SHEET_COLS } from '@/tabs/spreadsheet/univer/utils/constants';
import { useSidebarState } from '@/tabs/spreadsheet/managers/SidebarStateManager';

interface SpreadsheetTabProps {
  tabId: string;
}

const SpreadsheetContent: React.FC<SpreadsheetTabProps> = ({ tabId }) => {
  const { notifySelection } = useSelectionContext();
  // Check if we're in a detached window by looking at URL params
  const isDetachedWindow = new URLSearchParams(window.location.search).has('tabId');

  // Workbook data context for proper synchronization
  const { getPendingWorkbookData, clearPendingWorkbookData } = useWorkbookData();

  // Use centralized sidebar state management
  const { state: sidebarState, actions: sidebarActions } = useSidebarState();

  // Spreadsheet state - now persistent per tab
  const spreadsheetRef = useRef<SpreadsheetRef>(null);

  // Initialize services after mount (can't access refs during render)
  const handleSpreadsheetReady = useCallback(() => {
    sidebarActions.initializeServices(spreadsheetRef);
  }, [sidebarActions]);

  // NOTE: Window event listener for load-workbook-data removed - App.tsx now handles file opening directly
  // File association is handled by creating tabs with workbook data in App.tsx

  // Check for pending workbook data on mount (for proper synchronization)
  useEffect(() => {
    const pendingData = getPendingWorkbookData(tabId);
    if (pendingData && spreadsheetRef.current?.loadWorkbookSnapshot) {
      void spreadsheetRef.current.loadWorkbookSnapshot(pendingData as unknown);
      // Clear the pending data after loading
      clearPendingWorkbookData(tabId);
    }
  }, [tabId, getPendingWorkbookData, clearPendingWorkbookData]);

  // NOTE: Tab synchronization removed - tabs are now local-only

  // Sidebar state is now purely local - no backend persistence

  // Create initial workbook data - only load once to avoid circular updates
  const [spreadsheetData] = useState<WorkbookData>(() => {
    const sheetId = 'sheet-01';
    const workbookId = `workbook-${tabId}`;

    return {
      id: workbookId,
      name: `AnaFis Spreadsheet - ${tabId}`,
      appVersion: '1.0.0',
      locale: 'EN_US',
      styles: {},
      sheets: {
        [sheetId]: {
          id: sheetId,
          name: 'Sheet1',
          cellData: {},
          rowCount: DEFAULT_SHEET_ROWS,
          columnCount: DEFAULT_SHEET_COLS,
        }
      },
      sheetOrder: [sheetId],
    };
  });

  // NOTE: Backend data persistence removed - spreadsheet data is now local-only

  const handleCellChange = useCallback((_cellRef: string, _value: CellValue) => {
    // Cell change handler - Univer manages all data internally
    // NOTE: Backend persistence removed - cell data is now local-only
  }, []);

  const handleFormulaIntercept = useCallback((_cellRef: string, _formula: string) => {
    // Formula interception is no longer needed - Univer handles all formulas
    // Custom functions are registered directly with Univer's formula engine
    // This handler is kept for potential future use (e.g., formula validation)
  }, []);

  const handleSelectionChange = useCallback((cellRef: string) => {
    // Emit selection change event to all interested subscribers (sidebars)
    notifySelection(cellRef);
  }, [notifySelection]);

  // Handle sidebar toggle from toolbar
  const handleSidebarToggle = useCallback((sidebar: import('@/tabs/spreadsheet/managers/SidebarStateManager').SidebarType) => {
    sidebarActions.setActiveSidebar(sidebar);
  }, [sidebarActions]);

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* NOTE: Conflict resolution and error UI removed - no longer applicable for local-only tabs */}

      <SpreadsheetSidebarToolbar 
        activeSidebar={sidebarState.activeSidebar}
        onSidebarToggle={handleSidebarToggle}
        isDetachedWindow={isDetachedWindow}
      />

      {/* Main Content Area */}
      <SpreadsheetErrorBoundary title="Spreadsheet Error" componentName="SpreadsheetTab">
        <Box sx={{ display: 'flex', flex: 1, overflow: 'hidden', gap: 1 }}>
        {/* Spreadsheet Grid Container */}
        <Paper sx={{
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          minHeight: 0
        }}>

          {/* Grid Content Area */}
          <Box sx={{
            flex: 1,
            overflow: 'hidden',
            minHeight: 0,
            display: 'flex',
            position: 'relative',
            '& .dsg-container': {
              height: '100%',
              fontFamily: 'monospace',
            },
            '& .dsg-cell': {
              fontSize: '14px',
            },
            '& .dsg-cell-header': {
              backgroundColor: '#f5f5f5',
              fontWeight: 'bold',
            }
          }}>
            <Box sx={{ flex: 1, overflow: 'hidden' }}>
              <SpreadsheetAdapter
                ref={spreadsheetRef}
                initialData={spreadsheetData}
                onCellChange={handleCellChange}
                onFormulaIntercept={handleFormulaIntercept}
                onSelectionChange={handleSelectionChange}
                onReady={handleSpreadsheetReady}
                tabId={tabId}
              />
            </Box>
            {/* Uncertainty Propagation Sidebar - positioned within spreadsheet */}
            {sidebarState.activeSidebar === 'uncertainty' && (
              <SidebarErrorBoundary sidebarName="Uncertainty Propagation" onClose={() => sidebarActions.setActiveSidebar(null)}>
                <UncertaintySidebar
                  open={true}
                  onClose={() => sidebarActions.setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                  onPropagationComplete={(_resultRange: string) => {
                    // Could refresh spreadsheet or show notification here
                  }}
                />
              </SidebarErrorBoundary>
            )}
            {/* Unit Conversion Sidebar - positioned within spreadsheet */}
            {sidebarState.activeSidebar === 'unitConvert' && (
              <SidebarErrorBoundary sidebarName="Unit Conversion" onClose={() => sidebarActions.setActiveSidebar(null)}>
                <UnitConversionSidebar
                  open={true}
                  onClose={() => sidebarActions.setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                  category={sidebarState.unitConversion.category}
                  setCategory={sidebarActions.setUnitConversionCategory}
                  fromUnit={sidebarState.unitConversion.fromUnit}
                  setFromUnit={sidebarActions.setUnitConversionFromUnit}
                  toUnit={sidebarState.unitConversion.toUnit}
                  setToUnit={sidebarActions.setUnitConversionToUnit}
                  value={sidebarState.unitConversion.value}
                  setValue={sidebarActions.setUnitConversionValue}
                />
              </SidebarErrorBoundary>
            )}
            {/* Quick Plot Sidebar - positioned within spreadsheet */}
            {sidebarState.activeSidebar === 'quickPlot' && (
              <SidebarErrorBoundary sidebarName="Quick Plot" onClose={() => sidebarActions.setActiveSidebar(null)}>
                <QuickPlotSidebar
                  open={true}
                  onClose={() => sidebarActions.setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                />
              </SidebarErrorBoundary>
            )}
            {/* Export Sidebar - positioned within spreadsheet */}
            {sidebarState.activeSidebar === 'export' && sidebarState.services.exportService && (
              <SidebarErrorBoundary sidebarName="Export" onClose={() => sidebarActions.setActiveSidebar(null)}>
                <ExportSidebar
                  open={true}
                  onClose={() => sidebarActions.setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                  exportService={sidebarState.services.exportService}
                />
              </SidebarErrorBoundary>
            )}
            {/* Import Sidebar - positioned within spreadsheet */}
            {sidebarState.activeSidebar === 'import' && sidebarState.services.importService && (
              <SidebarErrorBoundary sidebarName="Import" onClose={() => sidebarActions.setActiveSidebar(null)}>
                <ImportSidebar
                  open={true}
                  onClose={() => sidebarActions.setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                  importService={sidebarState.services.importService}
                />
              </SidebarErrorBoundary>
            )}
          </Box>
        </Paper>
      </Box>
      </SpreadsheetErrorBoundary>
    </Box>
  );
};

const SpreadsheetTab: React.FC<SpreadsheetTabProps> = (props) => {
  return (
    <SelectionProvider>
      <SpreadsheetContent {...props} />
    </SelectionProvider>
  );
};

export default SpreadsheetTab;
