import { getCurrentWindow } from '@tauri-apps/api/window';

const WINDOW_READY_EVENT = 'anafis://ready';

export function notifyWindowReady(): void {
  requestAnimationFrame(() => {
    void (async () => {
      try {
        await getCurrentWindow().emit(WINDOW_READY_EVENT);
      } catch {
        // No-op outside Tauri runtime.
      }
    })();
  });
}
