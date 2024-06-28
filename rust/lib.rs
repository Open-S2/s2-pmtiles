#![no_std]
// #![deny(missing_docs)]
//! The `s2-pmtiles` Rust crate provides functionalities to read and write Open Vector Tile Spec messages.
//! This crate is a 0 dependency package that uses `no_std` and is intended to be used in
//! embedded systems and WASM applications.

/// All encoding and decoding is done via u64.
/// So all types must implement this trait to be able to be encoded and decoded.
pub mod bit_cast;

// pub mod buffer;
// pub mod pmtiles;

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
