mod cpu;
mod nes_rom;
mod ppu;
mod memory;

use crate::nes_rom::NesRom;
use crate::ppu::Ppu;
use cpu::cpu_memory::CpuMemory;
use cpu::Cpu;
use macroquad::prelude::*;

const RENDER_SCALE: f32 = 4.;
const TILE_SIZE: f32 = RENDER_SCALE * 8.;
const COLORS: [Color; 4] = [BLACK, DARKGRAY, LIGHTGRAY, WHITE];

#[macroquad::main("emurs")]
async fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    let rom = NesRom::read_from_file("./tetris.nes")?;
    println!("{rom:#?}");

    let mut memory_map = CpuMemory::new(rom.clone());
    println!("Entry point: {:#X}", memory_map.reset_vector());

    let cpu = Cpu::with_nes_options(memory_map, 1 << 16);

    debug_chr_rom(rom).await;

    Ok(())
}

async fn debug_chr_rom(rom: NesRom) {
    request_new_screen_size(
        RENDER_SCALE * (2 * 8 * 16) as f32,
        RENDER_SCALE * (8 * 16) as f32,
    );

    let chr_rom = rom.chr_rom;
    loop {
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
