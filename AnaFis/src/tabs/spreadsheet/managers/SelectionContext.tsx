import React, { createContext, useRef, ReactNode } from 'react';

type SelectionHandler = (cellRef: string) => void;

interface SelectionContextValue {
  registerHandler: (id: string, handler: SelectionHandler) => () => void;
  notifySelection: (cellRef: string) => void;
}

const SelectionContext = createContext<SelectionContextValue | null>(null);

export { SelectionContext };

interface SelectionProviderProps {
  children: ReactNode;
}

export const SelectionProvider: React.FC<SelectionProviderProps> = ({ children }) => {
  const handlersRef = useRef<Map<string, SelectionHandler>>(new Map());

  const registerHandler = (id: string, handler: SelectionHandler) => {
    handlersRef.current.set(id, handler);
    return () => {
      handlersRef.current.delete(id);
    };
  };

  const notifySelection = (cellRef: string) => {
    for (const handler of handlersRef.current.values()) {
      try {
        handler(cellRef);
      } catch (error) {
        console.error('Error in selection handler:', error);
      }
    }
  };

  const value: SelectionContextValue = {
    registerHandler,
    notifySelection,
  };

  return (
    <SelectionContext.Provider value={value}>
      {children}
    </SelectionContext.Provider>
  );
};