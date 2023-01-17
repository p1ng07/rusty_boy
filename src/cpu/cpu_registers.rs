pub enum Regs{
    A,B,C,D,E,F,H,L,Af,Bc,De,Hl
}

#[derive(Default)]
pub struct CpuRegisters {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
}

impl CpuRegisters {
    pub fn get_8bit_reg(&self, reg: Regs) -> u8{
	match reg {
	    Regs::A => self.a,
	    Regs::B => self.b,
	    Regs::C => self.c,
	    Regs::D => self.d,
	    Regs::E => self.e,
	    Regs::H => self.h,
	    Regs::L => self.l,
	    _ => panic!("function: get_8bit_reg is being used to try to get a 16-bit cpu register")
	}
    }

    pub fn get_16bit_reg(&self, reg: Regs) -> u16{
	match reg {
	    Regs::Af => (self.a as u16) << 8 | self.f as u16,
	    Regs::Bc => (self.b as u16) << 8 | self.c as u16,
	    Regs::De => (self.d as u16) << 8 | self.e as u16,
	    Regs::Hl => (self.h as u16) << 8 | self.l as u16,
	    _ => panic!("function: get_16bit_reg is being used to try to get a 8-bit cpu register")
	}
    }

    pub fn set_16bit_reg(&mut self, n: &u16, reg: Regs){
	match reg {
	    Regs::Af =>{
		self.a = n.to_be_bytes()[0];
		self.f = n.to_be_bytes()[1];
	    },
	    Regs::Bc => {
		self.b = n.to_be_bytes()[0];
		self.c = n.to_be_bytes()[1];
	    },
	    Regs::De => {
		self.d = n.to_be_bytes()[0];
		self.e = n.to_be_bytes()[1];
	    },
	    Regs::Hl => {
		self.h = n.to_be_bytes()[0];
		self.l = n.to_be_bytes()[1];
	    }
	    _ => panic!("function: get_16bit_reg is being used to try to get a 8-bit cpu register")
	};
    }
    pub fn set_8bit_reg(&mut self, n: &u8, reg:Regs){
	match reg {
	    Regs::A => self.a = *n,
	    Regs::B => self.b = *n,
	    Regs::C => self.c = *n,
	    Regs::D => self.d = *n,
	    Regs::E => self.e = *n,
	    Regs::H => self.h = *n,
	    Regs::L => self.l = *n,
	    _ => panic!("function: get_8bit_reg is being used to try to set a 16-bit cpu register")
	};
    }
}
