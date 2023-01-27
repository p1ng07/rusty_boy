#[derive(Default)]
pub struct Serial {
    pub serial_data_transfer: u8,
    pub serial_data_control: u8
}

impl Serial {
    pub fn write_to_transfer(&mut self, data: u8){
	println!("a{}", data as char)
    }
}
