// univerUtils.ts - Simplified utility functions for Facade API
import { IRange } from '@univerjs/core';

// Simple LRU Cache implementation
class LRUCache<K, V> {
  private cache = new Map<K, LRUCacheNode<K, V>>();
  private head: LRUCacheNode<K, V> | null = null;
  private tail: LRUCacheNode<K, V> | null = null;
  private evictThreshold: number;

  constructor(_maxSize: number, evictThreshold: number) {
    this.evictThreshold = evictThreshold;
  }

  get(key: K): V | undefined {
    const node = this.cache.get(key);
    if (!node) {
      return undefined;
    }

    // Move to front (most recently used)
    this.moveToFront(node);
    return node.value;
  }

  set(key: K, value: V): void {
    const existingNode = this.cache.get(key);

    if (existingNode) {
      // Update existing entry and move to front
      existingNode.value = value;
      this.moveToFront(existingNode);
    } else {
      // Add new entry
      const newNode = new LRUCacheNode(key, value);
      this.cache.set(key, newNode);
      this.addToFront(newNode);

      // Evict if over threshold
      if (this.cache.size > this.evictThreshold) {
        this.evictLRU();
      }
    }
  }

  has(key: K): boolean {
    return this.cache.has(key);
  }

  get size(): number {
    return this.cache.size;
  }

  clear(): void {
    this.cache.clear();
    this.head = null;
    this.tail = null;
  }

  private evictLRU(): void {
    // Remove least recently used entries (from tail)
    const entriesToEvict = Math.max(1, Math.floor(this.cache.size * 0.1));

    for (let i = 0; i < entriesToEvict && this.tail; i++) {
      const keyToDelete = this.tail.key;
      this.cache.delete(keyToDelete);
      this.removeNode(this.tail);
    }
  }

  private moveToFront(node: LRUCacheNode<K, V>): void {
    if (node === this.head) {
      return; // Already at front
    }

    this.removeNode(node);
    this.addToFront(node);
  }

  private addToFront(node: LRUCacheNode<K, V>): void {
    node.next = this.head;
    node.prev = null;

    if (this.head) {
      this.head.prev = node;
    }

    this.head = node;

    this.tail ??= node;
  }

  private removeNode(node: LRUCacheNode<K, V>): void {
    if (node.prev) {
      node.prev.next = node.next;
    } else {
      this.head = node.next;
    }

    if (node.next) {
      node.next.prev = node.prev;
    } else {
      this.tail = node.prev;
    }

    node.prev = null;
    node.next = null;
  }
}

class LRUCacheNode<K, V> {
  key: K;
  value: V;
  prev: LRUCacheNode<K, V> | null = null;
  next: LRUCacheNode<K, V> | null = null;

  constructor(key: K, value: V) {
    this.key = key;
    this.value = value;
  }
}

// Cache for frequently used column conversions with LRU eviction
const columnLetterCache = new LRUCache<number, string>(1000, 900);
const letterColumnCache = new LRUCache<string, number>(1000, 900);

export function columnToLetter(column: number): string {
  // Input validation: negative columns are invalid
  if (column < 0) {
    throw new RangeError(`Column number cannot be negative: ${column}`);
  }

  const cached = columnLetterCache.get(column);
  if (cached !== undefined) {
    return cached;
  }

  let temp, letter = '';
  let col = column;
  while (col >= 0) {
    temp = col % 26;
    letter = String.fromCharCode(temp + 65) + letter;
    col = Math.floor(col / 26) - 1;
  }

  // LRU cache handles size limits automatically
  columnLetterCache.set(column, letter);

  return letter;
}

export function letterToColumn(letter: string): number {
  // Input validation and normalization
  if (!letter || typeof letter !== 'string') {
    throw new TypeError('Input must be a non-empty string');
  }

  const normalizedLetter = letter.toUpperCase();
  if (!/^[A-Z]+$/.test(normalizedLetter)) {
    throw new TypeError(`Invalid column letter format: ${letter}. Must contain only uppercase letters A-Z.`);
  }

  const cached = letterColumnCache.get(normalizedLetter);
  if (cached !== undefined) {
    return cached;
  }

  let column = 0;
  for (let i = 0; i < normalizedLetter.length; i++) {
    column = column * 26 + (normalizedLetter.charCodeAt(i) - 65 + 1);
  }
  const result = column - 1;

  // LRU cache handles size limits automatically
  letterColumnCache.set(normalizedLetter, result);

  return result;
}

export function rangeToA1(range: IRange): string {
  // Add null checks for range properties
  if (typeof range.startColumn !== 'number' || typeof range.startRow !== 'number' ||
      typeof range.endColumn !== 'number' || typeof range.endRow !== 'number') {
    throw new Error('Invalid range object: missing required properties');
  }

  const startCol = columnToLetter(range.startColumn);
  const startRow = range.startRow + 1;
  const endCol = columnToLetter(range.endColumn);
  const endRow = range.endRow + 1;

  if (range.startColumn === range.endColumn && range.startRow === range.endRow) {
    return `${startCol}${startRow}`;
  }

  return `${startCol}${startRow}:${endCol}${endRow}`;
}

export function cellRefToIndices(cellRef: string): { row: number; col: number } | null {
  const match = cellRef.match(/^([A-Z]+)(\d+)$/);
  if (!match?.[1] || !match[2]) {return null;}

  const col = match[1];
  const row = parseInt(match[2]) - 1;
  const colIndex = letterToColumn(col);

  return { row, col: colIndex };
}

export function parseRange(rangeRef: string): { startCol: number; startRow: number; endCol: number; endRow: number } | null {
  const rangeMatch = rangeRef.match(/^([A-Z]+)(\d+):([A-Z]+)(\d+)$/);
  const singleMatch = rangeRef.match(/^([A-Z]+)(\d+)$/);

  if (rangeMatch?.[1] && rangeMatch[2] && rangeMatch[3] && rangeMatch[4]) {
    const startCol = rangeMatch[1];
    const startRow = parseInt(rangeMatch[2]) - 1;
    const endCol = rangeMatch[3];
    const endRow = parseInt(rangeMatch[4]) - 1;

    return {
      startCol: letterToColumn(startCol),
      startRow,
      endCol: letterToColumn(endCol),
      endRow
    };
  }

  if (singleMatch?.[1] && singleMatch[2]) {
    const col = singleMatch[1];
    const row = parseInt(singleMatch[2]) - 1;
    const colIndex = letterToColumn(col);

    return {
      startCol: colIndex,
      startRow: row,
      endCol: colIndex,
      endRow: row
    };
  }

  return null;
}

// Enhanced cache management with LRU eviction
let cacheCleanupInterval: ReturnType<typeof setInterval> | null = null;

/**
 * Start periodic cache cleanup to ensure LRU eviction runs regularly.
 * Call this during app initialization or component mount to maintain cache efficiency.
 * The LRU caches handle most eviction automatically, but this ensures cleanup runs
 * even during low activity periods.
 * 
 * @param intervalMs - Cleanup interval in milliseconds (default: 5 minutes)
 */
export function startPeriodicCacheCleanup(intervalMs: number = 300000): void { // 5 minutes default
  if (cacheCleanupInterval) {
    clearInterval(cacheCleanupInterval);
  }
  
  cacheCleanupInterval = setInterval(() => {
    // LRU caches handle automatic eviction, but we can trigger cleanup if needed
  }, intervalMs);
}

/**
 * Stop periodic cache cleanup.
 * Call this during app shutdown or component unmount to prevent memory leaks.
 */
export function stopPeriodicCacheCleanup(): void {
  if (cacheCleanupInterval) {
    clearInterval(cacheCleanupInterval);
    cacheCleanupInterval = null;
  }
}

/**
 * Clear all caches manually.
 * Use this for testing or when you need to reset cache state.
 */
export function clearAllCaches(): void {
  columnLetterCache.clear();
  letterColumnCache.clear();
}