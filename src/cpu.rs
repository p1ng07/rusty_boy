use strum::IntoEnumIterator;

use crate::cpu_registers::CpuRegisters;
use crate::interrupt_handler::*;
use crate::mmu::Mmu;

#[derive(PartialEq)]
pub enum CpuState {
    Boot,
    NonBoot,
    Stopped,
    DMA,
    Halt,
}

// Emulates the core cpu, is responsible for decoding instructions and executing them
// it "drives the whole system", basically the cpu is what ticks the other components
// and makes them do stuff, like the ppu or the timer
pub struct Cpu {
    state: CpuState,
    pub mmu: Mmu,
    pc: u16,
    sp: u16,
    elapsed_dma_cycles: u8, // m-cycles that a dma transfer has been active
    pub interrupt_handler: InterruptHandler,
    registers: CpuRegisters,
    delta_t_cycles: i32, // t-cycles performed in the current instruction
}

// Instructions and cb-prefixed instructions are on separate files
mod cb_instructions;
mod instructions;

impl Cpu {
    // TODO find a better way to run or not to run bootrom when the cpu starts
    pub fn new(run_boot: bool, mmu: Mmu) -> Cpu {
        let mut cpu = Cpu {
            pc: 0,
            sp: 0,
            mmu,
            state: CpuState::Boot,
            delta_t_cycles: 0,
            registers: CpuRegisters::default(),
            interrupt_handler: InterruptHandler::default(),
            elapsed_dma_cycles: 0,
        };

        // Skip the bootrom, and go straight to running the program
        if !run_boot {
            initialize_cpu_state_defaults(&mut cpu);
        }
        cpu
    }

    // Cycle the cpu once, fetch an instruction and run it, returns the number of t-cycles it took to run it
    pub fn cycle(&mut self) -> i32 {
        // Print state of emulator to logger
        self.log_to_file();

        if self.state == CpuState::Halt {
            self.tick();
            // If there are interrupts pending, and it is possible to service them, disable halt mode
            if self.interrupt_handler.IF & self.interrupt_handler.IE > 0 {
                self.state = CpuState::NonBoot;
            }

            if self.interrupt_handler.enabled && self.interrupt_handler.IE > 0 {
                self.handle_interrupts();
            }

            let instruction_delta_t_cycles = self.delta_t_cycles;
            self.delta_t_cycles = 0;
            return instruction_delta_t_cycles;
        }

        if self.state == CpuState::DMA {
            let dma_byte = self.mmu.fetch_byte(
                self.mmu.dma_register,
                &self.state,
                &mut self.interrupt_handler,
            );
            self.mmu.dma_register = self.mmu.dma_register.wrapping_add(1);

            let destination = 0xFE00u16 | (self.mmu.dma_register & 0xF) as u16;

            // Write the dma transfer byte
            self.mmu.ppu.oam_ram[destination as usize] = dma_byte;

            self.elapsed_dma_cycles += 1;

            // A dma transfer lasts 160 m-cycles
            if self.elapsed_dma_cycles >= 160 {
                self.state = CpuState::NonBoot;
                self.elapsed_dma_cycles = 0;
            }
        }

        let first_byte = self.fetch_byte_pc();

        // Service interrupts
        if self.interrupt_handler.enabled && self.interrupt_handler.IE > 0 {
            self.handle_interrupts();
        }

        // Cycle timing is done mid-instruction (i.e. inside the
        // instructions match statement using a self.tick() function
        // to tick the machine 1 m-cycle forward)

        self.execute(first_byte);

        let instruction_delta_t_cycles = self.delta_t_cycles;
        self.delta_t_cycles = 0;
        instruction_delta_t_cycles
    }

    // Ticks every component by 4 t-cycles
    fn tick(&mut self) {
        self.delta_t_cycles += 4;
        self.mmu
            .timer
            .step(&self.state, &mut self.interrupt_handler);

        // Advance the ppu 4 dots
        self.mmu.ppu.tick(&mut self.interrupt_handler);
        self.mmu.ppu.tick(&mut self.interrupt_handler);
        self.mmu.ppu.tick(&mut self.interrupt_handler);
        self.mmu.ppu.tick(&mut self.interrupt_handler);
    }

    fn fetch_byte_pc(&mut self) -> u8 {
        let byte = self
            .mmu
            .fetch_byte(self.pc, &self.state, &mut self.interrupt_handler);
        self.tick();
        self.pc = self.pc.wrapping_add(1);

        byte
    }

    pub fn fetch_word(&mut self) -> u16 {
        let fetch_byte_lower = self.fetch_byte_pc() as u16;
        let fetch_byte_high = self.fetch_byte_pc() as u16;

        fetch_byte_high << 8 | fetch_byte_lower
    }

    // Services all serviciable interrupts and returns the number of t-cycles this handling took
    fn handle_interrupts(&mut self) {
        // Go through every interrupt possible interrupt in order of priority (bit order ex: vblank is highest priority)
        // Check if it is requested and enabled, if it is then service it
        // IMPORTANT: This iterator uses the order in which the variants are set in the enum, therefore respecting the interrupt order
        for interrupt_type in Interrupt::iter() {
            if interrupt_type.mask() & self.interrupt_handler.IF > 0
                && interrupt_type.mask() & self.interrupt_handler.IE > 0
                && self.interrupt_handler.enabled
            {
                // Service interrupt, set ime to false and reset the respective IF bit on the handler
                self.interrupt_handler.consume_interrupt(&interrupt_type);

                // CALL interrupt_vector
                self.push_u16_to_stack(self.pc);
                self.pc = interrupt_type.jump_vector();

                // Disable IME
                self.interrupt_handler.enabled = false;
            }
        }
    }

    // Return from function stack, takes 3 m-cycles
    fn ret(&mut self) {
        self.pc = self.pop_u16_from_stack();
        self.tick();
    }

    // calls a sub routine, takes 3 m-cycles
    fn call_u16(&mut self, condition: bool) {
        let address = self.fetch_word();
        if condition {
            self.rst(address);
        }
    }

    fn push_u16_to_stack(&mut self, value_to_push: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.mmu.write_byte(
            self.sp,
            (value_to_push >> 8) as u8,
            &mut self.state,
            &mut self.interrupt_handler,
        );
        self.tick();
        self.sp = self.sp.wrapping_sub(1);
        self.mmu.write_byte(
            self.sp,
            value_to_push as u8,
            &mut self.state,
            &mut self.interrupt_handler,
        );
        self.tick();
    }

    fn pop_u16_from_stack(&mut self) -> u16 {
        self.tick();
        let lower_byte = self
            .mmu
            .fetch_byte(self.sp, &self.state, &mut self.interrupt_handler);
        self.sp = self.sp.wrapping_add(1);
        self.tick();
        let high_byte = self
            .mmu
            .fetch_byte(self.sp, &self.state, &mut self.interrupt_handler);
        self.sp = self.sp.wrapping_add(1);
        (high_byte as u16) << 8 | lower_byte as u16
    }

    fn rst(&mut self, address: u16) {
        self.push_u16_to_stack(self.pc);
        self.pc = address;
        self.tick();
    }

    fn jp_u16(&mut self, condition: bool) {
        let address = self.fetch_word();
        if condition {
            self.pc = address;
            self.tick();
        }
    }

    fn jr_i8(&mut self, jump_condition: bool) {
        let offset = self.fetch_byte_pc() as i8;
        if jump_condition {
            self.pc = ((self.pc as i32) + (offset as i32)) as u16;
            self.tick();
        }
    }

    fn daa(&mut self) {
        if !self.registers.is_n_flag_high() {
            // Last instruction was a addition
            if self.registers.is_carry_flag_high() || self.registers.a > 0x99 {
                self.registers.a = self.registers.a.wrapping_add(0x60);
                self.registers.set_carry_flag(true);
            };
            if self.registers.is_half_carry_flag_high() || (self.registers.a & 0x0F) > 0x9 {
                self.registers.a = self.registers.a.wrapping_add(0x6);
            }
        } else {
            // Last instruction was a subtraction
            if self.registers.is_carry_flag_high() {
                self.registers.a = self.registers.a.wrapping_sub(0x60);
            };
            if self.registers.is_half_carry_flag_high() {
                self.registers.a = self.registers.a.wrapping_sub(0x6);
            }
        }
        self.registers.set_zero_flag(self.registers.a == 0);
        self.registers.set_half_carry_flag(false);
    }

    fn log_to_file(&mut self) {
        //     if self.pc < 0x100 {
        //         return;
        //     }
        //     // log::info!(
        //     //     "A:{} F:{} B:{} C:{} D:{} E:{} H:{} L:{} SP:{} PC:{} PCMEM:{},{},{},{}",
        //     //     format!("{:0>2X}", self.registers.a),
        //     //     format!("{:0>2X}", self.registers.f),
        //     //     format!("{:0>2X}", self.registers.b),
        //     //     format!("{:0>2X}", self.registers.c),
        //     //     format!("{:0>2X}", self.registers.d),
        //     //     format!("{:0>2X}", self.registers.e),
        //     //     format!("{:0>2X}", self.registers.h),
        //     //     format!("{:0>2X}", self.registers.l),
        //     //     format!("{:0>4X}", self.sp),
        //     //     format!("{:0>4X}", self.pc - 1),
        //     //     format!("{:02X}", instruction),
        //     //     format!("{:02X}", self.mmu.fetch_byte(self.pc, &self.state)),
        //     //     format!("{:02X}", self.mmu.fetch_byte(self.pc + 1, &self.state)),
        //     //     format!("{:02X}", self.mmu.fetch_byte(self.pc + 2, &self.state))
        //     // );
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
            format!("{:0>4X}", self.pc),
            format!(
                "{:02X}",
                self.mmu
                    .fetch_byte(self.pc, &self.state, &mut self.interrupt_handler)
            ),
            format!(
                "{:02X}",
                self.mmu
                    .fetch_byte(self.pc + 1, &self.state, &mut self.interrupt_handler)
            ),
            format!(
                "{:02X}",
                self.mmu
                    .fetch_byte(self.pc + 2, &self.state, &mut self.interrupt_handler)
            ),
            format!(
                "{:02X}",
                self.mmu
                    .fetch_byte(self.pc + 3, &self.state, &mut self.interrupt_handler)
            ),
        );
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
    cpu.state = CpuState::NonBoot;
    cpu.mmu.ppu.lcdc = 0b1000_0000;
}
