// SpreadsheetEventBus.ts - Simple event system for spreadsheet-sidebar communication

type EventCallback<T = unknown> = (data: T) => void;

interface EventMap {
  'selection-change': string; // cellRef
  'range-selected': string; // rangeRef
}

class SpreadsheetEventBus {
  private listeners = new Map<string, Set<EventCallback>>();

  /**
   * Subscribe to an event
   */
  on<K extends keyof EventMap>(event: K, callback: (data: EventMap[K]) => void): () => void {
    const eventKey = event as string;
    if (!this.listeners.has(eventKey)) {
      this.listeners.set(eventKey, new Set());
    }

    this.listeners.get(eventKey)!.add(callback as EventCallback);

    return () => this.off(event, callback);
  }

  /**
   * Unsubscribe from an event
   */
  off<K extends keyof EventMap>(event: K, callback: (data: EventMap[K]) => void): void {
    const eventKey = event as string;
    const callbacks = this.listeners.get(eventKey);
    if (callbacks) {
      callbacks.delete(callback as EventCallback);
      if (callbacks.size === 0) {
        this.listeners.delete(eventKey);
      }
    }
  }

  /**
   * Emit an event
   */
  emit<K extends keyof EventMap>(event: K, data: EventMap[K]): void {
    const eventKey = event as string;
    const callbacks = this.listeners.get(eventKey);
    if (!callbacks) {return;}

    for (const callback of callbacks) {
      try {
        callback(data);
      } catch (error) {
        console.error(`Error in event listener for ${event}:`, error);
      }
    }
  }

  /**
   * Clear all listeners
   */
  clear(): void {
    this.listeners.clear();
  }
}

// Export singleton instance
export const spreadsheetEventBus = new SpreadsheetEventBus();

// Export class for testing
export { SpreadsheetEventBus };
