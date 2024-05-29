import { PMTilesReader } from '../src/reader';
import { PMTilesWriter } from '../src/writer';
import { S2_ROOT_SIZE } from '../src/s2pmtiles';
import tmp from 'tmp';
import { unlink } from 'node:fs/promises';
import { Compression, TileType } from '../src/pmtiles';
import { afterAll, describe, expect, test } from 'bun:test';

import { stat } from 'node:fs/promises';

import type { Metadata } from '../src/metadata';

describe('File Writer', async () => {
  const tmpFile = tmp.tmpNameSync();
  const writer = new PMTilesWriter(tmpFile, TileType.Pbf, Compression.None);
  // setup data
  const str = 'hello world';
  const buf = Buffer.from(str, 'utf8');
  const uint8 = new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
  // write data in tile
  await writer.writeTileXYZ(0, 0, 0, uint8);
  // finish
  await writer.flush({ metadata: true } as unknown as Metadata);
  test('write', () => {});

  const reader = new PMTilesReader(tmpFile);
  const tile = await reader.getTile(0, 0, 0);
  const metadata = reader.getMetadata();
  const header = await reader.getHeader();
  test('read', async () => {
    expect((await stat(tmpFile)).size).toEqual(98_315);
    expect(header).toEqual({
      clustered: true,
      etag: '',
      internalCompression: 1,
      jsonMetadataLength: 17,
      jsonMetadataOffset: 132,
      leafDirectoryLength: 0,
      leafDirectoryOffset: 98_315,
      maxZoom: 0,
      minZoom: 0,
      numAddressedTiles: 1,
      numTileContents: 1,
      numTileEntries: 1,
      rootDirectoryLength: 5,
      rootDirectoryOffset: 127,
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
