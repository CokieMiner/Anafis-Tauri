# Univer Implementation Research & Findings

## 1. Goal
The objective was to analyze the architecture, recommended design patterns, and documentation of the official Univer repositories and compare them against our `AnaFis` implementation.

## 2. Repositories Explored
We cloned and analyzed several repositories from the `dream-num` organization, focusing on:
- `univer`
- `univer-sheet-start-kit`
- `univer-sdk-skills` (containing plugin architecture and facade API guides)
- `Luckysheet` (predecessor to Univer)

## 3. Key Findings from Univer Official Documentation

### A. Facade API
Univer heavily promotes using the **Facade API** (`FUniver`, `FWorkbook`, `FWorksheet`, `FRange`) over their internal Command/Mutation core infrastructure.
- The Facade API provides a high-level, Google AppScript-like interface.
- It requires **side-effect imports** (e.g., `import '@univerjs/sheets/facade';`) to inject methods into the classes. Without these, methods like `.setBackground` will throw an error.
- Hook support: The Facade API now provides hooks such as `Hooks.onCellChange` for listening to spreadsheet events, replacing the need to listen to low-level mutations.

### B. Setup & Initialization (Presets System)
The official `univer-sheet-start-kit` demonstrates initializing Univer using a **Presets System** (`@univerjs/presets`).
- Instead of manually registering every single plugin (`UniverSheetsPlugin`, `UniverSheetsUIPlugin`, etc.), they now use grouped presets:
  - `UniverSheetsCorePreset()`
  - `UniverSheetsAdvancedPreset()`
  - `UniverSheetsConditionalFormattingPreset()`
- This severely reduces boilerplate and initialization errors.

### C. Plugin Architecture
For custom plugins, Univer uses Dependency Injection (DI) through an `Injector`. 
- Custom logic should inherit from the `Plugin` class.
- Lifecycle methods (`onStarting`, `onReady`, `onRendered`, `onSteady`) control initialization order.
- They recommend the Controller pattern for organizing plugin logic, utilizing `Disposable` and `disposeWithMe()` to prevent memory leaks.

## 4. Comparison Against AnaFis Implementation

### What We Are Doing Right:
1. **Facade API Migration**: We have successfully migrated the bulk of our operations to the Facade API. `AnaFis/src/tabs/spreadsheet/univer/operations/facadeOperations.ts` properly uses `FUniver`, `FWorkbook`, etc., wrapped in our own `safeSpreadsheetOperation` handler.
2. **Worker Offloading**: Our `DataMapperClient` uses Web Workers to process large datasets without blocking the UI thread. This is an advanced and highly performant pattern.
3. **Sequential Operation Queue**: Our `SequentialSpreadsheetQueue` avoids race conditions when multiple Facade API calls are triggered simultaneously.

### Areas for Improvement / Technical Debt:
1. **Manual Plugin Registration**: In `UniverSpreadsheet.tsx`, we manually register dozens of core and UI plugins instead of using the new `@univerjs/presets`. Migrating to Presets would greatly simplify this file.
2. **Low-Level Command Listening**: In `UniverSpreadsheet.tsx`, we use `commandService.onCommandExecuted` to track cell changes and intercept formulas (listening to `sheet.mutation.set-range-values`). Based on `facade-api-guide.md`, this could be rewritten using the Facade API's `hooks.onCellChange()`, resulting in cleaner and more maintainable code.
3. **Dead Code**: `UniverSpreadsheet.tsx` exports an imperative handle (`UniverSpreadsheetRef`) containing outdated `updateCell`, `getCellValue`, and `getRange` methods that use internal commands (e.g., `sheet.command.set-range-values`). However, our `UniverAdapter.tsx` actually calls `facadeOperations.ts` instead of these methods. These outdated methods in `UniverSpreadsheetRef` should be removed.

## 5. Design Patterns for Future Reference

If we ever need to build a heavy custom integration (like our custom scientific formula engine):
- Avoid writing directly to the `IWorkbookData` internal format if possible.
- If creating a custom tool for Univer, implement it as a standard Univer `Plugin` using their Dependency Injection (`@Inject(Injector)`).
- For large data updates where performance is critical, bypassing the Facade API is acceptable *only* if we insert purely formulas (which we already optimized in `isFormulaOnlyCell`) or by batching operations via Web Workers.

## 6. Conclusion
Our `AnaFis` spreadsheet integration is highly robust and performs well, leveraging advanced patterns like Web Workers and queues. However, as Univer evolves, we should plan a minor refactoring to adopt their `@univerjs/presets` system and switch from `ICommandService` event listeners to Facade API Hooks.
