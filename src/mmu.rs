use crate::cpu::CpuState;
use crate::interrupt_handler::InterruptHandler;
use crate::joypad::Joypad;
use crate::mbc::Mbc;
use crate::ppu::Ppu;
use crate::serial::Serial;
use crate::timer::Timer;

// Emulates the actions triggered by the reading and writing of bytes in the instructions
pub struct Mmu {
    hram: [u8; 0x7F],
    pub joypad: Joypad,
    pub mbc: Box<dyn Mbc>,
    pub timer: Timer,
    pub ppu: Ppu,
    serial: Serial,
    wram_banks: [[u8; 0x2000]; 8],
    wram_bank_index: usize,   // Index of the wram bank to use in the 0xD000-0xDFFF region
    pub dma_iterator: u8,
    pub dma_source: u8,
    pub key1: u8           // Prepare speed switch control register
}

impl Mmu {
    pub fn fetch_byte(
        &mut self,
        address: u16,
        cpu_state: &CpuState,
        interrupt_handler: &mut InterruptHandler,
    ) -> u8 {
        match address {
            0..=0x7FFF => match cpu_state {
                _ => self.mbc.read_byte(address),
            },
            0x8000..=0x9FFF => self.ppu.fetch_vram(address - 0x8000),
            0xA000..=0xBFFF => self.mbc.read_byte(address),
            0xC000..=0xCFFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_banks[0][local_address]
            }
            0xD000..=0xDFFF => {
                let local_address = (address & 0x1FFF) as usize;
		self.wram_banks[self.wram_bank_index][local_address]
            }
            0xE000..=0xFDFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_banks[0][local_address]
            }
            0xFE00..=0xFE9F => self.ppu.fetch_oam(address - 0xFE00),
            0xFF00 => self.joypad.byte,
            0xFF01 => self.serial.serial_data_transfer,
            0xFF02 => 0,
            0xFF04..=0xFF07 => self.timer.read_byte(address),
            0xFF0F => interrupt_handler.IF,
            0xFF40 => self.ppu.lcdc,
            0xFF41 => self.ppu.lcd_status,
            0xFF42 => self.ppu.scy,
            0xFF43 => self.ppu.scx,
            0xFF44 => self.ppu.ly,
            0xFF45 => self.ppu.lyc,
            0xFF46 => self.ppu.oam_ram[159],
            0xFF47 => self.ppu.bgp,
            0xFF48 => self.ppu.obp0,
            0xFF49 => self.ppu.obp1,
	    0xFF68 => self.ppu.bg_palette_index as u8,
	    0xFF69 => self.ppu.fetch_bg_palette_data(),
	    0xFF6A => self.ppu.sprite_palette_index as u8,
	    0xFF6B => self.ppu.fetch_sprite_palette_data(),
            0xFF4A => self.ppu.wy,
            0xFF4B => self.ppu.wx,
	    0xFF70 => self.wram_bank_index as u8,
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => interrupt_handler.IE,
            _ => 0xFF,
        }
    }

    pub fn write_word(
        &mut self,
        address: u16,
        word: u16,
        cpu_state: &mut CpuState,
        interrupt_handler: &mut InterruptHandler,
    ) {
        let lower = word as u8;
        self.write_byte(address, lower, cpu_state, interrupt_handler);
        let high = (word >> 8) as u8;
        self.write_byte(address + 1, high, cpu_state, interrupt_handler);
    }

    pub fn write_byte(
        &mut self,
        address: u16,
        received_byte: u8,
        cpu_state: &mut CpuState,
        interrupt_handler: &mut InterruptHandler,
    ) {
        match address {
            0..=0x7FFF => self.mbc.write_byte(address, received_byte), // Writing to ROM
            0x8000..=0x9FFF => self.ppu.write_vram(address - 0x8000, received_byte),
            0xA000..=0xBFFF => self.mbc.write_byte(address, received_byte),
            0xC000..=0xCFFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_banks[0][local_address]= received_byte;
            }
            0xD000..=0xDFFF => {
                let local_address = (address & 0x1FFF) as usize;
		self.wram_banks[self.wram_bank_index][local_address]= received_byte;
            }
            0xE000..=0xFDFF => {
                let local_address = (address & 0x1FFF) as usize;
                self.wram_banks[0][local_address]= received_byte;
            }
            0xFE00..=0xFE9F => self.ppu.write_oam(address - 0xFE00, received_byte),
            0xFF00 => self.joypad.write_to_byte(received_byte, interrupt_handler),
            0xFF01 => self.serial.write_to_transfer(interrupt_handler, received_byte),
            0xFF02 => self.serial.write_to_control(received_byte, interrupt_handler),
            0xFF04..=0xFF07 => self.timer.write_byte(address, received_byte),
            0xFF0F => {
		interrupt_handler.IF = received_byte & 0x1F;
		interrupt_handler.IF |= 0b1110_0000;
	    }
            0xFF40 => self.ppu.write_lcdc(received_byte),
            0xFF41 => self.ppu.write_to_lcd_status(received_byte),
            0xFF42 => self.ppu.scy = received_byte,
            0xFF43 => self.ppu.scx = received_byte,
            0xFF45 => self.ppu.lyc = received_byte,
            0xFF46 => self.request_dma(received_byte, cpu_state),
            0xFF47 => self.ppu.bgp = received_byte,
            0xFF48 => self.ppu.obp0 = received_byte,
            0xFF49 => self.ppu.obp1 = received_byte,
            0xFF4A => self.ppu.wy = received_byte,
            0xFF4B => self.ppu.wx = received_byte,
            0xFF4F => self.ppu.vram_bank_index = received_byte as usize & 0x1,
            0xFF50 => {
                if received_byte > 0 {
                    *cpu_state = CpuState::NonBoot
                }
            }
	    0xFF68 => self.ppu.bg_palette_index = received_byte as usize ,
	    0xFF69 => self.ppu.write_bg_palette_data(received_byte),
	    0xFF6A => self.ppu.sprite_palette_index = received_byte as usize ,
	    0xFF69 => self.ppu.write_sprite_palette_data(received_byte),
	    0xFF70 => {
		self.wram_bank_index = received_byte as usize & 0x7;
		if self.wram_bank_index == 0 {
		    self.wram_bank_index = 1;
		}
	    },
            0xFF80..=0xFFFE => {
                self.hram[(address - 0xFF80) as usize] = received_byte;
            }
            0xFFFF => {
		interrupt_handler.IE = received_byte & 0x1F;
		interrupt_handler.IE |= 0b1110_0000;
	    },
            _ => (),
        };
    }

    pub fn new(mbc: Box<dyn Mbc>) -> Self {
        Self {
            mbc,
            hram: [0x00; 0x7F],
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            serial: Serial::new(),
            timer: Timer::default(),
            dma_iterator: 0,
            dma_source: 0,
            key1: 0,
            wram_banks: [[0; 0x2000]; 8],
            wram_bank_index: 1,
        }
    }

    fn request_dma(&mut self, byte: u8, cpu_state: &mut CpuState) {
        self.dma_iterator = 0;
        self.dma_source = byte;
        *cpu_state = CpuState::DMA; // This requests the dma
    }
}
