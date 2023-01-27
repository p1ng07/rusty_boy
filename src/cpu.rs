use crate::cpu_registers::CpuRegisters;
use crate::mmu::Mmu;

#[derive(PartialEq)]
pub enum CpuState {
    Boot,
    NonBoot,
}

pub struct Cpu {
    state: CpuState,
    pc: u16,
    sp: u16,
    registers: CpuRegisters,
    pub mmu: Mmu,
    ime: bool
}

impl Cpu {
    pub fn new(rom_path: String, initial_state: CpuState) -> Cpu {
	let mut cpu = Cpu {
	    pc: if initial_state == CpuState::NonBoot {0x100} else {0},
	    sp: 0,
	    state: initial_state,
	    registers: Default::default(),
	    ime: false,
	    mmu: Mmu::new(rom_path),
	};

	if cpu.state == CpuState::NonBoot {
	    cpu.registers.a = 1;
	    cpu.registers.f = 0xB0;
	    cpu.registers.c = 0x13;
	    cpu.registers.e = 0xD8;
	    cpu.registers.h = 0x1;
	    cpu.registers.l = 0x4D;
	    cpu.sp = 0xfffe;
	}
	cpu
	
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.mmu.fetch_byte(self.pc, &self.state); // This is never going to launch an exception
        self.pc += 1;
        byte
    }

    pub(crate) fn fetch_word(&mut self) -> u16 {
        let fetch_byte_big = self.fetch_byte() as u16;
        let fetch_byte_small = self.fetch_byte() as u16;

        fetch_byte_small << 8 | fetch_byte_big
    }

    // Cycle the cpu once, fetch an instruction and run it, returns the number of t-cycles it took to run it
    pub(crate) fn cycle(&mut self) -> i32 {
        let fetch_cycles = 4;
        let first_byte = self.fetch_byte();
        fetch_cycles
            + match first_byte {
                0xCB => self.execute_cb(),
                _ => self.execute(first_byte),
            }
    }

    // Execute the instruction given and return the number of t-cycles it took to run it
    pub(crate) fn execute(&mut self, first_byte: u8) -> i32 {
        // Print state of emulator to logger
        log::info!(
            "A: {}, F: {}, B: {}, C: {}, D: {}, E: {}, H: {}, L: {}, sp: {}, pc: {}, {:X}",
            format!("{:0>2X}",self.registers.a),
            format!("{:0>2X}",self.registers.f),
            format!("{:0>2X}",self.registers.b),
            format!("{:0>2X}",self.registers.c),
            format!("{:0>2X}",self.registers.d),
            format!("{:0>2X}",self.registers.e),
            format!("{:0>2X}",self.registers.h),
            format!("{:0>2X}",self.registers.l),
            format!("{:0>4X}",self.sp),
            format!("{:0>4X}",self.pc - 1),
            first_byte
        );

        match first_byte {
            0x00 => 4,
            0x04 => {
                self.registers.b = self.inc_u8_reg(self.registers.b);
                4
            }
            0x05 => {
                // DEC B
                self.registers.b = self.dec_u8_reg(self.registers.b);
                5
            }
            0x06 => {
                // LD B, u8
                self.registers.b = self.fetch_byte();
                8
            }
            0x0C => {
                // INC C
                self.registers.c = self.inc_u8_reg(self.registers.c);
                4
            }
            0x0D => {
                self.registers.c = self.dec_u8_reg(self.registers.c);
                4
            }
            0x0E => {
                // LD C, u8
                self.registers.c = self.fetch_byte();
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
                let word = self.fetch_word();
                self.registers.set_de(word);
                12
            }
            0x12 => {
                self.mmu.write_byte(
                    &mut self.state,
                    self.registers.get_de() as i32,
                    self.registers.a,
                );
                8
            }
            0x13 => {
                // INC DE
                let new_de = self.registers.get_de().wrapping_add(1);
                self.registers.set_de(new_de);
                8
            }
            0x14 => {
                self.registers.d = self.inc_u8_reg(self.registers.d);
                4
            }
            0x15 => {
                self.registers.d = self.dec_u8_reg(self.registers.d);
                4
            }
            0x16 => {
                self.registers.d = self.fetch_byte();
                8
            }
            0x17 => {
                // RLA
                self.registers.set_flags(if self.registers.a & 0x80 > 0 {
                    0b1000
                } else {
                    0b0000
                });
                self.registers.a <<= 1;
                4
            }
            0x18 => {
                let offset = self.fetch_byte() as i8;
                self.pc += offset as u16;
                12
            }
            0x1A => {
                // LD A, (DE)
                self.registers.a = self
                    .mmu
                    .fetch_byte(self.registers.get_de().try_into().unwrap(), &self.state);
                8
            }
            0x1C => {
                self.registers.e = self.inc_u8_reg(self.registers.e);
                4
            }
            0x1D => {
                self.registers.e = self.dec_u8_reg(self.registers.e);
                4
            }
            0x1E => {
                self.registers.e = self.fetch_byte();
                8
            }
            0x22 => {
                // LD (HL++), A
                self.mmu.write_byte(
                    &mut self.state,
                    self.registers.get_hl() as i32,
                    self.registers.a,
                );
                self.registers.set_hl(self.registers.get_hl() + 1);
                8
            }
            0x24 => {
                self.registers.h = self.inc_u8_reg(self.registers.h);
                4
            }
            0x2A => {
                let add = self.registers.get_hl();
                self.registers.a = self.mmu.fetch_byte(add, &self.state);
                let new_hl = add.wrapping_add(1);
                self.registers.set_hl(new_hl);
                8
            }
            0x20 => {
                // JR NZ, i8
                if !self.registers.is_zero_flag_high() {
		    let offset = self.fetch_byte() as i8;
		    if offset < 0 {
			self.pc = self.pc.wrapping_sub(offset.abs() as u16);
		    }else{
			self.pc = self.pc.wrapping_sub(offset as u16);
		    }
                    return 12;
                }
                self.pc += 1;
                8
            }
            0x21 => {
                // LD HL, U16
                let word = self.fetch_word();
                self.registers.set_hl(word);
                12
            }
            0x23 => {
                // INC HL
                let new_hl = self.registers.get_hl().wrapping_add(1);
                self.registers.set_hl(new_hl);
                8
            }
            0x28 => {
		if self.registers.is_zero_flag_high() {
		    let offset = self.fetch_byte() as i8;
		    if offset < 0 {
			self.pc = self.pc.wrapping_sub(offset.abs() as u16);
		    }else{
			self.pc = self.pc.wrapping_sub(offset as u16);
		    }
		    return 12;
		}
		self.pc += 1;
		8
            }
            0x31 => {
                // LD SP, U16
                self.sp = self.fetch_word();
                12
            }
            0x32 => {
                // ld (hl-), A
                self.mmu.write_byte(
                    &mut self.state,
                    self.registers.get_hl().into(),
                    self.registers.a,
                );
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                4
            }
            0x3D => {
                self.registers.a = self.dec_u8_reg(self.registers.a);
                4
            }
            0x3E => {
                // LD A, u8
                self.registers.a = self.fetch_byte();
                8
            }
            0x44 => {
                self.registers.b = self.registers.h;
                4
            }
            0x47 => {
                self.registers.b = self.registers.a;
                4
            }
            0x4F => {
                // LD C,A
                self.registers.c = self.registers.a;
                4
            }
            0x57 => {
                self.registers.d = self.registers.a;
                4
            }
            0x67 => {
                self.registers.h = self.registers.a;
                4
            }
            0x77 => {
                // LD (hl), A
                self.mmu.write_byte(
                    &mut self.state,
                    self.registers.get_hl().into(),
                    self.registers.a,
                );
                8
            }
            0x78 => {
                self.registers.a = self.registers.b;
                4
            }
            0x7B => {
                self.registers.a = self.registers.e;
                4
            }
            0x7C => {
                // LD A, u8
                self.registers.a = self.registers.h;
                4
            }
            0x90 => {
                self.registers.a = self.sub_u8_reg(self.registers.b);
                4
            }
            0xAF => {
                // XOR A
                self.registers.a ^= self.registers.a;
                self.registers.unset_flags();
                4
            }
            0xBC => {
                // POP BC
                let new_bc = self.pop_u16_from_stack();
                self.registers.set_bc(new_bc);
                12
            }
            0xC1 => {
                let popped_value = self.pop_u16_from_stack();
                self.registers.set_bc(popped_value);
                12
            }
            0xC3 => {
                let address = self.fetch_word();
                self.pc = address;
                16
            }
            0xC5 => {
                // PUSH BC
                self.push_u16_to_stack(self.registers.get_bc());
                16
            }
            0xC9 => {
                // RET
                self.pc = self.pop_u16_from_stack();
                16
            }
            0xCD => {
                // CALL nn
                let new_address = self.fetch_word();
                self.push_u16_to_stack(self.pc);
                self.pc = new_address;
                24
            }
            0xE0 => {
                // LD ($FF00+u8), A
                let address: u16 = 0xFF00 + (self.fetch_byte() as u16);
                self.mmu
                    .write_byte(&mut self.state, address as i32, self.registers.a);
                12
            }
            0xE2 => {
                // LD (FF00 + C), A
                self.mmu.write_byte(
                    &mut self.state,
                    0xFF + self.registers.c as i32,
                    self.registers.a,
                );
                8
            }
            0xEA => {
                let address = self.fetch_word();
                self.mmu
                    .write_byte(&mut self.state, address as i32, self.registers.a);
                16
            }
            0xF0 => {
                let add_on = self.fetch_byte() as u16;
                self.registers.a = self.mmu.fetch_byte(0xFF00u16 + add_on, &self.state);
                12
            }
            0xF2 => {
                self.registers.a = self
                    .mmu
                    .fetch_byte(0xFF00u16.wrapping_add(self.registers.c as u16), &self.state);
                8
            }
	    0xF3 => {
		self.ime = false;
		4
	    }
            0xFE => {
                let number = self.fetch_byte();
                self.cp(number);
                8
            }
            _ => panic!(
                "Instruction {:x?} not implemented",
                first_byte.to_be_bytes()
            ),
        }
    }

    pub(crate) fn execute_cb(&mut self) -> i32 {
        let instruction = self.fetch_byte();

        // Print state of emulator to logger
        log::info!(
            "A: {:X}, F: {:X}, B: {:X}, C: {:X}, D: {:X}, E: {:X}, H: {:X}, L: {:X}, address: {:X}, instr: CB-{:X}",
            self.registers.a,
	    self.registers.f,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
	    self.registers.h,
	    self.registers.l,
            self.pc - 1,
	    instruction
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

    fn sub_u8_reg(&mut self, reg: u8) -> u8 {
        self.registers.set_carry_flag(self.registers.a < reg);
        let result = self.registers.a.wrapping_sub(reg);
        self.registers.set_zero_flag(self.registers.a == 0);
        self.registers.set_was_prev_instr_sub(true);
        self.registers
            .set_half_carry(((self.registers.a ^ reg ^ result) & 0x10) > 0);
        result
    }

    // Generic implementation for CP A, x opcode group
    fn cp(&mut self, value: u8) {
        let result = self.registers.a.wrapping_sub(value);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_was_prev_instr_sub(true);
        self.registers
            .set_carry_flag(((self.registers.a ^ value ^ result) & 0x10) > 0);
    }

    fn inc_u8_reg(&mut self, reg: u8) -> u8 {
        self.registers.set_half_carry((reg & 0x0F) as u16 + 1 > 0x0F);
        let new_reg = reg.wrapping_add(1);
        self.registers.set_zero_flag(new_reg == 0);
        self.registers.set_was_prev_instr_sub(false);
        new_reg
    }

    fn dec_u8_reg(&mut self, reg: u8) -> u8 {
        let new_reg = reg.wrapping_sub(1);
        self.registers.set_zero_flag(new_reg == 0);
        self.registers.set_half_carry((reg & 0x0F) == 0);
        self.registers.set_was_prev_instr_sub(true);
        new_reg
    }

    fn push_u16_to_stack(&mut self, value_to_push: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.mmu
            .write_byte(&mut self.state, self.sp as i32, (value_to_push >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.mmu
            .write_byte(&mut self.state, self.sp as i32, value_to_push as u8);
    }

    fn pop_u16_from_stack(&mut self) -> u16 {
        let lower_byte = self.mmu.fetch_byte(self.sp, &self.state);
        self.sp = self.sp.wrapping_add(1);
        let high_byte = self.mmu.fetch_byte(self.sp, &self.state);
        self.sp = self.sp.wrapping_add(1);
        (high_byte as u16) << 8 | lower_byte as u16
    }
}
