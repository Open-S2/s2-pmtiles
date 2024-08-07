import { PMTilesReader } from '../src/reader';
import { PMTilesWriter } from '../src/writer';
import { S2_ROOT_SIZE } from '../src/s2pmtiles';
import { TileType } from '../src/pmtiles';
import tmp from 'tmp';
import { unlink } from 'node:fs/promises';
import { afterAll, describe, expect, test } from 'bun:test';

import { stat } from 'node:fs/promises';

import type { Metadata } from '../src';

let tmpFile: string;
let tmpFile2: string;
let tmpFile3: string;

describe('File Writer WM', async () => {
  tmpFile = tmp.tmpNameSync({
    prefix: 'WM',
  });
  const writer = new PMTilesWriter(tmpFile, TileType.Pbf);
  // setup data
  const str = 'hello world';
  const buf = Buffer.from(str, 'utf8');
  const uint8 = new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
  const str2 = 'hello world 2';
  const buf2 = Buffer.from(str2, 'utf8');
  const uint8_2 = new Uint8Array(buf2.buffer, buf2.byteOffset, buf2.byteLength);
  // write data in tile
  await writer.writeTileXYZ(0, 0, 0, uint8);
  await writer.writeTileXYZ(1, 0, 1, uint8);
  await writer.writeTileXYZ(5, 2, 9, uint8_2);
  // finish
  await writer.commit({ metadata: true } as unknown as Metadata);

  const reader = new PMTilesReader(tmpFile);
  const metadata = await reader.getMetadata();
  const header = await reader.getHeader();
  test('read', async () => {
    expect((await stat(tmpFile)).size).toEqual(98_328);
    expect(header).toEqual({
      clustered: true,
      internalCompression: 1,
      jsonMetadataLength: 17,
      jsonMetadataOffset: 276,
      leafDirectoryLength: 0,
      leafDirectoryOffset: 98_328,
      maxZoom: 5,
      minZoom: 0,
      numAddressedTiles: 3,
      numTileContents: 2,
      numTileEntries: 3,
      rootDirectoryLength: 14,
      rootDirectoryOffset: 262,
      specVersion: 3,
      tileCompression: 1,
      tileDataLength: 24,
      tileDataOffset: S2_ROOT_SIZE,
      tileType: 1,
    });
    expect(metadata).toEqual({ metadata: true } as unknown as Metadata);

    const tile = await reader.getTile(0, 0, 0);
    expect(tile).toEqual(uint8);

    const tile2 = await reader.getTile(1, 0, 1);
    expect(tile2).toEqual(uint8);

    const tile3 = await reader.getTile(5, 2, 9);
    expect(tile3).toEqual(uint8_2);
  });
});

describe('File Writer S2', async () => {
  tmpFile2 = tmp.tmpNameSync({
    prefix: 'S2',
  });
  const writer = new PMTilesWriter(tmpFile2, TileType.Pbf);
  // setup data
  const str = 'hello world';
  const buf = Buffer.from(str, 'utf8');
  const uint8 = new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
  const str2 = 'hello world 2';
  const buf2 = Buffer.from(str2, 'utf8');
  const uint8_2 = new Uint8Array(buf2.buffer, buf2.byteOffset, buf2.byteLength);
  // write data in tile
  await writer.writeTileS2(0, 0, 0, 0, uint8);
  await writer.writeTileS2(1, 0, 0, 0, uint8);
  await writer.writeTileS2(3, 2, 1, 1, uint8_2);
  // finish
  await writer.commit({ metadata: true } as unknown as Metadata);

  const reader = new PMTilesReader(tmpFile2);
  const metadata = await reader.getMetadata();
  const header = await reader.getHeader();
  test('read', async () => {
    expect((await stat(tmpFile2)).size).toEqual(98_328);
    expect(header).toEqual({
      clustered: true,
      internalCompression: 1,
      jsonMetadataLength: 17,
      jsonMetadataOffset: 280,
      leafDirectoryLength: 0,
      leafDirectoryLength1: 0,
      leafDirectoryLength2: 0,
      leafDirectoryLength3: 0,
      leafDirectoryLength4: 0,
      leafDirectoryLength5: 0,
      leafDirectoryOffset: 98_328,
      leafDirectoryOffset1: 98_328,
      leafDirectoryOffset2: 98_328,
      leafDirectoryOffset3: 98_328,
      leafDirectoryOffset4: 98_328,
      leafDirectoryOffset5: 98_328,
      maxZoom: 0,
      minZoom: 0,
      numAddressedTiles: 3,
      numTileContents: 2,
      numTileEntries: 1,
      rootDirectoryLength: 5,
      rootDirectoryLength1: 5,
      rootDirectoryLength2: 1,
      rootDirectoryLength3: 5,
      rootDirectoryLength4: 1,
      rootDirectoryLength5: 1,
      rootDirectoryOffset: 262,
      rootDirectoryOffset1: 267,
      rootDirectoryOffset2: 272,
      rootDirectoryOffset3: 273,
      rootDirectoryOffset4: 278,
      rootDirectoryOffset5: 279,
      specVersion: 1,
      tileCompression: 1,
      tileDataLength: 24,
      tileDataOffset: S2_ROOT_SIZE,
      tileType: 1,
    });
    expect(metadata).toEqual({ metadata: true } as unknown as Metadata);

    const tile = await reader.getTileS2(0, 0, 0, 0);
    expect(tile).toEqual(uint8);

    const tile2 = await reader.getTileS2(1, 0, 0, 0);
    expect(tile2).toEqual(uint8);

    const tile3 = await reader.getTileS2(3, 2, 1, 1);
    expect(tile3).toEqual(uint8_2);
  });
});

describe('File Writer WM Large', async () => {
  tmpFile3 = tmp.tmpNameSync({
    prefix: 'S2-big-2',
  });
  const writer = new PMTilesWriter(tmpFile3, TileType.Pbf);
  // write lots of tiles
  for (let zoom = 0; zoom < 8; zoom++) {
    const size = 1 << zoom;
    for (let x = 0; x < size; x++) {
      for (let y = 0; y < size; y++) {
        const str = `${zoom}-${x}-${y}`;
        const buf = Buffer.from(str, 'utf8');
        const uint8 = new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
        await writer.writeTileXYZ(zoom, x, y, uint8);
      }
    }
  }
  // finish
  await writer.commit({ metadata: true } as unknown as Metadata);

  const reader = new PMTilesReader(tmpFile3);
  const header = await reader.getHeader();
  test('read', async () => {
    expect((await stat(tmpFile3)).size).toEqual(371_616);
    expect(header).toEqual({
      clustered: false,
      internalCompression: 1,
      jsonMetadataLength: 17,
      jsonMetadataOffset: 303,
      leafDirectoryLength: 118199,
      leafDirectoryOffset: 253417,
      maxZoom: 7,
      minZoom: 0,
      numAddressedTiles: 21845,
      numTileContents: 21845,
      numTileEntries: 21845,
      rootDirectoryLength: 41,
      rootDirectoryOffset: 262,
      specVersion: 3,
      tileCompression: 1,
      tileDataLength: 273312,
      tileDataOffset: 98304,
      tileType: 1,
    });
    const metadata = await reader.getMetadata();
    expect(metadata).toEqual({ metadata: true } as unknown as Metadata);

    // get a random tile
    const tile = await reader.getTile(6, 22, 45);
    const str = `6-22-45`;
    const buf = Buffer.from(str, 'utf8');
    const uint8 = new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
    expect(tile).toEqual(uint8);
  });
});

// cleanup
afterAll(async () => {
  try {
    await unlink(tmpFile);
  } catch (_) {
    // ignore
  }
  try {
    await unlink(tmpFile2);
  } catch (_) {
    // ignore
  }
  try {
    await unlink(tmpFile3);
  } catch (_) {
    // ignore
  }
});
