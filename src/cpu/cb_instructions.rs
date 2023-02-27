use super::Cpu;

impl Cpu {
    pub(crate) fn execute_cb(&mut self) {
        let instruction = self.fetch_byte();

        // Register used in the operation depends
        let register_to_use = match ((instruction >> 4) & 0xF) % 8 {
            0 => self.registers.b,
            1 => self.registers.c,
            2 => self.registers.d,
            3 => self.registers.e,
            4 => self.registers.h,
            5 => self.registers.l,
            6 => {
                self.tick();
                self.bus.fetch_byte(self.registers.get_hl(), &self.state)
            }
            7 => self.registers.a,
            _ => 0, // This is impossible to reach because we are comparing the remainder of a division by 8
        };

        match instruction {
            0x00 => self.registers.b = self.rlc(self.registers.b),
            0x01 => self.registers.c = self.rlc(self.registers.c),
            0x02 => self.registers.d = self.rlc(self.registers.d),
            0x03 => self.registers.e = self.rlc(self.registers.e),
            0x04 => self.registers.h = self.rlc(self.registers.h),
            0x05 => self.registers.l = self.rlc(self.registers.l),
            0x06 => {
                let _byte = self.bus.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.rlc(self.registers.c);
                self.bus
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
                let _byte = self.bus.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.rrc(self.registers.c);
                self.bus
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
                let _byte = self.bus.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.rl(self.registers.c);
                self.bus
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
                let _byte = self.bus.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.rr(self.registers.c);
                self.bus
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
                let _byte = self.bus.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.sla(self.registers.c);
                self.bus
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
                let _byte = self.bus.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.sra(self.registers.c);
                self.bus
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
                let _byte = self.bus.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.swap(self.registers.c);
                self.bus
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
                let _byte = self.bus.fetch_byte(self.registers.get_hl(), &self.state);
                let byte = self.srl(self.registers.c);
                self.bus
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x3F => self.registers.a = self.srl(self.registers.a),
            0x40..=0x47 => self.test_bit(register_to_use, 0),
            0x48..=0x4F => self.test_bit(register_to_use, 1),
            0x50..=0x57 => self.test_bit(register_to_use, 2),
            0x58..=0x5F => self.test_bit(register_to_use, 3),
            0x60..=0x67 => self.test_bit(register_to_use, 4),
            0x68..=0x6F => self.test_bit(register_to_use, 5),
            0x70..=0x77 => self.test_bit(register_to_use, 6),
            0x78..=0x7F => self.test_bit(register_to_use, 7),
            0x80 => self.registers.b &= !(1 << 0),
            0x81 => self.registers.c &= !(1 << 0),
            0x82 => self.registers.d &= !(1 << 0),
            0x83 => self.registers.e &= !(1 << 0),
            0x84 => self.registers.h &= !(1 << 0),
            0x85 => self.registers.l &= !(1 << 0),
            0x86 => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) & !(1 << 0),
                    &mut self.state,
                );
            }
            0x87 => self.registers.a &= !(1 << 1),
            0x88 => self.registers.b &= !(1 << 1),
            0x89 => self.registers.c &= !(1 << 1),
            0x8A => self.registers.d &= !(1 << 1),
            0x8B => self.registers.e &= !(1 << 1),
            0x8C => self.registers.h &= !(1 << 1),
            0x8D => self.registers.l &= !(1 << 1),
            0x8E => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) & !(1 << 1),
                    &mut self.state,
                );
            }
            0x8F => self.registers.a &= !(1 << 1),
            0x90 => self.registers.b &= !(1 << 2),
            0x91 => self.registers.c &= !(1 << 2),
            0x92 => self.registers.d &= !(1 << 2),
            0x93 => self.registers.e &= !(1 << 2),
            0x94 => self.registers.h &= !(1 << 2),
            0x95 => self.registers.l &= !(1 << 2),
            0x96 => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) & !(1 << 2),
                    &mut self.state,
                );
            }
            0x97 => self.registers.a &= !(1 << 3),
            0x98 => self.registers.b &= !(1 << 3),
            0x99 => self.registers.c &= !(1 << 3),
            0x9A => self.registers.d &= !(1 << 3),
            0x9B => self.registers.e &= !(1 << 3),
            0x9C => self.registers.h &= !(1 << 3),
            0x9D => self.registers.l &= !(1 << 3),
            0x9E => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) & !(1 << 3),
                    &mut self.state,
                );
            }
            0x9F => self.registers.a &= !(1 << 3),
            0xA0 => self.registers.b &= !(1 << 4),
            0xA1 => self.registers.c &= !(1 << 4),
            0xA2 => self.registers.d &= !(1 << 4),
            0xA3 => self.registers.e &= !(1 << 4),
            0xA4 => self.registers.h &= !(1 << 4),
            0xA5 => self.registers.l &= !(1 << 4),
            0xA6 => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) & !(1 << 4),
                    &mut self.state,
                );
            }
            0xA7 => self.registers.a &= !(1 << 4),
            0xA8 => self.registers.b &= !(1 << 5),
            0xA9 => self.registers.c &= !(1 << 5),
            0xAA => self.registers.d &= !(1 << 5),
            0xAB => self.registers.e &= !(1 << 5),
            0xAC => self.registers.h &= !(1 << 5),
            0xAD => self.registers.l &= !(1 << 5),
            0xAE => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) & !(1 << 5),
                    &mut self.state,
                );
            }
            0xAF => self.registers.a &= !(1 << 5),
            0xB0 => self.registers.b &= !(1 << 6),
            0xB1 => self.registers.c &= !(1 << 6),
            0xB2 => self.registers.d &= !(1 << 6),
            0xB3 => self.registers.e &= !(1 << 6),
            0xB4 => self.registers.h &= !(1 << 6),
            0xB5 => self.registers.l &= !(1 << 6),
            0xB6 => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) & !(1 << 6),
                    &mut self.state,
                );
            }
            0xB7 => self.registers.a &= !(1 << 6),
            0xB8 => self.registers.b &= !(1 << 7),
            0xB9 => self.registers.c &= !(1 << 7),
            0xBA => self.registers.d &= !(1 << 7),
            0xBB => self.registers.e &= !(1 << 7),
            0xBC => self.registers.h &= !(1 << 7),
            0xBD => self.registers.l &= !(1 << 7),
            0xBE => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) & !(1 << 7),
                    &mut self.state,
                );
            }
            0xBF => self.registers.a &= !(1 << 7),
            0xC0 => self.registers.b |= 1 << 0,
            0xC1 => self.registers.c |= 1 << 0,
            0xC2 => self.registers.d |= 1 << 0,
            0xC3 => self.registers.e |= 1 << 0,
            0xC4 => self.registers.h |= 1 << 0,
            0xC5 => self.registers.l |= 1 << 0,
            0xC6 => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) | (1 << 0),
                    &mut self.state,
                );
            }
            0xC7 => self.registers.a |= 1 << 1,
            0xC8 => self.registers.b |= 1 << 1,
            0xC9 => self.registers.c |= 1 << 1,
            0xCA => self.registers.d |= 1 << 1,
            0xCB => self.registers.e |= 1 << 1,
            0xCC => self.registers.h |= 1 << 1,
            0xCD => self.registers.l |= 1 << 1,
            0xCE => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) | (1 << 1),
                    &mut self.state,
                );
            }
            0xCF => self.registers.a |= 1 << 1,
            0xD0 => self.registers.b |= 1 << 2,
            0xD1 => self.registers.c |= 1 << 2,
            0xD2 => self.registers.d |= 1 << 2,
            0xD3 => self.registers.e |= 1 << 2,
            0xD4 => self.registers.h |= 1 << 2,
            0xD5 => self.registers.l |= 1 << 2,
            0xD6 => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) | (1 << 2),
                    &mut self.state,
                );
            }
            0xD7 => self.registers.a |= 1 << 3,
            0xD8 => self.registers.b |= 1 << 3,
            0xD9 => self.registers.c |= 1 << 3,
            0xDA => self.registers.d |= 1 << 3,
            0xDB => self.registers.e |= 1 << 3,
            0xDC => self.registers.h |= 1 << 3,
            0xDD => self.registers.l |= 1 << 3,
            0xDE => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) | (1 << 3),
                    &mut self.state,
                );
            }
            0xDF => self.registers.a |= 1 << 3,
            0xE0 => self.registers.b |= 1 << 4,
            0xE1 => self.registers.c |= 1 << 4,
            0xE2 => self.registers.d |= 1 << 4,
            0xE3 => self.registers.e |= 1 << 4,
            0xE4 => self.registers.h |= 1 << 4,
            0xE5 => self.registers.l |= 1 << 4,
            0xE6 => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) | (1 << 4),
                    &mut self.state,
                );
            }
            0xE7 => self.registers.a |= 1 << 4,
            0xE8 => self.registers.b |= 1 << 5,
            0xE9 => self.registers.c |= 1 << 5,
            0xEA => self.registers.d |= 1 << 5,
            0xEB => self.registers.e |= 1 << 5,
            0xEC => self.registers.h |= 1 << 5,
            0xED => self.registers.l |= 1 << 5,
            0xEE => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) | (1 << 5),
                    &mut self.state,
                );
            }
            0xEF => self.registers.a |= 1 << 5,
            0xF0 => self.registers.b |= 1 << 6,
            0xF1 => self.registers.c |= 1 << 6,
            0xF2 => self.registers.d |= 1 << 6,
            0xF3 => self.registers.e |= 1 << 6,
            0xF4 => self.registers.h |= 1 << 6,
            0xF5 => self.registers.l |= 1 << 6,
            0xF6 => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) | (1 << 6),
                    &mut self.state,
                );
            }
            0xF7 => self.registers.a |= 1 << 6,
            0xF8 => self.registers.b |= 1 << 7,
            0xF9 => self.registers.c |= 1 << 7,
            0xFA => self.registers.d |= 1 << 7,
            0xFB => self.registers.e |= 1 << 7,
            0xFC => self.registers.h |= 1 << 7,
            0xFD => self.registers.l |= 1 << 7,
            0xFE => {
                self.tick();
                self.bus.write_byte(
                    self.registers.get_hl(),
                    self.bus.fetch_byte(self.registers.get_hl(), &self.state) | (1 << 7),
                    &mut self.state,
                );
            }
            0xFF => self.registers.a |= 1 << 7,
        };
    }

    fn rr(&mut self, mut reg: u8) -> u8 {
        let old_carry = self.registers.is_carry_flag_high();
        let new_carry = reg & 0x1 > 0;
        self.registers.set_carry_flag(new_carry);
        reg >>= 1;
        reg |= (old_carry as u8) << 7;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn rlc(&mut self, mut reg: u8) -> u8 {
        let carry = reg & 0x80 > 0;
        self.registers.set_carry_flag(carry);
        reg = (reg << 1) | carry as u8;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn rrc(&mut self, mut reg: u8) -> u8 {
        let carry = reg & 0x1 > 0;
        self.registers.set_carry_flag(carry);
        reg = (reg >> 1) | (carry as u8) << 7;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn rl(&mut self, mut reg: u8) -> u8 {
        let carry = reg & 0x80 > 0;
        self.registers.set_carry_flag(carry);
        reg <<= 1;
        reg |= carry as u8;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn sla(&mut self, mut reg: u8) -> u8 {
        let carry = reg & 0x80 > 0;
        self.registers.set_carry_flag(carry);
        reg <<= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn sra(&mut self, mut reg: u8) -> u8 {
        let carry = reg & 0x1 > 0;
        self.registers.set_carry_flag(carry);
        reg >>= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn srl(&mut self, mut reg: u8) -> u8 {
        self.registers.set_carry_flag(reg & 0x1 > 0);
        reg >>= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn swap(&mut self, reg: u8) -> u8 {
        self.registers.set_zero_flag(reg == 0);
        reg.swap_bytes()
    }

    fn test_bit(&mut self, reg: u8, bit_index: u8) {
        self.registers.set_zero_flag((reg >> bit_index) & 0x1 == 0);
        self.registers.set_n_flag(false);
        self.registers.set_half_carry_flag(false);
    }
}
