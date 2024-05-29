/** Types */
export type DrawType = 1 | 2 | 3 | 4; // 1: points, 2: lines, 3: poly

/** Open S2 Projetion Face */
export type Face = 0 | 1 | 2 | 3 | 4 | 5;

/** OpenS2 layer fields */
export type LayerFields = Record<string, Array<'Number' | 'String' | 'Boolean'>>;

/** OpenS2 layer metadata */
export type LayerMetaData = Record<
  string,
  {
    // layer
    description?: string;
    minzoom: number;
    maxzoom: number;
    drawTypes: DrawType[];
    fields: LayerFields; // max fields size of 10
  }
>;

/** Mapbox Vector Tile layer metadata */
export interface VectorLayerMetaData {
  id: string;
  fields: Record<string, string>; // max fields size of 10; value is the description
  description?: string;
  minzoom?: number;
  maxzoom?: number;
}

/** Mapbox Vector Tile laters metadata */
export type VectorLayersMetaData = VectorLayerMetaData[];

/** Mapbox Vector Tile Layer Stats */
export interface VectorTileLayerStats {
  layer?: string;
  count: number;
  geometry: 'Polygon' | 'Point' | 'LineString';
  attributeCount: number;
  attributes: string[];
}

/** Mapbox Vector Tile Stats */
export interface VectorTileStats {
  layerCount: number;
  layers: VectorTileLayerStats[];
}

/** OpenS2 Tile Stats */
export interface TileStatsMetadata {
  total: number;
  0: { total: number } | undefined;
  1: { total: number } | undefined;
  2: { total: number } | undefined;
  3: { total: number } | undefined;
  4: { total: number } | undefined;
  5: { total: number } | undefined;
}

/** Bounds set in lon/lat */
export type LLBounds = [minLon: number, minLat: number, maxLon: number, maxLat: number];
/** The bounds of the tile, relative to the tiles zoom */
export type TileBounds = [minX: number, minY: number, maxX: number, maxY: number];

/** Base Metadata that's common to both Mapbox and Open S2 */
export interface MetadataBase {
  name: string;
  description: string;
}

/**
 * Check out the [OLD Mapbox Spec](https://github.com/mapbox/tilejson-spec/blob/22f5f91e643e8980ef2656674bef84c2869fbe76/3.0.0/README.md#315-scheme)
 * for details of how this was built.
 */
export interface Metadata extends MetadataBase {
  minzoom?: number;
  maxzoom?: number;
  scheme?: 'zxy';
  tilejson?: string;
  attribution?: string;
  center?: [x: number, y: number, zoom: number];
  bounds?: LLBounds;
  data?: string[];
  fillzoom?: number;
  legend?: string;
  version?: string;
  vector_layers: VectorLayersMetaData;
  tiles?: string[];
  type?: string;
  generator?: string;
  generator_options?: string;
  tilestats: VectorTileStats;
}

/** New Open S2 Maps Spec */
export interface S2Metadata extends MetadataBase {
  minzoom: number;
  maxzoom: number;
  format: 'fzxy';
  faces: Face[];
  type: 'vector' | 'raster' | 'rasterDEM' | 'rasterData' | 'json' | 'buffer';
  encoding: 'gz' | 'br' | 'none';
  attributions: Record<string, string>; // { ['human readable string']: 'href' }
  facesbounds: {
    // facesbounds[face][zoom] = [...]
    0: Record<number, TileBounds>;
    1: Record<number, TileBounds>;
    2: Record<number, TileBounds>;
    3: Record<number, TileBounds>;
    4: Record<number, TileBounds>;
    5: Record<number, TileBounds>;
  };
  layers: LayerMetaData;
  tilestats?: TileStatsMetadata;
}
