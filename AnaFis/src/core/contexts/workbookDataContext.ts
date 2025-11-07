// workbookDataContext.ts - Workbook data context creation
import { createContext } from 'react';
import type { WorkbookDataContextType } from '@/core/contexts/WorkbookDataContext';

export const WorkbookDataContext = createContext<WorkbookDataContextType | null>(null);

export type { WorkbookDataContextType };