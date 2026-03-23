use crate::context::Endian;
use crate::elf::ELF;

pub struct ROP {
    chain_data: Vec<u8>,
    word_size: usize,
    endian: Endian,
}

impl ROP {
    pub fn new(elf: &ELF) -> Self {
        Self {
            chain_data: Vec::new(),
            word_size: (elf.bits() / 8) as usize,
            endian: elf.endian(),
        }
    }

    pub fn raw(&mut self, value: u64) {
        match (self.word_size, self.endian) {
            (8, Endian::Little) => self.chain_data.extend_from_slice(&value.to_le_bytes()),
            (8, Endian::Big) => self.chain_data.extend_from_slice(&value.to_be_bytes()),
            (4, Endian::Little) => self
                .chain_data
                .extend_from_slice(&(value as u32).to_le_bytes()),
            (4, Endian::Big) => self
                .chain_data
                .extend_from_slice(&(value as u32).to_be_bytes()),
            _ => unreachable!(),
        }
    }

    pub fn raw_bytes(&mut self, data: &[u8]) {
        self.chain_data.extend_from_slice(data);
    }

    pub fn chain(&self) -> Vec<u8> {
        self.chain_data.clone()
    }

    pub fn len(&self) -> usize {
        self.chain_data.len()
    }
}
