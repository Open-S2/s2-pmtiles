#[cfg(feature = "alloc")]
extern crate alloc;

use alloc::vec::Vec;
use core::cmp::Ordering;
use alloc::string::String;

use crate::buffer::Buffer;

/// zoom values for each zoom level. Supports up to 27 zooms
pub const TZ_VALUES: [u64; 27] = [
  0, 1, 5, 21, 85, 341, 1365, 5461, 21845, 87381, 349525, 1398101, 5592405, 22369621, 89478485,
  357913941, 1431655765, 5726623061, 22906492245, 91625968981, 366503875925, 1466015503701,
  5864062014805, 23456248059221, 93824992236885, 375299968947541, 1501199875790165,
];
/// the number of bytes in the header
pub const HEADER_SIZE_BYTES: usize = 127;
/// the number of bytes in the root
pub const ROOT_SIZE: usize = 16_384;

/// An array of two numbers representing a point in 2D space
pub struct Point2D {
    /// x coordinate
    pub x: u64,
    /// y coordinate
    pub y: u64,
}

/// A tile, in the format of ZXY
pub struct Tile {
    /// zoom level
    pub zoom: u8,
    /// x coordinate
    pub x: u64,
    /// y coordinate
    pub y: u64,
}
impl Tile {
    /// Create a Tile instance from a zoom, x, and y
    pub fn new(zoom: u8, x: u64, y: u64) -> Tile {
        Tile { zoom, x, y }
    }

    /// Create a Tile instance from an ID
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

    /// Create a Tile instance from a zoom and position
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

    /// Convert a Tile instance to an ID
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
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Entry {
    /// tile ID
    pub tile_id: u64,
    /// offset relative to where tile data starts in the file
    pub offset: u64,
    /// length in bytes
    pub length: u32,
    /// run length
    pub run_length: u32,
}
impl Entry {
    /// Create a new directory entry
    pub fn new(tile_id: u64, offset: u64, length: u32, run_length: u32) -> Entry {
        Entry{ tile_id, offset, length, run_length }
    }
}

/// PMTiles v3 directory. A collection of Entry instances for storage
#[derive(Debug, Clone, Default)]
pub struct Directory {
    /// entries
    pub entries: Vec<Entry>,
}
impl Directory {
    /// Create a new directory
    pub fn new(entries: Vec<Entry>) -> Directory {
        Directory{ entries }
    }

    /// Create a new directory from a buffer
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

    /// Serialize the directory into a buffer
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Buffer::new();

        buffer.write_varint(self.entries.len() as u64);

        let mut last_id = 0;
        for e in &self.entries {
            buffer.write_varint(e.tile_id - last_id);
            last_id = e.tile_id;
        }

        for e in &self.entries { buffer.write_varint(e.run_length as u64); }
        for e in &self.entries { buffer.write_varint(e.length as u64); }
        for e in &self.entries { buffer.write_varint(e.offset + 1); }

        buffer.take()
    }

    /// Check if the directory is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Get an entry
    pub fn get(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|e| e.tile_id == id)
    }

    /// Get an entry mutable
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.tile_id == id)
    }

    /// Set an entry
    pub fn set(&mut self, id: u64, entry: Entry) {
        if let Some(e) = self.get_mut(id) {
            *e = entry;
        } else {
            self.entries.push(entry);
        }
    }

    /// Insert an entry
    pub fn insert(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Get the first entry
    pub fn first(&self) -> Option<&Entry> {
        self.entries.first()
    }

    /// Get the first entry mutable
    pub fn first_mut(&mut self) -> Option<&mut Entry> {
        self.entries.first_mut()
    }

    /// Get the last entry
    pub fn last(&self) -> Option<&Entry> {
        self.entries.last()
    }

    /// Get the last entry mutable
    pub fn last_mut(&mut self) -> Option<&mut Entry> {
        self.entries.last_mut()
    }
}

/// Enum representing a compression algorithm used.
/// 0 = unknown compression, for if you must use a different or unspecified algorithm.
/// 1 = no compression.
/// 2 = gzip
/// 3 = brotli
/// 4 = zstd
#[derive(Debug, Copy, Clone, Default)]
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
impl From<Compression> for u8 {
    fn from(compression: Compression) -> Self {
        match compression {
            Compression::Unknown => 0,
            Compression::None => 1,
            Compression::Gzip => 2,
            Compression::Brotli => 3,
            Compression::Zstd => 4,
        }
    }
}
impl From<Compression> for String {
    fn from(comp: Compression) -> Self {
        match comp {
            Compression::None => "none".into(),
            Compression::Gzip => "gzip".into(),
            Compression::Brotli => "br".into(),
            Compression::Zstd => "zstd".into(),
            Compression::Unknown => "unknown".into(),
        }
    }
}


/// Describe the type of tiles stored in the archive.
/// 0 is unknown/other, 1 is "MVT" vector tiles.
#[derive(Debug, Copy, Clone, Default)]
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
impl From<TileType> for u8 {
    fn from(t_type: TileType) -> Self {
        match t_type {
            TileType::Unknown => 0,
            TileType::Pbf => 1,
            TileType::Png => 2,
            TileType::Jpeg => 3,
            TileType::Webp => 4,
            TileType::Avif => 5,
        }
    }
}
impl From<TileType> for String {
    fn from(t_type: TileType) -> Self {
        match t_type {
            TileType::Unknown => "unknown".into(),
            TileType::Pbf => "pbf".into(),
            TileType::Png => "png".into(),
            TileType::Jpeg => "jpeg".into(),
            TileType::Webp => "webp".into(),
            TileType::Avif => "avif".into(),
        }
    }
}

/// PMTiles v3 header storing basic archive-level information.
#[derive(Debug, Default)]
pub struct Header {
    /// Only v3 PMTiles supported
    pub version: u8,
    /// the offset in the archive of the root directory
    pub root_directory_offset: u64,
    /// the length of the root directory
    pub root_directory_length: u64,
    /// the offset in the archive of the JSON metadata
    pub metadata_offset: u64,
    /// the length of the metadata
    pub metadata_length: u64,
    /// the offset in the archive of the leaf directory
    pub leaf_directory_offset: u64,
    /// the length of the leaf directory
    pub leaf_directory_length: u64,
    /// the offset in the archive of the tile data
    pub data_offset: u64,
    /// the length of the tile data
    pub data_length: u64,
    /// the number of addressed tiles
    pub n_addressed_tiles: Option<u64>,
    /// the number of tile entries
    pub n_tile_entries: Option<u64>,
    /// the number of tile contents
    pub n_tile_contents: Option<u64>,
    /// if the archive is clustered
    pub clustered: bool,
    /// what kind of compression is used for the Entries and metadata
    /// This is depreacted and will always be NONE for S2PMTiles
    pub internal_compression: Compression,
    /// what kind of compression is used for the tiles
    pub tile_compression: Compression,
    /// the type of the tiles
    pub tile_type: TileType,
    /// the min zoom level
    pub min_zoom: u8,
    /// the max zoom level
    pub max_zoom: u8,
    /// the min longitude
    pub min_longitude: f32,
    /// the min latitude
    pub min_latitude: f32,
    /// the max longitude
    pub max_longitude: f32,
    /// the max latitude
    pub max_latitude: f32,
    /// the center zoom level
    pub center_zoom: u8,
    /// the center longitude
    pub center_longitude: f32,
    /// the center latitude
    pub center_latitude: f32,
}
impl Header {
    /// Create a new Header from a buffer
    pub fn from_bytes(buffer: &mut Buffer) -> Header {
        buffer.set_pos(7);
        Header {
            version: buffer.get_u8_at(7),
            root_directory_offset: buffer.get_u64_at(8),
            root_directory_length: buffer.get_u64_at(16),
            metadata_offset: buffer.get_u64_at(24),
            metadata_length: buffer.get_u64_at(32),
            leaf_directory_offset: buffer.get_u64_at(40),
            leaf_directory_length: buffer.get_u64_at(48),
            data_offset: buffer.get_u64_at(56),
            data_length: buffer.get_u64_at(64),
            n_addressed_tiles: Some(buffer.get_u64_at(72)),
            n_tile_entries: Some(buffer.get_u64_at(80)),
            n_tile_contents: Some(buffer.get_u64_at(88)),
            clustered: buffer.get_u8_at(96) == 1,
            internal_compression: Compression::from(buffer.get_u8_at(97)),
            tile_compression: Compression::from(buffer.get_u8_at(98)),
            tile_type: TileType::from(buffer.get_u8_at(99)),
            min_zoom: buffer.get_u8_at(100),
            max_zoom: buffer.get_u8_at(101),
            min_longitude: (buffer.get_i32_at(102) as f32) / 10_000_000.0,
            min_latitude: (buffer.get_i32_at(106) as f32) / 10_000_000.0,
            max_longitude: (buffer.get_i32_at(110) as f32) / 10_000_000.0,
            max_latitude: (buffer.get_i32_at(114) as f32) / 10_000_000.0,
            center_zoom: buffer.get_u8_at(118),
            center_longitude: (buffer.get_i32_at(119) as f32) / 10_000_000.0,
            center_latitude: (buffer.get_i32_at(123) as f32) / 10_000_000.0,
        }
    }

    /// Write the header to a buffer
    pub fn to_bytes(&self) -> Buffer {
        let mut buffer = Buffer::new();

        // set id
        buffer.set_u16_at(0, 0x4d50); // set PM
        // Version number at position 7
        buffer.set_u8_at(7, 3);

        // Root directory offset and length at positions 8 and 16
        buffer.set_u64_at(8, self.root_directory_offset);
        buffer.set_u64_at(16, self.root_directory_length);

        // JSON metadata offset and length at positions 24 and 32
        buffer.set_u64_at(24, self.metadata_offset);
        buffer.set_u64_at(32, self.metadata_length);

        // Leaf directory offset and optional length at positions 40 and 48
        buffer.set_u64_at(40, self.leaf_directory_offset);
        buffer.set_u64_at(48, self.leaf_directory_length);

        // Tile data offset and optional length at positions 56 and 64
        buffer.set_u64_at(56, self.data_offset);
        buffer.set_u64_at(64, self.data_length);

        // Number of addressed tiles, tile entries, and tile contents at positions 72, 80, and 88
        buffer.set_u64_at(72, self.n_addressed_tiles.unwrap_or(0));
        buffer.set_u64_at(80, self.n_tile_entries.unwrap_or(0));
        buffer.set_u64_at(88, self.n_tile_contents.unwrap_or(0));

        // Flags and types at positions 96 through 101
        buffer.set_u8_at(96, if self.clustered { 1 } else { 0 });
        buffer.set_u8_at(97, self.internal_compression.into());
        buffer.set_u8_at(98, self.tile_compression.into());
        buffer.set_u8_at(99, self.tile_type.into());
        buffer.set_u8_at(100, self.min_zoom);
        buffer.set_u8_at(101, self.max_zoom);

        buffer
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
pub fn find_tile(entries: &[Entry], tile_id: u64) -> Option<Entry> {
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
