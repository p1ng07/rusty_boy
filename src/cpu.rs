use std::{io, fs};

use self::memory_mapper::MemoryMapper;

pub mod cpu_registers;
pub mod memory_mapper;

#[derive(PartialEq)]
pub enum CpuState {
    Boot,
    NonBoot
}

pub struct Cpu {
    boot_rom: [u8; 256],
    current_t_cycles: i32,
    state: CpuState,
    pc: usize,
    stack: Vec<u16>,
    pub registers: cpu_registers::CpuRegisters,
    memory_mapper: memory_mapper::MemoryMapper,
}

impl Cpu {
    pub fn new(rom_path: String) -> Cpu{
	Cpu {
	    boot_rom: [0; 256],
	    current_t_cycles: 0,
	    stack: vec![],
	    pc: 0,
	    state: CpuState::Boot,
	    registers: Default::default(),
	    memory_mapper: memory_mapper::MemoryMapper::new(rom_path)
	}
    }

    pub(crate) fn fetch(&mut self)->u16{
	let fetch_byte_big = self.memory_mapper.fetch_byte(self.pc, &self.state).expect("Invalid (or non-implemented memory) was requested") as u16;
	self.pc+=1;
	let fetch_byte_small = self.memory_mapper.fetch_byte(self.pc, &self.state).expect("Invalid (or non-implemented memory) was requested") as u16;
	self.pc +=1;

	fetch_byte_big << 8 | fetch_byte_small
    }

    pub(crate) fn execute(&mut self, first_byte_of_instruction: u16) -> u8 {

    }
}
