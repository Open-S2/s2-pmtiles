import type { Entry } from './pmtiles';

/**
 * A cache of directories.
 * The key is the offset in the data and the value is the directory entries.
 */
export default class DirCache<K = number, V = Entry[]> extends Map<K, V> {
  order: K[] = [];
  /**
   * @param maxSize - the max size of the cache before dumping old data
   */
  constructor(private readonly maxSize: number) {
    super();
  }

  /**
   * @param key - the offset position in the data
   * @param dir - the directory entries
   * @returns this
   */
  set(key: K, dir: V): this {
    // if key exists, we just update the place in the array
    if (super.has(key)) this.order.splice(this.order.indexOf(key), 1);
    // add the key to the start of the array
    this.order.unshift(key);
    while (this.order.length > this.maxSize) this.delete(this.order.pop() as K);

    return super.set(key, dir);
  }

  /**
   * @param key - the offset position in the data
   * @returns - the directories entries if found
   */
  get(key: K): V | undefined {
    // update the place in the array and than get
    if (super.has(key)) {
      this.order.splice(this.order.indexOf(key), 1);
      this.order.unshift(key);
    }
    return super.get(key);
  }

  /**
   * @param key - the offset position in the data
   * @returns - true if found
   */
  delete(key: K): boolean {
    return super.delete(key);
  }
}
