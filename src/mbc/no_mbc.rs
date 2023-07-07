// 'No memory bank controller' game type
// Up to 32 KiB of rom (with no rom banks)
// Optional up to 8KiB of RAM

use std::array;

use super::Mbc;

pub const KIBI_BYTE: usize = 1024;

pub struct NoMbc {
    rom: [u8; 32 * KIBI_BYTE],
    ram: Option<[u8; 8 * KIBI_BYTE]>,
}

impl Mbc for NoMbc {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            ..=0x7FFF => self.rom[address as usize],
            0xA000..=0xBFFF => match self.ram {
                Some(array) => array[(address - 0xA000u16) as usize],
                None => 0xFF,
            },
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0xA000..=0xBFFF => match self.ram {
                Some(mut array) => array[(address - 0xA000) as usize] = byte,
                None => (),
            },
            _ => (),
        }
    }

    fn get_rom_banks(&self) -> Vec<[u8; 16 * KIBI_BYTE]> {
        let mut rom1 = [0u8; 16 * KIBI_BYTE];
        let mut rom2 = [0u8; 16 * KIBI_BYTE];
	for i in 0..16 * KIBI_BYTE {
	    rom1[i] = self.rom[i];
	}
	for i in 16 * KIBI_BYTE..32 * KIBI_BYTE {
	    rom2[i - 16 * KIBI_BYTE] = self.rom[i];
	}
	let vec = vec![rom1, rom2];
	vec
    }

    fn get_ram_banks(&self) -> Option<Vec<[u8; 8 * KIBI_BYTE]>> {
	match self.ram {
	    Some(x) => {
		Some(vec![x])
	    },
	    None => None
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
            2 => Some([0u8; 8 * KIBI_BYTE]),
            _ => None,
        };

        Self { rom, ram }
    }
}
