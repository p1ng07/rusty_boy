pub struct Ppu {
    vram: [u8; 8196], // 8 kibibytes of vram
}
impl Ppu {
    pub fn new() -> Self {
        Self { vram: [0; 8196] }
    }

    pub fn get_vram(&self, address: i32) -> Option<u8> {
        self.vram.get(address as usize).copied()
    }

    pub fn set_vram(&mut self, address: i32, value: u8) {
        self.vram[address as usize] = value;
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self { vram: [0; 8196] }
    }
}
