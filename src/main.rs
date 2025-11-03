mod cpu;
mod memory;
mod nes_rom;
mod ppu;

use crate::cpu::controller::{
    CONTROLLER_BUTTON_A, CONTROLLER_BUTTON_B, CONTROLLER_BUTTON_DOWN, CONTROLLER_BUTTON_LEFT,
    CONTROLLER_BUTTON_RIGHT, CONTROLLER_BUTTON_SELECT, CONTROLLER_BUTTON_START,
    CONTROLLER_BUTTON_UP,
};
use crate::nes_rom::NesRom;
use crate::render::{debug_chr_rom, render_frame};
use cpu::bus::Bus;
use cpu::Cpu;
use macroquad::prelude::*;

mod render;

#[macroquad::main("emurs")]
async fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    // let rom = NesRom::read_from_file("vendor/nes-test-roms/blargg_litewall/litewall5.nes")?;
    let rom = NesRom::read_from_file("./lode_runner.nes")?;
    println!("{rom:#?}");

    let bus = Bus::new(rom.clone());
    println!("Entry point: {:#X}", bus.reset_vector());

    let mut cpu = Cpu::with_nes_options(bus, 1 << 31);
    cpu.reset();

    let mut show_chr_rom_debug = false;
    loop {
        const TOGGLE_CHR_DEBUG_KEY: KeyCode = KeyCode::C;
        if is_key_pressed(TOGGLE_CHR_DEBUG_KEY) {
            show_chr_rom_debug = !show_chr_rom_debug;
        }
        if show_chr_rom_debug {
            debug_chr_rom(&rom).await;
        } else {
            if cpu.poll_new_frame() {
                render_frame(&mut cpu).await;
            }
            cpu.tick();
            handle_keyboard_input(&mut cpu);
        }
    }
}

fn handle_keyboard_input(cpu: &mut Cpu) {
    cpu.bus.controller.button_states[CONTROLLER_BUTTON_A] = is_key_down(KeyCode::S);
    cpu.bus.controller.button_states[CONTROLLER_BUTTON_B] = is_key_down(KeyCode::A);
    cpu.bus.controller.button_states[CONTROLLER_BUTTON_SELECT] = is_key_down(KeyCode::LeftShift);
    cpu.bus.controller.button_states[CONTROLLER_BUTTON_START] = is_key_down(KeyCode::Enter);
    cpu.bus.controller.button_states[CONTROLLER_BUTTON_UP] = is_key_down(KeyCode::Up);
    cpu.bus.controller.button_states[CONTROLLER_BUTTON_DOWN] = is_key_down(KeyCode::Down);
    cpu.bus.controller.button_states[CONTROLLER_BUTTON_LEFT] = is_key_down(KeyCode::Left);
    cpu.bus.controller.button_states[CONTROLLER_BUTTON_RIGHT] = is_key_down(KeyCode::Right);

    if is_key_down(KeyCode::R) {
        cpu.reset();
    }
}
