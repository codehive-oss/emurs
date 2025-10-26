mod cpu;
mod nes_rom;
mod ram;
mod ppu;

use cpu::cpu_memory::CpuMemory;
use crate::nes_rom::NesRom;
use cpu::Cpu;

fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    let rom = NesRom::read_from_file("./vendor/nestest/nestest.nes")?;
    println!("{rom:#?}");

    let memory_map = CpuMemory::new(rom);
    println!("Entry point: {:#X}", memory_map.reset_vector());

    let mut cpu = Cpu::with_nes_options(memory_map, 1 << 16);
    // cpu.run();

    Ok(())
}

