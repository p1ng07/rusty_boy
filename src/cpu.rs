use std::{io, fs};

use self::cpu_registers::CpuRegisters;

pub mod cpu_registers;
pub mod memory_mapper;

#[derive(PartialEq)]
pub enum CpuState {
    Boot,
    NonBoot
}

pub struct Cpu {
    boot_rom: [u8; 256],
    state: CpuState,
    pc: usize,
    sp: u16,
    registers: cpu_registers::CpuRegisters,
    memory_mapper: memory_mapper::MemoryMapper,
}

impl Cpu {
    pub fn new(rom_path: String) -> Cpu{
	Cpu {
	    boot_rom: [0; 256],
	    pc: 0,
	    sp: 0,
	    state: CpuState::Boot,
	    registers: Default::default(),
	    memory_mapper: memory_mapper::MemoryMapper::new(rom_path)
	}
    }

    fn fetch_byte(&mut self) -> u8 {
	let byte = self.memory_mapper.fetch_byte(self.pc, &self.state).expect("Invalid (or non-implemented memory) was requested");
	self.pc+=1;
	byte
    }

    pub(crate) fn fetch_word(&mut self)->u16{
	let fetch_byte_big = self.fetch_byte() as u16;
	let fetch_byte_small = self.fetch_byte() as u16;

	fetch_byte_big << 8 | fetch_byte_small
    }

    // Execute the instruction given and return the number of t-cycles it took to run it
    pub(crate) fn execute(&mut self) -> i32 {
	let first_byte = self.fetch_byte();
	let fetch_cycles = 4;

	fetch_cycles + match first_byte {
	    0x21 => {let word = self.fetch_word(); self.registers.set_hl(word); 12}, // ld hl, u16
	    0x31 => {self.sp = self.fetch_word(); 12},                               // ld sp, u16
	    0x32 => {
		self.memory_mapper.set_internal_ram(self.registers.get_hl().into(), self.registers.a);
		self.registers.set_hl(self.registers.get_hl()-1);
		4},                                                                 // ld (hl-), A
	    0x7C => {self.registers.a = self.registers.h; 4}
	    0xAF => {self.registers.a ^= self.registers.a; 4},                      // XOR A
	    0xCB => {self.sp += 1; 1}, // TODO: CB
	    _ => panic!("Instruction {:x?} not implemented", first_byte.to_be_bytes()),
	}
    }

    // Cycle the cpu once, fetch an instruction and run it, returns the number of t-cycles it took to run it
    pub(crate) fn cycle(&mut self) -> i32{
	self.execute()
    }
}
