use egui::Ui;

use crate::cpu::is_bit_set;
use crate::interrupt_handler::Interrupt;
use crate::interrupt_handler::InterruptHandler;

pub struct Joypad {
    pub byte: u8,
    group_action: u8,
    group_direction: u8
}

// TODO joypad is broken, probably because this code sucks
impl Joypad {
    pub fn new() -> Self {
	Self {
	    byte: 0xFF,
	    group_action: 0,
	    group_direction: 0
	}
    }
    // Updates the interal byte represetation of the input, returns true if a key has been pressed
    pub fn update_input(&mut self, ui: &Ui, _interrupt_handler: &mut InterruptHandler) {
	// let p15_mask = 0b0010_0000;
	// let p14_mask = 0b0001_0000;
	let p13_mask = 0b0000_1000;
	let p12_mask = 0b0000_0100;
	let p11_mask = 0b0000_0010;
	let p10_mask = 0b0000_0001;


	if !is_bit_set(self.byte, 4) && !is_bit_set(self.byte, 5) {
	    self.byte = 0b1100_0000;
	    self.byte |= !(self.group_action & self.group_direction);
	    // if self.byte == 0b1100_1111 {self.byte = 0xFF;}
	} else if !is_bit_set(self.byte, 4) {
	    self.byte = 0b1110_0000;
	    self.byte |= !self.group_direction;
	    // if self.byte == 0b1110_1111 {self.byte = 0xFF;}
	} else if !is_bit_set(self.byte, 5) {
	    self.byte = 0b1101_0000;
	    self.byte |= !self.group_action;
	    // if self.byte == 0b1101_1111 {self.byte = 0xFF;}
	}
	self.byte = 0xFF;
	self.group_action = 0;
	self.group_direction = 0;

	// Go through every possible pressed button, if it pressed than unset it in the byte representation
	// All of the next operations are done in reverse, at the end of the function the byte is flipped
	// PS: This is some non ugly code but the raylib_handle.get_key_pressed() was not returning the key if it was held down
	// interrupt_handler.request_interrupt(Interrupt::Joypad);
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
    }

    pub(crate) fn write_to_byte(&mut self, received_byte: u8, interrupt_handler: &mut InterruptHandler) {
	if !is_bit_set(received_byte, 4) && !is_bit_set(received_byte, 5) {
	    self.byte = 0b1100_1111;
	    self.byte ^= self.group_direction & self.group_action;
	    interrupt_handler.request_interrupt(Interrupt::Joypad);
	} else if !is_bit_set(received_byte, 5) {
	    // Poll action buttons
	    self.byte = 0b1101_1111;
	    self.byte ^= self.group_action;
	    interrupt_handler.request_interrupt(Interrupt::Joypad);
	} else if !is_bit_set(received_byte, 4){
	    // Poll direction buttons
	    self.byte = 0b1110_1111;
	    self.byte ^= self.group_direction;
	    interrupt_handler.request_interrupt(Interrupt::Joypad);
	} else {
	    self.byte = 0xFF;
	}
    }

    pub fn read_joypad(&mut self) -> u8 {
	let byte = self.byte;
	self.byte = 0xFF;
	byte
    }
}
