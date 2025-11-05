// useWorkbookData.ts - Hook for workbook data context
import { useContext } from 'react';
import { WorkbookDataContext, WorkbookDataContextType } from '../contexts/workbookDataContext';

export const useWorkbookData = (): WorkbookDataContextType => {
  const context = useContext(WorkbookDataContext);
  if (!context) {
    throw new Error('useWorkbookData must be used within a WorkbookDataProvider');
  }
  return context;
};