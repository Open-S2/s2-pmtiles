#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
use std::fs::{File, OpenOptions};
#[cfg(feature = "std")]
use std::io::{self, Seek, SeekFrom, Write};

use alloc::vec::Vec;
use crate::{
    Compression, Directory, Entry, Header, S2Header, S2Entries, Tile, TileType,
    S2_HEADER_SIZE_BYTES, S2_ROOT_SIZE
};
use s2_tilejson::{Face, Metadata};
use sha2::{Sha256, Digest};

/// The result of an optimized directory computation
#[derive(Debug, Clone, Default)]
pub struct OptimizedDirectory {
    /// The root directory bytes
    pub root_bytes: Vec<u8>,
    /// The leaf directories bytes
    pub leaves_bytes: Vec<u8>,
    /// The number of leaf directories
    pub num_leaves: u64,
}
impl OptimizedDirectory {
    /// Optimize the directory for storage
    pub fn optimize_directories(directory: &Directory, target_root_length: usize) -> OptimizedDirectory {
        let test_bytes = directory.serialize();
        if test_bytes.len() < target_root_length {
            OptimizedDirectory {
                root_bytes: test_bytes,
                leaves_bytes: Vec::new(),
                num_leaves: 0
            }
        } else {
            let mut leaf_size = 4096;
            loop {
                let build = OptimizedDirectory::build_root_leaves(directory, leaf_size);
                if build.root_bytes.len() < target_root_length {
                    return build;
                }
                leaf_size *= 2;
            }
        }
    }

    /// Build the root and leaf directories
    pub fn build_root_leaves(directory: &Directory, leaf_size: usize) -> OptimizedDirectory {
        let mut root_entries = Directory::default();
        let mut leaves_bytes = Vec::<u8>::new();
        let mut num_leaves = 0;

        let mut i = 0;
        let entries = &directory.entries;
        while i < entries.len() {
            num_leaves += 1;
            let new_dir_slice = Directory::new(entries[i..i + leaf_size].to_vec());
            let serialized = new_dir_slice.serialize();
            let entry = Entry {
                tile_id: entries[i].tile_id,
                offset: leaves_bytes.len() as u64,
                length: serialized.len() as u32,
                run_length: 0,
            };
            root_entries.entries.push(entry);
            leaves_bytes.extend(serialized);
            i += leaf_size;
        }

        OptimizedDirectory { root_bytes: root_entries.serialize(), leaves_bytes, num_leaves }
    }
}

/// The data writer
pub trait DataWriter: core::fmt::Debug {
    /// Write data at the specified offset
    fn write_data(&mut self, data: &[u8], offset: u64);
    /// Append data to the end of the storage
    fn append_data(&mut self, data: &[u8]);
}

/// If `std` is enabled use the `FileWriter`
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct FileWriter {
    file: File,
}
#[cfg(feature = "std")]
impl FileWriter {
    /// Create a new `FileWriter`
    pub fn create(path: &str) -> Result<Self, io::Error> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        Ok(Self { file })
    }
}
#[cfg(feature = "std")]
impl DataWriter for FileWriter {
    fn write_data(&mut self, data: &[u8], offset: u64) {
        // Write bytes to the file at the specified offset
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.file.write_all(data).unwrap();
    }

    fn append_data(&mut self, data: &[u8]) {
        // Append bytes to the end of the file
        self.file.seek(SeekFrom::End(0)).unwrap();
        self.file.write_all(data).unwrap();
    }
}

/// The local writer is when not using `std` and stores everything to a `Vec<u8>`
#[derive(Debug, Default)]
pub struct LocalWriter {
    /// The data storage container
    data: Vec<u8>,
}
impl LocalWriter {
    /// When done writing, take ownership of the data
    pub fn take(self) -> Vec<u8> {
        self.data
    }
}
impl DataWriter for LocalWriter {
    fn write_data(&mut self, data: &[u8], offset: u64) {
        // Ensure data vector is large enough to accommodate the write
        if (offset as usize + data.len()) > self.data.len() {
            self.data.resize(offset as usize + data.len(), 0u8);
        }
        // Write data to the vector at the specified offset
        self.data[offset as usize..(offset as usize + data.len())].copy_from_slice(data);
    }

    fn append_data(&mut self, data: &[u8]) {
        // Append data to the end of the vector
        self.data.extend_from_slice(data);
    }
}

/// The File reader is to be used by the local filesystem.
#[derive(Debug)]
pub struct PMTileswriter {
    tile_entries: Directory,
    s2tile_entries: S2Entries,
    offset: u64,
    hash_to_offset: std::collections::HashMap<[u8; 32], u64>,
    addressed_tiles: u64,
    clustered: bool,
    compression: Compression,
    data_writer: Box<dyn DataWriter>
}
impl PMTileswriter {
    /// given a compression scheme and a data writer, create an instance to start storing tiles
    /// and metadata.
    /// Compression will only describle how tiles are stored, nothing more.
    pub fn new(compression: Compression, data_writer: Box<dyn DataWriter>) -> Self {
        PMTileswriter {
            tile_entries: Directory::default(),
            s2tile_entries: S2Entries::default(),
            hash_to_offset: std::collections::HashMap::new(),
            offset: 0,
            addressed_tiles: 0,
            clustered: false,
            compression,
            data_writer
        }
    }

    /// Write a tile to the PMTiles file given its (face, zoom, x, y) coordinates.
    pub fn write_tile_xyz(&mut self, zoom: u8, x: u64, y: u64, data: &[u8]) {
        let tile_id = Tile::new(zoom, x, y).to_id();
        self.write_tile(tile_id, data, None);
    }

    /// Write a tile to the PMTiles file given its (face, zoom, x, y) coordinates.
    pub fn write_tile_s2(&mut self, face: Face, zoom: u8, x: u64, y: u64, data: &[u8]) {
        let tile_id = Tile::new(zoom, x, y).to_id();
        self.write_tile(tile_id, data, Some(face));
    }

    /// Write a tile to the PMTiles file given its tile ID.
    pub fn write_tile(&mut self, tile_id: u64, data: &[u8], face: Option<Face>) {
        let length = data.len();
        let tile_entries = match face {
            None => &mut self.tile_entries,
            Some(f) => self.s2tile_entries.get_mut(f),
        };
        if !tile_entries.is_empty() && tile_id < tile_entries.last().unwrap().tile_id {
            self.clustered = false;
        }

        let hsh = hash_data(data);
        match self.hash_to_offset.get(&hsh) {
            Some(offset) => {
                let last = tile_entries.last_mut().unwrap();
                if tile_id == last.tile_id + last.run_length as u64 && last.offset == *offset {
                    last.run_length += 1;
                } else {
                    tile_entries.insert(Entry{ tile_id, offset: *offset, length: length as u32, run_length: 1 });
                }
            },
            None => {
                let offset = self.offset;
                self.data_writer.append_data(data);
                tile_entries.insert(Entry { tile_id, offset, length: length as u32, run_length: 1 });
                self.hash_to_offset.insert(hsh, offset);
                self.offset += length as u64;
            }
        }

        self.addressed_tiles += 1;
    }

    /// Finish writing by building the header with root and leaf directories
    pub fn commit(&mut self, metadata: &Metadata) {
        if !self.tile_entries.is_empty() {
            self.commit_wm(metadata);
        } else {
            self.commit_s2(metadata);
        }
    }

    /// Finish writing by building the header with root and leaf directories
    pub fn commit_wm(&mut self, metadata: &Metadata) {
        // build metadata
        let meta_buffer = serde_json::to_vec(metadata).unwrap();

        // optimize directories
        let od = OptimizedDirectory::optimize_directories(
            &self.tile_entries,
            S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - meta_buffer.len()
        );
        let OptimizedDirectory { root_bytes, leaves_bytes, .. } = od;

        // build header data
        let root_directory_offset = S2_HEADER_SIZE_BYTES as u64;
        let root_directory_length = root_bytes.len() as u64;
        let metadata_offset = root_directory_offset + root_directory_length;
        let metadata_length = meta_buffer.len() as u64;
        let leaf_directory_offset = self.offset + S2_ROOT_SIZE as u64;
        let leaf_directory_length = leaves_bytes.len() as u64;
        self.offset += leaves_bytes.len() as u64;

        // write data
        self.data_writer.append_data(&leaves_bytes);
        // to make writing fasters
        let min_zoom = Tile::from_id(self.tile_entries.first().unwrap().tile_id).zoom;
        let max_zoom = Tile::from_id(self.tile_entries.last().unwrap().tile_id).zoom;

        // build header
        let header = Header {
            version: 3,
            root_directory_offset,
            root_directory_length,
            metadata_offset,
            metadata_length,
            leaf_directory_offset,
            leaf_directory_length,
            data_offset: S2_ROOT_SIZE as u64,
            data_length: self.offset,
            n_addressed_tiles: self.addressed_tiles,
            n_tile_entries: self.tile_entries.len() as u64,
            n_tile_contents: self.hash_to_offset.len() as u64,
            clustered: self.clustered,
            internal_compression: Compression::None,
            tile_compression: self.compression,
            tile_type: TileType::Unknown,
            min_zoom,
            max_zoom,
            ..Default::default()
        };
        let serialized_header = header.to_bytes().take();

        // write header
        self.data_writer.write_data(&serialized_header, 0);
        self.data_writer.write_data(&root_bytes, root_directory_offset);
        self.data_writer.write_data(&meta_buffer, metadata_offset);
    }

    /// Finish writing by building the header with root and leaf directories
    pub fn commit_s2(&mut self, metadata: &Metadata) {
        // build metadata
        let meta_buffer = serde_json::to_vec(metadata).unwrap();

        // optimize directories
        let od = OptimizedDirectory::optimize_directories(
            self.s2tile_entries.get(Face::Face0),
            S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - meta_buffer.len()
        );
        let OptimizedDirectory { root_bytes, leaves_bytes, .. } = od;
        let od1 = OptimizedDirectory::optimize_directories(
            self.s2tile_entries.get(Face::Face1),
            S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - meta_buffer.len()
        );
        let OptimizedDirectory { root_bytes: root_bytes1, leaves_bytes: leaves_bytes1, .. } = od1;
        let od2 = OptimizedDirectory::optimize_directories(
            self.s2tile_entries.get(Face::Face2),
            S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - meta_buffer.len()
        );
        let OptimizedDirectory { root_bytes: root_bytes2, leaves_bytes: leaves_bytes2, .. } = od2;
        let od3 = OptimizedDirectory::optimize_directories(
            self.s2tile_entries.get(Face::Face3),
            S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - meta_buffer.len()
        );
        let OptimizedDirectory { root_bytes: root_bytes3, leaves_bytes: leaves_bytes3, .. } = od3;
        let od4 = OptimizedDirectory::optimize_directories(
            self.s2tile_entries.get(Face::Face4),
            S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - meta_buffer.len()
        );
        let OptimizedDirectory { root_bytes: root_bytes4, leaves_bytes: leaves_bytes4, .. } = od4;
        let od5 = OptimizedDirectory::optimize_directories(
            self.s2tile_entries.get(Face::Face5),
            S2_ROOT_SIZE - S2_HEADER_SIZE_BYTES - meta_buffer.len()
        );
        let OptimizedDirectory { root_bytes: root_bytes5, leaves_bytes: leaves_bytes5, .. } = od5;

        // build header data
        // roots
        let root_directory_offset = S2_HEADER_SIZE_BYTES as u64;
        let root_directory_length = root_bytes.len() as u64;
        let root_directory_offset1 = root_directory_offset + root_directory_length;
        let root_directory_length1 = root_bytes1.len() as u64;
        let root_directory_offset2 = root_directory_offset1 + root_directory_length1;
        let root_directory_length2 = root_bytes2.len() as u64;
        let root_directory_offset3 = root_directory_offset2 + root_directory_length2;
        let root_directory_length3 = root_bytes3.len() as u64;
        let root_directory_offset4 = root_directory_offset3 + root_directory_length3;
        let root_directory_length4 = root_bytes4.len() as u64;
        let root_directory_offset5 = root_directory_offset4 + root_directory_length4;
        let root_directory_length5 = root_bytes5.len() as u64;
        // metadata
        let metadata_offset = root_directory_offset + root_directory_length;
        let metadata_length = meta_buffer.len() as u64;
        // leafs
        let leaf_directory_offset = self.offset + S2_ROOT_SIZE as u64;
        let leaf_directory_length = leaves_bytes.len() as u64;
        self.offset += leaf_directory_length;
        self.data_writer.append_data(&leaves_bytes);
        let leaf_directory_offset1 = self.offset + S2_ROOT_SIZE as u64;
        let leaf_directory_length1 = leaves_bytes1.len() as u64;
        self.offset += leaf_directory_length1;
        self.data_writer.append_data(&leaves_bytes1);
        let leaf_directory_offset2 = self.offset + S2_ROOT_SIZE as u64;
        let leaf_directory_length2 = leaves_bytes2.len() as u64;
        self.offset += leaf_directory_length2;
        self.data_writer.append_data(&leaves_bytes2);
        let leaf_directory_offset3 = self.offset + S2_ROOT_SIZE as u64;
        let leaf_directory_length3 = leaves_bytes3.len() as u64;
        self.offset += leaf_directory_length3;
        self.data_writer.append_data(&leaves_bytes3);
        let leaf_directory_offset4 = self.offset + S2_ROOT_SIZE as u64;
        let leaf_directory_length4 = leaves_bytes4.len() as u64;
        self.offset += leaf_directory_length4;
        self.data_writer.append_data(&leaves_bytes4);
        let leaf_directory_offset5 = self.offset + S2_ROOT_SIZE as u64;
        let leaf_directory_length5 = leaves_bytes5.len() as u64;
        self.offset += leaf_directory_length5;
        self.data_writer.append_data(&leaves_bytes5);

        // write data
        self.data_writer.append_data(&leaves_bytes);
        // to make writing fasters
        let min_zoom = Tile::from_id(self.tile_entries.first().unwrap().tile_id).zoom;
        let max_zoom = Tile::from_id(self.tile_entries.last().unwrap().tile_id).zoom;

        // build header
        let header = S2Header {
            is_s2: true,
            version: 3,
            root_directory_offset,
            root_directory_length,
            root_directory_offset1,
            root_directory_length1,
            root_directory_offset2,
            root_directory_length2,
            root_directory_offset3,
            root_directory_length3,
            root_directory_offset4,
            root_directory_length4,
            root_directory_offset5,
            root_directory_length5,
            metadata_offset,
            metadata_length,
            leaf_directory_offset,
            leaf_directory_length,
            leaf_directory_offset1,
            leaf_directory_length1,
            leaf_directory_offset2,
            leaf_directory_length2,
            leaf_directory_offset3,
            leaf_directory_length3,
            leaf_directory_offset4,
            leaf_directory_length4,
            leaf_directory_offset5,
            leaf_directory_length5,
            data_offset: S2_ROOT_SIZE as u64,
            data_length: self.offset,
            n_addressed_tiles: self.addressed_tiles,
            n_tile_entries: self.tile_entries.len() as u64,
            n_tile_contents: self.hash_to_offset.len() as u64,
            clustered: self.clustered,
            internal_compression: Compression::None,
            tile_compression: self.compression,
            tile_type: TileType::Unknown,
            min_zoom,
            max_zoom,
            ..Default::default()
        };
        let serialized_header = header.to_bytes().take();

        // write header
        self.data_writer.write_data(&serialized_header, 0);
        self.data_writer.write_data(&root_bytes, root_directory_offset);
        self.data_writer.write_data(&root_bytes1, root_directory_offset1);
        self.data_writer.write_data(&root_bytes2, root_directory_offset2);
        self.data_writer.write_data(&root_bytes3, root_directory_offset3);
        self.data_writer.write_data(&root_bytes4, root_directory_offset4);
        self.data_writer.write_data(&root_bytes5, root_directory_offset5);
        self.data_writer.write_data(&meta_buffer, metadata_offset);
    }
}

fn hash_data(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
