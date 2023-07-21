use crate::{
    cpu::{is_bit_set, Cpu, CpuState},
    interrupt_handler::{self, InterruptHandler},
    mmu::{self, Mmu},
    ppu::{self, Ppu},
};

pub struct HdmaController {
    pub hdma1: u8,
    pub hdma2: u8,
    pub hdma3: u8,
    pub hdma4: u8,
    pub length: u8, // Number of blocks (of 16 bytes) minus 1 that still need to be transfered
    pub is_active: bool,
    pub iterator_hdma: u16,
    pub destination_hdma: u16,
}

impl HdmaController {
    pub fn new() -> Self {
        Self {
            hdma1: 0,
            hdma2: 0,
            hdma3: 0,
            hdma4: 0,
            length: 0,
            is_active: false,
	    iterator_hdma: 0,
            destination_hdma: 0,
        }
    }
}
