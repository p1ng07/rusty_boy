use crate::cpu::CpuState;
use crate::interrupt_handler::InterruptHandler;
use crate::joypad::Joypad;
use crate::mbc::Mbc;
use crate::ppu::Ppu;
use crate::serial::Serial;
use crate::timer::Timer;

// Emulates the actions triggered by the reading and writing of bytes in the instructions
pub struct Mmu {
    boot_rom: [u8; 256],
    hram: [u8; 0x7F],
    pub joypad: Joypad,
    mbc: Box<dyn Mbc>,
    pub timer: Timer,
    pub ppu: Ppu,
    serial: Serial,
    wram_0: [u8; 0x2000],
    wram_n: [u8; 0x2000],
    pub dma_register: u16,
    dma_source: u8,
}

// TODO: Add fetching and writing of ppu registers
impl<'a> Mmu {
    pub fn fetch_byte(&self, address: u16, cpu_state: &CpuState, interrupt_handler: &mut InterruptHandler) -> u8 {
	match address {
	    0..=0x7FFF => match cpu_state {
		CpuState::Boot => match address {
                    0..=255 => *self.boot_rom.get(address as usize).unwrap(),
                    _ => panic!("Tried to call boot rom after it was already ended"),
                },
                _ => self.mbc.read_byte(address),
            },
            0x8000..=0x9FFF => self.ppu.fetch_vram(address - 0x8000),
            0xA000..=0xBFFF => self.mbc.read_byte(address),
            0xC000..=0xCFFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_0[local_address]
            }
            0xD000..=0xDFFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_n[local_address]
            }
            0xE000..=0xFDFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_0[local_address]
            }
            0xFE00..=0xFE9F => self.ppu.fetch_oam(address - 0xFE00),
            0xFF00 => self.joypad.byte,
            0xFF01 => self.serial.serial_data_transfer,
            0xFF02 => self.serial.serial_data_control,
            0xFF04..=0xFF07 => self.timer.read_byte(address),
            0xFF0F => interrupt_handler.IF,
            0xFF40 => self.ppu.lcdc,
            0xFF41 => self.ppu.status,
            0xFF42 => self.ppu.scy,
            0xFF43 => self.ppu.scx,
            0xFF44 => self.ppu.ly,
            0xFF45 => self.ppu.lyc,
	    0xFF46 => self.dma_source,
	    0xFF47 => self.ppu.bgp,
	    0xFF48 => self.ppu.obp0,
	    0xFF49 => self.ppu.obp1,
	    0xFF4A => self.ppu.wy,
	    0xFF4B => self.ppu.wx,
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => interrupt_handler.IE,
            _ => 0xFF,
        }
    }

    pub fn write_word(&mut self, address: u16, word: u16, cpu_state: &mut CpuState, interrupt_handler: &mut InterruptHandler) {
        let lower = word as u8;
        self.write_byte(address, lower, cpu_state, interrupt_handler);
        let high = (word >> 8) as u8;
        self.write_byte(address + 1, high, cpu_state, interrupt_handler);
    }

    pub fn write_byte(&mut self, address: u16, received_byte: u8, cpu_state: &mut CpuState, interrupt_handler: &mut InterruptHandler) {
        match address {
            0..=0x7FFF => self.mbc.write_byte(address, received_byte), // Writing to ROM
            0x8000..=0x9FFF => self.ppu.write_vram(address - 0x8000, received_byte),
            0xA000..=0xBFFF => self.mbc.write_byte(address, received_byte),
            0xC000..=0xCFFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_0[local_address] = received_byte;
            }
            0xD000..=0xDFFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_n[local_address] = received_byte;
            }
            0xE000..=0xFDFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_0[local_address] = received_byte;
            }
            0xFE00..=0xFE9F => self.ppu.write_oam(address - 0xFE00, received_byte),
            0xFF00 => self.joypad.write_to_byte(received_byte),
            0xFF01 => self.serial.write_to_transfer(received_byte),
            0xFF02 => self.serial.serial_data_control = received_byte,
            0xFF04..=0xFF07 => self.timer.write_byte(address, received_byte),
            0xFF0F => interrupt_handler.IF = received_byte,
            0xFF40 => self.ppu.lcdc = received_byte,
            0xFF41 => self.ppu.status = received_byte,
            0xFF42 => self.ppu.scy = received_byte,
            0xFF43 => self.ppu.scx = received_byte,
            0xFF45 => self.ppu.lyc = received_byte,
	    0xFF46 => self.request_dma(received_byte, cpu_state),
	    0xFF47 => self.ppu.bgp = received_byte,
	    0xFF48 => self.ppu.obp0 = received_byte,
	    0xFF49 => self.ppu.obp1 = received_byte,
	    0xFF4A => self.ppu.wy = received_byte,
	    0xFF4B => self.ppu.wx = received_byte,
            0xFF50 => {
                if received_byte > 0 {
                    *cpu_state = CpuState::NonBoot
                }
            }
            0xFF80..=0xFFFE => {
                self.hram[(address - 0xFF80) as usize] = received_byte;
            }
            0xFFFF => interrupt_handler.IE = received_byte,
            _ => (),
        };
    }

    pub fn new(mbc: Box<dyn Mbc>) -> Self {
	Self {
            mbc,
            hram: [0x00; 0x7F],
            wram_0: [0x00; 0x2000],
            wram_n: [0x00; 0x2000],
	    ppu: Ppu::new(),
            joypad: Joypad::default(),
            serial: Serial::default(),
            timer: Timer::default(),
	    dma_register: 0,
	    dma_source: 0,
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

    fn request_dma(&mut self, byte: u8, cpu_state: &mut CpuState) {
	self.dma_register = ((byte as u16) << 8) | 0x00;
	self.dma_source = byte;
	*cpu_state = CpuState::DMA; // This requests the dma
    }
}
