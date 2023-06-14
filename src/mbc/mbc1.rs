use super::{Mbc, no_mbc::KIBI_BYTE};
//TODO: MBC1 is not behaving correctly

pub struct Mbc1 {
    ram_enabled: bool,
    ram_bank_index: usize,
    rom_bank_index: usize,
    rom_bank_mask: u16, // Used to mask the value written to the rom bank register
    rom_banks: Vec<[u8; 16 * KIBI_BYTE]>,
    ram_banks: Option<Vec<[u8; 8 * KIBI_BYTE]>>
}

impl Mbc for Mbc1 {
    fn read_byte(&self, address: u16) -> u8 {
	match address {
	    ..=0x3FFF => self.rom_banks[0][address as usize], // Reading rom bank 0
	    0x4000..=0x7FFF => {
		if let Some(ref rom_bank) = self.rom_banks.get(self.rom_bank_index) {
		    rom_bank[address as usize - 0x4000]
		}else{
		    panic!("{} rom bank", self.rom_bank_index);
		}
	    },
	    0xA000..=0xBFFF => { // Reading ram bank 00-04
		if !self.ram_enabled { return 0xFF; }

		if let Some(ref vector) = self.ram_banks {
		    vector[self.ram_bank_index][address as usize - 0xA000]
		}else {
		    0xFF
		}
	    },
	    _ => 0xFF
	}
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
	match address{
	    ..=0x1FFF => {
		if byte & 0xF == 0xA {
		    self.ram_enabled = true;
		} else {
		    self.ram_enabled = false;
		}
	    }
	    0x2000..=0x3FFF =>{
		self.rom_bank_index = (self.rom_bank_mask & (byte as u16)) as usize;
		if self.rom_bank_index == 0 {
		    self.rom_bank_index = 1;
		}
	    }
	    0x4000..=0x5FFF => {
		self.ram_bank_index = (byte & 2) as usize;
	    }
	    _ => ()
	}
    }
}

impl Mbc1 {
    pub fn new(total_rom: Vec<u8>) -> Self {
	let num_of_banks = match total_rom[0x148] {
	    0 => 2,
	    1 => 4,
	    2 => 8,
	    3 => 16,
	    4 => 32,
	    _ => panic!("{} is not a valid bank value", total_rom[0x148])
	};

	let mut cartridge_total_iterator = 0usize;

	let mut rom_banks: Vec<[u8; 16 * KIBI_BYTE]> = Vec::with_capacity(num_of_banks);

	let rom_bank_mask = match num_of_banks {
	    2 => 1u16,
	    4 => 2u16,
	    8 => 3u16,
	    16 => 4u16,
	    32 => 5u16,
	    _ => panic!("{} is not a valid rom bank number.", num_of_banks)
	};

	// Copy every rom bank on the cartridge
	// Every bank is comprised of 16 KiB
	for _ in 0..num_of_banks {
	    let mut new_vec: [u8; 16 * KIBI_BYTE] = [0; 16 * KIBI_BYTE];
	    for j in 0..new_vec.len() {
		new_vec[j] = match total_rom.get(cartridge_total_iterator) {
		    Some(x) => {
			cartridge_total_iterator += 1;
			x.to_owned()
		    }
		    None => 0x00,
		};
	    }
	    rom_banks.push(new_vec);
	}

	// Initialize ram based on the size given in the rom
	let mut ram_banks = match total_rom[0x149] {
	    0 | 1 => None,
	    2 => Some([[0u8; 8 * KIBI_BYTE]; 1].to_vec()), // 1 bank of 8 KiB
	    3 => Some([[0u8; 8 * KIBI_BYTE]; 4].to_vec()), // 4 banks of 8 KiB
	    _ => panic!("Ram size specified on the cartridge isn't available on this mbc.")
	};

	// Populate mirrors of external ram
	match ram_banks.as_mut(){
	    Some(array) => {
		for i in 0..array.len() - 1 {
		    for j in 0..array[0].len() - 1 {
			(*array)[i][j] = match total_rom.get(cartridge_total_iterator){
			    Some(x) => {
				cartridge_total_iterator += 1;
				x.to_owned()
			    },
			    None => 0
			};
		    };
		};
	    },
	    None => ()
	}

	Self{
	    ram_enabled: false,
	    ram_bank_index: 0,
	    rom_bank_index: 1,
	    rom_bank_mask,
	    rom_banks,
	    ram_banks,
	}
    }
}
