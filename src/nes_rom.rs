use std::fs::File;
use std::io::Read;
use std::fmt;

#[derive(Clone, Debug)]
pub enum NametableMirroring {
    Vertical,
    Horizontal,
}

impl NametableMirroring {
    fn from_bit(value: u8) -> Self {
        match value {
            0 => Self::Vertical,
            1 => Self::Horizontal,
            _ => panic!("Invalid nametable arrangement"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TvSystem {
    Ntsc,
    Pal,
}

impl TvSystem {
    fn from_bit(value: u8) -> Self {
        match value {
            0 => Self::Ntsc,
            1 => Self::Pal,
            _ => panic!("Invalid nametable arrangement"),
        }
    }
}

const HEADER_SIZE: usize = 16;
const PRG_ROM_CHUNK_SIZE: usize = 16384;
const CHR_ROM_CHUNK_SIZE: usize = 8192;
const TRAINER_SIZE: usize = 512;

#[derive(Clone)]
pub struct NesRom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    trainer: Option<[u8; TRAINER_SIZE]>,
    mapper: u8,
    alt_nametable: bool,
    nametable_mirroring: NametableMirroring,
    battery_backed_prg_ram: bool,
    prg_ram_size: u8,
    tv_system: TvSystem,
}

impl fmt::Debug for NesRom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        #[derive(Debug)]
        struct NesRom<'a> {
            _prg_rom_chunks: u8,
            _chr_rom_chunks: u8,
            _has_trainer: bool,
            _mapper: &'a u8,
            _alt_nametable: &'a bool,
            _nametable_arrangement: &'a NametableMirroring,
            _battery_backed_prg_ram: &'a bool,
            _prg_ram_size: &'a u8,
            _tv_system: &'a TvSystem,
        }

        let Self {
            prg_rom,
            chr_rom,
            trainer,
            mapper,
            alt_nametable,
            nametable_mirroring: nametable_arrangement,
            battery_backed_prg_ram: prg_ram,
            prg_ram_size,
            tv_system,
            ..
        } = self;

        fmt::Debug::fmt(
            &NesRom {
                _prg_rom_chunks: (prg_rom.len() / PRG_ROM_CHUNK_SIZE) as u8,
                _chr_rom_chunks: (chr_rom.len() / CHR_ROM_CHUNK_SIZE) as u8,
                _has_trainer: trainer.is_some(),
                _mapper: mapper,
                _alt_nametable: alt_nametable,
                _nametable_arrangement: nametable_arrangement,
                _battery_backed_prg_ram: prg_ram,
                _prg_ram_size: prg_ram_size,
                _tv_system: tv_system,
            },
            f,
        )
    }
}

impl NesRom {
    /// Reads an NES rom from the specified file and parses it according to the [iNES](https://www.nesdev.org/wiki/INES) format
    pub fn read_from_file(path: &str) -> Result<Self, anyhow::Error> {
        let mut file = File::open(path)?;
        let mut header = [0u8; HEADER_SIZE];
        file.read_exact(&mut header)?;

        if header[0..4] != [b'N', b'E', b'S', 0x1A] {
            anyhow::bail!("Invalid start of ROM Header");
        }

        let flags6 = header[6];
        let flags7 = header[7];
        let mapper = (flags7 & 0xF0) | (flags6 >> 4);
        let alt_nametable = (flags6 >> 3) & 1 == 1;
        let nametable_arrangement = NametableMirroring::from_bit(flags6 & 1);
        let has_trainer = (flags6 >> 2) & 1 == 1;
        let battery_backed_prg_ram = (flags6 >> 1) & 1 == 1;
        let prg_ram_size = header[8];
        let tv_system = TvSystem::from_bit(header[9] & 0x1);

        let mut trainer = None;
        if has_trainer {
            let mut buffer = [0u8; TRAINER_SIZE];
            file.read_exact(&mut buffer)?;
            trainer = Some(buffer);
        }

        let prg_rom_size = header[4] as usize;
        let mut prg_rom = vec![0; prg_rom_size * PRG_ROM_CHUNK_SIZE];
        file.read_exact(&mut prg_rom)?;

        let chr_rom_size = header[5] as usize;
        let mut chr_rom = vec![0; chr_rom_size * CHR_ROM_CHUNK_SIZE];
        file.read_exact(&mut chr_rom)?;

        Ok(Self {
            prg_rom,
            chr_rom,
            trainer,
            mapper,
            alt_nametable,
            nametable_mirroring: nametable_arrangement,
            battery_backed_prg_ram,
            prg_ram_size,
            tv_system,
        })
    }

}
