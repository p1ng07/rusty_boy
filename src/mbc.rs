use self::no_mbc::KIBI_BYTE;

pub trait Mbc {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, byte: u8);
    fn get_rom_banks(&self) -> Vec<[u8; 16 * KIBI_BYTE]>;
    fn get_ram_banks(&self) -> Option<Vec<[u8; 8 * KIBI_BYTE]>>;
}

pub mod mbc1;
pub mod mbc3;
pub mod mbc5;
pub mod no_mbc;
