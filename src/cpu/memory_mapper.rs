use std::fs;

pub struct MemoryMapper{
    rom0: [u8; 0x1000],
    rom_path: String,
    are_interrupts_enabled: bool,
}

impl MemoryMapper {
    pub fn fetch_byte(&self, address: usize) -> u8{
	self.rom0[address]
    }

    pub fn new(rom_path: String) -> MemoryMapper {
	MemoryMapper{
	    rom0: fs::read(&rom_path).expect("ficheiro {} indisponivel!")[..0x1000].try_into().unwrap(),
	    rom_path,
	    are_interrupts_enabled: false
	}
    }
}
