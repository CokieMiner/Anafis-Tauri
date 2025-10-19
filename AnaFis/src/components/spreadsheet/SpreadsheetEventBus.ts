// SpreadsheetEventBus.ts - Type-safe event system for spreadsheet-sidebar communication

type EventCallback = (data: any) => void;

interface EventMap {
  'selection-change': string; // cellRef
  'range-selected': string; // rangeRef
}

class SpreadsheetEventBus {
  private listeners: Map<keyof EventMap, Set<EventCallback>> = new Map();

  /**
   * Subscribe to an event
   */
  on<K extends keyof EventMap>(event: K, callback: (data: EventMap[K]) => void): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    
    this.listeners.get(event)!.add(callback as EventCallback);

    // Return unsubscribe function
    return () => {
      this.off(event, callback as EventCallback);
    };
  }

  /**
   * Unsubscribe from an event
   */
  off<K extends keyof EventMap>(event: K, callback: EventCallback): void {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      callbacks.delete(callback);
      if (callbacks.size === 0) {
        this.listeners.delete(event);
      }
    }
  }

  /**
   * Emit an event
   */
  emit<K extends keyof EventMap>(event: K, data: EventMap[K]): void {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      callbacks.forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          console.error(`Error in event listener for ${event}:`, error);
        }
      });
    }
  }

  /**
   * Clear all listeners for an event or all events
   */
  clear(event?: keyof EventMap): void {
    if (event) {
      this.listeners.delete(event);
    } else {
      this.listeners.clear();
    }
  }

  /**
   * Get number of listeners for an event
   */
  listenerCount(event: keyof EventMap): number {
    return this.listeners.get(event)?.size ?? 0;
  }
}

// Export singleton instance
export const spreadsheetEventBus = new SpreadsheetEventBus();
