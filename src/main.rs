mod cpu;
mod memory_map;
mod nes_rom;
mod ram;

use crate::memory_map::MemoryMap;
use crate::nes_rom::NesRom;
use cpu::CPU;

fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    let rom =
        NesRom::read_from_file("./vendor/nes-test-roms/nes_instr_test/rom_singles/01-implied.nes")?;
    println!("{rom:#?}");

    let memory_map = MemoryMap::new(rom);
    println!("Entry point: {:#X}", memory_map.reset_vector());

    let mut cpu = CPU::new(memory_map, 1 << 16);
    cpu.run();

    Ok(())
}
