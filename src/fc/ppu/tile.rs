use crate::bits::Bitwise;

/// The `Tile` type represents a 8x8 tile as interpreted by the Famicom's PPU.
/// 
/// Internally, this is a wrapper around a slice of bytes with length 16.
pub struct Tile<'a> {
    bytes: &'a [u8]
}

impl<'a> Tile<'a> {
    /// Creates a tile from a slice.
    /// Returns either a tile wrapped in `Some`, or `None` if the length of `buf` is not 16.
    pub fn from_slice(buf: &[u8]) -> Option<Tile> {
        if buf.len() == 16 {
            Some(Tile { bytes: buf })
        } else {
            None
        }
    }

    /// Get the color at the given x,y coordinate pair.
    /// 
    /// ```rust
    /// let t = Tile::from_slice(&[0; 16]).unwrap();
    /// assert_eq!(t.color_at(0,0), 0);
    /// assert_eq!(t.color_at(7,7), 0);
    /// ```
    pub fn color_at(&self, x: usize, y: usize) -> u8 {
        assert!(x < 8 && y < 8);
        let b0 = self.bytes[y].test_bit(7 - x) as u8;
        let b1 = self.bytes[y + 8].test_bit(7 - x) as u8;
        (b1 << 1) | b0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn color_test() {
        let t = Tile::from_slice(&[0; 16]).unwrap();
        assert_eq!(t.color_at(0,0), 0);
        assert_eq!(t.color_at(7,7), 0);
    }
}