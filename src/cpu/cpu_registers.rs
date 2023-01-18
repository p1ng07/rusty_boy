#[derive(Default)]
pub struct CpuRegisters {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
}

impl CpuRegisters {
    pub fn get_af(&self) -> u16 { 
	(self.a as u16) << 8 | self.f as u16
    }

    pub fn get_bc(&self) -> u16 {
	(self.b as u16) << 8 | self.c as u16
    }

    pub fn get_de(&self) -> u16 {
	(self.d as u16) << 8 | self.e as u16
    }

    pub fn get_hl(&self) -> u16 {
	(self.h as u16) << 8 | self.l as u16
    }

    pub fn set_af(&mut self, n: u16){ 
	self.a = n.to_be_bytes()[0];
	self.f = n.to_be_bytes()[1];
    }
    pub fn set_bc(&mut self, n: u16){ 
	self.b = n.to_be_bytes()[0];
	self.c = n.to_be_bytes()[1];
    }
    pub fn set_de(&mut self, n: u16){ 
	self.d = n.to_be_bytes()[0];
	self.e = n.to_be_bytes()[1];
    }
    pub fn set_hl(&mut self, n: u16){ 
	self.h = n.to_be_bytes()[0];
	self.l = n.to_be_bytes()[1];
    }
}
