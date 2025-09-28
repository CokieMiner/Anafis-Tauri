import { Column } from 'react-datasheet-grid';
import UncertaintyCell from './UncertaintyCell';

interface UncertaintyColumnOptions {
  cellRef: (rowIndex: number, columnId: string) => string;
  onCellValueChange?: (cellRef: string, value: string) => void;
}

// Custom uncertainty column type for react-datasheet-grid
export const uncertaintyColumn = (columnId: string, options: UncertaintyColumnOptions): Partial<Column<any, any>> => ({
  component: ({ rowData, rowIndex, focus, stopEditing }) => {
    const cellRef = options.cellRef(rowIndex, columnId);
    const currentValue = (rowData && typeof rowData[columnId] !== 'undefined') ? rowData[columnId] : '';

    return (
      <UncertaintyCell
        cellRef={cellRef}
        initialValue={currentValue}
        isEditing={focus}
        onValueChange={(value) => {
          // Ensure rowData exists and update it
          if (rowData) {
            rowData[columnId] = value;
            options.onCellValueChange?.(cellRef, value);
          }
        }}
        onEditingChange={(editing) => {
          if (!editing) {
            stopEditing();
          }
        }}
        onFocusAreaChange={(area) => {
          // Could be used for additional focus handling
          console.log('Focus area changed to:', area);
        }}
      />
    );
  },

  disableKeys: true, // Disable default keyboard handling
  keepFocus: true,   // Keep focus when editing
});

// Helper function to detect if a cell should use uncertainty column
export const shouldUseUncertaintyColumn = (value: string): boolean => {
  return value.includes('±') || value.includes('+/-') || value.includes('+-');
};

// Enhanced column factory that automatically switches between text and uncertainty
export const adaptiveColumn = (
  columnId: string,
  options: UncertaintyColumnOptions
): Partial<Column<any, any>> => ({
  component: ({ rowData, rowIndex, focus, stopEditing }) => {
    // Ensure rowData exists
    if (!rowData) {
      return <div>Loading...</div>;
    }

    const cellRef = options.cellRef(rowIndex, columnId);
    const currentValue = rowData[columnId] || '';

    // Check if this cell should use uncertainty rendering
    const useUncertainty = shouldUseUncertaintyColumn(currentValue);

    if (useUncertainty) {
      return (
        <UncertaintyCell
          cellRef={cellRef}
          initialValue={currentValue}
          isEditing={focus}
          onValueChange={(value) => {
            rowData[columnId] = value;
            options.onCellValueChange?.(cellRef, value);
          }}
          onEditingChange={(editing) => {
            if (!editing) {
              stopEditing();
            }
          }}
        />
      );
    }

    // Default text cell rendering
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          paddingLeft: '8px',
          paddingRight: '8px',
          cursor: 'text',
          fontSize: '14px',
          fontFamily: 'monospace',
        }}
      >
        {focus ? (
          <input
            type="text"
            value={currentValue}
            onChange={(e) => {
              rowData[columnId] = e.target.value;
              options.onCellValueChange?.(cellRef, e.target.value);
            }}
            onBlur={() => stopEditing()}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                stopEditing();
              } else if (e.key === 'Escape') {
                e.preventDefault();
                stopEditing();
              }
            }}
            style={{
              width: '100%',
              border: 'none',
              outline: 'none',
              background: 'transparent',
              fontSize: '14px',
              fontFamily: 'monospace',
            }}
            autoFocus
          />
        ) : (
          <span>{currentValue}</span>
        )}
      </div>
    );
  },

  disableKeys: true,
  keepFocus: true,
});

export default uncertaintyColumn;