// performance.ts - Performance monitoring utilities for spreadsheet operations

/**
 * Performance monitoring utility wrapper for operations
 * Logs execution time in development mode
 * 
 * @example
 * const monitoredFn = withPerformanceMonitoring(myFunction, 'My Operation');
 * monitoredFn(args); // Logs: [Performance] My Operation completed in 123.45ms
 */
export function withPerformanceMonitoring<T extends (...args: unknown[]) => unknown>(
  fn: T,
  operationName: string
): T {
  return ((...args: Parameters<T>) => {
    const startTime = performance.now();

    try {
      const result = fn(...args);

      // Handle both sync and async functions
      if (result instanceof Promise) {
        return result
          .then((res: Awaited<ReturnType<T>>) => {
            const endTime = performance.now();
            if (process.env.NODE_ENV === 'development') {
              console.log(`[Performance] ${operationName} completed in ${(endTime - startTime).toFixed(2)}ms`);
            }
            return res;
          })
          .catch((err: unknown) => {
            const endTime = performance.now();
            if (process.env.NODE_ENV === 'development') {
              console.error(`[Performance] ${operationName} failed after ${(endTime - startTime).toFixed(2)}ms:`, err);
            }
            throw err;
          });
      } else {
        const endTime = performance.now();
        if (process.env.NODE_ENV === 'development') {
          console.log(`[Performance] ${operationName} took ${(endTime - startTime).toFixed(2)}ms`);
        }
        return result;
      }
    } catch (error) {
      const endTime = performance.now();
      if (process.env.NODE_ENV === 'development') {
        console.error(`[Performance] ${operationName} failed after ${(endTime - startTime).toFixed(2)}ms:`, error);
      }
      throw error;
    }
  }) as T;
}
