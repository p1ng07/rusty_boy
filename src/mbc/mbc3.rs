use serde::{Serialize, Deserialize};

use crate::cpu::is_bit_set;

use super::{Mbc, mbc1::{LENGTH_ROM_BANK, LENGTH_RAM_BANK}};

#[derive(Serialize, Deserialize)]
pub struct Mbc3 {
    ram_rtc_enabled: bool,
    ram_bank_rtc_register_index: usize,
    rom_bank_index: usize,
    rom_bank_mask: u16, // Used to mask the value written to the rom bank register
    ram_bank_index_mask: usize,
    #[serde(with = "serde_bytes")]
    rom_banks: Vec<u8>,
    #[serde(with = "serde_bytes")]
    ram_banks: Vec<u8>,
    latch_clock_data: u8,
    latched_hours: u8,
    latched_minutes: u8,
    latched_seconds: u8,
    latched_low_byte_day_counter: u8,
    // bit7 - Day counter carry, bit6 - Halt, bit0 - most significant bit of day counter
    latched_high_byte_day_counter: u8, 
    hours: u8,
    minutes: u8,
    seconds: u8,
    low_byte_day_counter: u8,
    // bit7 - Day counter carry, bit6 - Halt, bit0 - most significant bit of day counter
    high_byte_day_counter: u8, 
}

#[typetag::serde]
impl Mbc for Mbc3 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            ..=0x3FFF => self.rom_banks[address as usize], // Reading rom bank 0
            0x4000..=0x7FFF => {
		let address = address as usize + (self.rom_bank_index & self.rom_bank_mask as usize) * LENGTH_ROM_BANK - 0x4000;
		self.rom_banks[address]
            }
            0xA000..=0xBFFF => {
                // Reading ram bank 00-04
                if !self.ram_rtc_enabled {
                    return 0xFF;
		}

		if (0..5).contains(&self.ram_bank_rtc_register_index) {
		    let address = address as usize +
			(self.ram_bank_rtc_register_index & self.ram_bank_index_mask as usize) * LENGTH_RAM_BANK - 0xA000; 
		     
		    self.ram_banks.get(address as usize).unwrap_or(&0xFF).to_owned()

		}else if (8..0xD).contains(&self.ram_bank_rtc_register_index) {
		    match self.ram_bank_rtc_register_index {
			0x8 => self.latched_seconds,
			0x9 => self.latched_minutes,
			0xA => self.latched_hours,
			0xB => self.latched_low_byte_day_counter,
			0xC => self.latched_high_byte_day_counter,
			_ => 0xFF,
		    }
		}else {
		    0xFF
		}
	    }
	    _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            ..=0x1FFF => {
                if byte & 0xF == 0xA {
                    self.ram_rtc_enabled = true;
                } else {
                    self.ram_rtc_enabled = false;
                }
            }
            0x2000..=0x3FFF => {
                // The rom bank register is a 7 bit register
                self.rom_bank_index = byte as usize & 0x7F;
                if self.rom_bank_index == 0 {
                    self.rom_bank_index = 1;
                }
            }
            0x4000..=0x5FFF => {
		if self.ram_rtc_enabled {
		    self.ram_bank_rtc_register_index = (byte & 0b1111) as usize;
                }
            }
	    0x6000..=0x7FFF => {
		if self.latch_clock_data == 0 {
		    self.latch_clock_data = byte;
		    if byte == 1 {
			self.latched_seconds = self.seconds;
			self.latched_minutes = self.minutes;
			self.latched_hours = self.hours;
			self.latched_high_byte_day_counter = self.high_byte_day_counter;
			self.latched_low_byte_day_counter = self.low_byte_day_counter;
		    }
		}
	    }
            0xA000..=0xBFFF => {
		if !self.ram_rtc_enabled {
		    return;
		}
		if (0..5).contains(&self.ram_bank_rtc_register_index) {
		    let address = address as usize +
			(self.ram_bank_rtc_register_index & self.ram_bank_index_mask as usize) * LENGTH_RAM_BANK - 0xA000; 

		    let value = self.ram_banks.get_mut(address);
		    match value {
			Some(_) => self.ram_banks[address] = byte,
			None => (),
		    }

		}else if (8..0xD).contains(&self.ram_bank_rtc_register_index) {
		    match self.ram_bank_rtc_register_index {
			0x8 => self.seconds = byte,
			0x9 => self.minutes = byte,
			0xA => self.hours = byte,
			0xB => self.low_byte_day_counter = byte,
			0xC => self.high_byte_day_counter = byte,
			_ => ()
		    }
		}
            }
            _ => (),
        }
    }

    // This function is only used by mbc3
    fn tick_second(&mut self) {
	// If timer is halted, don't add to current time
	if !is_bit_set(self.low_byte_day_counter, 6){
	    return;
	}

	self.seconds += 1;

	if self.seconds > 59 {
	    self.seconds = 0;
	    self.minutes += 1;

	    if self.minutes > 59 {
		self.minutes = 0;
		self.hours += 1;

		if self.hours > 23 {
		    self.hours = 0;
		    let (low_byte_day_counter, high_bit_day_counter) = self.low_byte_day_counter.overflowing_add(1);
		    self.low_byte_day_counter = low_byte_day_counter;

		    if high_bit_day_counter {
			if is_bit_set(self.high_byte_day_counter, 0){
			    // If the current high bit of day counter is already set, set the day counter carry
			    self.high_byte_day_counter |= 0x80;
			}
			self.high_byte_day_counter = self.high_byte_day_counter & 0xFE | high_bit_day_counter as u8;
		    }
		}
	    }
	}
    }
}

impl Mbc3 {
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
            32 => 0b1_1111u16,
            64 => 0b11_1111u16,
            128 => 0b111_1111u16,
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
            ram_rtc_enabled: false,
            ram_bank_rtc_register_index: 0,
            rom_bank_index: 1,
            rom_bank_mask,
            rom_banks,
            ram_banks,
            ram_bank_index_mask,
            latch_clock_data: 0,
            latched_hours: 0,
            latched_minutes: 0,
            latched_seconds: 0,
            latched_low_byte_day_counter: 0,
            latched_high_byte_day_counter: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            low_byte_day_counter: 0,
            high_byte_day_counter: 0,
        }
    }
}
