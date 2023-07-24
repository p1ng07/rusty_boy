// 'No memory bank controller' game type
// Up to 32 KiB of rom (with no rom banks)
// Optional up to 8KiB of RAM

use serde::{Serialize, Deserialize};

use super::Mbc;

pub const KIBI_BYTE: usize = 1024;

#[derive(Serialize, Deserialize)]
pub struct NoMbc {
    #[serde(with = "serde_arrays")]
    rom: [u8; 32 * KIBI_BYTE],
    #[serde(with = "serde_arrays")]
    ram: [u8; 8 * KIBI_BYTE],
}

#[typetag::serde]
impl Mbc for NoMbc {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            ..=0x7FFF => self.rom[address as usize],
            0xA000..=0xBFFF => self.ram.get(address as usize - 0xA000).unwrap_or(&0xFF).to_owned(),
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0xA000..=0xBFFF => {
		let value = self.ram.get_mut(address as usize - 0xA000);
		match value {
		    Some(_) => self.ram[address as usize - 0xA000] = byte,
		    None => (),
		}
	    }
            _ => (),
        }
    }
}

impl NoMbc {
    /// Creates a new mbc of type no_mbc
    pub fn new(total_rom: Vec<u8>) -> Self {
        let mut rom = [0u8; 32 * KIBI_BYTE];
        for i in 0..=total_rom.len() - 1 {
            rom[i] = total_rom[i];
        }

        let ram_type_code = total_rom[0x149];
        let ram = match ram_type_code {
            2 => [0u8; 8 * KIBI_BYTE],
            _ => [0u8; 8 * KIBI_BYTE],
        };

        Self { rom, ram }
    }
}
