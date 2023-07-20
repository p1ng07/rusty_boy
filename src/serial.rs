use crate::{interrupt_handler::{InterruptHandler, Interrupt}, cpu::is_bit_set};

pub struct Serial {
    pub serial_data_transfer: u8,
    pub serial_data_control: u8,

    pub current_word: String,
}

impl Serial {

    pub fn new() -> Self {
	Self {
	    serial_data_transfer: 0,
	    serial_data_control: 0,
	    current_word: String::new()
	}
    }
    pub fn write_to_transfer(&mut self, _interrupt_handler: &mut InterruptHandler, _data: u8) {
    }

    pub(crate) fn write_to_control(&mut self, received_byte: u8, interrupt_handler: &mut InterruptHandler) {
	if is_bit_set(received_byte, 7) && !is_bit_set(7,self.serial_data_control) {
	    self.serial_data_transfer = 0xFF;
	    interrupt_handler.request_interrupt(Interrupt::Serial);
	    self.serial_data_control = 0x01;
	}
    }
}
