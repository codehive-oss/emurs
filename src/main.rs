mod cpu;
use cpu::CPU;

fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    let mut cpu = CPU::new(
        "./6502_65C02_functional_tests/bin_files/6502_functional_test.bin",
        1 << 16,
    )?;
    cpu.run();

    Ok(())
}
