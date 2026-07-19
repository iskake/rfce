const SPRITE_ATTR_PALETTE: u8  = 0b00000011;
const _SPRITE_ATTR_UNUSED: u8  = 0b00011100;
const SPRITE_ATTR_PRIORITY: u8 = 0b00100000;
const SPRITE_ATTR_FLIP_H: u8   = 0b01000000;
const SPRITE_ATTR_FLIP_V: u8   = 0b10000000;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub idx: u8,
    pub y: u8,
    pub tile: u8,
    pub attrs: u8,
    pub x: u8,
}

impl Sprite {
    pub fn palette(&self) -> u8 {
        self.attrs & SPRITE_ATTR_PALETTE
    }

    /// Sprite priority. `true` if in front of background (bit 6 is 0), `false` if behind (bit 6 is 1).
    pub fn in_front(&self) -> bool {
        self.attrs & SPRITE_ATTR_PRIORITY == 0
    }

    pub fn flipped_horizontal(&self) -> bool {
        self.attrs & SPRITE_ATTR_FLIP_H != 0
    }

    pub fn flipped_vertical(&self) -> bool {
        self.attrs & SPRITE_ATTR_FLIP_V != 0
    }
}