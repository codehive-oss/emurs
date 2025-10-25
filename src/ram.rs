use std::{fs::File, io::Read};

pub struct RAM(Vec<u8>);

impl RAM {
    pub fn new(size: usize) -> Self {
        Self(vec![0; size])
    }

    pub fn load(&mut self, path: &str) -> Result<(), anyhow::Error> {
        let mut file = File::open(path)?;
        file.read_to_end(&mut self.0)?;
        Ok(())
    }

    #[inline]
    pub fn read(&self, a: u16) -> u8 {
        self.0[a as usize]
    }

    #[inline]
    pub fn write(&mut self, a: u16, v: u8) {
        self.0[a as usize] = v;
    }
}
