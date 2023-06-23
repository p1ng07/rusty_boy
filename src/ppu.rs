use crate::interrupt_handler::{InterruptHandler, Interrupt};

// Scanline based rendering of the ppu
pub struct Ppu {
    pub vram: [u8; 8196], // 8 kibibytes of vram
    pub oam_ram: [u8; 0xA0],
    mode: PpuModes,
    current_scanline: u8,
    current_elapsed_dots: u16,
    pub bgp: u8,        // Bg palette data
    pub obp0: u8,       // Obj palette 0
    pub obp1: u8,       // Obj palette 1
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub lcdc: u8,
    pub lcd_status: u8,
    pub wy: u8,        // Window y position
    pub wx: u8         // Window x position + 7
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
	    current_elapsed_dots: 0,
            ly: 0,
            lyc: 0,
            lcdc: 0,
	    lcd_status: 2, // the lcd status will start with in mode 2
            scy: 0,
            scx: 0,
	    wy: 0,
	    wx: 0,
	    current_scanline: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
        }
    }

    fn compare_ly_lyc(&mut self, interrupt_handler: &mut InterruptHandler){
	if self.ly == self.lyc {
	    self.lcd_status |= 0b0000_0100;

	    // If the 'ly==lyc' interrupt is enabled, fire it
	    if self.lcd_status & 0b0100_0000 {
		interrupt_handler.request_interrupt(Interrupt::Stat);
	    }
	}
    }

    pub fn fetch_oam(&self, address: u16) -> u8 {
	match self.mode {
	    PpuModes::Mode2 | PpuModes::Mode3 => 0xFF,
	    _ => self.oam_ram[address as usize]
	}
    }

    pub fn write_oam(&mut self, address: u16, byte: u8) {
	match self.mode {
	    PpuModes::Mode2 | PpuModes::Mode3 => (),
	    _ => self.oam_ram[address as usize] = byte
	}
    }

    pub fn write_vram(&mut self, address: u16, byte:u8) {
	match self.mode {
	    PpuModes::Mode3 => (),
	    _ => self.vram[address as usize] = byte
	}
    }

    pub fn fetch_vram(&self, address: u16) -> u8 {
	match self.mode {
	    PpuModes::Mode3 => 0xFF,
	    _ => self.vram[address as usize]
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

    // Advances the ppu state machine 1 dot forward
    pub fn tick(&mut self, interrupt_handler: &mut InterruptHandler){
	if !self.is_lcdc_bit_high(LCDCBit::LcdEnabled) {
	    return;
	}

	self.compare_ly_lyc(interrupt_handler);

	match self.mode {
	    PpuModes::Mode2 => self.oam_scan(interrupt_handler),
	    PpuModes::Mode3 => self.draw_pixels(interrupt_handler),
	    PpuModes::Mode0 => {
		self.horizontal_blank(interrupt_handler);
	    },
	    PpuModes::Mode1 => self.vertical_blank(interrupt_handler),
	}

	self.update_current_mode_in_lcd_status();
	self.current_elapsed_dots += 1;

    }

    // Perform the oam scan step of the ppu
    // As this is a scanline-based renderer, it just advances the ppu 1 dot and fires interrupts,
    // no actual drawing is done until the end of mode 3
    fn oam_scan(&self, interrupt_handler: &mut InterruptHandler) {

	// OAM scans takes only 79 dots
	if self.current_elapsed_dots > 79 {
	    self.current_elapsed_dots = 0;
	    self.mode = PpuModes::Mode3;
	}
    }

    // Performs the drawing pixels step of the ppu
    // This takes a fixed 172 dots
    // At the end of this mode, the screen should be drawn before we enter hblank
    fn draw_pixels(&self, interrupt_handler: &mut InterruptHandler) {

	// drawing pixels takes 172 dots
	// Change into hblank when that happens
	if self.current_elapsed_dots > 171 {
	    self.current_elapsed_dots = 0;

	    // Check if a hblank interrupt should fire
	    if self.lcd_status & 0b0000_1000 > 0 {
		interrupt_handler.request_interrupt(Interrupt::Stat);
	    }

	    // TODO draw line here
	    
	    self.mode = PpuModes::Mode0;
	}
    }

    // Performs the horizontal step of the ppu
    // This takes a fixed 204 dots
    // At the end of this mode, we can either go into vblank (if the new scanline is 144), or into another oam scan
    fn horizontal_blank(&self, interrupt_handler: &mut InterruptHandler) -> _ {

	if self.current_elapsed_dots > 203 {
	    self.current_elapsed_dots = 0;
	    self.ly += 1;

	    if self.ly == 144 {
		// Check if a vblank interrupt should fire
		if self.lcd_status & 0b0001_0000 > 0 {
		    interrupt_handler.request_interrupt(Interrupt::Stat);
		}

		self.mode = PpuModes::Mode1;
	    } else{
		// Check if a oam scan interrupt should fire
		if self.lcd_status & 0b0010_0000 > 0 {
		    interrupt_handler.request_interrupt(Interrupt::Stat);
		}

		self.mode = PpuModes::Mode2;
	    }
	}
    }

    fn vertical_blank(&self, interrupt_handler: &mut InterruptHandler) {

	// vertical blank takes 10 scanlines, 456 dots each
	if self.current_elapsed_dots > 455 {
	    self.current_elapsed_dots = 0;
	    self.ly += 1;

	    if self.ly > 153 {
		self.ly = 0;
		
		// check if a oam scan interrupt should occur
		if self.lcd_status & 0b0010_0000 > 0 {
		    interrupt_handler.request_interrupt(Interrupt::Stat);
		}

		self.mode = PpuModes::Mode2;
	    }
	}
    }

    pub(crate) fn write_to_lcd_status(&self, received_byte: u8) {
	self.lcd_status = (self.lcd_status & 0b1000_0111) | (received_byte & 0b0111_1000);
    }

    fn update_current_mode_in_lcd_status(&mut self) {
	self.lcd_status &= 0b1111_1100;
	match self.mode {
	    PpuModes::Mode0 => (),
	    PpuModes::Mode1 => lcd_status |= 0x1,
	    PpuModes::Mode2 => lcd_status |= 0x2,
	    PpuModes::Mode3 => lcd_status |= 0x3,
}
    }

}
