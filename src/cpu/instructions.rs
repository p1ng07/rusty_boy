use super::Cpu;

impl Cpu {
    // Execute the instruction given and return the number of t-cycles it took to run it
    pub(crate) fn execute(&mut self, first_byte: u8) {
        // Print state of emulator to logger
        self.log_to_file(first_byte);

        match first_byte {
            0x00 => {
                self.tick();
            }
            0x01 => {
                let n = self.fetch_word();
                self.registers.set_bc(n);
                self.tick();
            }
            0x02 => {
                self.mmu
                    .write_byte(self.registers.get_bc(), self.registers.a, &mut self.state);
                self.tick();
            }
            0x03 => {
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_add(1));
                self.tick();
            }
            0x04 => self.registers.b = self.registers.inc_u8_reg(self.registers.b),
            0x05 => self.registers.b = self.registers.dec_u8_reg(self.registers.b),
            0x06 => self.registers.b = self.fetch_byte(),
            0x07 => self.registers.rlca(),
            0x08 => {
                let word = self.fetch_word();
                self.mmu.write_word(word, self.sp, &mut self.state);
                self.tick();
                self.tick();
            }
            0x09 => {
                self.registers.add_to_hl_u16(self.registers.get_bc());
                self.tick();
            }
            0x0A => self.registers.a = self.mmu.fetch_byte(self.registers.get_bc(), &self.state),
            0x0B => {
                self.registers.dec_bc();
                self.tick();
            }
            0x0C => self.registers.c = self.registers.inc_u8_reg(self.registers.c),
            0x0D => self.registers.c = self.registers.dec_u8_reg(self.registers.c),
            0x0E => self.registers.c = self.fetch_byte(),
            0x0F => {
                let least_bit = self.registers.a & 0x1;
                self.registers.set_flags(0);
                self.registers.set_carry_flag(least_bit > 0);
                self.registers.a = (self.registers.a >> 1) | least_bit << 7;
            }
            0x11 => {
                let word = self.fetch_word();
                self.registers.set_de(word);
            }
            0x12 => {
                self.mmu
                    .write_byte(self.registers.get_de(), self.registers.a, &mut self.state);
                self.tick();
            }
            0x13 => {
                // INC DE
                let new_de = self.registers.get_de().wrapping_add(1);
                self.registers.set_de(new_de);
                self.tick();
            }
            0x14 => self.registers.d = self.registers.inc_u8_reg(self.registers.d),
            0x15 => self.registers.d = self.registers.dec_u8_reg(self.registers.d),
            0x16 => self.registers.d = self.fetch_byte(),
            0x17 => {
                // RLA
                let old_carry = if self.registers.is_carry_flag_high() {
                    1
                } else {
                    0
                };
                self.registers.set_flags(if self.registers.a & 0x80 > 0 {
                    0b1000
                } else {
                    0b0000
                });
                self.registers.a <<= 1;
                self.registers.a |= old_carry;
            }
            0x18 => {
                let offset = self.fetch_byte() as i8;
		self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                self.tick();
            }
            0x19 => {
                self.registers.add_to_hl_u16(self.registers.get_de());
                self.tick();
            }
            0x1A => {
                self.registers.a = self.mmu.fetch_byte(self.registers.get_de(), &self.state);
                self.tick();
            }
            0x1B => {
                self.registers.dec_de();
                self.tick();
            }
            0x1C => self.registers.e = self.registers.inc_u8_reg(self.registers.e),
            0x1D => self.registers.e = self.registers.dec_u8_reg(self.registers.e),
            0x1E => self.registers.e = self.fetch_byte(),
            0x1F => {
		// Rotate right through carry
                let new_carry = self.registers.a & 0x1 == 1;
		let old_carry = self.registers.is_carry_flag_high();
		self.registers.a >>= 1;
		self.registers.a |= (old_carry as u8) << 7;
		self.registers.set_carry_flag(new_carry);
		self.registers.set_was_prev_instr_sub(false);
		self.registers.set_half_carry_flag(false);
		self.registers.set_zero_flag(false);
            }
            0x20 => {
                if !self.registers.is_zero_flag_high() {
		    let offset = self.fetch_byte() as i8;
                    self.tick();
		    self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                } else {
		    self.pc += 1;
		}
            }
            0x21 => {
                let word = self.fetch_word();
                self.registers.set_hl(word);
            }
            0x22 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.a, &mut self.state);
                self.registers.set_hl(self.registers.get_hl() + 1);
                self.tick();
            }
            0x23 => {
                let new_hl = self.registers.get_hl().wrapping_add(1);
                self.registers.set_hl(new_hl);
                self.tick();
            }
            0x24 => self.registers.h = self.registers.inc_u8_reg(self.registers.h),
            0x25 => self.registers.h = self.registers.dec_u8_reg(self.registers.h),
            0x26 => self.registers.h = self.fetch_byte(),
            0x28 => {
                if self.registers.is_zero_flag_high() {
		    let offset = self.fetch_byte() as i8;
                    self.tick();
		    self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                } else {
		    self.pc += 1;
		}
            }
            0x29 => {
                self.registers.add_to_hl_u16(self.registers.get_hl());
                self.tick();
            }
            0x2A => {
                let add = self.registers.get_hl();
                self.registers.a = self.mmu.fetch_byte(add, &self.state);
                let new_hl = add.wrapping_add(1);
                self.registers.set_hl(new_hl);
            }
            0x2B => {
                self.registers.dec_hl();
                self.tick();
            }
            0x2C => self.registers.l = self.registers.inc_u8_reg(self.registers.l),
            0x2D => self.registers.l = self.registers.dec_u8_reg(self.registers.l),
            0x2E => self.registers.l = self.fetch_byte(),
            0x2F => {
                self.registers.cpl();
            }
            0x30 => {
                if !self.registers.is_carry_flag_high() {
		    let offset = self.fetch_byte() as i8;
                    self.tick();
		    self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                } else {
		    self.pc += 1;
		}
            }
            0x31 => self.sp = self.fetch_word(),
            0x32 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.a, &mut self.state);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                self.tick();
            }
            0x33 => {
                self.sp = self.sp.wrapping_add(1);
                self.tick();
            }
            0x34 => {
                let value = self
                    .mmu
                    .fetch_byte(self.registers.get_hl(), &self.state)
                    .wrapping_add(1);
                self.mmu
                    .write_byte(self.registers.get_hl(), value, &mut self.state);
                self.tick();
                self.tick();
            }
            0x35 => {
                let value = self
                    .mmu
                    .fetch_byte(self.registers.get_hl(), &self.state)
                    .wrapping_sub(1);
                self.mmu
                    .write_byte(self.registers.get_hl(), value, &mut self.state);
                self.tick();
                self.tick();
            }
            0x36 => {
                let byte = self.fetch_byte();
                self.mmu
                    .write_byte(self.registers.get_hl(), byte, &mut self.state);
                self.tick();
            }
            0x37 => self.registers.set_carry_flag(true),
            0x38 => {
                if self.registers.is_carry_flag_high() {
		    let offset = self.fetch_byte() as i8;
                    self.tick();
		    self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                } else {
		    self.pc += 1;
		}
            }
            0x39 => {
                self.registers.add_to_hl_u16(self.sp);
                self.tick();
            }
            0x3A => {
                self.registers.a = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                self.tick();
            }
            0x3B => {
                self.sp = self.sp.wrapping_sub(1);
                self.tick();
            }
            0x3C => self.registers.a = self.registers.inc_u8_reg(self.registers.a),
            0x3D => self.registers.a = self.registers.dec_u8_reg(self.registers.a),
            0x3E => self.registers.a = self.fetch_byte(),
            0x3F => {
                self.registers.set_was_prev_instr_sub(false);
                self.registers.set_half_carry_flag(false);
                self.registers
                    .set_carry_flag(!self.registers.is_carry_flag_high());
            }
            0x40 => self.registers.b = self.registers.b,
            0x41 => self.registers.b = self.registers.c,
            0x42 => self.registers.b = self.registers.d,
            0x43 => self.registers.b = self.registers.e,
            0x44 => self.registers.b = self.registers.h,
            0x45 => self.registers.b = self.registers.l,
            0x46 => {
                self.registers.b = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                self.tick();
            }
            0x47 => self.registers.b = self.registers.a,
            0x48 => self.registers.c = self.registers.b,
            0x49 => self.registers.c = self.registers.c,
            0x4A => self.registers.c = self.registers.d,
            0x4B => self.registers.c = self.registers.e,
            0x4C => self.registers.c = self.registers.h,
            0x4E => {self.tick();self.registers.c = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);},
            0x4F => self.registers.c = self.registers.a,
            0x50 => self.registers.d = self.registers.b,
            0x51 => self.registers.d = self.registers.c,
            0x52 => self.registers.d = self.registers.d,
            0x53 => self.registers.d = self.registers.e,
            0x54 => self.registers.d = self.registers.h,
            0x55 => self.registers.d = self.registers.l,
            0x56 => {
                self.registers.d = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                self.tick();
            }
            0x57 => self.registers.d = self.registers.a,
            0x58 => self.registers.e = self.registers.b,
            0x59 => self.registers.e = self.registers.c,
            0x5A => self.registers.e = self.registers.e,
            0x5B => self.registers.e = self.registers.e,
            0x5C => self.registers.e = self.registers.h,
            0x5D => self.registers.e = self.registers.l,
            0x5E => {
                self.registers.e = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                self.tick();
            }
            0x5F => self.registers.e = self.registers.a,
            0x60 => self.registers.h = self.registers.b,
            0x61 => self.registers.h = self.registers.c,
            0x62 => self.registers.h = self.registers.d,
            0x63 => self.registers.h = self.registers.e,
            0x64 => self.registers.h = self.registers.h,
            0x65 => self.registers.h = self.registers.l,
            0x66 => {
                self.registers.h = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                self.tick();
            }
            0x67 => self.registers.h = self.registers.a,
            0x68 => self.registers.l = self.registers.b,
            0x69 => self.registers.l = self.registers.c,
            0x6A => self.registers.l = self.registers.d,
            0x6B => self.registers.l = self.registers.e,
            0x6C => self.registers.l = self.registers.h,
            0x6D => self.registers.l = self.registers.l,
            0x6E => {
                self.registers.l = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                self.tick();
            }
            0x6F => self.registers.l = self.registers.a,
            0x70 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.b, &mut self.state);
                self.tick();
            }
            0x71 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.c, &mut self.state);
                self.tick();
            }
            0x72 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.d, &mut self.state);
                self.tick();
            }
            0x73 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.e, &mut self.state);
                self.tick();
            }
            0x74 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.h, &mut self.state);
                self.tick();
            }
            0x75 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.l, &mut self.state);
                self.tick();
            }
            0x77 => {
                self.mmu
                    .write_byte(self.registers.get_hl(), self.registers.a, &mut self.state);
                self.tick();
            }
            0x78 => self.registers.a = self.registers.b,
            0x79 => self.registers.a = self.registers.c,
            0x7A => self.registers.a = self.registers.d,
            0x7B => self.registers.a = self.registers.e,
            0x7C => self.registers.a = self.registers.h,
            0x7D => self.registers.a = self.registers.l,
            0x7E => {
                self.registers.a = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
                self.tick();
            }
            0x7F => self.registers.a = self.registers.a,
            0x80 => self.registers.add_u8(self.registers.b),
            0x81 => self.registers.add_u8(self.registers.c),
            0x82 => self.registers.add_u8(self.registers.d),
            0x83 => self.registers.add_u8(self.registers.e),
            0x84 => self.registers.add_u8(self.registers.h),
            0x85 => self.registers.add_u8(self.registers.l),
            0x86 => {
                self.registers
                    .add_u8(self.mmu.fetch_byte(self.registers.get_hl(), &self.state));
                self.tick();
            }
            0x87 => self.registers.add_u8(self.registers.a),
            0x88 => self.registers.add_u8(
                self.registers
                    .b
                    .wrapping_add(self.registers.is_carry_flag_high() as u8),
            ),
            0x89 => {
                self.registers.add_u8(
                    self.registers
                        .c
                        .wrapping_add(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x8A => {
                self.registers.add_u8(
                    self.registers
                        .d
                        .wrapping_add(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x8B => {
                self.registers.add_u8(
                    self.registers
                        .e
                        .wrapping_add(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x8C => {
                self.registers.add_u8(
                    self.registers
                        .h
                        .wrapping_add(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x8D => {
                self.registers.add_u8(
                    self.registers
                        .l
                        .wrapping_add(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x8E => {
                self.registers.add_u8(
                    self.mmu
                        .fetch_byte(self.registers.get_hl(), &self.state)
                        .wrapping_add(self.registers.is_carry_flag_high() as u8),
                );
                self.tick();
            }
            0x8F => {
                self.registers.add_u8(
                    self.registers
                        .a
                        .wrapping_add(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x90 => self.registers.sub_u8(self.registers.b),
            0x91 => self.registers.sub_u8(self.registers.c),
            0x92 => self.registers.sub_u8(self.registers.d),
            0x93 => self.registers.sub_u8(self.registers.e),
            0x94 => self.registers.sub_u8(self.registers.h),
            0x95 => self.registers.sub_u8(self.registers.l),
            0x96 => {
                self.registers
                    .sub_u8(self.mmu.fetch_byte(self.registers.get_hl(), &self.state));
                self.tick();
            }
            0x97 => self.registers.sub_u8(self.registers.a),
            0x98 => {
                self.registers.sub_u8(
                    self.registers
                        .b
                        .wrapping_sub(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x99 => {
                self.registers.sub_u8(
                    self.registers
                        .c
                        .wrapping_sub(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x9A => {
                self.registers.sub_u8(
                    self.registers
                        .d
                        .wrapping_sub(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x9B => {
                self.registers.sub_u8(
                    self.registers
                        .e
                        .wrapping_sub(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x9C => {
                self.registers.sub_u8(
                    self.registers
                        .h
                        .wrapping_sub(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x9D => {
                self.registers.sub_u8(
                    self.registers
                        .l
                        .wrapping_sub(self.registers.is_carry_flag_high() as u8),
                );
            }
            0x9E => {
                self.registers.sub_u8(
                    self.mmu
                        .fetch_byte(self.registers.get_hl(), &self.state)
                        .wrapping_sub(self.registers.is_carry_flag_high() as u8),
                );
                self.tick();
            }
            0x9F => {
                self.registers.sub_u8(
                    self.registers
                        .a
                        .wrapping_sub(self.registers.is_carry_flag_high() as u8),
                );
            }
            0xA0 => self.registers.and_u8(self.registers.b),
            0xA1 => self.registers.and_u8(self.registers.c),
            0xA2 => self.registers.and_u8(self.registers.d),
            0xA3 => self.registers.and_u8(self.registers.e),
            0xA4 => self.registers.and_u8(self.registers.h),
            0xA5 => self.registers.and_u8(self.registers.l),
            0xA6 => {
                self.registers
                    .and_u8(self.mmu.fetch_byte(self.registers.get_hl(), &self.state));
                self.tick();
            }
            0xA7 => self.registers.and_u8(self.registers.a),
            0xA8 => self.registers.xor_u8(self.registers.b),
            0xA9 => self.registers.xor_u8(self.registers.c),
            0xAA => self.registers.xor_u8(self.registers.d),
            0xAB => self.registers.xor_u8(self.registers.e),
            0xAC => self.registers.xor_u8(self.registers.h),
            0xAD => self.registers.xor_u8(self.registers.l),
            0xAE => {
                self.registers
                    .xor_u8(self.mmu.fetch_byte(self.registers.get_hl(), &self.state));
                self.tick();
            }
            0xAF => self.registers.xor_u8(self.registers.a),
            0xB0 => self.registers.or_u8(self.registers.b),
            0xB1 => self.registers.or_u8(self.registers.c),
            0xB2 => self.registers.or_u8(self.registers.d),
            0xB3 => self.registers.or_u8(self.registers.e),
            0xB4 => self.registers.or_u8(self.registers.h),
            0xB5 => self.registers.or_u8(self.registers.l),
            0xB6 => {
                self.registers
                    .or_u8(self.mmu.fetch_byte(self.registers.get_hl(), &self.state));
                self.tick();
            }
            0xB7 => self.registers.or_u8(self.registers.a),
            0xB8 => self.registers.cp_u8(self.registers.b),
            0xB9 => self.registers.cp_u8(self.registers.c),
            0xBA => self.registers.cp_u8(self.registers.d),
            0xBB => self.registers.cp_u8(self.registers.e),
            0xBC => self.registers.cp_u8(self.registers.h),
            0xBD => self.registers.cp_u8(self.registers.l),
            0xBE => {
                self.registers
                    .cp_u8(self.mmu.fetch_byte(self.registers.get_hl(), &self.state));
                self.tick();
            }
            0xBF => self.registers.cp_u8(self.registers.a),
            0xC0 => {
                self.tick();
                if !self.registers.is_zero_flag_high() {
                    self.ret();
                }
            }
            0xC1 => {
                let popped_value = self.pop_u16_from_stack();
                self.registers.set_bc(popped_value);
            }
            0xC2 => {
                let address = self.fetch_word();
                if self.registers.is_zero_flag_high() {
                    self.pc = address;
                    self.tick();
                }
            }
            0xC3 => {
                let address = self.fetch_word();
                self.pc = address;
                self.tick();
            }
            0xC4 => {
                let address = self.fetch_word();
                if !self.registers.is_zero_flag_high() {
                    self.call(address);
                }
            }
            0xC5 => {
                self.push_u16_to_stack(self.registers.get_bc());
                self.tick();
            }
            0xC6 => {
                let n = self.fetch_byte();
                self.registers.add_u8(n);
            }
            0xC7 => self.call(0x0u16),
            0xC8 => {
                self.tick();
                if self.registers.is_zero_flag_high() {
                    self.ret();
                }
            }
            0xC9 => self.ret(),
            0xCA => {
                let jump = self.fetch_word();
                if self.registers.is_zero_flag_high() {
                    self.pc = jump;
                    self.tick();
                }
            }
            0xCB => self.execute_cb(),
            0xCC => {
                let jump_address = self.fetch_word();
                if self.registers.is_zero_flag_high() {
                    self.call(jump_address);
                }
            }
            0xCD => {
                let new_address = self.fetch_word();
                self.call(new_address);
            }
            0xCE => {
                let number = self.fetch_byte();
                self.registers
                    .add_u8(number + self.registers.is_carry_flag_high() as u8);
            }
            0xCF => {
                self.call(0x08u16);
            }
            0xD0 => {
                self.tick();
                if !self.registers.is_carry_flag_high() {
                    self.ret();
                }
            }
            0xD1 => {
                let popped_value = self.pop_u16_from_stack();
                self.registers.set_de(popped_value);
            }
            0xD2 => {
                let address = self.fetch_word();
                if self.registers.is_carry_flag_high() {
                    self.pc = address;
                    self.tick();
                }
            }
            0xD4 => {
                let address = self.fetch_word();
                if !self.registers.is_carry_flag_high() {
                    self.call(address);
                }
            }
            0xD5 => {
                self.push_u16_to_stack(self.registers.get_de());
                self.tick();
            }
            0xD6 => {
                let n = self.fetch_byte();
                self.registers.sub_u8(n);
            }
            0xD7 => {
                self.call(0x10u16);
            }
            0xD8 => {
                self.tick();
                if self.registers.is_carry_flag_high() {
                    self.ret();
                }
            }
            0xD9 => {
                self.ret();
                self.mmu.interrupt_handler.enabled = true;
            }
            0xDA => {
                let jump = self.fetch_word();
                self.tick();
                if self.registers.is_carry_flag_high() {
                    self.pc = jump;
                    self.tick();
                }
            }
            0xDC => {
                let jump_address = self.fetch_word();
                if self.registers.is_carry_flag_high() {
                    self.call(jump_address);
                    self.tick();
                }
            }
            0xDE => {
                let number = self.fetch_byte();
                self.registers
                    .sub_u8(number.wrapping_sub(self.registers.is_carry_flag_high() as u8));
            }
            0xDF => {
                self.call(0x18u16);
            }
            0xE0 => {
                let address: u16 = 0xFF00 + (self.fetch_byte() as u16);
                self.mmu
                    .write_byte(address, self.registers.a, &mut self.state);
                self.tick();
            }
            0xE1 => {
                let address = self.pop_u16_from_stack();
                self.registers.set_hl(address);
            }
            0xE2 => {
                self.mmu.write_byte(
                    0xFFu16 + self.registers.c as u16,
                    self.registers.a,
                    &mut self.state,
                );
                self.tick();
            }
            0xE5 => {
                self.push_u16_to_stack(self.registers.get_hl());
                self.tick();
            }
            0xE6 => {
                let reg = self.fetch_byte();
                self.registers.and_u8(reg);
            }
            0xE7 => {
                self.call(0x20u16);
            }
            0xE8 => {
                let number = self.fetch_byte() as i8;
                self.registers.set_zero_flag(false);
                self.registers.set_was_prev_instr_sub(false);

                self.tick();
                self.tick();
                if number < 0 {
                    self.registers.set_carry_flag(self.sp < number as u16);
                    self.registers.set_half_carry_flag(
                        ((self.sp
                            ^ number as u16
                            ^ self.sp.wrapping_add(number.unsigned_abs() as u16))
                            & 0x10)
                            > 0,
                    );
                    self.sp = self.sp.wrapping_sub(number.unsigned_abs() as u16);
                } else {
                    self.registers.set_carry_flag(
                        self.sp > self.sp.wrapping_add(number.unsigned_abs() as u16),
                    );
                    self.registers.set_half_carry_flag(
                        (self.sp & 0xFFu16) + number.unsigned_abs() as u16 > 0xFF,
                    );
                    self.sp = self.sp.wrapping_add(number as u16);
                }
            }
            0xE9 => self.pc = self.registers.get_hl(),
            0xEA => {
                let address = self.fetch_word();
                self.mmu
                    .write_byte(address, self.registers.a, &mut self.state);
                self.tick();
            }
            0xEE => {
                let byte = self.fetch_byte();
                self.registers.xor_u8(byte);
            }
            0xEF => self.call(0x28u16),
            0xF0 => {
                let add_on = self.fetch_byte() as u16;
                self.registers.a = self.mmu.fetch_byte(0xFF00u16 + add_on, &self.state);
                self.tick();
            }
            0xF1 => {
                let address = self.pop_u16_from_stack();
                self.registers.set_af(address);
            }
            0xF2 => {
                self.registers.a = self
                    .mmu
                    .fetch_byte(0xFF00u16.wrapping_add(self.registers.c as u16), &self.state);
                self.tick();
            }
            0xF3 => self.mmu.interrupt_handler.enabled = false,
            0xF5 => {
                self.push_u16_to_stack(self.registers.get_af());
                self.tick();
            }
            0xF6 => {
                let byte = self.fetch_byte();
                self.registers.or_u8(byte);
            }
            0xF7 => self.call(0x30u16),
            0xF8 => {
                let number = self.fetch_byte() as i8;
                let adder: u16;

                self.tick();
                if number >= 0 {
                    adder = self.sp.wrapping_add(number.unsigned_abs() as u16);
                } else {
                    adder = self.sp.wrapping_sub(number.unsigned_abs() as u16);
                }
                self.registers.set_hl(adder);
            }
            0xF9 => {
                self.sp = self.registers.get_hl();
                self.tick();
            }
            0xFA => {
                let word = self.fetch_word();
                self.registers.a = self.mmu.fetch_byte(word, &self.state);
                self.tick();
            }
            0xFB => self.mmu.interrupt_handler.enabled = true,
            0xFE => {
                let number = self.fetch_byte();
                self.registers.cp_u8(number);
            }
            0xFF => {
                panic!(
                    "Something went wrong, instruction 0xFF called, pc: {:X}",
                    self.pc - 1
                )
            }
            _ => panic!(
                "Instruction {:x?} not implemented",
                first_byte.to_be_bytes()
            ),
        }
    }
}
