#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
//! The `s2-pmtiles` Rust crate provides functionalities to read and write S2-PMTiles Spec messages.
//! This crate supports `no_std` and is intended to be used in embedded systems and WASM
//! applications.

/// All encoding and decoding is done via u64.
/// So all types must implement this trait to be able to be encoded and decoded.
pub mod bit_cast;

/// The `Buffer` struct is used to read and write Buffer messages.
pub mod buffer;
/// A simple cache system with a maximum size.
pub mod cache;
/// The `PMTiles` specification tools
pub mod pmtiles;
/// The `S2PMTiles` tool for reading S2PMTiles and PMTiles messages
pub mod reader;
/// The `S2PMTiles` specification tools
pub mod s2pmtiles;
/// The `S2PMTiles` tool for writing S2PMTiles and PMTiles messages
pub mod writer;

pub use buffer::*;
pub use cache::*;
pub use pmtiles::*;
pub use s2pmtiles::*;

/// Add two usize numbers into one
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(1, 2);
        let result2 = add(1, 1);

        assert_eq!(result, 3);
        assert_eq!(result2, 2);
    }
}
