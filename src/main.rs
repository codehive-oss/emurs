mod cpu;
mod ram;
use cpu::CPU;
use ram::RAM;

fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    let mut ram = RAM::new();
    ram.load("./6502_65C02_functional_tests/bin_files/6502_functional_test.bin")?;

    let mut cpu = CPU::new(ram, 1 << 16);
    cpu.run();

    Ok(())
}
