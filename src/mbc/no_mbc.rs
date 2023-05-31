// 'No memory bank controller' game type
// Up to 32 KiB of rom (with no rom banks)
// Optional up to 8KiB of RAM

use super::Mbc;

const KIBI_BYTE: usize = 1024;

pub struct NoMbc {
    rom: [u8; 32 * KIBI_BYTE],
    ram: Option<[u8; 8 * KIBI_BYTE]>,
}

impl Mbc for NoMbc {
    fn read_byte(&self, address: u16) -> u8 {
	match address {
	    ..=0x7FFF => self.rom[address as usize],
	    0xA000..=0xBFFF => {
		match self.ram {
		    Some(array) => array[(address-0xA000u16) as usize],
		    None => 0xFF
		}
	    },
	    _ => 0xFF
	}
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
	match address {
	    0xA000..=0xBFFF => {
		match self.ram{
		    Some(mut array) => array[(address - 0xA000) as usize] = byte,
		    None => ()
		}
	    }
	    _ => ()
	}
    }

}

impl NoMbc {
    /// Creates a new mbc of type no_mbc
    pub fn new(total_rom: Vec<u8>) -> Self {
	let mut rom = [0u8; 32 * KIBI_BYTE];
	for i in 0..=total_rom.len() -1 {
	    rom[i] = total_rom[i];
	}

	let ram_type_code = total_rom[0x149];
	let ram = match ram_type_code {
	    2 => Some([0u8; 8 * KIBI_BYTE]),
	    _ => None
	};

	Self {
	    rom,
	    ram,
	}
    }
}
