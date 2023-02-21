use super::Cpu;

impl Cpu {
    pub(crate) fn execute_cb(&mut self){
	#[macro_export]
	macro_rules! rl {
	    ($reg:expr) => {
		{
		}
	    };
	}
        let instruction = self.fetch_byte();

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
            format!("{:02X}", self.mmu.fetch_byte(self.pc, &self.state)),
            format!("{:02X}", self.mmu.fetch_byte(self.pc + 1, &self.state)),
            format!("{:02X}", self.mmu.fetch_byte(self.pc + 2, &self.state))
        );

        match instruction {
                0x10 => self.registers.b = self.rl(self.registers.b),
                0x11 => self.registers.c = self.rl(self.registers.c),
                0x12 => self.registers.d = self.rl(self.registers.d),
                0x13 => self.registers.e = self.rl(self.registers.e),
                0x14 => self.registers.h = self.rl(self.registers.h),
                0x15 => self.registers.l = self.rl(self.registers.l),
		0x16 => {
		    let _byte = self.mmu.fetch_byte(self.registers.get_hl(), &self.state);
		    let byte = self.rl(self.registers.c);
		    self.mmu.write_byte(self.registers.get_hl(), byte, &mut self.state);
		    self.tick();
		},
                0x17 => self.registers.a = self.rl(self.registers.a),
                0x7C => {
                    self.registers.set_zero_flag(self.registers.h < 128);
                }
                _ => panic!(
                    "CB prefixed instruction {:X?} was not implemented",
                    instruction.to_be_bytes()
                ),
	};
    }

    fn rl(&mut self, mut reg: u8) -> u8 {
	self.registers.set_carry_flag(reg & 0x80 > 0);
	reg <<= 1;
	self.registers.set_zero_flag(reg == 0);
	self.registers.set_half_carry_flag(false);
	self.registers.set_was_prev_instr_sub(false);
	reg
    }
}
