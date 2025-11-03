pub struct Sprite {
    pub x: u16,
    pub y: u16,
    pub bank: u16,
    pub tile_index: u16,
    pub palette: u8,
    pub visible: bool,
    pub flip_horizontally: bool,
    pub flip_vertically: bool,
}

const ATTR_PALETTE_MASK: u8 = 0b11;
const ATTR_PRIORITY_BIT: u8 = 5;
const ATTR_FLIP_HORIZONTALLY_BIT: u8 = 6;
const ATTR_FLIP_VERTICALLY_BIT: u8 = 7;

impl Sprite {
    pub fn from_data(data: &[u8], tall_sprite: bool) -> Self {
        let x = data[3] as u16;
        let y = data[0] as u16;
        let bank = if tall_sprite && data[1] & 1 == 1 {
            0x1000
        } else {
            0
        };
        let tile_index = if tall_sprite {
            data[1] & 0b1111_1110
        } else {
            data[1]
        } as u16;
        let palette = data[2] & ATTR_PALETTE_MASK;
        let visible = (data[2] >> ATTR_PRIORITY_BIT) & 1 == 0;
        let flip_horizontally = (data[2] >> ATTR_FLIP_HORIZONTALLY_BIT) & 1 == 1;
        let flip_vertically = (data[2] >> ATTR_FLIP_VERTICALLY_BIT) & 1 == 1;
        Self {
            x,
            y,
            bank,
            tile_index,
            palette,
            visible,
            flip_horizontally,
            flip_vertically,
        }
    }
}
