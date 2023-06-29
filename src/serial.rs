use std::io::Write;

#[derive(Default)]
pub struct Serial {
    pub serial_data_transfer: u8,
    pub serial_data_control: u8,

    pub current_word: String,
}

impl Serial {
    pub fn write_to_transfer(&mut self, data: u8) {
    }
}
