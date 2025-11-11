import { useContext } from 'react';
import { SelectionContext } from './SelectionContext';

type SelectionHandler = (cellRef: string) => void;

interface SelectionContextValue {
  registerHandler: (id: string, handler: SelectionHandler) => () => void;
  notifySelection: (cellRef: string) => void;
}

export const useSelectionContext = (): SelectionContextValue => {
  const context = useContext(SelectionContext);
  if (!context) {
    throw new Error('useSelectionContext must be used within a SelectionProvider');
  }
  return context;
};