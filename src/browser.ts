import DirCache from './cache';
import { decompressSync } from 'fflate';
import {
  Compression,
  type Entry,
  type Header,
  bytesToHeader,
  deserializeDir,
  findTile,
  zxyToTileID,
} from './pmtiles';
import {
  type S2Entries,
  type S2Header,
  S2_HEADER_SIZE_BYTES,
  S2_ROOT_SIZE,
  s2BytesToHeader,
} from './s2pmtiles';

import type { Face, Metadata, S2Metadata } from './metadata';

// export DirCache for browsers to use (reduce code duplication)
export { default as DirCache } from './cache';

/** The File reader is to be used by bun/node/deno on the local filesystem. */
export default class S2PMTilesReader {
  #header: Header | S2Header | undefined;
  // root directory will exist if header does
  #rootDir: Entry[] = [];
  #rootDirS2: S2Entries = { 0: [], 1: [], 2: [], 3: [], 4: [], 5: [] };
  #metadata!: Metadata | S2Metadata;
  readonly #dirCache: DirCache;
  readonly #decoder = new TextDecoder('utf-8');

  /**
   * Given an input path, read in the header and root directory
   * @param path - the location of the PMTiles data
   * @param rangeRequests - enable range requests or use urlParam "bytes"
   * @param maxSize - the max size of the cache before dumping old data. Defaults to 20.
   */
  constructor(
    readonly path: string,
    readonly rangeRequests: boolean = false,
    maxSize = 20,
  ) {
    this.#dirCache = new DirCache(maxSize);
  }

  /**
   * @returns - the header of the archive along with the root directory,
   * including information such as tile type, min/max zoom, bounds, and summary statistics.
   */
  async #getMetadata(): Promise<Header> {
    if (this.#header !== undefined) return this.#header;
    const data = await this.#getRange(0, S2_ROOT_SIZE);
    const headerData = data.slice(0, S2_HEADER_SIZE_BYTES);
    // check if s2
    const isS2 = headerData[0] === 83 && headerData[1] === 50;
    // header
    const headerFunction = isS2 ? s2BytesToHeader : bytesToHeader;
    const header = (this.#header = headerFunction(headerData, ''));

    // json metadata
    const jsonMetadata = data.slice(
      header.jsonMetadataOffset,
      header.jsonMetadataOffset + header.jsonMetadataLength,
    );
    this.#metadata = JSON.parse(
      this.#arrayBufferToString(await decompress(jsonMetadata, header.internalCompression)),
    );

    // root directory data
    const rootDirData = data.slice(
      header.rootDirectoryOffset,
      header.rootDirectoryOffset + header.rootDirectoryLength,
    );
    this.#rootDir = deserializeDir(await decompress(rootDirData, header.internalCompression));

    if (isS2) await this.#getS2Metadata(data, header as S2Header);

    return header;
  }

  /**
   * If S2 Projection, pull in the rest of the data
   * @param data - the root data
   * @param header - the S2 header with pointers to the rest of the data
   */
  async #getS2Metadata(data: Uint8Array, header: S2Header): Promise<void> {
    // move the root directory to the s2 root
    this.#rootDirS2[0] = this.#rootDir;
    // add the 4 other faces
    for (const face of [1, 2, 3, 4, 5]) {
      const rootOffset = `rootDirectoryOffset${face}` as keyof S2Header;
      const rootLenght = `rootDirectoryLength${face}` as keyof S2Header;
      const faceDirData = data.slice(
        header[rootOffset] as number,
        (header[rootOffset] as number) + (header[rootLenght] as number),
      );
      this.#rootDirS2[face as keyof S2Entries] = deserializeDir(
        await decompress(faceDirData, header.internalCompression),
      );
    }
  }

  /** @returns - the header of the archive */
  async getHeader(): Promise<Header> {
    return await this.#getMetadata();
  }

  /** @returns - the metadata of the archive */
  async getMetadata(): Promise<Metadata | S2Metadata> {
    await this.#getMetadata(); // ensure loaded first
    return this.#metadata;
  }

  /**
   * @param face - the Open S2 projection face
   * @param zoom - the zoom level of the tile
   * @param x - the x coordinate of the tile
   * @param y - the y coordinate of the tile
   * @returns - the bytes of the tile at the given (face, zoom, x, y) coordinates, or undefined if the tile does not exist in the archive.
   */
  async getTileS2(face: Face, zoom: number, x: number, y: number): Promise<Uint8Array | undefined> {
    return await this.#getTile(face, zoom, x, y);
  }

  /**
   * @param zoom - the zoom level of the tile
   * @param x - the x coordinate of the tile
   * @param y - the y coordinate of the tile
   * @returns - the bytes of the tile at the given (z, x, y) coordinates, or undefined if the tile does not exist in the archive.
   */
  async getTile(zoom: number, x: number, y: number): Promise<Uint8Array | undefined> {
    return await this.#getTile(-1, zoom, x, y);
  }

  /**
   * @param face - the Open S2 projection face
   * @param zoom - the zoom level of the tile
   * @param x - the x coordinate of the tile
   * @param y - the y coordinate of the tile
   * @returns - the bytes of the tile at the given (z, x, y) coordinates, or undefined if the tile does not exist in the archive.
   */
  async #getTile(
    face: number,
    zoom: number,
    x: number,
    y: number,
  ): Promise<Uint8Array | undefined> {
    const header = await this.#getMetadata();
    const tileID = zxyToTileID(zoom, x, y);
    const { minZoom, maxZoom, rootDirectoryOffset, rootDirectoryLength, tileDataOffset } = header;
    if (zoom < minZoom || zoom > maxZoom) return undefined;

    let dO = rootDirectoryOffset;
    let dL = rootDirectoryLength;

    for (let depth = 0; depth <= 3; depth++) {
      const directory = await this.#getDirectory(dO, dL, face);
      if (directory === undefined) return undefined;
      const entry = findTile(directory, tileID);
      if (entry !== null) {
        if (entry.runLength > 0) {
          const entryData = await this.#getRange(tileDataOffset + entry.offset, entry.length);
          return await decompress(entryData, header.tileCompression);
        }
        dO = header.leafDirectoryOffset + entry.offset;
        dL = entry.length;
      } else return undefined;
    }
    throw Error('Maximum directory depth exceeded');
  }

  /**
   * @param offset - the offset of the directory
   * @param length - the length of the directory
   * @param face - -1 for WM root, 0-5 for S2
   * @returns - the entries in the directory if it exists
   */
  async #getDirectory(offset: number, length: number, face: number): Promise<Entry[] | undefined> {
    const dir = face === -1 ? this.#rootDir : this.#rootDirS2[face as Face];
    const header = await this.#getMetadata();
    const { internalCompression, rootDirectoryOffset } = header;
    // if rootDirectoryOffset, return roon
    if (offset === rootDirectoryOffset) return dir;
    // check cache
    const cache = this.#dirCache.get(offset);
    if (cache !== undefined) return cache;
    // get from archive
    const resp = await this.#getRange(offset, length);
    const data = await decompress(resp, internalCompression);
    const directory = deserializeDir(data);
    if (directory.length === 0) throw new Error('Empty directory is invalid');
    // save in cache
    this.#dirCache.set(offset, directory);

    return directory;
  }

  /**
   * @param offset - the offset of the data
   * @param length - the length of the data
   * @returns - the bytes of the data
   */
  async #getRange(offset: number, length: number): Promise<Uint8Array> {
    const bytes = String(offset) + '-' + String(offset + length);
    const fetchReq = this.rangeRequests
      ? fetch(this.path, { headers: { Range: `bytes=${offset}-${offset + length - 1}` } })
      : fetch(`${this.path}&bytes=${bytes}`);
    const res = await fetchReq.then(async (res) => await res.arrayBuffer());
    return new Uint8Array(res, 0, res.byteLength);
  }

  /**
   * @param buffer - the buffer to convert
   * @returns - the string result
   */
  #arrayBufferToString(buffer: Uint8Array): string {
    return this.#decoder.decode(buffer);
  }
}

/**
 * @param data - the data to decompress
 * @param compression - the compression type
 * @returns - the decompressed data
 */
async function decompress(data: Uint8Array, compression: Compression): Promise<Uint8Array> {
  switch (compression) {
    case Compression.Gzip:
      return decompressSync(data);
    case Compression.Brotli:
      throw new Error('Brotli decompression not implemented');
    case Compression.Zstd:
      throw new Error('Zstd decompression not implemented');
    case Compression.None:
    default:
      return data;
  }
}
