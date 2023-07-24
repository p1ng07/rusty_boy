use std::ops::Shl;

use serde::{Serialize, Deserialize};

use super::{Mbc, mbc1::{LENGTH_ROM_BANK, LENGTH_RAM_BANK}};

#[derive(Serialize, Deserialize)]
pub struct Mbc5 {
    ram_enabled: bool,
    ram_bank_index: usize,
    rom_bank_index: usize,
    rom_bank_extra_bit: usize,
    rom_bank_mask: usize, // Used to mask the value written to the rom bank register
    ram_bank_index_mask: usize,
    #[serde(with = "serde_bytes")]
    rom_banks: Vec<u8>,
    #[serde(with = "serde_bytes")]
    ram_banks: Vec<u8>,
}

#[typetag::serde]
impl Mbc for Mbc5 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            ..=0x3FFF => self.rom_banks[address as usize], // Reading rom bank 0
            0x4000..=0x7FFF => {
                let index = self.rom_bank_extra_bit.shl(8) | self.rom_bank_index;
		let address = address as usize + (index & self.rom_bank_mask as usize) * LENGTH_ROM_BANK - 0x4000usize;
		self.rom_banks[address as usize]
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
            0x2000..=0x2FFF => {
                self.rom_bank_index = byte as usize;
            }
            0x3000..=0x3FFF => {
                self.rom_bank_extra_bit = byte as usize & 1;
            }
            0x4000..=0x5FFF => {
                if self.ram_enabled {
                    self.ram_bank_index = (byte & 0xF) as usize;
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
}

impl Mbc5 {
    pub fn new(total_rom: Vec<u8>) -> Self {
        let num_of_banks: usize = match total_rom[0x148] {
            0 => 2,
            1 => 4,
            2 => 8,
            3 => 16,
            4 => 32,
            5 => 64,
            6 => 128,
            7 => 256,
            8 => 512,
            _ => panic!("{} is not a valid bank value", total_rom[0x148]),
        };

        let mut cartridge_total_iterator = 0usize;

        let mut rom_banks: Vec<u8> = Vec::new();

        let rom_bank_mask: usize = match num_of_banks {
            2 => 1,
            4 => 0b11,
            8 => 0b111,
            16 => 0b1111,
            32 => 0b1_1111,
            64 => 0b11_1111,
            128 => 0b111_1111,
            256 => 0b1111_1111,
            512 => 0b1_1111_1111,
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
            2 => 1,  // 1 bank of 8 KiB
            3 => 4,  // 4 banks of 8 KiB
            4 => 16, // 4 banks of 8 KiB
            5 => 8,  // 4 banks of 8 KiB
            _ => 0,
        };

        let ram_bank_index_mask: usize = match num_ram_banks {
            4 => 0b11,
            8 => 0b1111,
            16 => 0b1_1111,
            _ => 0,
        };

        let mut ram_banks: Vec<u8> = Vec::new();

        // Populate ram banks
	for _ in 0..num_ram_banks * LENGTH_RAM_BANK {
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
            rom_bank_extra_bit: 0,
            rom_bank_mask,
            rom_banks,
            ram_banks,
            ram_bank_index_mask,
        }
    }
}
