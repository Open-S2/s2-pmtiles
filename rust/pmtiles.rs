extern crate alloc;

use alloc::vec::Vec;
use core::num::NonZeroU64;
use core::cmp::Ordering;

use crate::buffer::Buffer;

pub const TZ_VALUES: [u64; 27] = [
  0, 1, 5, 21, 85, 341, 1365, 5461, 21845, 87381, 349525, 1398101, 5592405, 22369621, 89478485,
  357913941, 1431655765, 5726623061, 22906492245, 91625968981, 366503875925, 1466015503701,
  5864062014805, 23456248059221, 93824992236885, 375299968947541, 1501199875790165,
];

pub const HEADER_SIZE_BYTES: usize = 127;

pub const ROOT_SIZE: usize = 16_384;

/// An array of two numbers representing a point in 2D space
pub struct Point2D {
    x: u64,
    y: u64,
}

/// A tile, in the format of ZXY
pub struct Tile {
    zoom: u8,
    x: u64,
    y: u64,
}
impl Tile {
    pub fn from_id(id: u64) -> Tile {
        let mut acc = 0;
      
        for z in 0..27 {
          let num_tiles = (0x1 << z) * (0x1 << z);
          if acc + num_tiles > id {
            return Tile::from_zoom_pos(z, id - acc);
          }
          acc += num_tiles;
        }

        unreachable!()
    }

    pub fn from_zoom_pos(zoom: u8, pos: u64) -> Tile {
        let n: u64 = 2u64.pow(zoom as u32);
        let mut t = pos;
        let mut xy = Point2D{ x: 0, y: 0 };
        let mut s = 1;
        while s < n {
            let rx = 1 & (t / 2);
            let ry = 1 & (t ^ rx);
            rotate(s, &mut xy, rx, ry);
            xy.x += s * rx;
            xy.y += s * ry;
            t /= 4;
            s *= 2;
        }
        Tile { zoom, x: xy.x, y: xy.y }
    }

    pub fn to_id(&self) -> u64 {
        if 
            self.zoom > 26 ||
            self.x > 2u64.pow(self.zoom as u32) - 1 ||
            self.y > 2u64.pow(self.zoom as u32) - 1
        { unreachable!() }
      
        let acc: u64 = TZ_VALUES[self.zoom as usize];
        let n = 2u64.pow(self.zoom as u32);
        let mut d = 0;
        let mut xy = Point2D{ x: self.x, y: self.y };
        let mut s = n / 2;
        while s > 0 {
            let rx = if (xy.x & s) > 0 { 1 } else { 0 };
            let ry = if (xy.y & s) > 0 { 1 } else { 0 };
            d += s * s * ((3 * rx) ^ ry);
            rotate(s, &mut xy, rx, ry);
            s /= 2;
        }

        acc + d
    }
}

/// PMTiles v3 directory entry.
#[derive(Clone, Copy)]
pub struct Entry {
    tile_id: u64,
    offset: u64,
    length: u32,
    run_length: u32,
}
impl Entry {
    pub fn new(tile_id: u64, offset: u64, length: u32, run_length: u32) -> Entry {
        Entry{ tile_id, offset, length, run_length }
    }
}

pub struct Directory {
    pub entries: Vec<Entry>,
}
impl Directory {
    pub fn from_buffer(buffer: &mut Buffer) -> Directory {
        let num_entries = buffer.read_varint::<usize>();

        let mut entries: Vec<Entry> = Vec::new();

        let mut last_id = 0;
        for _ in 0..num_entries {
            let v = buffer.read_varint::<u64>();
            entries.push(Entry::new(last_id + v, 0, 0, 1));
            last_id += v;
        }

        // run lengths, lengths, and offsets
        entries.iter_mut().for_each(|e| e.run_length = buffer.read_varint::<u32>());
        entries.iter_mut().for_each(|e| e.length = buffer.read_varint::<u32>());
        for i in 0..num_entries {
            let v = buffer.read_varint::<u64>();
            if v == 0 && i > 0 {
                entries[i].offset = entries[i - 1].offset + entries[i - 1].length as u64;
            } else {
                entries[i].offset = v - 1;
            }
        }

        Directory{ entries }
    }
}

/// Enum representing a compression algorithm used.
/// 0 = unknown compression, for if you must use a different or unspecified algorithm.
/// 1 = no compression.
/// 2 = gzip
/// 3 = brotli
/// 4 = zstd
#[derive(Default)]
pub enum Compression {
  /// unknown compression, for if you must use a different or unspecified algorithm
  Unknown = 0,
  /// no compression
  #[default] None = 1,
  /// gzip
  Gzip = 2,
  /// brotli
  Brotli = 3,
  /// zstd
  Zstd = 4,
}
impl From<u8> for Compression {
  fn from(value: u8) -> Self {
    match value {
      1 => Compression::None,
      2 => Compression::Gzip,
      3 => Compression::Brotli,
      4 => Compression::Zstd,
      _ => Compression::Unknown
    }
  }
}

/// Describe the type of tiles stored in the archive.
/// 0 is unknown/other, 1 is "MVT" vector tiles.
#[derive(Default)]
pub enum TileType {
  /// unknown/other.
  Unknown = 0,
  /// Vector tiles.
  #[default] Pbf = 1,
  /// Image tiles.
  Png = 2,
  /// Image tiles.
  Jpeg = 3,
  /// Image tiles.
  Webp = 4,
  /// Image tiles.
  Avif = 5,
}
impl From<u8> for TileType {
  fn from(value: u8) -> Self {
    match value {
      1 => TileType::Pbf,
      2 => TileType::Png,
      3 => TileType::Jpeg,
      4 => TileType::Webp,
      5 => TileType::Avif,
      _ => TileType::Unknown
    }
  }
}

pub const MAX_INITIAL_BYTES: usize = 16_384;

pub const HEADER_SIZE: usize = 127;

/// PMTiles v3 header storing basic archive-level information.
#[derive(Default)]
pub struct Header {
    pub version: u8,
    pub root_offset: u64,
    pub root_length: u64,
    pub metadata_offset: u64,
    pub metadata_length: u64,
    pub leaf_offset: u64,
    pub leaf_length: u64,
    pub data_offset: u64,
    pub data_length: u64,
    pub n_addressed_tiles: Option<NonZeroU64>,
    pub n_tile_entries: Option<NonZeroU64>,
    pub n_tile_contents: Option<NonZeroU64>,
    pub clustered: bool,
    pub internal_compression: Compression,
    pub tile_compression: Compression,
    pub tile_type: TileType,
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub min_longitude: f32,
    pub min_latitude: f32,
    pub max_longitude: f32,
    pub max_latitude: f32,
    pub center_zoom: u8,
    pub center_longitude: f32,
    pub center_latitude: f32,
}
impl Header {
    pub fn from_bytes(buffer: &mut Buffer) -> Header {
        buffer.set_pos(7);
        Header {
            version: buffer.get_u8(),
            root_offset: buffer.get_u64(),
            root_length: buffer.get_u64(),
            metadata_offset: buffer.get_u64(),
            metadata_length: buffer.get_u64(),
            leaf_offset: buffer.get_u64(),
            leaf_length: buffer.get_u64(),
            data_offset: buffer.get_u64(),
            data_length: buffer.get_u64(),
            n_addressed_tiles: NonZeroU64::new(buffer.get_u64()),
            n_tile_entries: NonZeroU64::new(buffer.get_u64()),
            n_tile_contents: NonZeroU64::new(buffer.get_u64()),
            clustered: buffer.get_u8() == 1,
            internal_compression: Compression::from(buffer.get_u8()),
            tile_compression: Compression::from(buffer.get_u8()),
            tile_type: TileType::from(buffer.get_u8()),
            min_zoom: buffer.get_u8(),
            max_zoom: buffer.get_u8(),
            min_longitude: 0.0,
            min_latitude: 0.0,
            max_longitude: 0.0,
            max_latitude: 0.0,
            center_zoom: buffer.get_u8(),
            center_longitude: 0.0,
            center_latitude: 0.0,
        }
    }
}

/// rotate xy by n
pub fn rotate(n: u64, xy: &mut Point2D, rx: u64, ry: u64) {
  if ry == 0 {
    if rx == 1 {
      xy.x = n - 1 - xy.x;
      xy.y = n - 1 - xy.y;
    }
    core::mem::swap(&mut xy.x, &mut xy.y);
  }
}

/// Low-level function for looking up a Tile_id or leaf directory inside a directory.
pub fn find_tile(entries: Vec<Entry>, tile_id: u64) -> Option<Entry> {
    let mut m = 0;
    let mut n: isize = (entries.len() - 1).try_into().unwrap();
    while m <= n {
        let k = (n + m) >> 1;
        let cmp = tile_id - entries[k as usize].tile_id;
        match cmp.cmp(&0) {
            Ordering::Greater => m = k + 1,
            Ordering::Less => n = k - 1,
            Ordering::Equal => return Some(entries[k as usize]),
        }
    }

    // at this point, m > n
    if n >= 0 {
        let n: usize = n as usize;
        if entries[n].run_length == 0 {
            return Some(entries[n]);
        }
        if tile_id - entries[n].tile_id < entries[n].run_length as u64 {
            return Some(entries[n]);
        }
    }
    
    None
}

// /**
//  * @param header - the header object
//  * @returns the raw header bytes
//  */
// pub fn headerToBytes(header: Header): Uint8Array {
//   const dv = new DataView(new ArrayBuffer(HEADER_SIZE_BYTES));
//   dv.setUint16(0, 0x4d50, true);
//   dv.setUint8(7, header.specVersion);
//   setUint64(dv, 8, header.rootDirectoryOffset);
//   setUint64(dv, 16, header.rootDirectoryLength);
//   setUint64(dv, 24, header.jsonMetadataOffset);
//   setUint64(dv, 32, header.jsonMetadataLength);
//   setUint64(dv, 40, header.leafDirectoryOffset);
//   setUint64(dv, 48, header.leafDirectoryLength ?? 0);
//   setUint64(dv, 56, header.tileDataOffset);
//   setUint64(dv, 64, header.tileDataLength ?? 0);
//   setUint64(dv, 72, header.numAddressedTiles);
//   setUint64(dv, 80, header.numTileEntries);
//   setUint64(dv, 88, header.numTileContents);
//   dv.setUint8(96, header.clustered ? 1 : 0);
//   dv.setUint8(97, header.internalCompression);
//   dv.setUint8(98, header.tileCompression);
//   dv.setUint8(99, header.tileType);
//   dv.setUint8(100, header.minZoom);
//   dv.setUint8(101, header.maxZoom);

//   return new Uint8Array(dv.buffer, dv.byteOffset, dv.byteLength);
// }

// /// TODO
// pub fn deserialize_dir(buffer: Uint8Array): Entry[] {
//   const p = { buf: new Uint8Array(buffer), pos: 0 };
//   const numEntries = read_varint(p);

//   const entries: Entry[] = [];

//   let last_id = 0;
//   for (let i = 0; i < numEntries; i++) {
//     const v = read_varint(p);
//     entries.push({ tile_id: last_id + v, offset: 0, length: 0, runLength: 1 });
//     last_id += v;
//   }

//   // run lengths, lengths, and offsets
//   for (let i = 0; i < numEntries; i++) entries[i].runLength = read_varint(p);
//   for (let i = 0; i < numEntries; i++) entries[i].length = read_varint(p);
//   for (let i = 0; i < numEntries; i++) {
//     const v = read_varint(p);
//     if (v === 0 && i > 0) {
//       entries[i].offset = entries[i - 1].offset + entries[i - 1].length;
//     } else {
//       entries[i].offset = v - 1;
//     }
//   }

//   return entries;
// }

// /**
//  * @param entries - the directory entries
//  * @param compressor - the compressor to use, defaults to none
//  * @returns - the serialized directory
//  */
// pub fn serialize_dir(entries: Entry[], compressor: Compressor): Promise<Uint8Array> {
//   const data = { buf: new Uint8Array(0), pos: 0 };

//   writeVarint(entries.length, data);

//   let last_id = 0;
//   for (let i = 0; i < entries.length; i++) {
//     writeVarint(entries[i].tile_id - last_id, data);
//     last_id = entries[i].tile_id;
//   }

//   for (let i = 0; i < entries.length; i++) writeVarint(entries[i].runLength, data);
//   for (let i = 0; i < entries.length; i++) writeVarint(entries[i].length, data);
//   for (let i = 0; i < entries.length; i++) writeVarint(entries[i].offset + 1, data);

//   const buf = new Uint8Array(data.buf.buffer, data.buf.byteOffset, data.pos);

//   return await compressor(buf);
// }
