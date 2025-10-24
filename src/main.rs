mod cpu;
mod ram;
mod nes_rom;

use cpu::CPU;
use ram::RAM;
use crate::nes_rom::NesRom;

fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    let rom = NesRom::read_from_file("./vendor/nes-test-roms/other/manhole.nes");
    println!("{rom:#?}");

    let mut ram = RAM::new();
    ram.load("./vendor/6502_65C02_functional_tests/bin_files/6502_functional_test.bin")?;

    let mut cpu = CPU::new(ram, 1 << 16);
    cpu.run();

    Ok(())
}
