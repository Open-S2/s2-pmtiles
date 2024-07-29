import { createHash } from 'node:crypto';
import { Compression, headerToBytes, serializeDir, tileIDToZxy, zxyToTileID } from './pmtiles';
import { S2_HEADER_SIZE_BYTES, S2_ROOT_SIZE, s2HeaderToBytes } from './s2pmtiles';
import { appendFile, open } from 'node:fs/promises';

import type { Entry, Header, TileType } from './pmtiles';
import type { Face, Metadata } from 's2-tilejson';
import type { S2Entries, S2Header } from './s2pmtiles';

/** Write a PMTiles file. */
export class PMTilesWriter {
  #tileEntries: Entry[] = [];
  #s2tileEntries: S2Entries = { 0: [], 1: [], 2: [], 3: [], 4: [], 5: [] };
  #hashToOffset = new Map<string, number>();
  #offset = 0;
  #addressedTiles = 0;
  #clustered = true;
  compression: Compression = Compression.None;
  /**
   * @param file - the path to the file we want to write to
   * @param type - the tile type
   */
  constructor(
    readonly file: string,
    readonly type: TileType,
  ) {
    // append the headersize
    appendFile(this.file, new Uint8Array(S2_ROOT_SIZE));
  }

  /**
   * Write a tile to the PMTiles file given its (z, x, y) coordinates.
   * @param zoom - the zoom level
   * @param x - the tile X coordinate
   * @param y - the tile Y coordinate
   * @param data - the tile data to store
   */
  async writeTileXYZ(zoom: number, x: number, y: number, data: Uint8Array): Promise<void> {
    const tileID = zxyToTileID(zoom, x, y);
    await this.writeTile(tileID, data);
  }

  /**
   * Write a tile to the PMTiles file given its (face, zoom, x, y) coordinates.
   * @param face - the Open S2 projection face
   * @param zoom - the zoom level
   * @param x - the tile X coordinate
   * @param y - the tile Y coordinate
   * @param data - the tile data to store
   */
  async writeTileS2(
    face: Face,
    zoom: number,
    x: number,
    y: number,
    data: Uint8Array,
  ): Promise<void> {
    const tileID = zxyToTileID(zoom, x, y);
    await this.writeTile(tileID, data, face);
  }

  /**
   * Write a tile to the PMTiles file given its tile ID.
   * @param tileID - the tile ID
   * @param data - the tile data
   * @param face - If it exists, then we are storing S2 data
   */
  async writeTile(tileID: number, data: Uint8Array, face?: Face): Promise<void> {
    const length = data.length;
    const tileEntries = face !== undefined ? this.#s2tileEntries[face] : this.#tileEntries;
    if (tileEntries.length > 0 && tileID < (tileEntries.at(-1) as Entry).tileID) {
      this.#clustered = false;
    }

    const hsh = hashUint8Array(data);
    let offset = this.#hashToOffset.get(hsh);
    if (offset !== undefined) {
      const last = tileEntries.at(-1) as Entry;
      if (tileID == last.tileID + last.runLength && last.offset == offset) {
        tileEntries[-1].runLength++;
      } else {
        tileEntries.push({ tileID, offset, length, runLength: 1 });
      }
    } else {
      offset = this.#offset;
      await appendFile(this.file, data);
      tileEntries.push({ tileID, offset, length, runLength: 1 });
      this.#hashToOffset.set(hsh, this.#offset);
      this.#offset += length;
    }

    this.#addressedTiles++;
  }

  /**
   * Finish writing by building the header with root and leaf directories
   * @param metadata - the metadata to store
   */
  async commit(metadata: Metadata): Promise<void> {
    if (this.#tileEntries.length === 0) await this.#commitS2(metadata);
    else await this.#commit(metadata);
  }

  /**
   * Finish writing by building the header with root and leaf directories
   * @param metadata - the metadata to store
   */
  async #commit(metadata: Metadata): Promise<void> {
    const tileEntries = this.#tileEntries;
    // build metadata
    const metaBuffer = Buffer.from(JSON.stringify(metadata));
    const metauint8 = new Uint8Array(
      metaBuffer.buffer,
      metaBuffer.byteOffset,
      metaBuffer.byteLength,
    );

    // optimize directories
    const { rootBytes, leavesBytes } = optimizeDirectories(
      tileEntries,
      S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - metauint8.byteLength,
    );

    // build header data
    const rootDirectoryOffset = S2_HEADER_SIZE_BYTES;
    const rootDirectoryLength = rootBytes.byteLength;
    const jsonMetadataOffset = rootDirectoryOffset + rootDirectoryLength;
    const jsonMetadataLength = metauint8.byteLength;
    const leafDirectoryOffset = this.#offset + S2_ROOT_SIZE;
    const leafDirectoryLength = leavesBytes.byteLength;
    this.#offset += leavesBytes.byteLength;
    appendFile(this.file, leavesBytes);
    // to make writing faster
    const minZoom = tileIDToZxy((tileEntries.at(0) as Entry).tileID)[0];
    const maxZoom = tileIDToZxy((tileEntries.at(-1) as Entry).tileID)[0];

    // build header
    const header: Header = {
      specVersion: 3,
      rootDirectoryOffset,
      rootDirectoryLength,
      jsonMetadataOffset,
      jsonMetadataLength,
      leafDirectoryOffset,
      leafDirectoryLength,
      tileDataOffset: S2_ROOT_SIZE,
      tileDataLength: this.#offset,
      numAddressedTiles: this.#addressedTiles,
      numTileEntries: tileEntries.length,
      numTileContents: this.#hashToOffset.size,
      clustered: this.#clustered,
      internalCompression: Compression.None,
      tileCompression: this.compression,
      tileType: this.type,
      minZoom,
      maxZoom,
    };
    const serialzedHeader = headerToBytes(header);

    // write header
    const fileHandle = await open(this.file, 'r+');
    await fileHandle.write(serialzedHeader, 0, serialzedHeader.byteLength, 0);
    await fileHandle.write(rootBytes, 0, rootBytes.byteLength, rootDirectoryOffset);
    await fileHandle.write(metauint8, 0, metauint8.byteLength, jsonMetadataOffset);
    await fileHandle.close();
  }

  /**
   * Finish writing by building the header with root and leaf directories
   * @param metadata - the metadata to store
   */
  async #commitS2(metadata: Metadata): Promise<void> {
    const tileEntries = this.#s2tileEntries[0];
    const tileEntries1 = this.#s2tileEntries[1];
    const tileEntries2 = this.#s2tileEntries[2];
    const tileEntries3 = this.#s2tileEntries[3];
    const tileEntries4 = this.#s2tileEntries[4];
    const tileEntries5 = this.#s2tileEntries[5];
    // build metadata
    const metaBuffer = Buffer.from(JSON.stringify(metadata));
    const metauint8 = new Uint8Array(
      metaBuffer.buffer,
      metaBuffer.byteOffset,
      metaBuffer.byteLength,
    );

    // optimize directories
    const { rootBytes, leavesBytes } = optimizeDirectories(
      tileEntries,
      S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - metauint8.byteLength,
    );
    const { rootBytes: rootBytes1, leavesBytes: leavesBytes1 } = optimizeDirectories(
      tileEntries1,
      S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - metauint8.byteLength,
    );
    const { rootBytes: rootBytes2, leavesBytes: leavesBytes2 } = optimizeDirectories(
      tileEntries2,
      S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - metauint8.byteLength,
    );
    const { rootBytes: rootBytes3, leavesBytes: leavesBytes3 } = optimizeDirectories(
      tileEntries3,
      S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - metauint8.byteLength,
    );
    const { rootBytes: rootBytes4, leavesBytes: leavesBytes4 } = optimizeDirectories(
      tileEntries4,
      S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - metauint8.byteLength,
    );
    const { rootBytes: rootBytes5, leavesBytes: leavesBytes5 } = optimizeDirectories(
      tileEntries5,
      S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - metauint8.byteLength,
    );

    // build header data
    const rootDirectoryOffset = S2_HEADER_SIZE_BYTES;
    const rootDirectoryLength = rootBytes.byteLength;
    const rootDirectoryOffset1 = rootDirectoryOffset + rootDirectoryLength;
    const rootDirectoryLength1 = rootBytes1.byteLength;
    const rootDirectoryOffset2 = rootDirectoryOffset1 + rootDirectoryLength1;
    const rootDirectoryLength2 = rootBytes2.byteLength;
    const rootDirectoryOffset3 = rootDirectoryOffset2 + rootDirectoryLength2;
    const rootDirectoryLength3 = rootBytes3.byteLength;
    const rootDirectoryOffset4 = rootDirectoryOffset3 + rootDirectoryLength3;
    const rootDirectoryLength4 = rootBytes4.byteLength;
    const rootDirectoryOffset5 = rootDirectoryOffset4 + rootDirectoryLength4;
    const rootDirectoryLength5 = rootBytes5.byteLength;
    // metadata
    const jsonMetadataOffset = rootDirectoryOffset5 + rootDirectoryLength5;
    const jsonMetadataLength = metauint8.byteLength;
    // leafs
    const leafDirectoryOffset = this.#offset + S2_ROOT_SIZE;
    const leafDirectoryLength = leavesBytes.byteLength;
    this.#offset += leafDirectoryLength;
    appendFile(this.file, leavesBytes);
    const leafDirectoryOffset1 = this.#offset + S2_ROOT_SIZE;
    const leafDirectoryLength1 = leavesBytes1.byteLength;
    this.#offset += leafDirectoryLength1;
    appendFile(this.file, leavesBytes1);
    const leafDirectoryOffset2 = this.#offset + S2_ROOT_SIZE;
    const leafDirectoryLength2 = leavesBytes2.byteLength;
    this.#offset += leafDirectoryLength2;
    appendFile(this.file, leavesBytes2);
    const leafDirectoryOffset3 = this.#offset + S2_ROOT_SIZE;
    const leafDirectoryLength3 = leavesBytes3.byteLength;
    this.#offset += leafDirectoryLength3;
    appendFile(this.file, leavesBytes3);
    const leafDirectoryOffset4 = this.#offset + S2_ROOT_SIZE;
    const leafDirectoryLength4 = leavesBytes4.byteLength;
    this.#offset += leafDirectoryLength4;
    appendFile(this.file, leavesBytes4);
    const leafDirectoryOffset5 = this.#offset + S2_ROOT_SIZE;
    const leafDirectoryLength5 = leavesBytes5.byteLength;
    this.#offset += leafDirectoryLength5;
    appendFile(this.file, leavesBytes5);
    // to make writing faster
    const minZoom = tileIDToZxy((tileEntries.at(0) as Entry).tileID)[0];
    const maxZoom = tileIDToZxy((tileEntries.at(-1) as Entry).tileID)[0];

    // build header
    const header: S2Header = {
      specVersion: 3,
      rootDirectoryOffset,
      rootDirectoryLength,
      rootDirectoryOffset1,
      rootDirectoryLength1,
      rootDirectoryOffset2,
      rootDirectoryLength2,
      rootDirectoryOffset3,
      rootDirectoryLength3,
      rootDirectoryOffset4,
      rootDirectoryLength4,
      rootDirectoryOffset5,
      rootDirectoryLength5,
      jsonMetadataOffset,
      jsonMetadataLength,
      leafDirectoryOffset,
      leafDirectoryLength,
      leafDirectoryOffset1,
      leafDirectoryLength1,
      leafDirectoryOffset2,
      leafDirectoryLength2,
      leafDirectoryOffset3,
      leafDirectoryLength3,
      leafDirectoryOffset4,
      leafDirectoryLength4,
      leafDirectoryOffset5,
      leafDirectoryLength5,
      tileDataOffset: S2_ROOT_SIZE,
      tileDataLength: this.#offset,
      numAddressedTiles: this.#addressedTiles,
      numTileEntries: tileEntries.length,
      numTileContents: this.#hashToOffset.size,
      clustered: this.#clustered,
      internalCompression: Compression.None,
      tileCompression: this.compression,
      tileType: this.type,
      minZoom,
      maxZoom,
    };
    const serialzedHeader = s2HeaderToBytes(header);

    // write header
    const fileHandle = await open(this.file, 'r+');
    await fileHandle.write(serialzedHeader, 0, serialzedHeader.byteLength, 0);
    await fileHandle.write(rootBytes, 0, rootBytes.byteLength, rootDirectoryOffset);
    await fileHandle.write(rootBytes1, 0, rootBytes1.byteLength, rootDirectoryOffset1);
    await fileHandle.write(rootBytes2, 0, rootBytes2.byteLength, rootDirectoryOffset2);
    await fileHandle.write(rootBytes3, 0, rootBytes3.byteLength, rootDirectoryOffset3);
    await fileHandle.write(rootBytes4, 0, rootBytes4.byteLength, rootDirectoryOffset4);
    await fileHandle.write(rootBytes5, 0, rootBytes5.byteLength, rootDirectoryOffset5);
    await fileHandle.write(metauint8, 0, metauint8.byteLength, jsonMetadataOffset);
    await fileHandle.close();
  }
}

/** The result of an optimized directory computation */
interface OptimizedDirectory {
  /** The root directory bytes */
  rootBytes: Uint8Array;
  /** The leaf directories bytes */
  leavesBytes: Uint8Array;
  /** The number of leaf directories */
  numLeaves: number;
}

/**
 * @param entries - the tile entries
 * @param leafSize - the max leaf size
 * @returns - the optimized directories
 */
function buildRootsLeaves(entries: Entry[], leafSize: number): OptimizedDirectory {
  const rootEntries: Entry[] = [];
  let leavesBytes = new Uint8Array(0);
  let numLeaves = 0;

  let i = 0;
  while (i < entries.length) {
    numLeaves += 1;
    const serialized = serializeDir(entries.slice(i, i + leafSize));
    rootEntries.push({
      tileID: entries[i].tileID,
      offset: leavesBytes.length,
      length: serialized.length,
      runLength: 0,
    });
    leavesBytes = concatUint8Arrays(leavesBytes, serialized);
    i += leafSize;
  }

  return { rootBytes: serializeDir(rootEntries), leavesBytes, numLeaves };
}

/**
 * @param entries - the tile entries
 * @param targetRootLength - the max leaf size
 * @returns - the optimized directories
 */
function optimizeDirectories(entries: Entry[], targetRootLength: number): OptimizedDirectory {
  const testBytes = serializeDir(entries);
  if (testBytes.length < targetRootLength)
    return { rootBytes: testBytes, leavesBytes: new Uint8Array(0), numLeaves: 0 };

  let leafSize = 4096;
  while (true) {
    const build = buildRootsLeaves(entries, leafSize);
    if (build.rootBytes.length < targetRootLength) return build;
    leafSize *= 2;
  }
}

/**
 * @param a - the first array
 * @param b - the second array
 * @returns - the combined array of the two starting with "a"
 */
function concatUint8Arrays(a: Uint8Array, b: Uint8Array): Uint8Array {
  const result = new Uint8Array(a.length + b.length);
  result.set(a, 0);
  result.set(b, a.length);
  return result;
}

/**
 * @param data - the data to hash
 * @param algorithm - the hashing algorithm
 * @returns - the hashed string
 */
function hashUint8Array(data: Uint8Array, algorithm: string = 'sha256'): string {
  const hash = createHash(algorithm);
  hash.update(Buffer.from(data));
  return hash.digest('hex'); // Change 'hex' to 'base64' or other formats if needed
}
