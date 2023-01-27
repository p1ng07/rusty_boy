pub struct Ppu {
    pub vram: [u8; 8196], // 8 kibibytes of vram
    pub oam_ram: [u8; 0xA0],
}
impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; 8196],
            oam_ram: [0; 0xA0],
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            vram: [0; 8196],
            oam_ram: [0; 0xA0],
        }
    }
}
