use egui::Ui;
use serde::Deserialize;
use serde::Serialize;

use crate::cpu::is_bit_set;
use crate::interrupt_handler::Interrupt;
use crate::interrupt_handler::InterruptHandler;

#[derive(Serialize, Deserialize)]
pub struct Joypad {
    pub byte: u8,
    group_action: u8,
    group_direction: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            byte: 0xFF,
            group_action: 0,
            group_direction: 0,
        }
    }
    // Updates the interal byte represetation of the input, returns true if a key has been pressed
    pub fn update_input(&mut self, ui: &Ui, interrupt_handler: &mut InterruptHandler) {
        // let p15_mask = 0b0010_0000;
        // let p14_mask = 0b0001_0000;
        let p13_mask = 0b0000_1000;
        let p12_mask = 0b0000_0100;
        let p11_mask = 0b0000_0010;
        let p10_mask = 0b0000_0001;

        self.group_action = 0;
        self.group_direction = 0;

        // Go through every possible pressed button, if it pressed than unset it in the byte representation
        if ui.input(|i| i.key_down(egui::Key::D)) {
            // Right button
            self.group_direction |= p10_mask;
        }
        if ui.input(|i| i.key_down(egui::Key::A)) {
            // Left button
            self.group_direction |= p11_mask;
        }
        if ui.input(|i| i.key_down(egui::Key::W)) {
            // High button
            self.group_direction |= p12_mask;
        }
        if ui.input(|i| i.key_down(egui::Key::S)) {
            // Down button
            self.group_direction |= p13_mask;
        }

        if ui.input(|i| i.key_down(egui::Key::K)) {
            // A button
            self.group_action |= p10_mask;
        }
        if ui.input(|i| i.key_down(egui::Key::J)) {
            // B button
            self.group_action |= p11_mask;
        }
        if ui.input(|i| i.key_down(egui::Key::U)) {
            //Select button
            self.group_action |= p12_mask;
        }
        if ui.input(|i| i.key_down(egui::Key::I)) {
            //Start button
            self.group_action |= p13_mask;
        }

        if !is_bit_set(self.byte, 5) && !is_bit_set(self.byte, 4) {
            self.byte = 0xC0;
            self.byte ^= self.group_direction & self.group_action;
            return;
        }
        if !is_bit_set(self.byte, 4) {
            self.byte = 0xE0;
            self.byte |= !self.group_direction;
            // if self.byte == 0b1110_1111 {self.byte = 0xFF;}
            return;
        }
        if !is_bit_set(self.byte, 5) {
            self.byte = 0xD0;
            println!("polling action buttons");
            self.byte |= !self.group_action;
            // if self.byte == 0b1101_1111 {self.byte = 0xFF;}
            return;
        }
        self.byte = 0xFF;
    }

    pub(crate) fn write_to_byte(
        &mut self,
        received_byte: u8,
        interrupt_handler: &mut InterruptHandler,
    ) {
        self.byte = (received_byte & 0b0011_0000) | 0b1100_1111;

        if !is_bit_set(self.byte, 5) && !is_bit_set(self.byte, 4) {
            self.byte ^= self.group_direction & self.group_action;
            if self.byte != 0b1100_1111 {
                interrupt_handler.request_interrupt(Interrupt::Joypad);
            }
            return;
        }
        if !is_bit_set(self.byte, 4) {
            self.byte ^= self.group_direction;
            if self.byte != 0b1110_1111 {
                interrupt_handler.request_interrupt(Interrupt::Joypad);
            }
            return;
        }
        if !is_bit_set(self.byte, 5) {
            self.byte ^= self.group_action;
            if self.byte != 0b1101_1111 {
                interrupt_handler.request_interrupt(Interrupt::Joypad);
            }
            return;
        }
    }
}
