use egui::Ui;
use raylib::prelude::KeyboardKey::*;
use raylib::RaylibHandle;

use crate::interrupt_handler::Interrupt;
use crate::interrupt_handler::InterruptHandler;

#[derive(Default)]
pub struct Joypad {
    pub byte: u8,
}

impl Joypad {
    // Updates the interal byte represetation of the input, returns true if a key has been pressed
    pub fn update_input(
        &mut self,
        ui: &Ui,
        interrupt_handler: &mut InterruptHandler,
    ) {
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
        if ui.input(|i| i.key_pressed(egui::Key::A))
            || ui.input(|i| i.key_pressed(egui::Key::W))
            || ui.input(|i| i.key_pressed(egui::Key::S))
            || ui.input(|i| i.key_pressed(egui::Key::D))
        {
            interrupt_handler.request_interrupt(Interrupt::Joypad);
            byte = p14_mask;
            if ui.input(|i| i.key_pressed(egui::Key::D)) {
                byte |= p10_mask;
            }
            if ui.input(|i| i.key_pressed(egui::Key::A)) {
                byte |= p11_mask;
            }
            if ui.input(|i| i.key_pressed(egui::Key::W)) {
                byte |= p12_mask;
            }
            if ui.input(|i| i.key_pressed(egui::Key::S)) {
                byte |= p13_mask;
            }
        } else if ui.input(|i| i.key_pressed(egui::Key::I))
            || ui.input(|i| i.key_pressed(egui::Key::U))
            || ui.input(|i| i.key_pressed(egui::Key::J))
            || ui.input(|i| i.key_pressed(egui::Key::K))
        {
            interrupt_handler.request_interrupt(Interrupt::Joypad);
            byte = p15_mask;
            if ui.input(|i| i.key_pressed(egui::Key::K)) {
                byte |= p10_mask;
            }
            if ui.input(|i| i.key_pressed(egui::Key::J)) {
                byte |= p11_mask;
            }
            if ui.input(|i| i.key_pressed(egui::Key::U)) {
                byte |= p12_mask;
            }
            if ui.input(|i| i.key_pressed(egui::Key::I)) {
                byte |= p13_mask;
            }
        }
        self.byte = !byte;
    }

    pub(crate) fn write_to_byte(&mut self, received_byte: u8) {
        self.byte = (self.byte & 0xCF) | (received_byte & 0x30);
    }
}
