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
}
