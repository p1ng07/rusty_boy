use std::{io, fs};

use self::memory_mapper::MemoryMapper;

pub mod cpu_registers;
pub mod memory_mapper;

enum CpuState {
    Boot,
    NonBoot
}

pub struct Cpu {
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
	    current_t_cycles: 0,
	    stack: vec![],
	    pc: 0,
	    state: CpuState::Boot,
	    registers: Default::default(),
	    memory_mapper: memory_mapper::MemoryMapper::new(rom_path)
	}
    }

    pub fn fetch(&mut self)->u16{
	let mut x: u16 = (self.memory_mapper.fetch_byte(self.pc) as u16 )<< 8;
	self.pc+=1;
	println!("First byte {}" , format!("{:X}", x));
	x |= self.memory_mapper.fetch_byte(self.pc) as u16;
	self.pc +=1;
	x
    }
}
