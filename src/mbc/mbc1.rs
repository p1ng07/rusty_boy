use super::Mbc;

pub struct Mbc1 {
    
}

impl Mbc for Mbc1 {
    fn read_byte(&self, address: u16) -> u8 {
        todo!()
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        todo!()
    }
}

impl Mbc1 {
    pub fn new(total_rom: Vec<u8>) -> Self {
	Self{}
    }
}
