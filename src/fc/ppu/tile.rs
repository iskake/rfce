use crate::bits::Bitwise;

pub struct Tile<'a> {
    bytes: &'a [u8]
}

impl<'a> Tile<'a> {
    pub fn from_slice(buf: &[u8]) -> Option<Tile> {
        if buf.len() == 16 {
            Some(Tile { bytes: buf })
        } else {
            None
        }
    }

    pub fn color_at(&self, x: usize, y: usize) -> u8 {
        assert!(x < 8 && y < 8);
        let b0 = self.bytes[y].test_bit(7 - x) as u8;
        let b1 = self.bytes[y + 8].test_bit(7 - x) as u8;
        (b1 << 1) | b0
    }
}