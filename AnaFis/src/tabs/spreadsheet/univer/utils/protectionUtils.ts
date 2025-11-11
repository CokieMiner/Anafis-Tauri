/**
 * Utility functions for applying protection rules to Univer workbooks
 */

import { ERROR_MESSAGES } from './constants';
import { ICommandService, IUniverInstanceService, IPermissionService } from '@univerjs/core';
import { SpreadsheetOperationError } from './errors';
export interface ProtectionResource {
  name: string;
  data: string;
}

export interface ProtectionData {
  worksheetProtections?: Record<string, unknown>;
  rangeProtections?: Record<string, unknown[]>;
}

/**
 * Applies protection rules from resources to a Univer workbook instance
 * @param resources Array of protection resources containing serialized protection data
 * @param univerInstance The Univer instance to apply protection to
 * @param signal AbortSignal for cancellation support
 * @returns Promise that resolves when protection rules are applied (best-effort)
 */
export async function applyProtectionRules(
  resources: ProtectionResource[],
  univerInstance: unknown,
  signal: AbortSignal
): Promise<void> {
  try {
    const {
      AddWorksheetProtectionMutation,
      AddRangeProtectionMutation,
      WorksheetEditPermission,
      RangeProtectionPermissionEditPoint
    } = await import('@univerjs/sheets');

    interface UniverseInstance {
      __getInjector: () => { get: (service: unknown) => unknown };
    }

    const injector = (univerInstance as UniverseInstance).__getInjector();
    const commandService = injector.get(ICommandService) as {
      executeCommand: (commandId: string, params: unknown) => Promise<boolean>
    };
    const instanceService = injector.get(IUniverInstanceService) as{
      getFocusedUnit: () => { getUnitId: () => string } | null;
    };
    const permissionService = injector.get(IPermissionService) as {
      updatePermissionPoint: (pointId: string, value: boolean) => void;
    };

    // Validate focused workbook exists and get unitId
    const workbook = instanceService.getFocusedUnit();
    if (!workbook) {
      throw new SpreadsheetOperationError(
        'applyProtectionRules',
        new Error(ERROR_MESSAGES.NO_ACTIVE_WORKBOOK_PROTECTION),
        { operation: 'getFocusedUnit' }
      );
    }

    const expectedUnitId = workbook.getUnitId();
    let protectionCount = 0;

    // Check for cancellation before starting
    if (signal.aborted) {
      throw new SpreadsheetOperationError(
        'applyProtectionRules',
        new Error(ERROR_MESSAGES.PROTECTION_APPLICATION_CANCELLED),
        { operation: 'checkAbortSignal', aborted: signal.aborted }
      );
    }

    for (const resource of resources) {
      if (resource.name.includes('PROTECTION') || resource.name.includes('PERMISSION')) {
        try {
          const protectionData = JSON.parse(resource.data || '{}') as ProtectionData;

          // Re-validate workbook and unitId before each resource to avoid stale references
          const currentWorkbook = instanceService.getFocusedUnit();
          if (!currentWorkbook || currentWorkbook.getUnitId() !== expectedUnitId) {
            console.warn('⚠️ Workbook changed during protection application, skipping protection');
            continue;
          }

          if (protectionData.worksheetProtections) {
            for (const [sheetId, protection] of Object.entries(protectionData.worksheetProtections)) {
              if (typeof protection === 'object' && protection !== null) {
                try {
                  const success = await commandService.executeCommand(AddWorksheetProtectionMutation.id, {
                    unitId: expectedUnitId,
                    subUnitId: sheetId,
                    rule: protection,
                  });

                  if (success) {
                    const editPermission = new WorksheetEditPermission(expectedUnitId, sheetId);
                    permissionService.updatePermissionPoint(editPermission.id, false);
                    protectionCount++;
                  } else {
                    console.warn(`⚠️ Worksheet protection command returned false for ${sheetId}, may not be supported`);
                  }
                } catch (cmdError) {
                  console.warn(`⚠️ Failed to execute worksheet protection command for ${sheetId}:`, cmdError);
                }
              }
            }
          }

          if (protectionData.rangeProtections) {
            for (const [sheetId, protections] of Object.entries(protectionData.rangeProtections)) {
              if (Array.isArray(protections)) {
                for (const protection of protections) {
                  // Re-validate workbook before each command
                  const currentWorkbook = instanceService.getFocusedUnit();
                  if (!currentWorkbook || currentWorkbook.getUnitId() !== expectedUnitId) {
                    console.warn('⚠️ Workbook changed during protection application, skipping this protection');
                    continue;
                  }

                  try {
                    const success = await commandService.executeCommand(AddRangeProtectionMutation.id, {
                      unitId: expectedUnitId,
                      subUnitId: sheetId,
                      rules: [protection],
                    });

                    if (success) {
                      const protectionRule = protection as { permissionId: string; name?: string };
                      const editPermission = new RangeProtectionPermissionEditPoint(
                        expectedUnitId,
                        sheetId,
                        protectionRule.permissionId
                      );
                      permissionService.updatePermissionPoint(editPermission.id, false);
                      protectionCount++;
                    } else {
                      console.warn(`⚠️ Range protection command returned false for ${sheetId}, may not be supported`);
                    }
                  } catch (cmdError) {
                    console.warn(`⚠️ Failed to execute range protection command for ${sheetId}:`, cmdError);
                  }
                }
              }
            }
          }

          if (protectionCount > 0) {
            // Successfully applied protection rules
          } else {
            console.warn(`⚠️ No protection rules applied from ${resource.name}`);
          }
        } catch (error) {
          console.warn(`⚠️ Error processing resource "${resource.name}": ${error instanceof Error ? error.message : String(error)}`);
          // Don't throw - continue with next resource
        }
      }
    }

    if (protectionCount === 0) {
      console.warn('⚠️ No protection rules were successfully applied from any resource');
    }
  } catch (error) {
    console.error(`❌ Protection application error: ${error instanceof Error ? error.message : String(error)}`);
    // Don't re-throw - let this be a best-effort operation
  }
}