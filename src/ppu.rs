use std::ops::Rem;

use crate::constants::*;
use crate::cpu::is_bit_set;
use epaint::Color32;
use serde::{Serialize, Deserialize};

use crate::constants::{GAMEBOY_HEIGHT, GAMEBOY_WIDTH};
use crate::{
    constants,
    interrupt_handler::{Interrupt, InterruptHandler},
};

// Scanline based rendering of the ppu
#[derive(Serialize, Deserialize)]
pub struct Ppu {
    is_dmg: bool,
    #[serde(with = "serde_arrays")]
    pub vram_0: [u8; 0x2000], // 8 kibibytes of vram
    #[serde(with = "serde_arrays")]
    pub vram_1: [u8; 0x2000], // 8 kibibytes of vram
    pub vram_bank_index: usize,
    #[serde(with = "serde_arrays")]
    pub oam_ram: [u8; 160],
    pub mode: PpuModes,
    current_elapsed_dots: u16,
    pub bgp: u8,  // Bg palette data
    pub obp0: u8, // Obj palette 0
    pub obp1: u8, // Obj palette 1
    color_lookup_table: [Color32; 4],
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub lcdc: u8,
    #[serde(with = "serde_arrays")]
    pub current_framebuffer: [Color32; GAMEBOY_WIDTH * GAMEBOY_HEIGHT],
    // Saves the color index and bg tile attribute priority of the drawn bg pixels
    #[serde(with = "serde_arrays")]
    current_framebuffer_bg_pixel_info: [u8; GAMEBOY_WIDTH * GAMEBOY_HEIGHT],
    pub lcd_status: u8,
    pub wy: u8, // Window y position
    pub wx: u8, // Window x position + 7
    win_ly: u8,
    wy_condition: bool,
    stat_requested_on_current_line: bool,

    #[serde(with = "serde_arrays")]
    pub bg_color_ram: [u8; 64],
    #[serde(with = "serde_arrays")]
    pub sprite_color_ram: [u8; 64],
    pub bg_palette_index: usize,
    pub sprite_palette_index: usize,
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

#[derive(Serialize, Deserialize)]
pub enum PpuModes {
    HBlank,     // Horizontal blank
    Vblank,     // Vertical Blank
    OamScan,    // OAM Scan
    DrawPixels, // Drawing pixels
}

impl Ppu {
    pub fn new(is_dmg: bool) -> Ppu {
        Self {
            is_dmg,
            oam_ram: [0; 0xA0],
            mode: PpuModes::OamScan,
            current_elapsed_dots: 1,
            current_framebuffer: [Color32::from_rgb(155, 188, 15); GAMEBOY_WIDTH * GAMEBOY_HEIGHT],
            current_framebuffer_bg_pixel_info: [0; GAMEBOY_WIDTH * GAMEBOY_HEIGHT],
            lcd_status: 2, // the lcd status will start with in mode 2
            vram_0: [0; 0x2000],
            vram_1: [0; 0x2000],
            bg_color_ram: [0; 64],
            sprite_color_ram: [0; 64],
            vram_bank_index: 0,
            bg_palette_index: 0,
            sprite_palette_index: 0,
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
            color_lookup_table: [
                Color32::from_rgb(155, 188, 15),
                Color32::from_rgb(139, 172, 15),
                Color32::from_rgb(48, 98, 48),
                Color32::from_rgb(15, 56, 15),
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
        if self.vram_bank_index & 1 == 0 {
            self.vram_0[address as usize] = byte;
        } else {
            self.vram_1[address as usize] = byte;
        }
    }

    pub fn fetch_vram(&self, address: u16) -> u8 {
        // FIXME vram locking
        // match self.mode {
        //     PpuModes::Mode3 => 0xFF,
        //     _ =>
        // }
        if self.vram_bank_index & 1 == 0 {
            self.vram_0[address as usize]
        } else {
            self.vram_1[address as usize]
        }
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
            if !self.is_dmg {
                self.render_background();
                if is_bit_set(self.lcdc, 1) {
                    self.render_sprites();
                }
            } else {
                if is_bit_set(self.lcdc, 0) {
                    self.render_background();
                }
                if is_bit_set(self.lcdc, 1) {
                    self.render_sprites();
                }
            }

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
                if !self.wy_condition {
                    if self.wy == self.ly && is_bit_set(self.lcdc, WINDOW_ENABLED_BIT) {
                        self.wy_condition = true;
                    }
                }
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
            let mut tilemap_tile_y = tilemap_pixel_y % 32;
            let tile_index: u16 = (tilemap_pixel_x / 8) as u16 + (tilemap_pixel_y / 8) as u16 * 32;

            let tile_id_address = tilemap as usize + tile_index as usize;

            // Actual tile id to be used in tilemap addressing
            let tile_id = self.vram_0[tile_id_address];
            let tile_attributes = self.vram_1[tile_id_address];

            // Vertical flip background tile
            if !self.is_dmg && is_bit_set(tile_attributes, 6) {
                tilemap_tile_y = 7u8 - (tilemap_pixel_y as u8 % 8);
            }

            let row_start_address: usize = if is_bit_set(self.lcdc, BG_WIN_TILEDATA_AREA_BIT) {
                // unsigned addressing
                tile_id as usize * 16 + (tilemap_tile_y & 7) as usize * 2
            } else {
                // signed addressing
                (0x1000i32 + (tile_id as i8 as i32 * 16) + (tilemap_tile_y as i8 as i32 & 7) * 2)
                    as usize
            };

            // Get the tiledata with the offset to get the data of the line that is being rendered
            // This data represents the whole line that is to be drawn
            let tiledata_lsb = if !self.is_dmg && is_bit_set(tile_attributes, 3) {
                self.vram_1[row_start_address]
            } else {
                self.vram_0[row_start_address]
            };
            let tiledata_msb = if !self.is_dmg && is_bit_set(tile_attributes, 3) {
                self.vram_1[row_start_address + 1]
            } else {
                self.vram_0[row_start_address + 1]
            };

            // Compute the color id of the given pixel
            let mut x_offset: u8 = tilemap_tile_x % 8;

            // Horizontal flip background tile
            if !self.is_dmg && is_bit_set(tile_attributes, 5) {
                x_offset = 7u8.saturating_sub(tilemap_tile_x % 8);
            }

            let lsb = (tiledata_lsb >> (7 - x_offset)) & 1;
            let msb = (tiledata_msb >> (7 - x_offset)) & 1;

            let color_index = (msb << 1) | lsb;

            let buffer_index = pixel_x as usize + self.ly as usize * GAMEBOY_WIDTH;

            let color = if !self.is_dmg {
                // Start of the ram location of the color to be used
                let color_lsb_index =
                    (tile_attributes & 0b111) as usize * 8 + color_index as usize * 2;

                // Get the xbbbbbgg gggrrrrr color format in a unique number;
                let color_rgb555 = self.bg_color_ram[color_lsb_index] as u16
                    | ((self.bg_color_ram[color_lsb_index + 1] as u16) << 8);

                let red = (color_rgb555 & 0b1_1111) as u8;
                let green = ((color_rgb555 >> 5) & 0b1_1111) as u8;
                let blue = ((color_rgb555 >> 10) & 0b1_1111) as u8;

                Color32::from_rgb(
                    (red << 3) | (red >> 2),
                    (green << 3) | (green >> 2),
                    (blue << 3) | (blue >> 2),
                )
            } else {
                self.color_lookup_table[(self.bgp as usize >> (color_index * 2)) & 0b11 as usize]
            };

            // Paint the current pixel onto the current framebuffer
            self.current_framebuffer[buffer_index] = color;

            // Save the used bg/win color index
            self.current_framebuffer_bg_pixel_info[buffer_index] =
                color_index | tile_attributes & 0x80
        }

        if window_was_drawn && self.win_ly < 144 {
            self.win_ly += 1;
        }
    }

    fn render_sprites(&mut self) {
        // Vector of tuples (index, sprite) that saves the sprites on the current scanline
        let mut sprites: Vec<(usize, &[u8])> = Vec::new();

        // Get the indices of up to 10 sprites to be rendered on this line
        for i in (0..self.oam_ram.len()).step_by(4) {
            let obj_y = self.oam_ram[i].saturating_sub(16);

            let obj_size = if is_bit_set(self.lcdc, 2) { 16 } else { 8 };

            let range = obj_y..self.oam_ram[i].wrapping_sub(16).wrapping_add(obj_size);

            if range.contains(&self.ly) && sprites.len() < 10 {
                sprites.push((i, &self.oam_ram[i..(i + 4)]));
            }
        }

        /* Sort the array and put the higher indices first (lower priority objects) */
        sprites.sort_by(|a, b| b.0.cmp(&a.0));

        if self.is_dmg {
            /* Sort the array again but this time by the x coord, and put the lower priority objects first */
            sprites.sort_by(|a, b| b.1[1].cmp(&a.1[1]));
        }

        for sprite in sprites.iter() {
            let obj_size: usize = if is_bit_set(self.lcdc, 2) { 16 } else { 8 };
            let obj_y = sprite.1[0];
            let obj_x = sprite.1[1];

            // For 8x16 sprites, the bit 0 of tile_index should be ignored
            let tile_index = if obj_size == 8 {
                sprite.1[2] as usize
            } else {
                sprite.1[2] as usize & 0xFE
            };
            let attributes = sprite.1[3];

            let horizontal_flip = is_bit_set(attributes, 5);
            let vertical_flip = is_bit_set(attributes, 6);

            for pixel_x in obj_x.saturating_sub(8)..obj_x {
		if pixel_x >= 168 {
		    // If the current sprite is partly off-screen, don't draw those off-screen pixels
		    continue;
		}

                let mut tilemap_tile_y = (self.ly as usize + 16).saturating_sub(obj_y as usize);

                if vertical_flip {
                    tilemap_tile_y =
                        obj_size as usize - 1 - (self.ly as usize + 16 - obj_y as usize) % obj_size;
                }

                let row_start_address = tilemap_tile_y * 2 + tile_index * 16;

                let mut lsb = if is_bit_set(attributes, 3) && !self.is_dmg {
                    self.vram_1[row_start_address]
                } else {
                    self.vram_0[row_start_address]
                };

                let mut msb = if is_bit_set(attributes, 3) && !self.is_dmg {
                    self.vram_1[row_start_address + 1]
                } else {
                    self.vram_0[row_start_address + 1]
                };
                let mut x_offset: u8 = pixel_x.wrapping_sub(obj_x) % 8;

                if horizontal_flip {
                    x_offset = 7u8.saturating_sub(pixel_x.wrapping_sub(obj_x).rem(8));
                }

                lsb = (lsb >> (7 - x_offset)) & 1;
                msb = (msb >> (7 - x_offset)) & 1;

                let color_index = (msb << 1) | lsb;

                let buffer_index = pixel_x as usize + self.ly as usize * GAMEBOY_WIDTH;

                if !self.is_dmg {
                    let color_lsb_index =
                        (attributes & 0b111) as usize * 8 + color_index as usize * 2;

                    // Get the xbbbbbgg gggrrrrr color format in a unique number;
                    let color_rgb555 = self.sprite_color_ram[color_lsb_index] as u16
                        | ((self.sprite_color_ram[color_lsb_index + 1] as u16) << 8);

                    let red = (color_rgb555 & 0b1_1111) as u8;
                    let green = ((color_rgb555 >> 5) & 0b1_1111) as u8;
                    let blue = ((color_rgb555 >> 10) & 0b1_1111) as u8;

                    let color = Color32::from_rgb(
                        (red << 3) | (red >> 2),
                        (green << 3) | (green >> 2),
                        (blue << 3) | (blue >> 2),
                    );

                    // Don't paint the current pixel if it's transparent
                    if color_index != 0 {
                        if self.current_framebuffer_bg_pixel_info[buffer_index] & 0x7 == 0
                            || !is_bit_set(self.lcdc, 0)
                        {
                            // If there isn't any kind of bg/win priority, just draw it
                            self.current_framebuffer[buffer_index] = color;
                        } else {
                            // If bg/win colors 1-3 has priority over the current sprite, we need to check if the used color_index was 0
                            if !is_bit_set(attributes, 7)
                                && !is_bit_set(
                                    self.current_framebuffer_bg_pixel_info[buffer_index],
                                    7,
                                )
                            {
                                self.current_framebuffer[buffer_index] = color;
                            }
                        }
                    }
                } else {
                    let palette = if is_bit_set(attributes, 4) {
                        self.obp1
                    } else {
                        self.obp0
                    } as usize;
                    let color_lookup =
                        self.color_lookup_table[(palette >> (color_index * 2)) & 0b11 as usize];

                    // Don't paint the current pixel if it's transparent
                    if color_index != 0 {
                        if !is_bit_set(attributes, 7) {
                            // If there isn't any kind of bg/win priority, just draw it
                            self.current_framebuffer[buffer_index] = color_lookup;
                        } else {
                            // If bg/win colors 1-3 has priority over the current sprite, we need to check if the used color_index was 0
                            if self.current_framebuffer_bg_pixel_info[buffer_index] & 0b111 == 0 {
                                self.current_framebuffer[buffer_index] = color_lookup;
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn write_lcdc(&mut self, byte: u8) {
        // If the ppu has been turned off, reset it
        self.lcdc = byte;

        if !is_bit_set(self.lcdc, 7) {
            self.lcd_status &= 0b1111_1100;
        }
    }

    pub(crate) fn fetch_bg_palette_data(&self) -> u8 {
        self.bg_color_ram[self.bg_palette_index]
    }

    pub(crate) fn fetch_sprite_palette_data(&self) -> u8 {
        self.sprite_color_ram[self.sprite_palette_index]
    }

    pub(crate) fn write_sprite_palette_data(&mut self, received_byte: u8) {
        self.sprite_color_ram[self.sprite_palette_index & 0x3F] = received_byte;

        let increment_bit = self.sprite_palette_index & 0x80;
        if is_bit_set(self.sprite_palette_index as u8, 7) {
            self.sprite_palette_index += 1;
            if self.sprite_palette_index & 0b0111_1111 > 0b11_1111 {
                self.sprite_palette_index = increment_bit;
            }
        }
    }

    pub(crate) fn write_bg_palette_data(&mut self, received_byte: u8) {
        self.bg_color_ram[self.bg_palette_index & 0x3F] = received_byte;

        let increment_bit = self.bg_palette_index & 0x80;
        if is_bit_set(self.bg_palette_index as u8, 7) {
            self.bg_palette_index += 1;
            if self.bg_palette_index & 0b0111_1111 > 0b11_1111 {
                self.bg_palette_index = increment_bit;
            }
        }
    }
}
