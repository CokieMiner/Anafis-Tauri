import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
  Box,
  Paper
} from '@mui/material';

import { UniverAdapter as SpreadsheetAdapter } from '@/tabs/spreadsheet/univer';
import { SpreadsheetRef, WorkbookData, CellValue } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { spreadsheetEventBus } from '@/tabs/spreadsheet/managers/SpreadsheetEventBus';

import UncertaintySidebar from '@/tabs/spreadsheet/components/sidebar/UncertaintySidebar';
import UnitConversionSidebar from '@/tabs/spreadsheet/components/sidebar/UnitConversionSidebar';
import QuickPlotSidebar from '@/tabs/spreadsheet/components/sidebar/QuickPlotSidebar';
import ExportSidebar from '@/tabs/spreadsheet/components/sidebar/ExportSidebar';
import ImportSidebar from '@/tabs/spreadsheet/components/sidebar/ImportSidebar';
import SpreadsheetSidebarToolbar from '@/tabs/spreadsheet/components/SpreadsheetSidebarToolbar';
import { SidebarErrorBoundary, SpreadsheetErrorBoundary } from '@/shared/components/error-boundaries';
import { ExportService } from '@/core/types/export';
import { ImportService } from '@/core/types/import';
import { useWorkbookData } from '@/core/managers/WorkbookDataProvider';

interface SpreadsheetTabProps {
  tabId: string;
}

const SpreadsheetTab: React.FC<SpreadsheetTabProps> = ({ tabId }) => {
  // Check if we're in a detached window by looking at URL params
  const isDetachedWindow = new URLSearchParams(window.location.search).has('tabId');

  // Workbook data context for proper synchronization
  const { getPendingWorkbookData, clearPendingWorkbookData } = useWorkbookData();

  // Sidebar state management - now using simple local state since tabs stay mounted
  type SidebarType = 'uncertainty' | 'unitConvert' | 'quickPlot' | 'export' | 'import' | null;
  const [activeSidebar, setActiveSidebar] = useState<SidebarType>(null);

  // Consolidated sidebar state - better performance and organization
  // Uncertainty sidebar state - REMOVED: now managed by useUncertaintyPropagation hook

  // Unit conversion sidebar state
  const [unitConversionCategory, setUnitConversionCategory] = useState<string>('');
  const [unitConversionFromUnit, setUnitConversionFromUnit] = useState<string>('');
  const [unitConversionToUnit, setUnitConversionToUnit] = useState<string>('');
  const [unitConversionValue, setUnitConversionValue] = useState<string>('1');

  // Export sidebar state - REMOVED: now managed by useExport hook

  // Spreadsheet state - now persistent per tab
  const spreadsheetRef = useRef<SpreadsheetRef>(null);

  // Get services from the spreadsheet implementation (maintains abstraction)
  // Must use state because we can't access refs during render
  const [exportService, setExportService] = useState<ExportService | null>(null);
  const [importService, setImportService] = useState<ImportService | null>(null);

  // Initialize services after mount (can't access refs during render)
  useEffect(() => {
    // Poll for services until spreadsheet is ready
    const checkServices = () => {
      if (spreadsheetRef.current) {
        const expSvc = spreadsheetRef.current.getExportService();
        const impSvc = spreadsheetRef.current.getImportService();
        setExportService(expSvc);
        setImportService(impSvc);
        return true;
      }
      return false;
    };

    // Try immediately
    if (checkServices()) {
      return;
    }

    // Poll every 100ms if not ready yet
    const interval = setInterval(() => {
      if (checkServices()) {
        clearInterval(interval);
      }
    }, 100);

    return () => clearInterval(interval);
  }, []); // Only run once on mount

  // Listen for workbook data loading events (e.g., when opening .anafispread files)
  useEffect(() => {
    const handleLoadWorkbookData = (event: CustomEvent<WorkbookData>) => {
      if (spreadsheetRef.current?.loadWorkbookSnapshot) {
        void spreadsheetRef.current.loadWorkbookSnapshot(event.detail);
      }
    };

    window.addEventListener('load-workbook-data', handleLoadWorkbookData as EventListener);

    return () => {
      window.removeEventListener('load-workbook-data', handleLoadWorkbookData as EventListener);
    };
  }, []);

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
          rowCount: 1000,
          columnCount: 26,
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
    spreadsheetEventBus.emit('selection-change', cellRef);
  }, []);

  // Handle sidebar toggle from toolbar
  const handleSidebarToggle = useCallback((sidebar: SidebarType) => {
    setActiveSidebar(sidebar);
  }, []);

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* NOTE: Conflict resolution and error UI removed - no longer applicable for local-only tabs */}

      {/* Main Toolbar - show in both main window and detached windows */}
      <SpreadsheetSidebarToolbar 
        activeSidebar={activeSidebar}
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
                tabId={tabId}
              />
            </Box>
            {/* Uncertainty Propagation Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'uncertainty' && (
              <SidebarErrorBoundary sidebarName="Uncertainty Propagation" onClose={() => setActiveSidebar(null)}>
                <UncertaintySidebar
                  open={true}
                  onClose={() => setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                  onPropagationComplete={(_resultRange: string) => {
                    // Could refresh spreadsheet or show notification here
                  }}
                />
              </SidebarErrorBoundary>
            )}
            {/* Unit Conversion Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'unitConvert' && (
              <SidebarErrorBoundary sidebarName="Unit Conversion" onClose={() => setActiveSidebar(null)}>
                <UnitConversionSidebar
                  open={true}
                  onClose={() => setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                  category={unitConversionCategory}
                  setCategory={setUnitConversionCategory}
                  fromUnit={unitConversionFromUnit}
                  setFromUnit={setUnitConversionFromUnit}
                  toUnit={unitConversionToUnit}
                  setToUnit={setUnitConversionToUnit}
                  value={unitConversionValue}
                  setValue={setUnitConversionValue}
                />
              </SidebarErrorBoundary>
            )}
            {/* Quick Plot Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'quickPlot' && (
              <SidebarErrorBoundary sidebarName="Quick Plot" onClose={() => setActiveSidebar(null)}>
                <QuickPlotSidebar
                  open={true}
                  onClose={() => setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                />
              </SidebarErrorBoundary>
            )}
            {/* Export Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'export' && exportService && (
              <SidebarErrorBoundary sidebarName="Export" onClose={() => setActiveSidebar(null)}>
                <ExportSidebar
                  open={true}
                  onClose={() => setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                  exportService={exportService}
                />
              </SidebarErrorBoundary>
            )}
            {/* Import Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'import' && importService && (
              <SidebarErrorBoundary sidebarName="Import" onClose={() => setActiveSidebar(null)}>
                <ImportSidebar
                  open={true}
                  onClose={() => setActiveSidebar(null)}
                  spreadsheetRef={spreadsheetRef}
                  onSelectionChange={handleSelectionChange}
                  importService={importService}
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

export default SpreadsheetTab;
