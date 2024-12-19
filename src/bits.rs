use std::fmt::Binary; // todo? replace this with something better?

pub trait Bitwise: Binary {
    fn test_bit(&self, i: Self) -> bool;
}

impl Bitwise for u8 {
    fn test_bit(&self, i: u8) -> bool {
        ((self & (1 << i)) >> i) == 1
    }
}

impl Bitwise for u16 {
    fn test_bit(&self, i: u16) -> bool {
        ((self & (1 << i)) >> i) == 1
    }
}

pub trait Addr: Bitwise {
    fn lsb(self) -> u8;
    fn msb(self) -> u8;
}

impl Addr for u16 {
    fn lsb(self) -> u8 {
        ((self & 0xff00) >> 8) as u8
    }

    fn msb(self) -> u8 {
        (self & 0xff) as u8
    }
}

/// Combine two `u8` into a `u16`.
///
/// The first argument is the least significant byte, and the second is the most significant byte.
pub fn as_address(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}
