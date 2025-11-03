pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub tile_index: u8,
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
    pub fn from_data(data: &[u8]) -> Self {
        let x = data[3];
        let y = data[0];
        let tile_index = data[1]; // TODO support 8x16 sprites
        let palette = data[2] & 0b11;
        let visible = (data[2] >> ATTR_PRIORITY_BIT) & 1 == 0;
        let flip_horizontally = (data[2] >> ATTR_FLIP_HORIZONTALLY_BIT) & 1 == 1;
        let flip_vertically = (data[2] >> ATTR_FLIP_VERTICALLY_BIT) & 1 == 1;
        Self {
            x,
            y,
            tile_index,
            palette,
            visible,
            flip_horizontally,
            flip_vertically,
        }
    }
}
