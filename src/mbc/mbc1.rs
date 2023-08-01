use serde::{Serialize, Deserialize};

use super::{no_mbc::KIBI_BYTE, Mbc};

// Size of a rom bank = 8 * KIBI_BYTE
pub const LENGTH_ROM_BANK: usize = 16 * KIBI_BYTE;
pub const LENGTH_RAM_BANK: usize = 8 * KIBI_BYTE;

#[derive(Serialize, Deserialize)]
pub struct Mbc1 {
    ram_enabled: bool,
    ram_bank_index: usize,
    rom_bank_index: usize,
    rom_bank_mask: u16, // Used to mask the value written to the rom bank register
    ram_bank_index_mask: usize,
    #[serde(with = "serde_bytes")]
    rom_banks: Vec<u8>,
    #[serde(with = "serde_bytes")]
    ram_banks: Vec<u8>,
}

#[typetag::serde]
impl Mbc for Mbc1 {
    fn read_byte(&self, address: u16) -> u8 {
	match address {
	    ..=0x3FFF => self.rom_banks[address as usize], // Reading rom bank 0
	    0x4000..=0x7FFF => {
		let address = address as usize + (self.rom_bank_index & self.rom_bank_mask as usize) * LENGTH_ROM_BANK - 0x4000;
		self.rom_banks[address]
	    }
	    0xA000..=0xBFFF => {
		// Reading ram bank 00-04
		if !self.ram_enabled {
		    return 0xFF;
		}

		let address = address as usize +
		    (self.ram_bank_index & self.ram_bank_index_mask as usize) * LENGTH_RAM_BANK - 0xA000; 
		self.ram_banks.get(address as usize).unwrap_or(&0xFF).to_owned()
	    }
	    _ => 0xFF,
	}
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
	match address {
	    ..=0x1FFF => {
		if byte & 0xF == 0xA {
		    self.ram_enabled = true;
		} else {
		    self.ram_enabled = false;
		}
	    }
	    0x2000..=0x3FFF => {
		self.rom_bank_index = byte as usize & 0x1F;
		if self.rom_bank_index == 0 {
		    self.rom_bank_index = 1;
		}
	    }
	    0x4000..=0x5FFF => {
		if self.ram_enabled {
		    self.ram_bank_index = (byte & 0b11) as usize;
		}
	    }
	    0xA000..=0xBFFF => {
		if !self.ram_enabled {
		    return;
		}
		let address = address as usize +
		    (self.ram_bank_index & self.ram_bank_index_mask as usize) * LENGTH_RAM_BANK - 0xA000; 

		let value = self.ram_banks.get_mut(address);
		match value {
		    Some(_) => self.ram_banks[address] = byte,
		    None => (),
		}
	    }
	    _ => (),
	}
    }

    // This function is only used by mbc3
    fn tick(&mut self) {}
}

impl Mbc1 {
    pub fn new(total_rom: Vec<u8>) -> Self {
	let num_of_banks = match total_rom[0x148] {
	    0 => 2,
	    1 => 4,
	    2 => 8,
	    3 => 16,
	    4 => 32,
	    5 => 64,
	    6 => 128,
	    _ => panic!("{} is not a valid bank value", total_rom[0x148]),
	};

	let mut cartridge_total_iterator = 0usize;

	let mut rom_banks: Vec<u8> = Vec::new();

	let rom_bank_mask = match num_of_banks {
	    2 => 1u16,
	    4 => 0b11u16,
	    8 => 0b111u16,
	    16 => 0b1111u16,
	    32 => 0b11111u16,
	    _ => panic!("{} is not a valid rom bank number.", num_of_banks),
	};

	// Copy every rom bank on the cartridge
	// Every bank is comprised of 16 KiB
	for _ in 0..num_of_banks * LENGTH_ROM_BANK {
	    let x = match total_rom.get(cartridge_total_iterator) {
		Some(x) => {
		    cartridge_total_iterator += 1;
		    x.to_owned()
		}
		None => 0x00,
	    };
	    rom_banks.push(x);
	}

	// Initialize ram based on the size given in the rom
	let num_ram_banks = match total_rom[0x149] {
	    2 => 1, // 1 bank of 8 KiB
	    3 => 4, // 4 banks of 8 KiB
	    _ => 0,
	};

	let ram_bank_index_mask: usize = match num_ram_banks {
	    4 => 0b11,
	    _ => 0,
	};

	let mut ram_banks: Vec<u8> = Vec::new();

	// Populate ram banks
	for _ in 0..num_of_banks * LENGTH_RAM_BANK {
	    let x = if let Some(x) = total_rom.get(cartridge_total_iterator) {
		cartridge_total_iterator += 1;
		x.clone()
	    } else {
		0
	    };
	    ram_banks.push(x);
	}

	Self {
	    ram_enabled: false,
	    ram_bank_index: 0,
	    rom_bank_index: 1,
	    rom_bank_mask,
	    rom_banks,
	    ram_banks,
	    ram_bank_index_mask,
	}
    }
}
