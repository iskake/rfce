use std::num::ParseIntError;

use num_traits::PrimInt;

pub trait Bitwise: PrimInt {
    fn test_bit(&self, i: usize) -> bool;
}

impl Bitwise for u8 {
    fn test_bit(&self, i: usize) -> bool {
        ((self & (1 << i)) >> i) == 1
    }
}

impl Bitwise for u16 {
    fn test_bit(&self, i: usize) -> bool {
        ((self & (1 << i)) >> i) == 1
    }
}

pub trait Addr: Bitwise {
    fn lsb(self) -> u8;
    fn msb(self) -> u8;
}

impl Addr for u16 {
    fn msb(self) -> u8 {
        (self >> 8) as u8
    }

    fn lsb(self) -> u8 {
        (self & 0xff) as u8
    }
}

/// Combine two `u8` into a `u16`.
///
/// The first argument is the least significant byte, and the second is the most significant byte.
pub fn as_address(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

/// Parse a hexadecimal number using to a value of type `T: PrimInt`
pub fn parse_hex<T: PrimInt<FromStrRadixErr = ParseIntError>>(val: &str) -> Result<T, ParseIntError> {
    T::from_str_radix(&val.replace("$", "").replace("0x", ""), 16)
}