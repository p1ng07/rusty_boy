use crate::cpu::CpuState;
use crate::interrupt_handler::InterruptHandler;
use crate::joypad::Joypad;
use crate::ppu::Ppu;
use crate::serial::Serial;
use crate::timer::Timer;

const KIBI_BYTE: usize = 1024;

pub struct Bus {
    boot_rom: [u8; 256],
    hram: [u8; 0x7F],
    pub interrupt_handler: InterruptHandler,
    pub joypad: Joypad,
    rom_0: Vec<u8>, // [u8; KIBI_BYTE * 16],
    pub rom_path: String,
    pub timer: Timer,
    ppu: Ppu,
    serial: Serial,
    wram: [u8; 0x2000],
}

impl Bus {
    pub fn fetch_byte(&self, address: u16, cpu_state: &CpuState) -> u8 {
        match address {
            0..=0x7FFF => match cpu_state {
                CpuState::Boot => match address {
                    0..=255 => *self.boot_rom.get(address as usize).unwrap(),
                    _ => panic!("Tried to call boot rom after it was already ended"),
                },
                CpuState::NonBoot => *self.rom_0.get(address as usize).unwrap_or(&0xFFu8),
                CpuState::Stopped => 0xFF,
            },
            0x8000..=0x9FFF => self
                .ppu
                .vram
                .get(address.wrapping_sub(0x8000) as usize)
                .unwrap()
                .to_owned(),
            0xA000..=0xBFFF => todo!("Reading from external ram ({:X})", address),
            0xC000..=0xDFFF => {
                let local_address = (address - 0xC000) as usize;
                self.wram.get(local_address).unwrap().to_owned()
            }
            0xE000..=0xFDFF => {
                let local_address = (address.wrapping_sub(0xE000u16)) as usize;
                self.wram.get(local_address).unwrap().to_owned()
            }
            0xFE00..=0xFE9F => self
                .ppu
                .oam_ram
                .get(address.wrapping_sub(0xFE00) as usize)
                .unwrap()
                .to_owned(),
            0xFF00 => self.joypad.byte,
            0xFF01 => self.serial.serial_data_transfer,
            0xFF02 => self.serial.serial_data_control,
            0xFF04..=0xFF07 => self.timer.read_byte(address),
            0xFF0F => self.interrupt_handler.IF,
            0xFF42 => 0, // TODO: Stubbed to 0x0 because 0xFF42 is SCY and some roms wait for SCY to be set to 0
            0xFF44 => 0x90, // TODO: Stubbed to 0x90 because 0xFF40 is LY and some roms wait for LY to be set to 0x90
            0xFF40..=0xFF4B => todo!(
                "Reading LCD control, status, position, scroll and palletes, address {:X}",
                address
            ),
            0xFF80..=0xFFFE => self
                .hram
                .get((address - 0xFF80) as usize)
                .unwrap()
                .to_owned(),
            0xFFFF => self.interrupt_handler.IE,
            _ => 0xFF,
        }
    }

    pub fn write_word(&mut self, address: u16, word: u16, cpu_state: &mut CpuState) {
        let lower = word as u8;
        self.write_byte(address, lower, cpu_state);
        let high = (word >> 8) as u8;
        self.write_byte(address + 1, high, cpu_state);
    }

    pub fn write_byte(&mut self, address: u16, received_byte: u8, cpu_state: &mut CpuState) {
        match address {
            0..=0x7FFF => (), // Writing to ROM
            0x8000..=0x9FFF => self.ppu.vram[(address - 0x8000) as usize] = received_byte,
            0xA000..=0xBFFF => todo!(
                "Writing to External RAM: ({:X}), {}",
                address,
                received_byte
            ),
            0xC000..=0xDFFF => {
                let local_address = (address - 0xC000u16) as usize;
                self.wram[local_address] = received_byte;
            }
            0xE000..=0xFDFF => todo!("Writing to ECHO RAM ({:X}), {}", address, received_byte),
            0xFE00..=0xFE9F => todo!("Writing to OAM RAM ({:X}), {}", address, received_byte),
            0xFF00 => self.joypad.write_to_byte(received_byte),
            0xFF01 => self.serial.write_to_transfer(received_byte),
            0xFF02 => self.serial.serial_data_control = received_byte,
            0xFF04..=0xFF07 => self.timer.write_byte(address, received_byte),
            0xFF0F => self.interrupt_handler.IF = received_byte,
            0xFF40..=0xFF4B => (), // TODO: bunch off ppu status and controls
            0xFF50 => {
                if received_byte > 0 {
                    *cpu_state = CpuState::NonBoot
                }
            }
            0xFF80..=0xFFFE => self.hram[((address - 0xFF80u16) as usize)] = received_byte,
            0xFFFF => self.interrupt_handler.IE = received_byte,
            _ => (),
        };
    }

    pub fn new() -> Self {
        // Load the rom only cartridge, if there isn't a rom, load a load of nothing
        Self {
            rom_0: [0; KIBI_BYTE * 16].to_vec(), //[0; KIBI_BYTE * 16],
            hram: [0; 0x7F],
            wram: [0; 0x2000],
            ppu: Ppu::new(),
            joypad: Joypad::default(),
            serial: Serial::default(),
            timer: Timer::default(),
            rom_path: String::from(""),
            interrupt_handler: InterruptHandler::default(),
            boot_rom: [
                0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26,
                0xFF, 0x0E, 0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77,
                0x77, 0x3E, 0xFC, 0xE0, 0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95,
                0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B, 0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06,
                0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9, 0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21,
                0x2F, 0x99, 0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20, 0xF9, 0x2E, 0x0F, 0x18,
                0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04, 0x1E, 0x02,
                0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0x1D, 0x20, 0xF2,
                0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62, 0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64,
                0x20, 0x06, 0x7B, 0xE2, 0x0C, 0x3E, 0x87, 0xF2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15,
                0x20, 0xD2, 0x05, 0x20, 0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB,
                0x11, 0x17, 0xC1, 0xCB, 0x11, 0x17, 0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9,
                0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C,
                0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6,
                0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC,
                0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E, 0x3c, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x4C,
                0x21, 0x04, 0x01, 0x11, 0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x20, 0xFE, 0x23, 0x7D, 0xFE,
                0x34, 0x20, 0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x20, 0xFE,
                0x3E, 0x01, 0xE0, 0x50,
            ],
        }
    }

    // Tries to load the rom, returns true if it was sucessful, false otherwise
    pub fn load_rom(&mut self, rom_path: String) -> bool {
        // TODO: This is only dumping the rom into the temporary rom_0 vector, this will be an mbc controller
        self.rom_path = rom_path;
        if let Ok(vec) = std::fs::read(&self.rom_path) {
            self.rom_0 = vec;
        } else {
            return false;
        }

        true
    }
}
