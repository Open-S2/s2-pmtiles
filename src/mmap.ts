import type { Reader } from './reader';

/** The Memory Mapped reader is to be used by bun on the local filesystem. */
export class MMapReader implements Reader {
  #buffer: Uint8Array;

  /** @param file - the location of the PMTiles data in the FS */
  constructor(readonly file: string) {
    this.#buffer = Bun.mmap(file);
  }

  /**
   * @param offset - the offset of the range
   * @param length - the length of the range
   * @returns - the ranged buffer
   */
  async getRange(offset: number, length: number): Promise<Uint8Array> {
    return this.#buffer.slice(offset, offset + length);
  }
}
