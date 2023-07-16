use crate::{cpu::{is_bit_set, Cpu, CpuState}, mmu::{self, Mmu}, ppu::{self, Ppu}, interrupt_handler::{self, InterruptHandler}};

pub struct HdmaController {
    pub hdma1: u8,
    pub hdma2: u8,
    pub hdma3: u8,
    pub hdma4: u8,
    pub hdma5: u8,
    pub is_active: bool,
}

impl HdmaController {
    pub fn new() -> Self {
	Self {
	    hdma1: 0,
	    hdma2: 0,
	    hdma3: 0,
	    hdma4: 0,
	    hdma5: 0,
	    is_active: false
	}
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
	match address {
	    0xFF51 => self.hdma1 = byte,
	    0xFF52 => self.hdma2 = byte & 0xF0,
	    0xFF53 => self.hdma3 = byte,
	    0xFF54 => self.hdma4 = byte & 0xF0,
	    _ => ()
	}
    }

    pub fn start_dma(&mut self, hdma5: u8, ppu: &mut Ppu, mmu: &mut Mmu, interrupt_handler: &mut InterruptHandler){
	self.hdma5 = hdma5;
	self.is_active = true;

	
	if is_bit_set(hdma5, 7) {
	    // Start an hblank dma
	    todo!("Implement hblank dma");
	}else {
	    // Start a general purpose dma
	    // This transfer is instant
	    let start = (self.hdma1 as u16) << 8 | self.hdma2 as u16;
	    let destination = ((self.hdma3 as u16) << 8 | self.hdma4 as u16) & 0b0001_1111_1111_0000;
	    let mut state = CpuState::NonBoot;
	    let length = (hdma5 as u16 & 0b111_1111) * 16 + 1;

	    for i in 0..length {
		let byte = mmu.fetch_byte(start + i, interrupt_handler);

		mmu.write_byte(destination + i, byte, &mut state, interrupt_handler);
	    }
	}
    }
    
}
