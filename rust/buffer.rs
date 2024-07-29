#[cfg(feature = "alloc")]
extern crate alloc;

use alloc::vec::Vec;
use core::cell::RefCell;
use crate::bit_cast::BitCast;

const MAX_VARINT_LENGTH: usize = u64::BITS as usize * 8 / 7 + 1;
const BIT_SHIFT: [u64; 10] = [0, 7, 14, 21, 28, 35, 42, 49, 56, 63];

/// The `Buffer` struct is used to read and write Buffer messages.
///
/// # Example
/// Create a new Buffer instance:
/// ```
/// use s2_pmtiles::Buffer;
///
/// let mut buf = Buffer::new();
/// ```
/// Create a Buffer instance from a byte buffer:
/// ```
/// use s2_pmtiles::Buffer;
/// use std::cell::RefCell; // or use core::cell::RefCell; if sticking with no_std
///
/// let mut buf = vec![0x0A, 0x03, 0x74, 0x65, 0x73, 0x74];
/// let mut pbf = Buffer::from_input(RefCell::new(buf));
/// ```
#[derive(Default)]
pub struct Buffer {
    buf: RefCell<Vec<u8>>,
    pos: usize,
}
impl From<&[u8]> for Buffer {
    fn from(value: &[u8]) -> Self {
        Buffer::from_input(RefCell::new(value.to_vec()))
    }
}
impl Buffer {
    /// Create a new Buffer instance.
    pub fn new() -> Buffer {
        let buf = RefCell::new(Vec::new());
        Buffer { buf, pos: 0 }
    }

    /// Create a Buffer instance from a byte buffer.
    pub fn from_input(buf: RefCell<Vec<u8>>) -> Buffer {
        Buffer { buf, pos: 0 }
    }

    /// Set the position to read from the buffer next.
    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// get the length of the bufer
    pub fn len(&self) -> usize {
        self.buf.borrow().len()
    }

    /// check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// return the current u8 under the buffer
    pub fn get_u8(&mut self) -> u8 {
        let value = self.get_u8_at(self.pos);
        self.pos += 1;

        value
    }

    /// return the current u8 at position
    pub fn get_u8_at(&mut self, pos: usize) -> u8 {
        self.buf.borrow()[pos]
    }

    /// set the current u8 under the buffer
    pub fn set_u8(&mut self, value: u8) {
        self.set_u8_at(self.pos, value);
        self.pos += 1;
    }

    /// set the current u8 at position
    pub fn set_u8_at(&mut self, pos: usize, value: u8) {
        self.buf.borrow_mut()[pos] = value;
    }

    /// return the current i32 under the buffer
    pub fn get_i32(&mut self) -> i32 {
        let value = self.get_i32_at(self.pos);
        // Update the position
        self.pos += 4;

        value
    }

    /// return the current i32 at position
    pub fn get_i32_at(&mut self, pos: usize) -> i32 {
        // Borrow the buffer and slice the next 4 bytes
        let buf = self.buf.borrow();
        let bytes = &buf[pos..pos + 4];
        
        i32::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
    }


    /// set the current i32 under the buffer
    pub fn set_i32(&mut self, value: i32) {
        self.set_i32_at(self.pos, value);
        self.pos += 4;
    }

    /// set the current i32 at position
    pub fn set_i32_at(&mut self, pos: usize, value: i32) {
        // Borrow the buffer and slice the next 4 bytes
        let mut buf = self.buf.borrow_mut();
        let bytes = &mut buf[pos..pos + 4];
        
        bytes.copy_from_slice(&value.to_le_bytes());
    }

    /// return the current u16 under the buffer
    pub fn get_u16(&mut self) -> u16 {
        let value = self.get_u16_at(self.pos);
        // Update the position
        self.pos += 2;
        
        value
    }

    /// return the current u16 at position
    pub fn get_u16_at(&mut self, pos: usize) -> u16 {
        // Borrow the buffer and slice the next 2 bytes
        let buf = self.buf.borrow();
        let bytes = &buf[pos..pos + 2];
        
        u16::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
    }


    /// set the current u16 under the buffer
    pub fn set_u16(&mut self, value: u16) {
        self.set_u16_at(self.pos, value);
        self.pos += 2;
    }

    /// set the current u16 at position
    pub fn set_u16_at(&mut self, pos: usize, value: u16) {
        // Borrow the buffer and slice the next 2 bytes
        let mut buf = self.buf.borrow_mut();
        let bytes = &mut buf[pos..pos + 2];
        
        bytes.copy_from_slice(&value.to_le_bytes());
    }

    /// return the current u64 under the buffer
    pub fn get_u64(&mut self) ->u64 {
        let value = self.get_u64_at(self.pos);
        // Update the position
        self.pos += 8;
        
        value
    }

    /// return the current u64 at position
    pub fn get_u64_at(&mut self, pos: usize) ->u64 {
        // Borrow the buffer and slice the next 8 bytes
        let buf = self.buf.borrow();
        let bytes = &buf[pos..pos + 8];
        
        u64::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
    }

    /// set the current u64 under the buffer
    pub fn set_u64(&mut self, value: u64) {
        self.set_u64_at(self.pos, value);
        self.pos += 8;
    }

    /// set the current u64 at position
    pub fn set_u64_at(&mut self, pos: usize, value: u64) {
        // Borrow the buffer and slice the next 8 bytes
        let mut buf = self.buf.borrow_mut();
        let bytes = &mut buf[pos..pos + 8];
        
        bytes.copy_from_slice(&value.to_le_bytes());
    }

    /// Decode a varint from the buffer at the current position.
    pub fn decode_varint(&mut self) -> u64 {
        if self.pos >= self.len() {
            unreachable!();
        }

        let mut val: u64 = 0;
        let buf = self.buf.borrow();

        for (n, shift) in BIT_SHIFT.iter().enumerate().take(MAX_VARINT_LENGTH) {
            let b = buf[self.pos] as u64;
            self.pos += 1;
            if n == 0 {
                if b & 0x80 == 0 {
                    return b;
                }
                val = b & 0x7f;
            } else {
                val |= (b & 0x7f) << shift;
            }
            if b < 0x80 {
                break;
            }
        }

        val
    }

    /// Read in a variable size value from the buffer.
    pub fn read_varint<T>(&mut self) -> T
    where
        T: BitCast,
    {
        let val = self.decode_varint();
        T::from_u64(val)
    }

    /// Write a u64 to the buffer.
    pub fn write_varint(&mut self, val: u64) {
        let mut buf = self.buf.borrow_mut();
        let mut val = val;

        while val > 0x80 {
            buf.push((val & 0x7f) as u8 | 0x80);
            val >>= 7;
        }
        buf.push(val as u8);
    }

    /// When done writing to the buffer, call this function to take ownership
    pub fn take(&mut self) -> Vec<u8> {
        self.buf.take()
    }
}
