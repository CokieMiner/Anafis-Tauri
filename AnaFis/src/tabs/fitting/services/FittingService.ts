import { invoke } from '@tauri-apps/api/core';
import type {
  OdrFitRequest,
  OdrFitResponse,
} from '@/tabs/fitting/types/fittingTypes';

/**
 * Service to handle communications with the Rust Fitting backend.
 * Decoupled from React to allow for easier testing and reuse.
 */
export const FittingService = {
  /**
   * Executes a Custom ODR Fit on the backend
   */
  async runCustomOdr(request: OdrFitRequest): Promise<OdrFitResponse> {
    try {
      const response = await invoke<OdrFitResponse>('fit_custom_odr', {
        request,
      });
      return response;
    } catch (error) {
      console.error('[FittingService] Fit execution failed:', error);
      throw error;
    }
  },

  /**
   * Checks if a fit response has enough data to be displayed
   */
  isValidResult(response: OdrFitResponse): boolean {
    return (
      response.success &&
      response.parameterValues.length > 0 &&
      response.fittedValues.length > 0
    );
  },
};
