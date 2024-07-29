import { PMTilesReader } from '../src/reader';
import { PMTilesWriter } from '../src/writer';
import { S2_ROOT_SIZE } from '../src/s2pmtiles';
import { TileType } from '../src/pmtiles';
import tmp from 'tmp';
import { unlink } from 'node:fs/promises';
import { afterAll, describe, expect, test } from 'bun:test';

import { stat } from 'node:fs/promises';

import type { Metadata } from '../src';

describe('File Writer WM', async () => {
  const tmpFile = tmp.tmpNameSync();
  const writer = new PMTilesWriter(tmpFile, TileType.Pbf);
  // setup data
  const str = 'hello world';
  const buf = Buffer.from(str, 'utf8');
  const uint8 = new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
  // write data in tile
  await writer.writeTileXYZ(0, 0, 0, uint8);
  // finish
  await writer.commit({ metadata: true } as unknown as Metadata);

  const reader = new PMTilesReader(tmpFile);
  const tile = await reader.getTile(0, 0, 0);
  const metadata = reader.getMetadata();
  const header = await reader.getHeader();
  test('read', async () => {
    expect((await stat(tmpFile)).size).toEqual(98_315);
    expect(header).toEqual({
      clustered: true,
      internalCompression: 1,
      jsonMetadataLength: 17,
      jsonMetadataOffset: 267,
      leafDirectoryLength: 0,
      leafDirectoryOffset: 98_315,
      maxZoom: 0,
      minZoom: 0,
      numAddressedTiles: 1,
      numTileContents: 1,
      numTileEntries: 1,
      rootDirectoryLength: 5,
      rootDirectoryOffset: 262,
      specVersion: 3,
      tileCompression: 1,
      tileDataLength: 11,
      tileDataOffset: S2_ROOT_SIZE,
      tileType: 1,
    });
    expect(metadata).toEqual({ metadata: true } as unknown as Metadata);
    expect(tile).toEqual(uint8);
  });

  // cleanup
  afterAll(async () => {
    await unlink(tmpFile);
  });
});

describe('File Writer S2', async () => {
  const tmpFile = tmp.tmpNameSync();
  const writer = new PMTilesWriter(tmpFile, TileType.Pbf);
  // setup data
  const str = 'hello world';
  const buf = Buffer.from(str, 'utf8');
  const uint8 = new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
  // write data in tile
  await writer.writeTileS2(0, 0, 0, 0, uint8);
  // finish
  await writer.commit({ metadata: true } as unknown as Metadata);

  const reader = new PMTilesReader(tmpFile);
  const tile = await reader.getTileS2(0, 0, 0, 0);
  const metadata = reader.getMetadata();
  const header = await reader.getHeader();
  test('read', async () => {
    expect((await stat(tmpFile)).size).toEqual(98_315);
    expect(header).toEqual({
      clustered: true,
      internalCompression: 1,
      jsonMetadataLength: 17,
      jsonMetadataOffset: 272,
      leafDirectoryLength: 0,
      leafDirectoryLength1: 0,
      leafDirectoryLength2: 0,
      leafDirectoryLength3: 0,
      leafDirectoryLength4: 0,
      leafDirectoryLength5: 0,
      leafDirectoryOffset: 98_315,
      leafDirectoryOffset1: 98_315,
      leafDirectoryOffset2: 98_315,
      leafDirectoryOffset3: 98_315,
      leafDirectoryOffset4: 98_315,
      leafDirectoryOffset5: 98_315,
      maxZoom: 0,
      minZoom: 0,
      numAddressedTiles: 1,
      numTileContents: 1,
      numTileEntries: 1,
      rootDirectoryLength: 5,
      rootDirectoryLength1: 1,
      rootDirectoryLength2: 1,
      rootDirectoryLength3: 1,
      rootDirectoryLength4: 1,
      rootDirectoryLength5: 1,
      rootDirectoryOffset: 262,
      rootDirectoryOffset1: 267,
      rootDirectoryOffset2: 268,
      rootDirectoryOffset3: 269,
      rootDirectoryOffset4: 270,
      rootDirectoryOffset5: 271,
      specVersion: 1,
      tileCompression: 1,
      tileDataLength: 11,
      tileDataOffset: S2_ROOT_SIZE,
      tileType: 1,
    });
    expect(metadata).toEqual({ metadata: true } as unknown as Metadata);
    expect(tile).toEqual(uint8);
  });

  // cleanup
  afterAll(async () => {
    await unlink(tmpFile);
  });
});
