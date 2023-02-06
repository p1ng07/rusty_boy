use strum::IntoEnumIterator;

use crate::cpu_registers::CpuRegisters;
use crate::interrupt_handler::*;
use crate::mmu::Mmu;

#[derive(PartialEq)]
pub enum CpuState {
    Boot,
    NonBoot,
    Stopped,
}

pub struct Cpu {
    state: CpuState,
    pc: u16,
    sp: u16,
    registers: CpuRegisters,
}

impl Cpu {
    pub fn new(initial_state: CpuState) -> Cpu {
        let mut cpu = Cpu {
            pc: 0,
            sp: 0,
            state: initial_state,
            registers: CpuRegisters::default(),
        };

        // Skip the bootrom, and go straight to running the program
        if cpu.state == CpuState::NonBoot {
            initialize_cpu_state_defaults(&mut cpu);
        }
        cpu
    }

    // Cycle the cpu once, fetch an instruction and run it, returns the number of t-cycles it took to run it
    pub fn cycle(&mut self, mmu: &mut Mmu) -> i32 {
        let mut delta_cycles = 0;
        let first_byte = self.fetch_byte(mmu);

        // Fetch cycles are already included in te execute_* functions, this shouldn't happen but I am too lazy to fix it for now
        delta_cycles += match first_byte {
            0xCB => self.execute_cb(mmu),
            _ => self.execute(first_byte, mmu),
        };

	// TODO: Update the timers
	mmu.timer.step(&self.state, delta_cycles, &mut mmu.interrupt_handler);

        delta_cycles += self.handle_interrupts(mmu);

        delta_cycles
    }

    fn fetch_byte(&mut self, mmu: &Mmu) -> u8 {
        let byte = mmu.fetch_byte(self.pc, &self.state);
        self.pc += 1;
        byte
    }

    pub fn fetch_word(&mut self, mmu: &Mmu) -> u16 {
        let fetch_byte_big = self.fetch_byte(mmu) as u16;
        let fetch_byte_small = self.fetch_byte(mmu) as u16;

        fetch_byte_small << 8 | fetch_byte_big
    }

    // Services all serviciable interrupts and returns the number of t-cycles this handling took
    fn handle_interrupts(&mut self, mmu: &mut Mmu) -> i32 {
        if !mmu.interrupt_handler.enabled || mmu.interrupt_handler.IE == 0 {
            // It isn't possible to service any interrupt
            return 0;
        }

        // Go through every interrupt possible interrupt in order of priority (bit order ex: vblank is highest priority)
        // Check if it is requested and enabled, if it is then service it
        // IMPORTANT: This iterator uses the order in which the variants are set in the enum, therefore respecting the interrupt order
        for interrupt_type in Interrupt::iter() {
            if interrupt_type.mask() & mmu.interrupt_handler.IF > 0
                && interrupt_type.mask() & mmu.interrupt_handler.IE > 0
                && mmu.interrupt_handler.enabled
            {
                // Service interrupt, set ime to false and reset the respective IF bit on the handler
                mmu.interrupt_handler.unrequest_interrupt(&interrupt_type);

                // CALL interrupt_vector
		self.call(interrupt_type.jump_vector(), mmu);

                // Disable IME
                mmu.interrupt_handler.enabled = false;
                return 20;
            }
        }
        0
    }

    // Execute the instruction given and return the number of t-cycles it took to run it
    pub(crate) fn execute(&mut self, first_byte: u8, mmu: &mut Mmu) -> i32 {
        // Print state of emulator to logger
        log::info!(
            "A: {} F: {} B: {} C: {} D: {} E: {} H: {} L: {} SP: {} PC: 00:{} ({} {} {} {})",
            format!("{:0>2X}", self.registers.a),
            format!("{:0>2X}", self.registers.f),
            format!("{:0>2X}", self.registers.b),
            format!("{:0>2X}", self.registers.c),
            format!("{:0>2X}", self.registers.d),
            format!("{:0>2X}", self.registers.e),
            format!("{:0>2X}", self.registers.h),
            format!("{:0>2X}", self.registers.l),
            format!("{:0>4X}", self.sp),
            format!("{:0>4X}", self.pc - 1),
            format!("{:0>2X}", first_byte),
            format!("{:0>2X}", mmu.fetch_byte(self.pc, &self.state)),
            format!("{:0>2X}", mmu.fetch_byte(self.pc + 1, &self.state)),
            format!("{:0>2X}", mmu.fetch_byte(self.pc + 2, &self.state))
        );

        match first_byte {
            0x00 => 4,
	    0x01 => {
		let n = self.fetch_word(mmu);
		self.registers.set_bc(n);
		12
	    }
	    0x02 => {
		mmu.write_byte(self.registers.get_bc(), self.registers.a, &mut self.state);
		8
	    }
	    0x03 => {
		self.registers.set_bc(self.registers.get_bc().wrapping_add(1));
		8
	    }
            0x04 => {
                self.registers.b = self.registers.inc_u8_reg(self.registers.b);
                4
            }
            0x05 => {
                // DEC B
                self.registers.b = self.registers.dec_u8_reg(self.registers.b);
                5
            }
            0x06 => {
                // LD B, u8
                self.registers.b = self.fetch_byte(mmu);
                8
            }
	    0x07 => {
		self.registers.rlca();
		4
	    }
	    0x08 => {
		mmu.write_word(self.fetch_word(mmu), self.sp,&mut self.state, );
		20
	    }
	    0x09 => {
		self.registers.add_to_hl_u16(self.registers.get_bc());
		8
	    }
	    0x0A => {
		self.registers.a = mmu.fetch_byte(self.registers.get_bc(), &self.state);
		8
	    }
	    0x0B => {
		self.registers.dec_bc();
		8
	    }
            0x0C => {
                // INC C
                self.registers.c = self.registers.inc_u8_reg(self.registers.c);
                4
            }
            0x0D => {
                self.registers.c = self.registers.dec_u8_reg(self.registers.c);
                4
            }
            0x0E => {
                // LD C, u8
                self.registers.c = self.fetch_byte(mmu);
                8
            }
            0x0F => {
                let least_bit = self.registers.a & 0x1;
                self.registers.set_flags(0);
                self.registers.set_carry_flag(least_bit > 0);
                self.registers.a = (self.registers.a >> 1) | least_bit << 7;
                4
            }
            0x11 => {
                // LD BC, u16
                let word = self.fetch_word(mmu);
                self.registers.set_de(word);
                12
            }
            0x12 => {
                mmu.write_byte(self.registers.get_de(), self.registers.a, &mut self.state);
                8
            }
            0x13 => {
                // INC DE
                let new_de = self.registers.get_de().wrapping_add(1);
                self.registers.set_de(new_de);
                8
            }
            0x14 => {
                self.registers.d = self.registers.inc_u8_reg(self.registers.d);
                4
            }
            0x15 => {
                self.registers.d = self.registers.dec_u8_reg(self.registers.d);
                4
            }
            0x16 => {
                self.registers.d = self.fetch_byte(mmu);
                8
            }
            0x17 => {
                // RLA
		let old_carry = if self.registers.is_carry_flag_high() {1} else {0};
                self.registers.set_flags(if self.registers.a & 0x80 > 0 {
                    0b1000
                } else {
                    0b0000
                });
                self.registers.a <<= 1;
		self.registers.a |= old_carry;
                4
            }
            0x18 => {
                let offset = self.fetch_byte(mmu) as i8;
                self.pc += offset as u16;
                12
            }
	    0x19 => {
		self.registers.add_to_hl_u16(self.registers.get_de());
		8
	    }
            0x1A => {
                // LD A, (DE)
                self.registers.a = mmu.fetch_byte(self.registers.get_de(), &self.state);
                8
            }
	    0x1B => {
		self.registers.dec_de();
		8
	    }
            0x1C => {
                self.registers.e = self.registers.inc_u8_reg(self.registers.e);
                4
            }
            0x1D => {
                self.registers.e = self.registers.dec_u8_reg(self.registers.e);
                4
            }
            0x1E => {
                self.registers.e = self.fetch_byte(mmu);
                8
            }
	    0x1F => {
		let old_carry = if self.registers.is_carry_flag_high() {1} else {0};
		self.registers.set_flags(if self.registers.a & 0x01 > 0 {
                    0b1000
                } else {
                    0b0000
                });
                self.registers.a >>= 1;
		self.registers.a |= old_carry << 7;
                4
	    }
            0x20 => {
                // JR NZ, i8
                if !self.registers.is_zero_flag_high() {
                    let offset = self.fetch_byte(mmu) as i8;
                    if offset < 0 {
                        self.pc = self.pc.wrapping_sub(offset.unsigned_abs() as u16);
                    } else {
                        self.pc = self.pc.wrapping_sub(offset as u16);
                    }
                    return 12;
                }
                self.pc += 1;
                8
            }
            0x21 => {
                // LD HL, U16
                let word = self.fetch_word(mmu);
                self.registers.set_hl(word);
                12
            }
            0x22 => {
                // LD (HL++), A
                mmu.write_byte(self.registers.get_hl(), self.registers.a, &mut self.state);
                self.registers.set_hl(self.registers.get_hl() + 1);
                8
            }
            0x23 => {
                // INC HL
                let new_hl = self.registers.get_hl().wrapping_add(1);
                self.registers.set_hl(new_hl);
                8
            }
            0x24 => {
                self.registers.h = self.registers.inc_u8_reg(self.registers.h);
                4
            }
	    0x25 => {
		self.registers.h = self.registers.dec_u8_reg(self.registers.h);
		4
	    }
	    0x26 => {
		self.registers.h = self.fetch_byte(mmu);
		8
	    }
            0x28 => {
                if self.registers.is_zero_flag_high() {
                    let offset = self.fetch_byte(mmu) as i8;
                    if offset < 0 {
                        self.pc = self.pc.wrapping_sub(offset.unsigned_abs() as u16);
                    } else {
                        self.pc = self.pc.wrapping_sub(offset as u16);
                    }
                    return 12;
                }
                self.pc += 1;
                8
            }
	    0x29 => {
		self.registers.add_to_hl_u16(self.registers.get_hl());
		8
	    }
            0x2A => {
                let add = self.registers.get_hl();
                self.registers.a = mmu.fetch_byte(add, &self.state);
                let new_hl = add.wrapping_add(1);
                self.registers.set_hl(new_hl);
                8
            }
	    0x2B => {
		self.registers.dec_hl();
		8
	    }
	    0x2C => {
		self.registers.l = self.registers.inc_u8_reg(self.registers.l);
		4
	    }
	    0x2D => {
		self.registers.l = self.registers.dec_u8_reg(self.registers.l);
		4
	    }
	    0x2E => {
		self.registers.l = self.fetch_byte(mmu);
		8
	    }
	    0x2F => {
		self.registers.cpl();
		4
	    }
	    0x30 => {
                // JR NC, i8
                if !self.registers.is_carry_flag_high() {
                    let offset = self.fetch_byte(mmu) as i8;
                    if offset < 0 {
                        self.pc = self.pc.wrapping_sub(offset.unsigned_abs() as u16);
                    } else {
                        self.pc = self.pc.wrapping_sub(offset as u16);
                    }
                    return 12;
                }
                self.pc += 1;
                8
	    }
            0x31 => {
                // LD SP, U16
                self.sp = self.fetch_word(mmu);
                12
            }
            0x32 => {
                // ld (hl-), A
                mmu.write_byte(self.registers.get_hl(), self.registers.a, &mut self.state);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                4
            }
	    0x33 => {
		self.sp = self.sp.wrapping_add(1);
		8
	    }
	    0x34 => {
		let value = mmu.fetch_byte(self.registers.get_hl(), &self.state).wrapping_add(1);
		mmu.write_byte(self.registers.get_hl(), value, &mut self.state);
		12
	    }
	    0x35 => {
		let value = mmu.fetch_byte(self.registers.get_hl(), &self.state).wrapping_sub(1);
		mmu.write_byte(self.registers.get_hl(), value, &mut self.state);
		12
	    }
	    0x36 => {
		mmu.write_byte(self.registers.get_hl(), self.fetch_byte(mmu), &mut self.state);
		12
	    }
	    0x37 => {
		self.registers.set_carry_flag(true);
		12
	    }
	    0x38 => {
                if self.registers.is_carry_flag_high() {
                    let offset = self.fetch_byte(mmu) as i8;
                    if offset < 0 {
                        self.pc = self.pc.wrapping_sub(offset.unsigned_abs() as u16);
                    } else {
                        self.pc = self.pc.wrapping_sub(offset as u16);
                    }
                    return 12;
                }
                self.pc += 1;
                8
	    }
	    0x39 => {
		self.registers.add_to_hl_u16(self.sp);
		8
	    }
	    0x3A => {
		self.registers.a = mmu.fetch_byte(self.registers.get_hl(), &self.state);
		self.registers.set_hl(self.registers.get_hl().wrapping_sub(1));
		8
	    }
	    0x3B => {
		self.sp = self.sp.wrapping_sub(1);
		8
	    }
	    0x3C => {
		self.registers.a = self.registers.inc_u8_reg(self.registers.a);
		8
	    }
            0x3D => {
                self.registers.a = self.registers.dec_u8_reg(self.registers.a);
                4
            }
            0x3E => {
                // LD A, u8
                self.registers.a = self.fetch_byte(mmu);
                8
            }
	    0x3F => {
		self.registers.set_was_prev_instr_sub(false);
		self.registers.set_half_carry(false);
		self.registers.set_carry_flag(!self.registers.is_carry_flag_high());
		4
	    }
	    0x40 => {
		self.registers.b = self.registers.b;
		4
	    }
	    0x41 => {
		self.registers.b = self.registers.c;
		4
	    }
	    0x42 => {
		self.registers.b = self.registers.d;
		4
	    }
	    0x43 => {
		self.registers.b = self.registers.e;
		4
	    }
            0x44 => {
                self.registers.b = self.registers.h;
                4
            }
	    0x45 => {
		self.registers.b = self.registers.l;
		4
	    }
	    0x46 => {
		self.registers.b = mmu.fetch_byte(self.registers.get_hl(), &self.state);
		8
	    }
            0x47 => {
                self.registers.b = self.registers.a;
                4
            }
            0x48 => {
                self.registers.c = self.registers.b;
                4
            }
            0x49 => {
                self.registers.c = self.registers.c;
                4
            }
            0x4A => {
                self.registers.c = self.registers.d;
                4
            }
            0x4B => {
                self.registers.c = self.registers.e;
                4
            }
            0x4C => {
                self.registers.c = self.registers.h;
                4
            }
            0x4E => {
                self.registers.c = self.registers.l;
                4
            }
            0x4F => {
                // LD C,A
                self.registers.c = self.registers.a;
                4
            }
            0x50 => {
                self.registers.d = self.registers.b;
                4
            }
            0x51 => {
                self.registers.d = self.registers.c;
                4
            }
            0x52 => {
                self.registers.d = self.registers.d;
                4
            }
            0x53 => {
                self.registers.d = self.registers.e;
                4
            }
            0x54 => {
                self.registers.d = self.registers.h;
                4
            }
            0x55 => {
                self.registers.d = self.registers.l;
                4
            }
            0x56 => {
                self.registers.d = mmu.fetch_byte(self.registers.get_hl(), &self.state);
                8
            }
            0x57 => {
                self.registers.d = self.registers.a;
                4
            }
            0x58 => {
                self.registers.e = self.registers.b;
                4
            }
            0x59 => {
                self.registers.e = self.registers.c;
                4
            }
            0x5A => {
                self.registers.e = self.registers.e;
                4
            }
            0x5B => {
                self.registers.e = self.registers.e;
                4
            }
            0x5C => {
                self.registers.e = self.registers.h;
                4
            }
            0x5D => {
                self.registers.e = self.registers.l;
                4
            }
            0x5E => {
                self.registers.e = mmu.fetch_byte(self.registers.get_hl(), &self.state);
                8
            }
            0x5F => {
                self.registers.e = self.registers.a;
                4
            }
            0x60 => {
                self.registers.h = self.registers.b;
                4
            }
            0x61 => {
                self.registers.h = self.registers.c;
                4
            }
            0x62 => {
                self.registers.h = self.registers.d;
                4
            }
            0x63 => {
                self.registers.h = self.registers.e;
                4
            }
            0x64 => {
                self.registers.h = self.registers.h;
                4
            }
            0x65 => {
                self.registers.h = self.registers.l;
                4
            }
            0x66 => {
                self.registers.h = mmu.fetch_byte(self.registers.get_hl(), &self.state);
                8
            }
            0x67 => {
                self.registers.h = self.registers.a;
                4
            }
            0x68 => {
                self.registers.l = self.registers.b;
                4
            }
            0x69 => {
                self.registers.l = self.registers.c;
                4
            }
            0x6A => {
                self.registers.l = self.registers.d;
                4
            }
            0x6B => {
                self.registers.l = self.registers.e;
                4
            }
            0x6C => {
                self.registers.l = self.registers.h;
                4
            }
            0x6D => {
                self.registers.l = self.registers.l;
                4
            }
            0x6E => {
                self.registers.l = mmu.fetch_byte(self.registers.get_hl(), &self.state);
                8
            }
            0x6F => {
                self.registers.l = self.registers.a;
                4
            }
            0x70 => {
		mmu.write_byte(self.registers.get_hl(), self.registers.b, &mut self.state);
                8
            }
            0x71 => {
		mmu.write_byte(self.registers.get_hl(), self.registers.c, &mut self.state);
                8
            }
            0x72 => {
		mmu.write_byte(self.registers.get_hl(), self.registers.d, &mut self.state);
                8
            }
            0x73 => {
		mmu.write_byte(self.registers.get_hl(), self.registers.e, &mut self.state);
                8
            }
            0x74 => {
		mmu.write_byte(self.registers.get_hl(), self.registers.h, &mut self.state);
                8
            }
            0x75 => {
		mmu.write_byte(self.registers.get_hl(), self.registers.l, &mut self.state);
                8
            }
            0x77 => {
                mmu.write_byte(self.registers.get_hl(), self.registers.a, &mut self.state);
                8
            }
            0x78 => {
                self.registers.a = self.registers.b;
                4
            }
            0x79 => {
                self.registers.a = self.registers.c;
                4
            }
            0x7A => {
                self.registers.a = self.registers.d;
                4
            }
            0x7B => {
                self.registers.a = self.registers.e;
                4
            }
            0x7C => {
                self.registers.a = self.registers.h;
                4
            }
	    0x7D => {
		self.registers.a = self.registers.l;
		4
	    }
	    0x7E => {
		self.registers.a = mmu.fetch_byte(self.registers.get_hl(), &self.state);
		8
	    }
	    0x7F => {
		self.registers.a = self.registers.a;
		8
	    }
	    0x80 => {
		self.registers.add_u8(self.registers.b);
		4
	    }
	    0x81 => {
		self.registers.add_u8(self.registers.c);
		4
	    }
	    0x82 => {
		self.registers.add_u8(self.registers.d);
		4
	    }
	    0x83 => {
		self.registers.add_u8(self.registers.e);
		4
	    }
	    0x84 => {
		self.registers.add_u8(self.registers.h);
		4
	    }
	    0x85 => {
		self.registers.add_u8(self.registers.l);
		4
	    }
	    0x86 => {
		self.registers.add_u8(mmu.fetch_byte(self.registers.get_hl(), &self.state));
		8
	    }
	    0x87 => {
		self.registers.add_u8(self.registers.a);
		4
	    }
	    0x88 => {
		self.registers.add_u8(self.registers.b + self.registers.is_carry_flag_high() as u8);
		4
	    }
	    0x89 => {
		self.registers.add_u8(self.registers.c + self.registers.is_carry_flag_high() as u8);
		4
	    }
	    0x8A => {
		self.registers.add_u8(self.registers.d + self.registers.is_carry_flag_high() as u8);
		4
	    }
	    0x8B => {
		self.registers.add_u8(self.registers.e + self.registers.is_carry_flag_high() as u8);
		4
	    }
	    0x8C => {
		self.registers.add_u8(self.registers.h + self.registers.is_carry_flag_high() as u8);
		4
	    }
	    0x8D => {
		self.registers.add_u8(self.registers.l + self.registers.is_carry_flag_high() as u8);
		4
	    }
	    0x8E => {
		self.registers.add_u8(mmu.fetch_byte(self.registers.get_hl(), &self.state) + self.registers.is_carry_flag_high() as u8);
		8
	    }
	    0x8F => {
		self.registers.add_u8(self.registers.a + self.registers.is_carry_flag_high() as u8);
		4
	    }
            0x90 => {
                self.registers.sub_u8_reg(self.registers.b);
                4
            }
            0x91 => {
                self.registers.sub_u8_reg(self.registers.c);
                4
            }
            0x92 => {
                self.registers.sub_u8_reg(self.registers.d);
                4
            }
            0x93 => {
                self.registers.sub_u8_reg(self.registers.e);
                4
            }
            0x94 => {
                self.registers.sub_u8_reg(self.registers.h);
                4
            }
            0x95 => {
                self.registers.sub_u8_reg(self.registers.l);
                4
            }
            0x96 => {
                self.registers.sub_u8_reg(mmu.fetch_byte(self.registers.get_hl(), &self.state));
                8
            }
            0x97 => {
                self.registers.sub_u8_reg(self.registers.a);
                4
            }
            0x98 => {
                self.registers.sub_u8_reg(self.registers.b - self.registers.is_carry_flag_high() as u8);
                4
            }
            0x99 => {
                self.registers.sub_u8_reg(self.registers.c - self.registers.is_carry_flag_high() as u8);
                4
            }
            0x9A => {
                self.registers.sub_u8_reg(self.registers.d - self.registers.is_carry_flag_high() as u8);
                4
            }
            0x9B => {
                self.registers.sub_u8_reg(self.registers.e - self.registers.is_carry_flag_high() as u8);
                4
            }
            0x9C => {
                self.registers.sub_u8_reg(self.registers.h - self.registers.is_carry_flag_high() as u8);
                4
            }
            0x9D => {
                self.registers.sub_u8_reg(self.registers.l - self.registers.is_carry_flag_high() as u8);
                4
            }
            0x9E => {
                self.registers.sub_u8_reg(mmu.fetch_byte(self.registers.get_hl(), &self.state) - self.registers.is_carry_flag_high() as u8);
                8
            }
            0x9F => {
                self.registers.sub_u8_reg(self.registers.a - self.registers.is_carry_flag_high() as u8);
                4
            }
	    0xA9 => {
		self.registers.xor_u8(self.registers.c);
		4
	    }
            0xAF => {
                // XOR A
                self.registers.xor_u8(self.registers.a);
                4
            }
	    0xB1 => {
		self.registers.or_u8(self.registers.c);
		4
	    }
            0xBC => {
                // POP BC
                let new_bc = self.pop_u16_from_stack(mmu);
                self.registers.set_bc(new_bc);
                12
            }
            0xC1 => {
                let popped_value = self.pop_u16_from_stack(mmu);
                self.registers.set_bc(popped_value);
                12
            }
            0xC3 => {
                let address = self.fetch_word(mmu);
                self.pc = address;
                16
            }
	    0xC4 => {
		if !self.registers.is_zero_flag_high() {
		    let address = self.fetch_word(mmu);
		    self.call(address, mmu);
		    return 24;
		}
		self.pc += 2;
		12
	    }
            0xC5 => {
                // PUSH BC
                self.push_u16_to_stack(self.registers.get_bc(), mmu);
                16
            }
	    0xC6 => {
		let n = self.fetch_byte(mmu);
		self.registers.add_u8(n);
		8
	    }
            0xC9 => {
                // RET
                self.pc = self.pop_u16_from_stack(mmu);
                16
            }
            0xCD => {
                // CALL nn
                let new_address = self.fetch_word(mmu);
		self.call(new_address, mmu);
                24
            }
            0xE0 => {
                // LD ($FF00+u8), A
                let address: u16 = 0xFF00 + (self.fetch_byte(mmu) as u16);
                mmu.write_byte(address, self.registers.a, &mut self.state);
                12
            }
	    0xE1 => {
		let address = self.pop_u16_from_stack(mmu);
		self.registers.set_hl(address);
		12
	    }
            0xE2 => {
                // LD (FF00 + C), A
                mmu.write_byte(0xFFu16 + self.registers.c as u16, self.registers.a, &mut self.state);
                8
            }
	    0xE5 => {
		self.push_u16_to_stack(self.registers.get_hl(), mmu);
		16
	    }
	    0xE6 => {
		let reg = self.fetch_byte(mmu);
		self.registers.and_u8(reg);
		8
	    }
            0xEA => {
                let address = self.fetch_word(mmu);
                mmu.write_byte(address, self.registers.a, &mut self.state);
                16
            }
            0xF0 => {
                let add_on = self.fetch_byte(mmu) as u16;
                self.registers.a = mmu.fetch_byte(0xFF00u16 + add_on, &self.state);
                12
            }
	    0xF1 => {
		let new_af = self.pop_u16_from_stack(mmu);
		self.registers.set_af(new_af);
		12
	    }
            0xF2 => {
                self.registers.a =
                    mmu.fetch_byte(0xFF00u16.wrapping_add(self.registers.c as u16), &self.state);
                8
            }
	    0xF3 => {
		mmu.interrupt_handler.enabled = false;
		4
	    }
	    0xF5 => {
		self.push_u16_to_stack(self.registers.get_af(), mmu);
		16
	    }
	    0xFA => {
		self.registers.a = mmu.fetch_byte(self.fetch_word(mmu), &self.state);
		16
	    }
            0xFE => {
                let number = self.fetch_byte(mmu);
                self.registers.cp(number);
                8
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


    pub(crate) fn execute_cb(&mut self, mmu: &mut Mmu) -> i32 {
        let instruction = self.fetch_byte(mmu);

        // Print state of emulator to logger
        log::info!(
            "A: {} F: {} B: {} C: {} D: {} E: {} H: {} L: {} SP: {} PC: 00:{} ({} {} {} {})",
            format!("{:0>2X}", self.registers.a),
            format!("{:0>2X}", self.registers.f),
            format!("{:0>2X}", self.registers.b),
            format!("{:0>2X}", self.registers.c),
            format!("{:0>2X}", self.registers.d),
            format!("{:0>2X}", self.registers.e),
            format!("{:0>2X}", self.registers.h),
            format!("{:0>2X}", self.registers.l),
            format!("{:0>4X}", self.sp),
            format!("{:0>4X}", self.pc - 1),
            format!("{:0>4X}", instruction),
            format!("{:02X}", mmu.fetch_byte(self.pc, &self.state)),
            format!("{:02X}", mmu.fetch_byte(self.pc + 1, &self.state)),
            format!("{:02X}", mmu.fetch_byte(self.pc + 2, &self.state))
        );

        let instruction_cycles = 4;
        instruction_cycles
            + match instruction {
                0x11 => {
                    // RL C
                    self.registers.set_carry_flag(self.registers.c & 0x80 > 0);
                    self.registers.c <<= 1;
                    self.registers.set_zero_flag(self.registers.c == 0);
                    self.registers.set_half_carry(false);
                    self.registers.set_was_prev_instr_sub(false);
                    8
                }
                0x7C => {
                    self.registers.set_zero_flag(self.registers.h < 128);
                    4
                }
                _ => panic!(
                    "CB prefixed instruction {:X?} was not implemented",
                    instruction.to_be_bytes()
                ),
            }
    }

    fn call(&mut self, address: u16, mmu: &mut Mmu) {
	self.push_u16_to_stack(self.pc, mmu);
	self.pc = address;
    }


    fn push_u16_to_stack(&mut self, value_to_push: u16, mmu: &mut Mmu) {
        self.sp = self.sp.wrapping_sub(1);
        mmu.write_byte(self.sp, (value_to_push >> 8) as u8, &mut self.state);
        self.sp = self.sp.wrapping_sub(1);
        mmu.write_byte(self.sp, value_to_push as u8, &mut self.state);
    }

    fn pop_u16_from_stack(&mut self, mmu: &Mmu) -> u16 {
        let lower_byte = mmu.fetch_byte(self.sp, &self.state);
        self.sp = self.sp.wrapping_add(1);
        let high_byte = mmu.fetch_byte(self.sp, &self.state);
        self.sp = self.sp.wrapping_add(1);
        (high_byte as u16) << 8 | lower_byte as u16
    }

}

fn initialize_cpu_state_defaults(cpu: &mut Cpu) {
    cpu.registers.a = 1;
    cpu.registers.f = 0xB0;
    cpu.registers.c = 0x13;
    cpu.registers.e = 0xD8;
    cpu.registers.h = 0x1;
    cpu.registers.l = 0x4D;
    cpu.pc = 0x100;
    cpu.sp = 0xfffe;
}
