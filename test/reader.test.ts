import { Compression } from '../src/pmtiles';
import { PMTilesReader } from '../src/reader';
import { describe, expect, test } from 'bun:test';

import type { Metadata } from '../src/metadata';

describe('File Reader', async () => {
  test('test_fixture_1', async () => {
    const testFixture1 = new PMTilesReader(`${__dirname}/fixtures/test_fixture_1.pmtiles`);
    expect(testFixture1).toBeInstanceOf(PMTilesReader);
    const header = await testFixture1.getHeader();
    // header
    expect(header).toEqual({
      clustered: false,
      etag: '',
      internalCompression: Compression.Gzip,
      jsonMetadataLength: 247,
      jsonMetadataOffset: 152,
      leafDirectoryLength: 0,
      leafDirectoryOffset: 0,
      maxZoom: 0,
      minZoom: 0,
      numAddressedTiles: 1,
      numTileContents: 1,
      numTileEntries: 1,
      rootDirectoryLength: 25,
      rootDirectoryOffset: 127,
      specVersion: 3,
      tileCompression: 2,
      tileDataLength: 69,
      tileDataOffset: 399,
      tileType: 1,
    });
    // metadata
    expect(testFixture1.getMetadata()).toEqual({
      name: 'test_fixture_1.pmtiles',
      description: 'test_fixture_1.pmtiles',
      version: '2',
      type: 'overlay',
      generator: 'tippecanoe v2.5.0',
      generator_options: './tippecanoe -zg -o test_fixture_1.pmtiles --force',
      vector_layers: [
        {
          id: 'test_fixture_1pmtiles',
          description: '',
          minzoom: 0,
          maxzoom: 0,
          fields: {},
        },
      ],
      tilestats: {
        layerCount: 1,
        layers: [
          {
            layer: 'test_fixture_1pmtiles',
            count: 1,
            geometry: 'Polygon',
            attributeCount: 0,
            attributes: [],
          },
        ],
      },
    } as Metadata);
    // TILE
    const tile = await testFixture1.getTile(0, 0, 0);
    expect(tile).toBeInstanceOf(Uint8Array);
    expect(new Uint8Array(tile as Uint8Array)).toEqual(
      new Uint8Array([
        26, 47, 120, 2, 10, 21, 116, 101, 115, 116, 95, 102, 105, 120, 116, 117, 114, 101, 95, 49,
        112, 109, 116, 105, 108, 101, 115, 40, 128, 32, 18, 17, 24, 3, 34, 13, 9, 150, 32, 232, 31,
        26, 0, 24, 21, 0, 0, 23, 15,
      ]),
    );
  });

  test('test_fixture_2', async () => {
    const testFixture2 = new PMTilesReader(`${__dirname}/fixtures/test_fixture_2.pmtiles`);
    expect(testFixture2).toBeInstanceOf(PMTilesReader);
    const header = await testFixture2.getHeader();
    // header
    expect(header).toEqual({
      clustered: false,
      etag: '',
      internalCompression: Compression.Gzip,
      jsonMetadataLength: 247,
      jsonMetadataOffset: 152,
      leafDirectoryLength: 0,
      leafDirectoryOffset: 0,
      maxZoom: 0,
      minZoom: 0,
      numAddressedTiles: 1,
      numTileContents: 1,
      numTileEntries: 1,
      rootDirectoryLength: 25,
      rootDirectoryOffset: 127,
      specVersion: 3,
      tileCompression: 2,
      tileDataLength: 67,
      tileDataOffset: 399,
      tileType: 1,
    });
    // metadata
    expect(testFixture2.getMetadata()).toEqual({
      name: 'test_fixture_2.pmtiles',
      description: 'test_fixture_2.pmtiles',
      version: '2',
      type: 'overlay',
      generator: 'tippecanoe v2.5.0',
      generator_options: './tippecanoe -zg -o test_fixture_2.pmtiles --force',
      vector_layers: [
        {
          id: 'test_fixture_2pmtiles',
          description: '',
          minzoom: 0,
          maxzoom: 0,
          fields: {},
        },
      ],
      tilestats: {
        layerCount: 1,
        layers: [
          {
            layer: 'test_fixture_2pmtiles',
            count: 1,
            geometry: 'Polygon',
            attributeCount: 0,
            attributes: [],
          },
        ],
      },
    });
    // TILE
    const tile = await testFixture2.getTile(0, 0, 0);
    expect(tile).toBeInstanceOf(Uint8Array);
    expect(new Uint8Array(tile as Uint8Array)).toEqual(
      new Uint8Array([
        26, 45, 120, 2, 10, 21, 116, 101, 115, 116, 95, 102, 105, 120, 116, 117, 114, 101, 95, 50,
        112, 109, 116, 105, 108, 101, 115, 40, 128, 32, 18, 15, 24, 3, 34, 11, 9, 128, 32, 232, 31,
        18, 22, 24, 21, 0, 15,
      ]),
    );
  });
});
