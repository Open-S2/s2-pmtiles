import { describe, expect, test } from 'bun:test';

import DirCache from '../src/cache';

describe('dirCache', () => {
  const dirCache = new DirCache<number, number>(5);

  test('test functionality', () => {
    expect(dirCache.set(1, 2)).toBe(dirCache);
    expect(dirCache.get(1)).toEqual(2);
    expect(dirCache.delete(1)).toBe(true);
  });

  test('test max size', () => {
    dirCache.set(1, 2);
    dirCache.set(2, 3);
    dirCache.set(3, 4);
    dirCache.set(4, 5);
    dirCache.set(5, 6);
    dirCache.set(6, 7);
    dirCache.set(7, 8);

    expect(dirCache.size).toBe(5);
    expect(dirCache.get(2)).toEqual(undefined as unknown as number);
    expect(dirCache.get(3)).toEqual(4);
  });
});
