// UniverSpreadsheet.tsx - High-stability Univer integration with Presets & Facade API

import { useTheme } from '@mui/material';
import {
  type ICommandInfo,
  type IRange,
  type IWorkbookData,
  type Univer,
  UniverInstanceType,
} from '@univerjs/core';
import {
  createUniver,
  defaultTheme,
  LocaleType,
  LogLevel,
  merge,
} from '@univerjs/presets';
// Extra Free Features
import { UniverSheetsConditionalFormattingPreset } from '@univerjs/presets/preset-sheets-conditional-formatting';
import sheetsConditionalFormattingEnUs from '@univerjs/presets/preset-sheets-conditional-formatting/locales/en-US';
// Extra Free Features
import { UniverSheetsCorePreset } from '@univerjs/presets/preset-sheets-core';
import sheetsCoreEnUs from '@univerjs/presets/preset-sheets-core/locales/en-US';
import { UniverSheetsDataValidationPreset } from '@univerjs/presets/preset-sheets-data-validation';
import sheetsDataValidationEnUs from '@univerjs/presets/preset-sheets-data-validation/locales/en-US';
import { UniverSheetsFilterPreset } from '@univerjs/presets/preset-sheets-filter';
import sheetsFilterEnUs from '@univerjs/presets/preset-sheets-filter/locales/en-US';
import { UniverSheetsFindReplacePreset } from '@univerjs/presets/preset-sheets-find-replace';
import sheetsFindReplaceEnUs from '@univerjs/presets/preset-sheets-find-replace/locales/en-US';
import { UniverSheetsHyperLinkPreset } from '@univerjs/presets/preset-sheets-hyper-link';
import sheetsHyperLinkEnUs from '@univerjs/presets/preset-sheets-hyper-link/locales/en-US';
import { UniverSheetsSortPreset } from '@univerjs/presets/preset-sheets-sort';
import sheetsSortEnUs from '@univerjs/presets/preset-sheets-sort/locales/en-US';

import { useEffect, useRef } from 'react';

// Styles
import '@univerjs/presets/lib/styles/preset-sheets-core.css';
import '@univerjs/presets/lib/styles/preset-sheets-filter.css';
import '@univerjs/presets/lib/styles/preset-sheets-find-replace.css';
import '@univerjs/presets/lib/styles/preset-sheets-conditional-formatting.css';
import '@univerjs/presets/lib/styles/preset-sheets-data-validation.css';
import '@univerjs/presets/lib/styles/preset-sheets-sort.css';
import '@univerjs/presets/lib/styles/preset-sheets-hyper-link.css';

import '@univerjs/ui/lib/index.css';
import '@univerjs/sheets-ui/lib/index.css';
import '@univerjs/sheets-formula-ui/lib/index.css';
import '@univerjs/sheets-numfmt-ui/lib/index.css';
import '@univerjs/design/lib/index.css';

// Facade APIs
import '@univerjs/sheets/facade';
import '@univerjs/sheets-formula/facade';
import '@univerjs/sheets-numfmt/facade';
import '@univerjs/engine-formula/facade';

import { ICommandService } from '@univerjs/core';
import { IRegisterFunctionService } from '@univerjs/sheets-formula';
import { registerCustomFunctions } from '@/tabs/spreadsheet/univer/index';
import { rangeToA1 } from '@/tabs/spreadsheet/univer/utils/univerUtils';

interface Props {
  initialData: IWorkbookData;
  onSelectionChange?: (cellRef: string) => void;
  onUniverReady?: (univerInstance: Univer) => void;
  tabId?: string;
}

const UniverSpreadsheet = ({
  initialData,
  onSelectionChange,
  onUniverReady,
  tabId,
}: Props) => {
  const theme = useTheme();
  const containerRef = useRef<HTMLDivElement>(null);
  const univerRef = useRef<Univer | null>(null);
  const isInitializedRef = useRef(false);

  // Stable references for callbacks to prevent re-registration loops
  const callbacks = useRef({
    onSelectionChange,
    onUniverReady,
  });

  useEffect(() => {
    callbacks.current = {
      onSelectionChange,
      onUniverReady,
    };
  }, [onSelectionChange, onUniverReady]);

  const containerId = useRef(
    tabId
      ? `univer-container-${tabId}`
      : `univer-container-${Math.random().toString(36).substring(2, 11)}`
  ).current;

  useEffect(() => {
    if (!containerRef.current || isInitializedRef.current) return;

    let isDisposed = false;
    let initTimer: ReturnType<typeof setTimeout> | null = null;
    const hookDisposables: Array<{ dispose: () => void }> = [];

    window.__UNIVER_INSTANCES__ ??= new Set();

    // SAFETY INIT: 50ms buffer prevents HMR race conditions
    initTimer = setTimeout(() => {
      if (isDisposed || isInitializedRef.current) return;

      isInitializedRef.current = true;
      window.__UNIVER_INSTANCES__?.add(containerId);

      try {
        const { univer } = createUniver({
          // Force dark mode since the application is always dark
          // Also apply defaultTheme to ensure UI elements are styled properly
          theme: defaultTheme,
          darkMode: true,
          locale: LocaleType.EN_US,
          locales: {
            [LocaleType.EN_US]: merge(
              {},
              sheetsCoreEnUs,
              sheetsFilterEnUs,
              sheetsFindReplaceEnUs,
              sheetsConditionalFormattingEnUs,
              sheetsDataValidationEnUs,
              sheetsSortEnUs,
              sheetsHyperLinkEnUs
            ),
          },
          logLevel: LogLevel.VERBOSE,
          presets: [
            UniverSheetsCorePreset({
              container: containerId,
              ribbonType: 'collapsed',
              header: true,
            }),
            UniverSheetsFilterPreset(),
            UniverSheetsFindReplacePreset(),
            UniverSheetsConditionalFormattingPreset(),
            UniverSheetsDataValidationPreset(),
            UniverSheetsSortPreset(),
            UniverSheetsHyperLinkPreset(),
          ],
        });

        univerRef.current = univer;
        univer.createUnit(UniverInstanceType.UNIVER_SHEET, initialData);

        // Formula Engine Setup
        const injector = univer.__getInjector();
        registerCustomFunctions(injector.get(IRegisterFunctionService));

        // Global Command Listener for robust Cell Changes across all sheets
        const commandService = injector.get(ICommandService);
        const commandDisposable = commandService.onCommandExecuted(
          (command: ICommandInfo) => {
            if (isDisposed) return;

            // Handle Selection fallback if Facade hook didn't catch it
            if (command.id === 'sheet.operation.set-selections') {
              const params = command.params as {
                selections?: Array<{ range: IRange }>;
              };
              const range = params?.selections?.[0]?.range;
              if (range) {
                callbacks.current.onSelectionChange?.(rangeToA1(range));
              }
            }
          }
        );
        hookDisposables.push(commandDisposable);

        if (callbacks.current.onUniverReady) {
          callbacks.current.onUniverReady(univer);
        }
      } catch (error) {
        console.error('[Univer] Init failed:', error);
        isInitializedRef.current = false;
      }
    }, 50);

    return () => {
      isDisposed = true;
      if (initTimer) clearTimeout(initTimer);

      hookDisposables.forEach((d) => {
        d.dispose();
      });

      if (univerRef.current) {
        const u = univerRef.current;
        setTimeout(() => u.dispose(), 0);
        univerRef.current = null;
      }

      window.__UNIVER_INSTANCES__?.delete(containerId);
      isInitializedRef.current = false;
    };
  }, [initialData, containerId]);

  return (
    <div
      ref={containerRef}
      id={containerId}
      className="univer-spreadsheet-container"
      style={{
        width: '100%',
        height: '100%',
        minHeight: '400px',
        backgroundColor: theme.palette.background.paper,
      }}
    />
  );
};

UniverSpreadsheet.displayName = 'UniverSpreadsheet';
export default UniverSpreadsheet;
