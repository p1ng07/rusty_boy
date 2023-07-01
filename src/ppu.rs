use crate::constants::*;
use crate::cpu::is_bit_set;
use epaint::Color32;

use crate::constants::{GAMEBOY_HEIGHT, GAMEBOY_WIDTH};
use crate::{
    constants::{self, BG_WIN_ENABLED_BIT},
    interrupt_handler::{Interrupt, InterruptHandler},
};

// Scanline based rendering of the ppu
pub struct Ppu {
    pub vram: [u8; 8196], // 8 kibibytes of vram
    pub oam_ram: [u8; 160],
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
    pub current_framebuffer: [Color32; GAMEBOY_WIDTH * GAMEBOY_HEIGHT],
    pub lcd_status: u8,
    pub wy: u8, // Window y position
    pub wx: u8, // Window x position + 7
    win_ly: u8,
    wy_condition: bool,
    bg_color_lookup_table: [Color32; 4],
    stat_requested_on_current_line: bool,
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
    HBlank,     // Horizontal blank
    Vblank,     // Vertical Blank
    OamScan,    // OAM Scan
    DrawPixels, // Drawing pixels
}

impl Ppu {
    pub fn new() -> Ppu {
        Self {
            vram: [0; 8196],
            oam_ram: [0; 0xA0],
            mode: PpuModes::OamScan,
            current_elapsed_dots: 1,
            current_framebuffer: [Color32::WHITE; GAMEBOY_WIDTH * GAMEBOY_HEIGHT],
            lcd_status: 2, // the lcd status will start with in mode 2
            ly: 0,
            lyc: 0,
            lcdc: 0,
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            wy_condition: false,
            win_ly: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            stat_requested_on_current_line: false,
            bg_color_lookup_table: [
                Color32::WHITE,
                Color32::LIGHT_GRAY,
                Color32::DARK_GRAY,
                Color32::BLACK,
            ],
        }
    }

    fn compare_ly_lyc(&mut self, interrupt_handler: &mut InterruptHandler) {
        if self.ly == self.lyc && !self.stat_requested_on_current_line {
            self.lcd_status |= 0b0000_0100;

            // If the 'ly==lyc' interrupt is enabled, fire it
            if is_bit_set(self.lcd_status, 6) {
                interrupt_handler.request_interrupt(Interrupt::Stat);
                self.stat_requested_on_current_line = true;
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

    // Advances the ppu state machine 1 dot forward
    pub fn tick(&mut self, interrupt_handler: &mut InterruptHandler) {
        if !is_bit_set(self.lcdc, constants::LCD_ENABLED_BIT) {
            return;
        }

        self.current_elapsed_dots += 1;

        match self.mode {
            PpuModes::OamScan => self.oam_scan(),
            PpuModes::DrawPixels => self.draw_pixels(interrupt_handler),
            PpuModes::HBlank => {
                self.horizontal_blank(interrupt_handler);
            }
            PpuModes::Vblank => self.vertical_blank(interrupt_handler),
        }

        self.compare_ly_lyc(interrupt_handler);
        self.update_current_mode_in_lcd_status();
    }

    // Perform the oam scan step of the ppu
    // As this is a scanline-based renderer, it just advances the ppu 1 dot and fires interrupts,
    // no actual drawing is done until the end of mode 3
    fn oam_scan(&mut self) {
        // OAM scans takes only 80 dots
        if self.current_elapsed_dots > 76 {
            self.mode = PpuModes::DrawPixels;
        }
    }

    // Performs the drawing pixels step of the ppu
    // This takes a fixed 172 dots
    // At the end of this mode, the screen should be drawn before we enter hblank
    fn draw_pixels(&mut self, interrupt_handler: &mut InterruptHandler) {
        // drawing pixels takes 172 dots
        // Change into hblank when that ellapses and render the current line
        if self.current_elapsed_dots > 247 {
            if is_bit_set(self.lcdc, 0) {
                self.render_background();
            }

            self.render_sprites();

            // Check if a hblank stat interrupt should fire
            if is_bit_set(self.lcd_status, 3) {
                interrupt_handler.request_interrupt(Interrupt::Stat);
            }

            self.mode = PpuModes::HBlank;
        }
    }

    // Performs the horizontal step of the ppu
    // This takes a fixed 204 dots
    // At the end of this mode, we can either go into vblank (if the new scanline is 144) and render the screen, or into another oam scan
    fn horizontal_blank(&mut self, interrupt_handler: &mut InterruptHandler) {
        if self.current_elapsed_dots > 451 {
            self.current_elapsed_dots = 1;

            self.compare_ly_lyc(interrupt_handler);
            self.ly += 1;
            self.compare_ly_lyc(interrupt_handler);

            if self.ly == 144 {
                interrupt_handler.request_interrupt(Interrupt::Vblank);

                // Check if a stat interrupt should fire
                if is_bit_set(self.lcd_status, 4) {
                    interrupt_handler.request_interrupt(Interrupt::Stat);
                }
                self.stat_requested_on_current_line = false;

                self.mode = PpuModes::Vblank;
            } else {
                // Check if a oam scan interrupt should fire
                if is_bit_set(self.lcd_status, 5) {
                    interrupt_handler.request_interrupt(Interrupt::Stat);
                }

                self.stat_requested_on_current_line = false;
                self.mode = PpuModes::OamScan;

                // Check for wy == ly at the start of every mode 2
                if !self.wy_condition {
                    if self.wy == self.ly && is_bit_set(self.lcdc, WINDOW_ENABLED_BIT) {
                        self.wy_condition = true;
                    }
                }
            }
        }
    }

    fn vertical_blank(&mut self, interrupt_handler: &mut InterruptHandler) {
        // vertical blank takes 10 scanlines, 456 dots each
        if self.current_elapsed_dots > 451 {
            self.current_elapsed_dots = 1;
            self.ly += 1;
            self.compare_ly_lyc(interrupt_handler);

            if self.ly > 153 {
                self.ly = 0;
                self.compare_ly_lyc(interrupt_handler);

                // check if a oam scan interrupt should occur
                if is_bit_set(self.lcd_status, 5) {
                    interrupt_handler.request_interrupt(Interrupt::Stat);
                }

                self.stat_requested_on_current_line = false;

                self.mode = PpuModes::OamScan;

                self.wy_condition = false;
                self.win_ly = 0;
                // if self.wy_condition == false {
                //     if self.wy == self.ly && is_bit_set(self.lcdc, WINDOW_ENABLED_BIT){
                // 	self.wy_condition = true;
                //     }
                // }
            }
        }
    }

    pub(crate) fn write_to_lcd_status(&mut self, received_byte: u8) {
        self.lcd_status = 0b1000_0111 | (received_byte & 0b0111_1000);
    }

    fn update_current_mode_in_lcd_status(&mut self) {
        self.lcd_status &= 0b1111_1100;
        match self.mode {
            PpuModes::HBlank => (),
            PpuModes::Vblank => self.lcd_status |= 0x1,
            PpuModes::OamScan => self.lcd_status |= 0x2,
            PpuModes::DrawPixels => self.lcd_status |= 0x3,
        }
    }

    // Renders the background of the current scanline
    fn render_background(&mut self) {
        let bg_tilemap: u16 = if is_bit_set(self.lcdc, BG_TILEMAP_AREA_BIT) {
            0x1C00
        } else {
            0x1800
        };
        let win_tilemap: u16 = if is_bit_set(self.lcdc, WINDOW_TILEMAP_AREA_BIT) {
            0x1C00
        } else {
            0x1800
        };

        let pixel_y = self.ly;
        let mut window_was_drawn = false;

        for pixel_x in 0..GAMEBOY_WIDTH as u8 {
            // Render window if all conditions are met, otherwise render background
            let window_draw = is_bit_set(self.lcdc, WINDOW_ENABLED_BIT)
                && pixel_x + 7 >= self.wx
                && self.wy_condition;

            let (tilemap_pixel_x, tilemap_pixel_y, tilemap) = if window_draw {
                window_was_drawn = true;
                (pixel_x + 7 - self.wx, self.win_ly, win_tilemap)
            } else {
                (
                    pixel_x.wrapping_add(self.scx),
                    pixel_y.wrapping_add(self.scy),
                    bg_tilemap,
                )
            };

            // Get the tile indexes inside of the tilemap
            // This is in case the background is to be drawn

            let tilemap_tile_x = tilemap_pixel_x % 32;
            let tilemap_tile_y = tilemap_pixel_y % 32;
            let tile_index: u16 = (tilemap_pixel_x / 8) as u16 + (tilemap_pixel_y / 8) as u16 * 32;
            let tile_id_address = tilemap as usize + tile_index as usize;

            // Actual tile id to be used in tilemap addressing
            let tile_id = self.vram[tile_id_address];

            let row_start_address: usize = if is_bit_set(self.lcdc, BG_WIN_TILEDATA_AREA_BIT) {
                // unsigned addressing
                tile_id as usize * 16 + (tilemap_tile_y & 7) as usize * 2
            } else {
                // signed addressing
                let address =
                    0x1000i32 + (tile_id as i8 as i32 * 16) + (tilemap_tile_y as i32 & 7) * 2;
                address as usize
            };

            // Get the tiledata with the offset to get the data of the line that is being rendered
            // This data represents the whole line that is to be drawn
            let tiledata_lsb = self.vram[row_start_address];
            let tiledata_msb = self.vram[row_start_address + 1];

            // Compute the color id of the given pixel
            let x_offset: u8 = tilemap_tile_x % 8;

            let lsb = (tiledata_lsb >> (7 - x_offset)) & 1;
            let msb = (tiledata_msb >> (7 - x_offset)) & 1;

            let color_index = (msb << 1) | lsb;

            let buffer_index = pixel_x as usize + self.ly as usize * GAMEBOY_WIDTH;

            let color_lookup = self.bg_color_lookup_table
                [(self.bgp as usize >> (color_index * 2)) & 0b11 as usize];

            // Paint the current pixel onto the current framebuffer
            self.current_framebuffer[buffer_index] = color_lookup;
        }

        if window_was_drawn && self.win_ly < 144 {
            self.win_ly += 1;
        }
    }

    fn render_sprites(&mut self) {}
}
