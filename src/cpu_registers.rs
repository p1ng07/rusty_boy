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

#[allow(dead_code)]
impl CpuRegisters {
    pub fn get_af(&self) -> u16 {
        u16::from(self.a) << 8 | u16::from(self.f)
    }

    pub fn get_bc(&self) -> u16 {
        u16::from(self.b) << 8 | u16::from(self.c)
    }

    pub fn get_de(&self) -> u16 {
        u16::from(self.d) << 8 | u16::from(self.e)
    }

    pub fn get_hl(&self) -> u16 {
        u16::from(self.h) << 8 | u16::from(self.l)
    }

    pub fn set_af(&mut self, n: u16) {
        self.a = n.to_be_bytes()[0];
        self.f = n.to_be_bytes()[1];
    }
    pub fn set_bc(&mut self, n: u16) {
        self.b = n.to_be_bytes()[0];
        self.c = n.to_be_bytes()[1];
    }
    pub fn set_de(&mut self, n: u16) {
        self.d = n.to_be_bytes()[0];
        self.e = n.to_be_bytes()[1];
    }
    pub fn set_hl(&mut self, n: u16) {
        self.h = n.to_be_bytes()[0];
        self.l = n.to_be_bytes()[1];
    }

    pub(crate) fn rlca(&mut self) {
	let high_bit = (self.a & 0x80) >> 7;
	self.set_carry_flag(high_bit > 0);
	self.set_was_prev_instr_sub(false);
	self.set_half_carry(false);
	self.set_zero_flag(false);
	self.a <<= 1;
	self.a |= high_bit;
    }

    pub(crate) fn rrca(&mut self) {
	let low_bit = self.a & 0x1;
	self.set_carry_flag(low_bit > 0);
	self.set_was_prev_instr_sub(false);
	self.set_half_carry(false);
	self.set_zero_flag(false);
	self.a >>= 1;
	self.a |= low_bit << 7;
    }
    
    pub(crate) fn and_u8(&mut self, reg: u8) {
	self.a &= reg;
        self.set_zero_flag(self.a == 0);
        self.set_was_prev_instr_sub(false);
        self.set_half_carry(true);
        self.set_carry_flag(false);
    }

    pub(crate) fn or_u8(&mut self, c: u8) {
	self.a |= c;
	self.set_zero_flag(self.a == 0);
	self.set_was_prev_instr_sub(false);
	self.set_half_carry(false);
	self.set_carry_flag(false);
    }

    pub(crate) fn xor_u8(&mut self, c: u8){
	self.a ^= c;
	self.set_zero_flag(self.a == 0);
	self.set_was_prev_instr_sub(false);
	self.set_half_carry(false);
	self.set_carry_flag(false);
    }

    pub(crate) fn dec_u8_reg(&mut self, reg: u8) -> u8 {
        let new_reg = reg.wrapping_sub(1);
        self.set_zero_flag(new_reg == 0);
        self.set_half_carry((reg & 0x0F) == 0);
        self.set_was_prev_instr_sub(true);
        new_reg
    }

    pub(crate) fn dec_bc(&mut self) {
	let new = self.get_bc().wrapping_sub(1);
	self.set_bc(new);
    }
    pub(crate) fn dec_de(&mut self) {
	let new = self.get_de().wrapping_sub(1);
	self.set_de(new);
    }
    pub(crate) fn dec_hl(&mut self) {
	let new = self.get_hl().wrapping_sub(1);
	self.set_hl(new);
    }

    pub(crate) fn inc_u8_reg(&mut self, reg: u8) -> u8 {
        self.set_half_carry((reg & 0x0F) as u16 + 1 > 0x0F);
        let new_reg = reg.wrapping_add(1);
        self.set_zero_flag(new_reg == 0);
        self.set_was_prev_instr_sub(false);
        new_reg
    }

    // Generic implementation for CP A, x opcode group
    pub(crate) fn cp(&mut self, value: u8) {
        let result = self.a.wrapping_sub(value);
        self.set_zero_flag(result == 0);
        self.set_was_prev_instr_sub(true);
        self.set_carry_flag(((self.a ^ value ^ result) & 0x10) > 0);
    }

    pub(crate) fn add_u8(&mut self, n: u8) {
	self.set_carry_flag(self.a > self.a.wrapping_add(n));
	self.set_half_carry((self.a & 0x0F) + (n & 0x0F) > 0x0F);
	self.a = self.a.wrapping_add(n);
	self.set_zero_flag(self.a == 0);
	self.set_was_prev_instr_sub(false);
    }

    pub(crate) fn add_hl_u16(&mut self, n: u16) {
	let new_reg = self.get_hl().wrapping_add(n);
	self.set_zero_flag(new_reg == 0);
	self.set_carry_flag(self.get_hl() > self.get_hl());
	self.set_half_carry((self.get_hl() & 0x00FF) + (n & 0x00FF) > 0xFF);
	self.set_was_prev_instr_sub(false);
	self.set_hl(new_reg);
    }

    pub(crate) fn sub_u8_reg(&mut self, reg: u8) -> u8 {
        self.set_carry_flag(self.a < reg);
        let result = self.a.wrapping_sub(reg);
        self.set_zero_flag(self.a == 0);
        self.set_was_prev_instr_sub(true);
        self.set_half_carry(((self.a ^ reg ^ result) & 0x10) > 0);
        result
    }


    pub fn set_zero_flag(&mut self, is_high: bool) {
        if is_high {
            self.f |= 0b1000_0000;
        } else {
            self.f &= 0b0111_0000;
        }
    }

    pub fn set_carry_flag(&mut self, is_high: bool) {
        if is_high {
            self.f |= 0b0001_0000;
        } else {
            self.f &= 0b1110_0000;
        }
    }

    pub fn set_was_prev_instr_sub(&mut self, is_high: bool) {
        if is_high {
            self.f |= 0b0100_0000;
        } else {
            self.f &= 0b1011_0000;
        }
    }

    pub fn set_half_carry(&mut self, is_high: bool) {
        if is_high {
            self.f |= 0b0010_0000;
        } else {
            self.f &= 0b1101_0000;
        }
    }

    pub fn is_lower_carry_high(&self) -> bool {
        self.f >> 5 == 1
    }

    pub fn is_carry_flag_high(&self) -> bool {
        (self.f >> 4) == 1
    }

    pub fn is_zero_flag_high(&self) -> bool {
        (self.f >> 7) == 1
    }

    // Receives a u8 and uses the 4 lower bits of the u8 as the flags register
    pub(crate) fn set_flags(&mut self, flags: u8) {
        self.f = flags << 4;
    }

    pub(crate) fn unset_flags(&mut self) {
        self.f = 0;
    }

    pub(crate) fn cpl(&mut self){
	self.a ^= 0xFF;
	self.set_was_prev_instr_sub(true);
	self.set_half_carry(true);
    }
}
