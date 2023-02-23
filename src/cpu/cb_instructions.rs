use super::Cpu;

impl Cpu {
    pub(crate) fn execute_cb(&mut self) {
        let instruction = self.fetch_byte();

        // Print state of emulator to logger
	self.log_to_file(first_byte);

        match instruction {
            0x00 => self.registers.b = self.rlc(self.registers.b),
            0x01 => self.registers.c = self.rlc(self.registers.c),
            0x02 => self.registers.d = self.rlc(self.registers.d),
            0x03 => self.registers.e = self.rlc(self.registers.e),
            0x04 => self.registers.h = self.rlc(self.registers.h),
            0x05 => self.registers.l = self.rlc(self.registers.l),
            0x06 => {
                let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.rlc(self.registers.c);
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x07 => self.registers.a = self.rlc(self.registers.a),
            0x08 => self.registers.b = self.rrc(self.registers.b),
            0x09 => self.registers.c = self.rrc(self.registers.c),
            0x0A => self.registers.d = self.rrc(self.registers.d),
            0x0B => self.registers.e = self.rrc(self.registers.e),
            0x0C => self.registers.h = self.rrc(self.registers.h),
            0x0D => self.registers.l = self.rrc(self.registers.l),
            0x0E => {
                let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.rrc(self.registers.c);
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x0F => self.registers.a = self.rrc(self.registers.a),
            0x10 => self.registers.b = self.rl(self.registers.b),
            0x11 => self.registers.c = self.rl(self.registers.c),
            0x12 => self.registers.d = self.rl(self.registers.d),
            0x13 => self.registers.e = self.rl(self.registers.e),
            0x14 => self.registers.h = self.rl(self.registers.h),
            0x15 => self.registers.l = self.rl(self.registers.l),
            0x16 => {
                let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.rl(self.registers.c);
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x17 => self.registers.a = self.rr(self.registers.a),
            0x18 => self.registers.b = self.rr(self.registers.b),
            0x19 => self.registers.c = self.rr(self.registers.c),
            0x1A => self.registers.d = self.rr(self.registers.d),
            0x1B => self.registers.e = self.rr(self.registers.e),
            0x1C => self.registers.h = self.rr(self.registers.h),
            0x1D => self.registers.l = self.rr(self.registers.l),
            0x1E => {
                let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.rr(self.registers.c);
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x1F => self.registers.a = self.sla(self.registers.a),
            0x20 => self.registers.b = self.sla(self.registers.b),
            0x21 => self.registers.c = self.sla(self.registers.c),
            0x22 => self.registers.d = self.sla(self.registers.d),
            0x23 => self.registers.e = self.sla(self.registers.e),
            0x24 => self.registers.h = self.sla(self.registers.h),
            0x25 => self.registers.l = self.sla(self.registers.l),
            0x26 => {
                let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.sla(self.registers.c);
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x27 => self.registers.a = self.sla(self.registers.a),
            0x28 => self.registers.b = self.sra(self.registers.b),
            0x29 => self.registers.c = self.sra(self.registers.c),
            0x2A => self.registers.d = self.sra(self.registers.d),
            0x2B => self.registers.e = self.sra(self.registers.e),
            0x2C => self.registers.h = self.sra(self.registers.h),
            0x2D => self.registers.l = self.sra(self.registers.l),
            0x2E => {
                let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.sra(self.registers.c);
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x2F => self.registers.a = self.swap(self.registers.a),
            0x30 => self.registers.b = self.swap(self.registers.b),
            0x31 => self.registers.c = self.swap(self.registers.c),
            0x32 => self.registers.d = self.swap(self.registers.d),
            0x33 => self.registers.e = self.swap(self.registers.e),
            0x34 => self.registers.h = self.swap(self.registers.h),
            0x35 => self.registers.l = self.swap(self.registers.l),
            0x36 => {
                let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.swap(self.registers.c);
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x37 => self.registers.a = self.swap(self.registers.a),
            0x38 => self.registers.b = self.srl(self.registers.b),
            0x39 => self.registers.c = self.srl(self.registers.c),
            0x3A => self.registers.d = self.srl(self.registers.d),
            0x3B => self.registers.e = self.srl(self.registers.e),
            0x3C => self.registers.h = self.srl(self.registers.h),
            0x3D => self.registers.l = self.srl(self.registers.l),
            0x3E => {
                let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.srl(self.registers.c);
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x3F => self.registers.a = self.srl(self.registers.a),
            0x7C => {
                self.registers.set_zero_flag(self.registers.h < 128);
            }
            _ => panic!(
                "CB prefixed instruction {:X?} was not implemented",
                instruction.to_be_bytes()
            ),
        };
    }

    fn rr(&mut self, mut reg: u8) -> u8 {
        self.registers.set_carry_flag(reg & 0x80 > 0);
        reg >>= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_was_prev_instr_sub(false);
        reg
    }

    fn rlc(&mut self, mut reg: u8) -> u8 {
	let carry = reg & 0x80 > 0;
        self.registers.set_carry_flag(carry);
	reg = (reg << 1) | carry as u8;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_was_prev_instr_sub(false);
	reg
    }

    fn rrc(&mut self, mut reg: u8) -> u8 {
	let carry = reg & 0x1 > 0;
        self.registers.set_carry_flag(carry);
	reg = (reg >> 1) | (carry as u8) << 8;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_was_prev_instr_sub(false);
	reg
    }

    fn rl(&mut self, mut reg: u8) -> u8 {
        self.registers.set_carry_flag(reg & 0x80 > 0);
        reg <<= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_was_prev_instr_sub(false);
        reg
    }

    fn sla(&mut self, mut reg: u8) -> u8 {
	let carry = reg & 0x80 > 0;
        self.registers.set_carry_flag(carry);
	reg <<= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_was_prev_instr_sub(false);
	reg
    }

    fn sra(&mut self, mut reg: u8) -> u8 {
	let carry = reg & 0x1 > 0;
        self.registers.set_carry_flag(carry);
	reg >>= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_was_prev_instr_sub(false);
	reg
    }

    fn srl(&mut self, mut reg: u8) -> u8 {
	self.registers.set_carry_flag(reg & 0x80 > 0);
	reg >>= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_was_prev_instr_sub(false);
	reg
    }

    fn swap(&mut self, reg: u8) -> u8 {
	self.registers.set_zero_flag(reg == 0);
	reg.swap_bytes()
    }


    fn test_bit(&mut self, reg: u8, bit_index: u8){
	self.registers.set_zero_flag((reg >> bit_index) & 0x1 == 0);
	self.registers.set_was_prev_instr_sub(false);
	self.registers.set_half_carry_flag(false);
    }
}

