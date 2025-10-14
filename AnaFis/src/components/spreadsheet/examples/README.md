# Spreadsheet Adapter Examples

This folder contains template/example adapters for different spreadsheet libraries.

## AlternativeAdapter.tsx

A template adapter that shows the structure needed to implement support for any spreadsheet library.

### To create a new adapter:

1. Copy `AlternativeAdapter.tsx` to a new file (e.g., `ExcelJSAdapter.tsx`)
2. Implement the 3 required methods in the `useImperativeHandle` hook:
   - `updateCell(cellRef: string, value: CellValue)` - Update a cell with a value or formula
   - `getCellValue(cellRef: string)` - Get the value of a cell
   - `getRange(rangeRef: string)` - Get a 2D array of values from a range

3. Replace the placeholder JSX with your spreadsheet library's component
4. Update the import in `SpreadsheetTab.tsx`
5. Update `package.json` with the new library dependencies

### Example for ExcelJS:

```typescript
// ExcelJSAdapter.tsx
import ExcelJS from 'exceljs';

export const ExcelJSAdapter = forwardRef<SpreadsheetRef, SpreadsheetProps>(
  ({ initialData, onCellChange, onFormulaIntercept, onSelectionChange }, ref) => {
    const workbook = useRef(new ExcelJS.Workbook());

    useImperativeHandle(ref, () => ({
      updateCell: (cellRef, value) => {
        const [col, row] = parseCellRef(cellRef);
        const worksheet = workbook.current.getWorksheet(1);
        worksheet.getCell(row, col).value = value.f || value.v;
      },
      getCellValue: (cellRef) => {
        const [col, row] = parseCellRef(cellRef);
        const worksheet = workbook.current.getWorksheet(1);
        return worksheet.getCell(row, col).value;
      },
      getRange: async (rangeRef) => {
        // Implement range extraction
        return [];
      }
    }));

    return <ExcelJSComponent workbook={workbook.current} />;
  }
);
```

## Adding New Examples

When you implement support for a new spreadsheet library, add the adapter here as an example for others to reference.