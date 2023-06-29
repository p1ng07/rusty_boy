use epaint::Color32;

use crate::{
    game_app::{GAME_SCREEN_HEIGHT, GAME_SCREEN_WIDTH},
    interrupt_handler::{Interrupt, InterruptHandler},
};

// Scanline based rendering of the ppu
pub struct Ppu {
    pub vram: [u8; 8196], // 8 kibibytes of vram
    pub oam_ram: [u8; 0xA0],
    mode: PpuModes,
    current_elapsed_dots: u16,
    pub bgp: u8,  // Bg palette data
    pub obp0: u8, // Obj palette 0
    pub obp1: u8, // Obj palette 1
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub lcdc: u8,
    pub current_framebuffer: [Color32; GAME_SCREEN_WIDTH * GAME_SCREEN_HEIGHT],
    pub lcd_status: u8,
    pub wy: u8, // Window y position
    pub wx: u8, // Window x position + 7
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
    BgWinEnablePriority,
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
            current_elapsed_dots: 1,
            ly: 0,
            lyc: 0,
            lcdc: 0,
            lcd_status: 2, // the lcd status will start with in mode 2
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            current_framebuffer: [Color32::BLUE; GAME_SCREEN_WIDTH * GAME_SCREEN_HEIGHT],
        }
    }

    fn compare_ly_lyc(&mut self, interrupt_handler: &mut InterruptHandler) {
        if self.ly == self.lyc {
            self.lcd_status |= 0b0000_0100;

            // If the 'ly==lyc' interrupt is enabled, fire it
            if self.lcd_status & 0b0100_0000 > 0 {
                interrupt_handler.request_interrupt(Interrupt::Stat);
            }
        }
    }

    pub fn fetch_oam(&self, address: u16) -> u8 {
        // FIXME vram locking
        // match self.mode {
        //     PpuModes::Mode2 | PpuModes::Mode3 => 0xFF,
        //     _ => self.oam_ram[address as usize]
        // }
        self.oam_ram[address as usize]
    }

    pub fn write_oam(&mut self, address: u16, byte: u8) {
        // FIXME vram locking
        // match self.mode {
        //     PpuModes::Mode2 | PpuModes::Mode3 => (),
        //     _ => self.oam_ram[address as usize] = byte
        // }
        self.oam_ram[address as usize] = byte;
    }

    pub fn write_vram(&mut self, address: u16, byte: u8) {
        // FIXME Vram locking
        // match self.mode {
        //     PpuModes::Mode3 => (),
        //     _ => self.vram[address as usize] = byte
        // }
        self.vram[address as usize] = byte;
    }

    pub fn fetch_vram(&self, address: u16) -> u8 {
        // FIXME vram locking
        // match self.mode {
        //     PpuModes::Mode3 => 0xFF,
        //     _ =>
        // }
        self.vram[address as usize]
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
        };
    }

    // Advances the ppu state machine 1 dot forward
    pub fn tick(&mut self, interrupt_handler: &mut InterruptHandler) {
        if !self.is_lcdc_bit_high(LCDCBit::LcdEnabled) {
            return;
        }

        self.compare_ly_lyc(interrupt_handler);
        self.current_elapsed_dots += 1;

        match self.mode {
            PpuModes::Mode2 => self.oam_scan(),
            PpuModes::Mode3 => self.draw_pixels(interrupt_handler),
            PpuModes::Mode0 => {
                self.horizontal_blank(interrupt_handler);
            }
            PpuModes::Mode1 => self.vertical_blank(interrupt_handler),
        }

        self.compare_ly_lyc(interrupt_handler);
        self.update_current_mode_in_lcd_status();
    }

    // Perform the oam scan step of the ppu
    // As this is a scanline-based renderer, it just advances the ppu 1 dot and fires interrupts,
    // no actual drawing is done until the end of mode 3
    fn oam_scan(&mut self) {
        // OAM scans takes only 79 dots
        if self.current_elapsed_dots > 80 {
            self.current_elapsed_dots = 1;
            self.mode = PpuModes::Mode3;
        }
    }

    // Performs the drawing pixels step of the ppu
    // This takes a fixed 172 dots
    // At the end of this mode, the screen should be drawn before we enter hblank
    fn draw_pixels(&mut self, interrupt_handler: &mut InterruptHandler) {
        // drawing pixels takes 172 dots
        // Change into hblank when that ellapses and render the current line
        if self.current_elapsed_dots > 172 {
            self.current_elapsed_dots = 1;

	    if self.is_lcdc_bit_high(LCDCBit::BgWinEnablePriority){
		self.render_background_current_scanline();
	    }

            // Check if a hblank stat interrupt should fire
            if self.lcd_status & 0b0000_1000 > 0 {
                interrupt_handler.request_interrupt(Interrupt::Stat);
            }

            self.mode = PpuModes::Mode0;
        }
    }

    // Performs the horizontal step of the ppu
    // This takes a fixed 204 dots
    // At the end of this mode, we can either go into vblank (if the new scanline is 144) and render the screen, or into another oam scan
    fn horizontal_blank(&mut self, interrupt_handler: &mut InterruptHandler) {
        if self.current_elapsed_dots > 204 {
            self.current_elapsed_dots = 1;

            self.ly += 1;

            if self.ly == 144 {
                interrupt_handler.request_interrupt(Interrupt::Vblank);

                // Check if a stat interrupt should fire
                if self.lcd_status & 0b0001_0000 > 0 {
                    interrupt_handler.request_interrupt(Interrupt::Stat);
                }

                self.mode = PpuModes::Mode1;
            } else {
                // Check if a oam scan interrupt should fire
                if self.lcd_status & 0b0010_0000 > 0 {
                    interrupt_handler.request_interrupt(Interrupt::Stat);
                }

                self.mode = PpuModes::Mode2;
            }
        }
    }

    fn reset_current_framebuffer(&mut self) {
        self.current_framebuffer = [Color32::DEBUG_COLOR; GAME_SCREEN_WIDTH * GAME_SCREEN_HEIGHT];
    }

    fn vertical_blank(&mut self, interrupt_handler: &mut InterruptHandler) {
        // vertical blank takes 10 scanlines, 456 dots each
        if self.current_elapsed_dots > 456 {
            self.current_elapsed_dots = 1;
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

    pub(crate) fn write_to_lcd_status(&mut self, received_byte: u8) {
        self.lcd_status = 0b1000_0111 | (received_byte & 0b0111_1000);
    }

    fn update_current_mode_in_lcd_status(&mut self) {
        self.lcd_status &= 0b1111_1100;
        match self.mode {
            PpuModes::Mode0 => (),
            PpuModes::Mode1 => self.lcd_status |= 0x1,
            PpuModes::Mode2 => self.lcd_status |= 0x2,
            PpuModes::Mode3 => self.lcd_status |= 0x3,
        }
    }

    // Renders the background of the current scanline
    fn render_background_current_scanline(&mut self) {
        let bg_tilemap: u16 = if self.is_lcdc_bit_high(LCDCBit::BgTileMapArea) {
            0x1C00
        } else {
            0x1800
        };

        let pixel_y = self.ly;

        for pixel_x in 0..GAME_SCREEN_WIDTH as u8 {
            // Get the pixel indexes inside of the tilemap
            let tilemap_pixel_x = pixel_x.wrapping_add(self.scx);
            let tilemap_pixel_y = pixel_y.wrapping_add(self.scy);

            // Get the tile indexes inside of the tilemap
            let tilemap_tile_x = tilemap_pixel_x % 32;
            let tilemap_tile_y = tilemap_pixel_y % 32;

            let tile_x = tilemap_pixel_x / 8;
            let tile_y = tilemap_pixel_y / 8;

            let tile_index: u16 = tile_x as u16 + tile_y as u16 * 32;
            let tile_id_address = bg_tilemap as usize + tile_index as usize;

            // Actual tile id to be used in tilemap addressing
            let tile_id = self.vram[tile_id_address];

            let row_start_address: usize = if self.is_lcdc_bit_high(LCDCBit::BgWinTileDataArea) {
                // unsigned addressing
                tile_id as usize * 16 + (tilemap_tile_y & 7) as usize * 2
            } else {
                // signed addressing
		let mut address = 0x1000i32 + (tile_id as i8 as i32 * 16) + (tilemap_tile_y as i32 & 7) * 2;
                address as usize 
            };

            // Get the tiledata with the offset to get the data of the line that is being rendered
            // This data represents the whole line that is to be drawn
            let tiledata_least_significant_bits =
                self.vram[row_start_address];
            let tiledata_most_significant_bits =
                self.vram[row_start_address + 1];

            // Compute the color id of the given pixel
	    let x_offset_to_pixel: u8 = tilemap_tile_x % 8;
	    let color_index = (tiledata_most_significant_bits >> (6 - x_offset_to_pixel) & 2)
		| (tiledata_least_significant_bits >> (7 - x_offset_to_pixel as u32) & 1);

            // Paint the current pixel onto the current framebuffer
            let buffer_index = pixel_x as usize + self.ly as usize * GAME_SCREEN_WIDTH;
            self.current_framebuffer[buffer_index] =
                get_background_color_by_index(color_index, self.bgp);
        }
    }
}

fn get_background_color_by_index(color_index: u8, bgp: u8) -> Color32 {
    match color_index {
        0 => match bgp & 0b11 {
            0 => Color32::WHITE,
            1 => Color32::LIGHT_GRAY,
            2 => Color32::DARK_GRAY,
            3 => Color32::BLACK,
            _ => Color32::DEBUG_COLOR,
        },
        1 => match (bgp & 0b11_00) >> 2 {
            0 => Color32::WHITE,
            1 => Color32::LIGHT_GRAY,
            2 => Color32::DARK_GRAY,
            3 => Color32::BLACK,
            _ => Color32::DEBUG_COLOR,
        },
        2 => match (bgp & 0b11_00_00) >> 4 {
            0 => Color32::WHITE,
            1 => Color32::LIGHT_GRAY,
            2 => Color32::DARK_GRAY,
            3 => Color32::BLACK,
            _ => Color32::DEBUG_COLOR,
        },
        3 => match (bgp & 0b11_00_00_00) >> 6 {
            0 => Color32::WHITE,
            1 => Color32::LIGHT_GRAY,
            2 => Color32::DARK_GRAY,
            3 => Color32::BLACK,
            _ => Color32::DEBUG_COLOR,
        },
        _ => panic!("Cannot resolve color for index {}", color_index),
    }
}
