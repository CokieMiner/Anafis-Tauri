// WorkbookDataContext.tsx - Context for managing pending workbook data loading
import React, { useRef, ReactNode } from 'react';
import type { WorkbookData } from '@/core/types/import';
import { WorkbookDataContext } from '@/core/contexts/workbookDataContext';

export interface WorkbookDataContextType {
  setPendingWorkbookData: (tabId: string, data: WorkbookData) => void;
  getPendingWorkbookData: (tabId: string) => WorkbookData | undefined;
  clearPendingWorkbookData: (tabId: string) => void;
}

interface WorkbookDataProviderProps {
  children: ReactNode;
}

export const WorkbookDataProvider: React.FC<WorkbookDataProviderProps> = ({ children }) => {
  const pendingDataRef = useRef(new Map<string, WorkbookData>());

  const setPendingWorkbookData = (tabId: string, data: WorkbookData) => {
    pendingDataRef.current.set(tabId, data);
  };

  const getPendingWorkbookData = (tabId: string): WorkbookData | undefined => {
    return pendingDataRef.current.get(tabId);
  };

  const clearPendingWorkbookData = (tabId: string) => {
    pendingDataRef.current.delete(tabId);
  };

  const contextValue: WorkbookDataContextType = {
    setPendingWorkbookData,
    getPendingWorkbookData,
    clearPendingWorkbookData,
  };

  return (
    <WorkbookDataContext.Provider value={contextValue}>
      {children}
    </WorkbookDataContext.Provider>
  );
};