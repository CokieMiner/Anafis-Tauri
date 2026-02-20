// SequentialSpreadsheetQueue.ts - Simple sequential queue for spreadsheet operations
// Prevents race conditions by processing operations one at a time with debouncing

export interface QueuedOperation<T = void> {
  id: string;
  operation: () => Promise<T>;
  resolve: (value: T | PromiseLike<T>) => void;
  reject: (error: unknown) => void;
  timestamp: number;
}

/**
 * Simple sequential queue for spreadsheet operations.
 * Processes operations one at a time to prevent race conditions.
 * Includes debouncing to batch rapid consecutive operations.
 */
export class SequentialSpreadsheetQueue {
  private queue: QueuedOperation<unknown>[] = [];
  private isProcessing = false;
  private debounceTimer: number | undefined = undefined;
  private readonly debounceDelay: number;

  constructor(debounceDelay = 50) {
    this.debounceDelay = debounceDelay;
  }

  /**
   * Add an operation to the queue.
   * Returns a promise that resolves when the operation completes.
   */
  enqueue<T = void>(operation: () => Promise<T>): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      const queuedOp: QueuedOperation<unknown> = {
        id: `op_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        operation: operation as () => Promise<unknown>,
        resolve: resolve as (value: unknown) => void,
        reject,
        timestamp: Date.now(),
      };

      this.queue.push(queuedOp);

      // If not currently processing, start processing
      if (!this.isProcessing) {
        this.scheduleProcessing();
      }
    });
  }

  /**
   * Schedule processing with debouncing.
   * This allows rapid consecutive operations to be batched.
   */
  private scheduleProcessing(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    this.debounceTimer = setTimeout(() => {
      void this.processQueue();
    }, this.debounceDelay);
  }

  /**
   * Process the queue sequentially.
   */
  private async processQueue(): Promise<void> {
    if (this.isProcessing || this.queue.length === 0) {
      return;
    }

    this.isProcessing = true;

    while (this.queue.length > 0) {
      const operation = this.queue.shift();
      if (!operation) continue;
      try {
        const result = await operation.operation();
        operation.resolve(result);
      } catch (error) {
        operation.reject(error);
      }
    }

    this.isProcessing = false;
  }

  /**
   * Get current queue status for debugging/monitoring.
   */
  getStatus() {
    return {
      queueLength: this.queue.length,
      isProcessing: this.isProcessing,
      debounceActive: !!this.debounceTimer,
    };
  }

  /**
   * Clear all pending operations.
   * Useful for cleanup or when operations become invalid.
   */
  clear(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = undefined;
    }

    // Reject all pending operations
    for (const operation of this.queue) {
      operation.reject(new Error('Queue cleared'));
    }

    this.queue = [];
    this.isProcessing = false;
  }

  /**
   * Wait for all current operations to complete.
   * Useful for ensuring state consistency before proceeding.
   */
  async waitForCompletion(): Promise<void> {
    if (this.queue.length === 0 && !this.isProcessing) {
      return;
    }

    return new Promise<void>((resolve) => {
      const checkComplete = () => {
        if (this.queue.length === 0 && !this.isProcessing) {
          resolve();
        } else {
          setTimeout(checkComplete, 10);
        }
      };
      checkComplete();
    });
  }
}
