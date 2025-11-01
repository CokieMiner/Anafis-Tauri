// SpreadsheetEventBus.ts - Optimized type-safe event system for spreadsheet-sidebar communication

// Environment check for development mode using Vite's import.meta.env
const isDevelopment = import.meta.env.MODE === 'development';

type EventCallback<T = unknown> = (data: T) => void;

interface EventMap {
  'selection-change': string; // cellRef
  'range-selected': string; // rangeRef
}

type ErrorHandler = (error: unknown, event: keyof EventMap, listener: EventCallback) => void;

class SpreadsheetEventBus {
  // Type-erased internal storage for better flexibility - public API maintains type safety
  private listeners: Map<string, Set<EventCallback<unknown>>> = new Map();
  private isEmitting = false; // Prevent recursive emissions
  private emissionQueue: Array<{ event: keyof EventMap; data: EventMap[keyof EventMap] }> = [];
  private errorHandler: ErrorHandler | undefined;

  constructor(errorHandler?: ErrorHandler) {
    this.errorHandler = errorHandler;
  }

  /**
   * Set the global error handler for listener errors
   */
  setErrorHandler(handler: ErrorHandler): void {
    this.errorHandler = handler;
  }

  /**
   * Clear the global error handler
   */
  clearErrorHandler(): void {
    this.errorHandler = undefined;
  }

  /**
   * Subscribe to an event with automatic cleanup tracking
   */
  on<K extends keyof EventMap>(event: K, callback: (data: EventMap[K]) => void): () => void {
    const eventKey = event as string;
    if (!this.listeners.has(eventKey)) {
      this.listeners.set(eventKey, new Set());
    }
    
    this.listeners.get(eventKey)!.add(callback as EventCallback<unknown>);

    // Return memoized unsubscribe function for better performance
    return () => {
      this.off(event, callback);
    };
  }

  /**
   * Unsubscribe from an event with optimized cleanup
   */
  off<K extends keyof EventMap>(event: K, callback: (data: EventMap[K]) => void): void {
    const eventKey = event as string;
    const callbacks = this.listeners.get(eventKey);
    if (callbacks) {
      callbacks.delete(callback as EventCallback<unknown>);
      // Clean up empty event maps to prevent memory leaks
      if (callbacks.size === 0) {
        this.listeners.delete(eventKey);
      }
    }
  }

  /**
   * Emit an event with batching and error isolation
   */
  emit<K extends keyof EventMap>(event: K, data: EventMap[K]): void {
    // Prevent recursive emissions that could cause infinite loops
    if (this.isEmitting) {
      this.emissionQueue.push({ event, data });
      return;
    }

    const eventKey = event as string;
    const callbacks = this.listeners.get(eventKey);
    if (!callbacks || callbacks.size === 0) {
      return; // Early exit for performance
    }

    this.isEmitting = true;
    
    try {
      // Use for...of for better performance than forEach
      for (const callback of callbacks) {
        try {
          callback(data);
        } catch (error) {
          // Isolate errors to prevent one bad listener from breaking others
          if (this.errorHandler) {
            // Use custom error handler if provided
            this.errorHandler(error, event, callback);
          } else {
            // Use structured logging as fallback
            const errorMessage = `Error in event listener for ${String(event)}`;
            const errorDetails = {
              event: String(event),
              listener: `${callback.toString().slice(0, 100)}...`, // Truncate for readability
              stack: error instanceof Error ? error.stack : undefined,
              timestamp: new Date().toISOString(),
            };

            if (isDevelopment) {
              console.error(errorMessage, errorDetails);
            } else {
              // In production, log as warning to ensure visibility
              console.warn(errorMessage, errorDetails);
            }
          }
        }
      }
    } finally {
      this.isEmitting = false;
      
      // Process queued emissions
      if (this.emissionQueue.length > 0) {
        const queued = this.emissionQueue.splice(0);
        for (const { event: queuedEvent, data: queuedData } of queued) {
          this.emit(queuedEvent, queuedData);
        }
      }
    }
  }

  /**
   * Clear all listeners for an event or all events
   */
  clear(event?: keyof EventMap): void {
    if (event) {
      this.listeners.delete(event as string);
    } else {
      this.listeners.clear();
      this.emissionQueue.length = 0; // Clear queue as well
    }
  }

  /**
   * Get number of listeners for an event
   */
  listenerCount(event: keyof EventMap): number {
    return this.listeners.get(event as string)?.size ?? 0;
  }

  /**
   * Get total number of listeners across all events
   */
  totalListenerCount(): number {
    let total = 0;
    for (const callbacks of this.listeners.values()) {
      total += callbacks.size;
    }
    return total;
  }

  /**
   * Check if there are any listeners for an event
   */
  hasListeners(event: keyof EventMap): boolean {
    return (this.listeners.get(event as string)?.size ?? 0) > 0;
  }

  /**
   * Batch multiple emissions for better performance
   */
  batchEmit(emissions: Array<{ event: keyof EventMap; data: EventMap[keyof EventMap] }>): void {
    for (const { event, data } of emissions) {
      this.emit(event, data);
    }
  }
}

// Export singleton instance
export const spreadsheetEventBus = new SpreadsheetEventBus();

// Export class for testing
export { SpreadsheetEventBus };
