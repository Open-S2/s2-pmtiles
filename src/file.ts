import { open } from 'fs/promises';
import { promisify } from 'util';
import { openSync, read, write, writeSync } from 'fs';

import type { Reader } from './reader';
import type { Writer } from './writer';

const readAsync = promisify(read);
const writeAsync = promisify(write);

/** The File reader is to be used by bun/node/deno on the local filesystem. */
export class FileReader implements Reader {
  #fileFD: number;

  /** @param file - the location of the PMTiles data in the FS */
  constructor(readonly file: string) {
    this.#fileFD = openSync(file, 'r');
  }

  /**
   * @param offset - the offset of the range
   * @param length - the length of the range
   * @returns - the ranged buffer
   */
  async getRange(offset: number, length: number): Promise<Uint8Array> {
    const buffer = Buffer.alloc(length);
    await readAsync(this.#fileFD, buffer, 0, length, offset);
    return new Uint8Array(buffer.buffer, 0, length);
  }
}

/** The File writer is to be used by bun/node/deno on the local filesystem. */
export class FileWriter implements Writer {
  #fileFD: number;

  /** @param file - the location of the PMTiles data in the FS */
  constructor(readonly file: string) {
    this.#fileFD = openSync(file, 'a+');
  }

  /**
   * @param data - the data to write
   * @param offset - where in the buffer to start
   */
  async write(data: Uint8Array, offset: number): Promise<void> {
    const fd = await open(this.file, 'r+'); // Open file for reading and writing
    try {
      await fd.write(data, 0, data.length, offset); // Write at the specified offset
    } finally {
      await fd.close(); // Close the file after writing
    }
  }

  /** @param data - the data to append */
  async append(data: Uint8Array): Promise<void> {
    await writeAsync(this.#fileFD, data, 0, data.byteLength);
  }

  /** @param data - the data to append */
  appendSync(data: Uint8Array): void {
    writeSync(this.#fileFD, data, 0, data.length); // Append data using the open file descriptor
  }
}
