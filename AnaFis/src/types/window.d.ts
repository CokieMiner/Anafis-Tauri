/**
 * Global window augmentation for Univer instance tracking
 */

declare global {
  interface Window {
    /**
     * Univer instance tracking (prevents multiple instances)
     */
    __UNIVER_INSTANCES__?: Set<string>;
  }
}

export {};
