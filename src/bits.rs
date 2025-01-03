use std::num::ParseIntError;

use num_traits::PrimInt;

pub trait Bitwise: PrimInt {
    /// Tests the bit at position `i` in the given integer.
    /// 
    /// ```
    /// assert_eq!(1.test_bit(1), true);
    /// assert_eq!(0xff.test_bit(7), true);
    /// assert_eq!(0xefff.test_bit(15), false);
    /// ```
    fn test_bit(&self, i: usize) -> bool;
}

// I know it is the same exact code, but since it's primitives...
impl Bitwise for i8 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for i16 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for i32 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for i64 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for i128 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for isize {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for u8 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for u16 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for u32 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for u64 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for u128 {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

impl Bitwise for usize {
    fn test_bit(&self, i: usize) -> bool {
        (self & (1 << i)) != 0
    }
}

pub trait Addr: Bitwise {
    /// Get the least significant byte of an address.
    /// 
    /// ```
    /// assert_eq!(0x1234.lsb(), 0x34);
    /// ```
    fn lsb(self) -> u8;

    /// Get the most significant byte of an address.
    /// 
    /// ```
    /// assert_eq!(0x1234.lsb(), 0x12);
    /// ```
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
/// ```
/// let least = 0x34;
/// let most = 0x12;
/// assert_eq!(as_address(least, most), 0x1234);
/// ```
pub fn as_address(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

/// Parse a hexadecimal number using to a value of type `T: PrimInt`
/// 
/// ```
/// assert_eq!(parse_hex("$1234"), 0x1234);
/// assert_eq!(parse_hex("0xbeef"), 0xbeef);
/// ```
pub fn parse_hex<T: PrimInt<FromStrRadixErr = ParseIntError>>(val: &str) -> Result<T, ParseIntError> {
    T::from_str_radix(&val.replace("$", "").replace("0x", ""), 16)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bit_test() {
        assert_eq!(1.test_bit(0), true);
        assert_eq!(0xff.test_bit(7), true);
        assert_eq!(0x7fffu16.test_bit(15), false);
        assert_eq!((u128::MAX - (1 << 127)).test_bit(127), false);
        println!("{:b}", i128::MIN);
        let a = 0b10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000u128 as i128;
        println!("{}", (a & (1 << 127)) == (1 << 127));
        assert_eq!(i128::MIN.test_bit(127), true);
    }

    #[test]
    fn lsb_msb_test() {
        assert_eq!(0x1234.msb(), 0x12);
        assert_eq!(0x1234.lsb(), 0x34);
    }
}