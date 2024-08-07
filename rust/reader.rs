#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::{Read, Seek};
#[cfg(feature = "std")]
use flate2::read::GzDecoder;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use crate::{
    Tile, S2Header, S2Entries, DirCache, Compression,
    S2_ROOT_SIZE, S2_HEADER_SIZE_BYTES, Directory, find_tile
};
use s2_tilejson::{Face, Metadata};

/// The data manager trait for the reader
pub trait DataManager: core::fmt::Debug {
    /// Get a range of bytes using the offset and length (both in byte sizes)
    fn get_range(&mut self, offset: u64, length: u64) -> Vec<u8>;
}

/// The file manager if using STD
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct FileManager {
    file: File,
}
#[cfg(feature = "std")]
impl FileManager {
    /// Open a file
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        Ok(Self { file })
    }
}

#[cfg(feature = "std")]
impl DataManager for FileManager {
    fn get_range(&mut self, offset: u64, length: u64) -> Vec<u8> {
        // Read bytes from the file
        let mut buf = vec![0u8; length as usize];
        self.file.seek(std::io::SeekFrom::Start(offset)).unwrap();  
        let _ = self.file.read(&mut buf).unwrap();

        buf
    }
}

/// The local manager if not using STD
#[derive(Debug)]
pub struct LocalManager {
    data: Vec<u8>,
}
impl LocalManager {
    /// Create a new local manager
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
impl DataManager for LocalManager {
    fn get_range(&mut self, offset: u64, length: u64) -> Vec<u8> {
        let offset = offset as usize;
        let length = (length as usize).min(self.data.len() - offset);
        self.data[offset..(offset + length)].to_vec()
    }
}

/// The File reader is to be used by the local filesystem.
#[derive(Debug)]
pub struct PMTilesReader {
    header: Option<S2Header>,
    root_dir: Directory,
    root_dir_s2: S2Entries,
    metadata: Metadata,
    dir_cache: DirCache<u64, Directory>,
    data_manager: Box<dyn DataManager>
}
impl PMTilesReader {
    /// Given an input path, read in the header and root directory
    pub fn new(data_manager: Box<dyn DataManager>, max_size: Option<usize>) -> Self {
        let max_size = max_size.unwrap_or(20);
        Self {
            header: None,
            root_dir: Directory::default(),
            root_dir_s2: S2Entries::default(),
            metadata: Metadata::default(),
            dir_cache: DirCache::new(max_size),
            data_manager,
        }
    }

    /// fetch the s2 metadata as needed
    pub fn get_header(&mut self) -> S2Header {
        if self.header.is_some() {
            return self.header.unwrap();
        }

        let data = self.data_manager.get_range(0, S2_ROOT_SIZE as u64);
        let header_data = &data[0..S2_HEADER_SIZE_BYTES];
        // header
        let mut header = S2Header::from_bytes(&mut header_data.into());

        // json metadata
        let json_offset = header.metadata_offset as usize;
        let json_length = header.metadata_length as usize;
        let json_metadata = decompress(
            &data[json_offset..(json_offset + json_length)],
            header.internal_compression
        );
        self.metadata = serde_json::from_str(
            &String::from_utf8_lossy(&json_metadata)
        ).unwrap_or_else(|e| panic!("ERROR: {}", e));

        // root directory data
        let root_dir_offset = header.root_directory_offset as usize;
        let root_dir_length = header.root_directory_length as usize;
        let root_dir_data = decompress(
            &data[
                root_dir_offset..
                (root_dir_offset + root_dir_length)
            ],
            header.internal_compression
        );
        self.root_dir = Directory::from_buffer(&mut (&root_dir_data[..]).into());

        if header.is_s2 { self.get_s2_metadata(&data, &mut header); }

        self.header = Some(header);

        header
    }

    /// If S2, we need to build the other face's root directories
    pub fn get_s2_metadata(&mut self, data: &[u8], header: &mut S2Header) {
        // move the root directory to the s2 root
        self.root_dir_s2.face_0 = self.root_dir.clone();
        // add the 5 other faces
        for face in [Face::Face1, Face::Face2, Face::Face3, Face::Face4, Face::Face5] {
            let root_offset = header.get_root_offset(face) as usize;
            let root_length = header.get_root_length(face) as usize;
            let face_dir_data = decompress(
                &data[
                    root_offset..
                    (root_offset + root_length)
                ],
                header.internal_compression
            );
            self.root_dir_s2.set_dir(face, Directory::from_buffer(&mut (&face_dir_data[..]).into()));
        }
    }

    /// get the metadata
    pub fn get_metadata(&mut self) -> &Metadata {
        &self.metadata
    }

    /// get an S2 tile
    pub fn get_tile_s2(&mut self, face: Face, zoom: u8, x: u64, y: u64) -> Option<Vec<u8>> {
        self.get_tile(Some(face), zoom, x, y)
    }

    /// get an WM tile
    pub fn get_tile_zxy(&mut self, zoom: u8, x: u64, y: u64) -> Option<Vec<u8>> {
        self.get_tile(None, zoom, x, y)
    }

    /// get a tile, wheather WM or S2
    pub fn get_tile(&mut self, face: Option<Face>, zoom: u8, x: u64, y: u64) -> Option<Vec<u8>> {
        let header = self.get_header();
        let tile_id = Tile::new(zoom, x, y).to_id();
        // if zoom < header.min_zoom || zoom > header.max_zoom { return None; }

        let mut d_o = header.root_directory_offset;
        let mut d_l = header.root_directory_length;

        for _ in 0..4 {
            let directory = self.get_directory(d_o, d_l, face);
            if directory.is_empty() { return None; }
            let entry = find_tile(&directory.entries, tile_id);
            match entry {
                None => { return None; }
                Some(entry) => {
                    if entry.run_length > 0 {
                        let entry_data = self.get_range(header.data_offset + entry.offset, entry.length as u64);
                        return Some(decompress(&entry_data, header.internal_compression));
                    } else {
                        d_o = header.leaf_directory_offset + entry.offset;
                        d_l = entry.length as u64;
                    }
                }
            }
        }

        panic!("Maximum directory depth exceeded");
    }

    /// Get a full directory
    fn get_directory(&mut self, offset: u64, length: u64, face: Option<Face>) -> Directory {
        let dir = match face {
            None => &self.root_dir,
            Some(f) => self.root_dir_s2.get(f),
        };
        let internal_compression = self.header.unwrap().internal_compression;
        let root_directory_offset = self.header.unwrap().root_directory_offset;
        // if root_directory_offset, return roon
        if offset == root_directory_offset { return dir.clone(); }
        // check cache
        if let Some(cache) = self.dir_cache.get(&offset) {
            cache.clone()
        } else {
            // get from archive
            let resp = self.get_range(offset, length);
            let data = decompress(&resp, internal_compression);
            let directory = Directory::from_buffer(&mut (&data[..]).into());
            if directory.is_empty() { panic!("Empty directory is invalid"); }
            // save in cache
            self.dir_cache.set(offset, Directory::from_buffer(&mut (&data[..]).into()));

            directory
        }
    }

    /// Get a range of bytes given an offset and length
    fn get_range(&mut self, offset: u64, length: u64) -> Vec<u8> {
        self.data_manager.get_range(offset, length)
    }
}

/// Decompress the data based on the compression type
/// NOTE: Currently only supports `Compression::None`
fn decompress(data: &[u8], compression: Compression) -> Vec<u8> {
    match compression {
        Compression::None => data.to_vec(),
        #[cfg(feature = "std")]
        Compression::Gzip => {
            let mut gz = GzDecoder::new(data);
            let mut decompressed_data = Vec::new();
            gz.read_to_end(&mut decompressed_data).expect("Failed to decompress gzip data");
            decompressed_data
        },
        _ => panic!("Decompression error"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TileType;
    use s2_tilejson::{Scheme, SourceType, VectorLayer, Encoding};

    #[test]
    fn test_fixture_1() {
        let file_manager = FileManager::new("./test/fixtures/test_fixture_1.pmtiles").unwrap();
        let mut reader = PMTilesReader::new(Box::new(file_manager), None);

        let header = reader.get_header();
        assert_eq!(header, S2Header {
            is_s2: false,
            version: 3,
            root_directory_offset: 127,
            root_directory_length: 25,
            metadata_offset: 152,
            metadata_length: 247,
            leaf_directory_offset: 0,
            leaf_directory_length: 0,
            data_offset: 399,
            data_length: 69,
            n_addressed_tiles: 1,
            n_tile_entries: 1,
            n_tile_contents: 1,
            clustered: false,
            internal_compression: Compression::Gzip,
            tile_compression: Compression::Gzip,
            tile_type: TileType::Pbf,
            min_zoom: 0,
            max_zoom: 0,
            min_longitude: 0.0,
            min_latitude: 0.0,
            max_longitude: 0.9999999,
            max_latitude: 1.0,
            center_zoom: 0,
            center_longitude: 0.0,
            center_latitude: 0.0,
            root_directory_offset1: 0,
            root_directory_length1: 0,
            root_directory_offset2: 0,
            root_directory_length2: 0,
            root_directory_offset3: 0,
            root_directory_length3: 0,
            root_directory_offset4: 0,
            root_directory_length4: 0,
            root_directory_offset5: 0,
            root_directory_length5: 0,
            leaf_directory_offset1: 0,
            leaf_directory_length1: 0,
            leaf_directory_offset2: 0,
            leaf_directory_length2: 0,
            leaf_directory_offset3: 0,
            leaf_directory_length3: 0,
            leaf_directory_offset4: 0,
            leaf_directory_length4: 0,
            leaf_directory_offset5: 0,
            leaf_directory_length5: 0,
        });

        let metadata = reader.get_metadata();
        assert_eq!(*metadata, Metadata {
            s2tilejson: "".into(),
            version: "2".into(),
            name: "test_fixture_1.pmtiles".into(),
            scheme: Scheme::Fzxy,
            description: "test_fixture_1.pmtiles".into(),
            type_: SourceType::Unknown,
            extension: "".into(),
            encoding: Encoding::None,
            minzoom: 0,
            maxzoom: 0,
            vector_layers: vec![
                VectorLayer {
                    id: "test_fixture_1pmtiles".into(),
                    description: Some("".into()),
                    minzoom: Some(0),
                    maxzoom: Some(0),
                    ..Default::default()
                }
            ],
            ..Default::default()
        });

        let tile = reader.get_tile(None, 0, 0, 0).unwrap();
        assert_eq!(tile, vec![
            26, 47, 120, 2, 10, 21, 116, 101, 115, 116, 95, 102, 105, 120, 116, 117, 114, 101, 95, 49,
            112, 109, 116, 105, 108, 101, 115, 40, 128, 32, 18, 17, 24, 3, 34, 13, 9, 150, 32, 232, 31,
            26, 0, 24, 21, 0, 0, 23, 15,
        ]);
    }

    #[test]
    fn test_fixture_1_local_manager() {
        // read in "./test/fixtures/test_fixture_1.pmtiles" to a Vec<u8>
        let data = std::fs::read("./test/fixtures/test_fixture_1.pmtiles").unwrap();
        let local_manager = LocalManager::new(data);
        let mut reader = PMTilesReader::new(Box::new(local_manager), None);

        let header = reader.get_header();
        assert_eq!(header, S2Header {
            is_s2: false,
            version: 3,
            root_directory_offset: 127,
            root_directory_length: 25,
            metadata_offset: 152,
            metadata_length: 247,
            leaf_directory_offset: 0,
            leaf_directory_length: 0,
            data_offset: 399,
            data_length: 69,
            n_addressed_tiles: 1,
            n_tile_entries: 1,
            n_tile_contents: 1,
            clustered: false,
            internal_compression: Compression::Gzip,
            tile_compression: Compression::Gzip,
            tile_type: TileType::Pbf,
            min_zoom: 0,
            max_zoom: 0,
            min_longitude: 0.0,
            min_latitude: 0.0,
            max_longitude: 0.9999999,
            max_latitude: 1.0,
            center_zoom: 0,
            center_longitude: 0.0,
            center_latitude: 0.0,
            root_directory_offset1: 0,
            root_directory_length1: 0,
            root_directory_offset2: 0,
            root_directory_length2: 0,
            root_directory_offset3: 0,
            root_directory_length3: 0,
            root_directory_offset4: 0,
            root_directory_length4: 0,
            root_directory_offset5: 0,
            root_directory_length5: 0,
            leaf_directory_offset1: 0,
            leaf_directory_length1: 0,
            leaf_directory_offset2: 0,
            leaf_directory_length2: 0,
            leaf_directory_offset3: 0,
            leaf_directory_length3: 0,
            leaf_directory_offset4: 0,
            leaf_directory_length4: 0,
            leaf_directory_offset5: 0,
            leaf_directory_length5: 0,
        });

        let metadata = reader.get_metadata();
        assert_eq!(*metadata, Metadata {
            s2tilejson: "".into(),
            version: "2".into(),
            name: "test_fixture_1.pmtiles".into(),
            scheme: Scheme::Fzxy,
            description: "test_fixture_1.pmtiles".into(),
            type_: SourceType::Unknown,
            extension: "".into(),
            encoding: Encoding::None,
            minzoom: 0,
            maxzoom: 0,
            vector_layers: vec![
                VectorLayer {
                    id: "test_fixture_1pmtiles".into(),
                    description: Some("".into()),
                    minzoom: Some(0),
                    maxzoom: Some(0),
                    ..Default::default()
                }
            ],
            ..Default::default()
        });

        let tile = reader.get_tile(None, 0, 0, 0).unwrap();
        assert_eq!(tile, vec![
            26, 47, 120, 2, 10, 21, 116, 101, 115, 116, 95, 102, 105, 120, 116, 117, 114, 101, 95, 49,
            112, 109, 116, 105, 108, 101, 115, 40, 128, 32, 18, 17, 24, 3, 34, 13, 9, 150, 32, 232, 31,
            26, 0, 24, 21, 0, 0, 23, 15,
        ]);
    }

    #[test]
    fn decompress_test() {
        let data = vec![0, 1, 2, 3, 4];
        let decompressed = decompress(&data, Compression::None);
        assert_eq!(decompressed, data);
    }

    #[test]
    #[should_panic(expected = "Decompression error")]
    fn decompress_test_panic() {
        let data = vec![0, 1, 2, 3, 4];
        let _ = decompress(&data, Compression::Brotli);
    }
}
