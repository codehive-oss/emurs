mod cpu;
mod memory;
mod nes_rom;
mod ppu;

use crate::memory::Memory;
use crate::nes_rom::NesRom;
use crate::ppu::Ppu;
use cpu::bus::Bus;
use cpu::Cpu;
use macroquad::prelude::*;

const RENDER_SCALE: f32 = 4.;
const TILE_SIZE: f32 = RENDER_SCALE * 8.;
const COLORS: [Color; 4] = [BLACK, DARKGRAY, LIGHTGRAY, WHITE];

#[macroquad::main("emurs")]
async fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    // let rom = NesRom::read_from_file("./vendor/nestest/nestest.nes")?;
    let rom = NesRom::read_from_file("./pacman.nes")?;
    println!("{rom:#?}");

    let mut memory_map = Bus::new(rom.clone());
    println!("Entry point: {:#X}", memory_map.reset_vector());

    let mut cpu = Cpu::with_nes_options(memory_map, 1 << 31);
    cpu.reset();
    loop {
        if cpu.poll_new_frame() {
            render_frame(&mut cpu).await;
            println!("frame")
        }
        cpu.tick()
    }

    // debug_chr_rom(rom).await;
    //
    // Ok(())
}

async fn render_frame(cpu: &mut Cpu) {
    let ppu = &mut cpu.bus.ppu;
    let bank = ppu.background_pattern_addr();

    request_new_screen_size(
        RENDER_SCALE * (8 * 32) as f32,
        RENDER_SCALE * (8 * 30) as f32,
    );

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
        let nametable_addr = (0x2000 + (ppu.base_nametable_index() as u16 * 0x400)) + tile_index;
        let tile = ppu.memory.read(nametable_addr) as u16;
        let chr_data = ppu.memory.chr_rom
            [(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize]
            .to_vec();
        let pixels = chr_data_to_pixels(chr_data);
        for i in 0..pixels.len() {
            let (screen_x, screen_y) = calc_screen_pos(tile_index as usize, i);
            draw_rectangle(
                screen_x,
                screen_y,
                RENDER_SCALE,
                RENDER_SCALE,
                COLORS[pixels[i] as usize],
            )
        }
    }
    next_frame().await
}

async fn debug_chr_rom(rom: NesRom) {
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

    let chr_rom = rom.chr_rom;
    loop {
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
