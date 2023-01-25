use raylib::prelude::KeyboardKey::*;
use raylib::RaylibHandle;

#[derive(Default)]
pub struct Joypad {
    pub byte: u8,
}

impl Joypad {
    // Updates the interal byte represetation of the input
    pub fn update_input(&mut self, raylib_handle: &mut RaylibHandle) {
        let p15_mask = 0b0010_0000;
        let p14_mask = 0b0001_0000;
        let p13_mask = 0b0000_1000;
        let p12_mask = 0b0000_0100;
        let p11_mask = 0b0000_0010;
        let p10_mask = 0b0000_0001;

        let mut byte = 0u8;

        // Go through every possible pressed button, if it pressed than unset it in the byte representation
        // All of the next operations are done in reverse, at the end of the function the byte is flipped
        // PS: This is some non ugly code but the raylib_handle.get_key_pressed() was not returning the key if it was held down
        if raylib_handle.is_key_down(KEY_A)
            || raylib_handle.is_key_down(KEY_W)
            || raylib_handle.is_key_down(KEY_S)
            || raylib_handle.is_key_down(KEY_D)
        {
            byte = p14_mask;
            if raylib_handle.is_key_down(KEY_D) {
                byte |= p10_mask;
            }
            if raylib_handle.is_key_down(KEY_A) {
                byte |= p11_mask;
            }
            if raylib_handle.is_key_down(KEY_W) {
                byte |= p12_mask;
            }
            if raylib_handle.is_key_down(KEY_S) {
                byte |= p13_mask;
            }
            println!("Byte: {:b}", !byte);
        } else if raylib_handle.is_key_down(KEY_I)
            || raylib_handle.is_key_down(KEY_U)
            || raylib_handle.is_key_down(KEY_J)
            || raylib_handle.is_key_down(KEY_K)
        {
            byte = p15_mask;
            if raylib_handle.is_key_down(KEY_K) {
                byte |= p10_mask;
            }
            if raylib_handle.is_key_down(KEY_J) {
                byte |= p11_mask;
            }
            if raylib_handle.is_key_down(KEY_U) {
                byte |= p12_mask;
            }
            if raylib_handle.is_key_down(KEY_I) {
                byte |= p13_mask;
            }
            println!("Byte: {:b}", !byte);
        }
        self.byte = !byte;
    }
}
