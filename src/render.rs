mod sprite;

use crate::cpu::Cpu;
use crate::memory::Memory;
use crate::nes_rom::NesRom;
use crate::ppu::ppu_memory::PpuMemory;
use crate::ppu::{Ppu, OAM_SIZE};
use crate::render::sprite::Sprite;
use macroquad::color::{Color, BLACK, BLUE, RED, WHITE};
use macroquad::prelude::{draw_rectangle, next_frame, request_new_screen_size};

const SCREEN_WIDTH: u16 = 256;
const SCREEN_HEIGHT: u16 = 240;

const RENDER_SCALE: f32 = 4.;
const TILE_SIZE: f32 = RENDER_SCALE * 8.;

const SYSTEM_PALLETE: [u32; 64] = [
    0x808080, 0x003DA6, 0x0012B0, 0x440096, 0xA1005E, 0xC70028, 0xBA0600, 0x8C1700, 0x5C2F00,
    0x104500, 0x054A00, 0x00472E, 0x004166, 0x000000, 0x050505, 0x050505, 0xC7C7C7, 0x0077FF,
    0x2155FF, 0x8237FA, 0xEB2FB5, 0xFF2950, 0xFF2200, 0xD63200, 0xC46200, 0x358000, 0x058F00,
    0x008A55, 0x0099CC, 0x212121, 0x090909, 0x090909, 0xFFFFFF, 0x0FD7FF, 0x69A2FF, 0xD480FF,
    0xFF45F3, 0xFF618B, 0xFF8833, 0xFF9C12, 0xFABC20, 0x9FE30E, 0x2BF035, 0x0CF0A4, 0x05FBFF,
    0x5E5E5E, 0x0D0D0D, 0x0D0D0D, 0xFFFFFF, 0xA6FCFF, 0xB3ECFF, 0xDAABEB, 0xFFA8F9, 0xFFABB3,
    0xFFD2B0, 0xFFEFA6, 0xFFF79C, 0xD7E895, 0xA6EDAF, 0xA2F2DA, 0x99FFFC, 0xDDDDDD, 0x111111,
    0x111111,
];

pub fn get_color(idx: u8) -> Color {
    Color::from_hex(SYSTEM_PALLETE[idx as usize % 64])
}

pub fn get_bg_palette(ppu: &Ppu<PpuMemory>, palette_idx: u16) -> [Color; 4] {
    [
        get_color(ppu.memory.palette_table.read(0)),
        get_color(ppu.memory.palette_table.read(palette_idx * 4 + 1)),
        get_color(ppu.memory.palette_table.read(palette_idx * 4 + 2)),
        get_color(ppu.memory.palette_table.read(palette_idx * 4 + 3)),
    ]
}

pub fn get_sprite_palette(ppu: &Ppu<PpuMemory>, palette_idx: u16) -> [Color; 4] {
    get_bg_palette(ppu, palette_idx + 4)
}

pub async fn render_frame(cpu: &mut Cpu) {
    let ppu = &mut cpu.bus.ppu;

    request_new_screen_size(
        RENDER_SCALE * (8 * 32) as f32,
        RENDER_SCALE * (8 * 30) as f32,
    );

    render_background(ppu).await;
    render_sprites(ppu).await;

    next_frame().await
}

async fn render_background(ppu: &mut Ppu<PpuMemory>) {
    let bank = ppu.background_pattern_addr();

    fn calc_screen_pos(tile_index: usize, pixel_index: usize) -> (f32, f32) {
        let tile_x = (tile_index % 32) as f32;
        let tile_y = (tile_index / 32) as f32;
        let base_x = tile_x * TILE_SIZE;
        let base_y = tile_y * TILE_SIZE;
        let pixel_x = (pixel_index % 8) as f32;
        let pixel_y = (pixel_index / 8) as f32;
        let screen_x = base_x + pixel_x * RENDER_SCALE;
        let screen_y = base_y + pixel_y * RENDER_SCALE;
        (screen_x, screen_y)
    }

    for tile_index in 0..32 * 30 {
        let nametable_addr = 0x2000 + (ppu.base_nametable_index() as u16 * 0x400);
        // which tile are we rendering?
        let tile = ppu.memory.read(nametable_addr + tile_index) as u16;
        let chr_data = ppu.memory.chr_rom
            [(bank + tile * 16) as usize..(bank + tile * 16 + 16) as usize]
            .to_vec();
        let pixels = chr_data_to_pixels(chr_data);

        // which palette should be used?
        let tile_x = tile_index % 32;
        let tile_y = tile_index / 32;
        let attr_table_idx = tile_x / 4 + tile_y / 4 * 8;
        let meta_palette = ppu.memory.read(nametable_addr + 0x3c0 + attr_table_idx) as u16;
        let palette_idx = match (tile_x % 2, tile_y % 2) {
            (0, 0) => meta_palette & 0b11,
            (1, 0) => (meta_palette >> 2) & 0b11,
            (0, 1) => (meta_palette >> 4) & 0b11,
            (1, 1) => (meta_palette >> 6) & 0b11,
            _ => panic!("unexpected tile position"),
        };
        let colors = get_bg_palette(ppu, palette_idx);

        for i in 0..pixels.len() {
            let (screen_x, screen_y) = calc_screen_pos(tile_index as usize, i);
            draw_rectangle(
                screen_x,
                screen_y,
                RENDER_SCALE,
                RENDER_SCALE,
                colors[pixels[i] as usize],
            )
        }
    }
}

async fn render_sprites(ppu: &Ppu<PpuMemory>) {
    for oam_idx in (0..OAM_SIZE).step_by(4).rev() {
        let sprite = Sprite::from_data(&ppu.oam[oam_idx..oam_idx + 4]);
        if !sprite.visible {
            continue;
        }
        let bank = ppu.sprite_pattern_addr();
        let tile = sprite.tile_index as u16;
        let chr_data = ppu.memory.chr_rom
            [(bank + tile * 16) as usize..(bank + tile * 16 + 16) as usize]
            .to_vec();
        let pixels = chr_data_to_pixels(chr_data);

        let colors = get_sprite_palette(ppu, sprite.palette as u16);

        for x in 0..8 {
            for y in 0..8 {
                let pixel_idx = (y * 8 + x) as usize;
                if pixels[pixel_idx] == 0 {
                    continue;
                }

                let x = if sprite.flip_horizontally { 7 - x } else { x };
                let y = if sprite.flip_vertically { 7 - y } else { y };

                if sprite.x as u16 + x > SCREEN_WIDTH || sprite.y as u16 + y > SCREEN_HEIGHT {
                    continue;
                }
                let (screen_x, screen_y) = (sprite.x as u16 + x, sprite.y as u16 + y);

                draw_rectangle(
                    screen_x as f32 * RENDER_SCALE,
                    screen_y as f32 * RENDER_SCALE,
                    RENDER_SCALE,
                    RENDER_SCALE,
                    colors[pixels[pixel_idx] as usize],
                )
            }
        }
    }
}

pub async fn debug_chr_rom(rom: &NesRom) {
    request_new_screen_size(
        RENDER_SCALE * (2 * 8 * 16) as f32,
        RENDER_SCALE * (8 * 16) as f32,
    );

    fn calc_screen_pos(tile_index: usize, pixel_index: usize) -> (f32, f32) {
        let tile_x = (tile_index % 16) as f32;
        let tile_y = (tile_index / 16) as f32;
        let base_x = tile_x * TILE_SIZE;
        let base_y = tile_y * TILE_SIZE;
        let pixel_x = (pixel_index % 8) as f32;
        let pixel_y = (pixel_index / 8) as f32;
        let screen_x = base_x + pixel_x * RENDER_SCALE;
        let screen_y = base_y + pixel_y * RENDER_SCALE;
        (screen_x, screen_y)
    }

    const COLORS: [Color; 4] = [BLACK, RED, BLUE, WHITE];

    let chr_rom = &rom.chr_rom;
    for tile in 0..256 {
        let chr_data = chr_rom[tile * 16..(tile + 1) * 16].to_vec();
        let pixels = chr_data_to_pixels(chr_data);
        for i in 0..pixels.len() {
            let (screen_x, screen_y) = calc_screen_pos(tile, i);
            draw_rectangle(
                screen_x,
                screen_y,
                RENDER_SCALE,
                RENDER_SCALE,
                COLORS[pixels[i] as usize],
            )
        }
    }
    for tile in 256..512 {
        let chr_data = chr_rom[tile * 16..(tile + 1) * 16].to_vec();
        let pixels = chr_data_to_pixels(chr_data);
        for i in 0..pixels.len() {
            let (mut screen_x, mut screen_y) = calc_screen_pos(tile, i);
            screen_x += TILE_SIZE * 16.;
            screen_y -= TILE_SIZE * 16.;
            draw_rectangle(
                screen_x,
                screen_y,
                RENDER_SCALE,
                RENDER_SCALE,
                COLORS[pixels[i] as usize],
            )
        }
    }

    next_frame().await;
}

fn chr_data_to_pixels(chr_data: Vec<u8>) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(64);
    for byte in 0..8 {
        for bit in (0..8).rev() {
            let pixel_value =
                (chr_data[byte] >> bit & 1) | (((chr_data[byte + 8] >> bit) & 1) << 1);
            pixels.push(pixel_value);
        }
    }
    pixels
}
