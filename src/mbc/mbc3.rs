use super::{no_mbc::KIBI_BYTE, Mbc};

pub struct Mbc3 {
    ram_enabled: bool,
    ram_bank_index: usize,
    rom_bank_index: usize,
    rom_bank_mask: u16, // Used to mask the value written to the rom bank register
    ram_bank_index_mask: usize,
    rom_banks: Vec<[u8; 16 * KIBI_BYTE]>,
    ram_banks: Option<Vec<[u8; 8 * KIBI_BYTE]>>,
}

impl Mbc for Mbc3 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            ..=0x3FFF => self.rom_banks[0][address as usize], // Reading rom bank 0
            0x4000..=0x7FFF => {
                if let Some(ref rom_bank) = self
                    .rom_banks
                    .get(self.rom_bank_index & self.rom_bank_mask as usize)
                {
                    rom_bank[address as usize - 0x4000]
                } else {
                    panic!("{} rom bank", self.rom_bank_index);
                }
            }
            0xA000..=0xBFFF => {
                // Reading ram bank 00-04
                if !self.ram_enabled {
                    return 0xFF;
                }

                if let Some(ref ram_bank) = self.ram_banks {
                    if let Some(bank) = ram_bank.get(self.ram_bank_index & self.ram_bank_index_mask)
                    {
                        bank[address as usize - 0xA000]
                    } else {
                        0xFF
                    }
                } else {
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
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
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
                if self.ram_enabled {
                    self.ram_bank_index = (byte & 0b11) as usize;
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if let Some(ref mut ram_bank) = self.ram_banks {
                        if let Some(ref mut bank) =
                            ram_bank.get_mut(self.ram_bank_index & self.ram_bank_index_mask)
                        {
                            bank[address as usize - 0xA000] = byte;
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn get_rom_banks(&self) -> Vec<[u8; 16 * KIBI_BYTE]> {
        self.rom_banks.clone()
    }

    fn get_ram_banks(&self) -> Option<Vec<[u8; 8 * KIBI_BYTE]>> {
        self.ram_banks.clone()
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

        let mut rom_banks: Vec<[u8; 16 * KIBI_BYTE]> = Vec::with_capacity(num_of_banks);

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
        let num_ram_banks = match total_rom[0x149] {
            2 => 1, // 1 bank of 8 KiB
            3 => 4, // 4 banks of 8 KiB
            _ => 0,
        };

        let ram_bank_index_mask: usize = match num_ram_banks {
            4 => 0b11,
            _ => 0,
        };

        let mut raw_ram_banks: Vec<[u8; 8 * KIBI_BYTE]> = Vec::new();

        // Populate ram banks
        for _ in 0..num_ram_banks {
            let mut ram_bank = [0u8; 8 * KIBI_BYTE];
            for i in 0..ram_bank.len() {
                ram_bank[i] = if let Some(x) = total_rom.get(cartridge_total_iterator) {
                    cartridge_total_iterator += 1;
                    x.clone()
                } else {
                    0
                };
            }

            raw_ram_banks.push(ram_bank);
        }

        let ram_banks = if raw_ram_banks.is_empty() {
            None
        } else {
            Some(raw_ram_banks)
        };

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
