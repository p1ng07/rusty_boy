use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AudioController {
    pub nr50: u8
}

impl AudioController {
    pub fn new() -> Self{
	Self{
	    nr50: 0
	}
    }
}
