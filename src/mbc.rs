pub trait Mbc {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, byte: u8);
}

pub mod mbc1;
pub mod mbc3;
pub mod mbc5;
pub mod no_mbc;
