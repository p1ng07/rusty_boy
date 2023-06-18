use crate::interrupt_handler::{InterruptHandler, Interrupt};

// Scanline based rendering of the ppu
pub struct Ppu {
    pub vram: [u8; 8196], // 8 kibibytes of vram
    pub oam_ram: [u8; 0xA0],
    mode: PpuModes,
    ly: u8,
    lyc: u8,
    lcdc: u8,
    status: u8,
}

pub enum LCDCBit {
    LcdEnabled,
    WindowTileMapArea,
    WindowEnabled,
    BgWinTileDataArea,
    BgTileMapArea,
    ObjSize,
    ObjEnable,
    BgWinEnablePriority
}

enum PpuModes {
    Mode0, // Horizontal blank
    Mode1, // Vertical Blank
    Mode2, // OAM Scan
    Mode3, // Drawing pixels
}

impl Ppu {
    pub fn new() -> Ppu {
        Self {
            vram: [0; 8196],
            oam_ram: [0; 0xA0],
	    mode: PpuModes::Mode2,
            ly: 0,
            lyc: 0,
            lcdc: 0,
	    status: 0
        }
    }

    fn check_ly_lyc(&mut self, interrupt_handler: &mut InterruptHandler){
	if self.ly == self.lyc {
	    self.status |= 0b0000_0100;
	    // TODO: maybe request an interrupt here, i dont know
	    interrupt_handler.request_interrupt(Interrupt::Stat);
	}
    }

    fn is_lcdc_bit_high(&self, lcdc_bit: LCDCBit) -> bool {
	return match lcdc_bit {
	    LCDCBit::LcdEnabled => self.lcdc & 0x80 > 0,
	    LCDCBit::WindowTileMapArea => self.lcdc & 0x40 > 0,
	    LCDCBit::WindowEnabled => self.lcdc & 0x20 > 0,
	    LCDCBit::BgWinTileDataArea => self.lcdc & 0x10 > 0,
	    LCDCBit::BgTileMapArea => self.lcdc & 0x08 > 0,
	    LCDCBit::ObjSize => self.lcdc & 0x04 > 0,
	    LCDCBit::ObjEnable => self.lcdc & 0x02 > 0,
	    LCDCBit::BgWinEnablePriority => self.lcdc & 0x01 > 0,
	}
    }


    // Advances the ppu state machine 1 m-cycle forward
    pub fn tick(&mut self, interrupt_handler: &mut InterruptHandler){
	
    }

    pub fn fetch_byte(&self, _address: u16) -> u8 {
	todo!()
    }
}
