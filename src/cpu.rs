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
    pub mmu: Mmu,
    pc: u16,
    sp: u16,
    registers: CpuRegisters,
    delta_t_cycles: i32, // t-cycles performed in the current instruction
}

mod cb_instructions;
mod instructions;

impl Cpu {
    pub fn new(initial_state: CpuState, mmu: Mmu) -> Cpu {
        let mut cpu = Cpu {
            pc: 0,
            sp: 0,
            mmu,
            state: initial_state,
            delta_t_cycles: 0,
            registers: CpuRegisters::default(),
        };

        // Skip the bootrom, and go straight to running the program
        if cpu.state == CpuState::NonBoot {
            initialize_cpu_state_defaults(&mut cpu);
        }
        cpu
    }

    // Cycle the cpu once, fetch an instruction and run it, returns the number of t-cycles it took to run it
    pub fn cycle(&mut self) -> i32 {
        let first_byte = self.fetch_byte();
        let instruction_delta_t_cycles = self.delta_t_cycles;

        // Cycle timing is done mid-instruction (i.e. is inside the instructions match statement using a self.tick() function to tick the machine 1 m-cycle forward)
        self.execute(first_byte);

        // TODO: Update the timers
        // self.mmu
        //     .timer
        //     .step(&self.state, delta_cycles, &mut self.mmu.interrupt_handler);

        self.handle_interrupts();

        self.delta_t_cycles = 0;
        instruction_delta_t_cycles
    }

    // Ticks every component by 4 t-cycles
    fn tick(&mut self) {
        self.delta_t_cycles += 4;
        self.mmu
            .timer
            .step(&self.state, 4, &mut self.mmu.interrupt_handler);
    }

    fn fetch_byte(&mut self) -> u8 {
        self.tick();
        let byte = self.mmu.fetch_byte(self.pc, &self.state);
        self.pc += 1;
        byte
    }

    pub fn fetch_word(&mut self) -> u16 {
        let fetch_byte_big = self.fetch_byte() as u16;
        let fetch_byte_small = self.fetch_byte() as u16;

        fetch_byte_small << 8 | fetch_byte_big
    }

    // Services all serviciable interrupts and returns the number of t-cycles this handling took
    fn handle_interrupts(&mut self) {
        if !self.mmu.interrupt_handler.enabled || self.mmu.interrupt_handler.IE == 0 {
            // It isn't possible to service any interrupt
            return;
        }

        // Go through every interrupt possible interrupt in order of priority (bit order ex: vblank is highest priority)
        // Check if it is requested and enabled, if it is then service it
        // IMPORTANT: This iterator uses the order in which the variants are set in the enum, therefore respecting the interrupt order
        for interrupt_type in Interrupt::iter() {
            if interrupt_type.mask() & self.mmu.interrupt_handler.IF > 0
                && interrupt_type.mask() & self.mmu.interrupt_handler.IE > 0
                && self.mmu.interrupt_handler.enabled
            {
                // Service interrupt, set ime to false and reset the respective IF bit on the handler
                self.mmu
                    .interrupt_handler
                    .unrequest_interrupt(&interrupt_type);

                // CALL interrupt_vector
                self.call(interrupt_type.jump_vector());

                // Disable IME
                self.mmu.interrupt_handler.enabled = false;
            }
        }
    }

    // Return from function stack, takes 3 m-cycles
    fn ret(&mut self) {
        self.pc = self.pop_u16_from_stack();
        self.tick();
    }
    // calls a sub routine, takes 3 m-cycles
    fn call(&mut self, address: u16) {
        self.push_u16_to_stack(self.pc);
        self.pc = address;
        self.tick();
    }

    fn push_u16_to_stack(&mut self, value_to_push: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.mmu
            .write_byte(self.sp, (value_to_push >> 8) as u8, &mut self.state);
        self.tick();
        self.sp = self.sp.wrapping_sub(1);
        self.mmu
            .write_byte(self.sp, value_to_push as u8, &mut self.state);
        self.tick();
    }

    fn pop_u16_from_stack(&mut self) -> u16 {
        self.tick();
        let lower_byte = self.mmu.fetch_byte(self.sp, &self.state);
        self.sp = self.sp.wrapping_add(1);
        self.tick();
        let high_byte = self.mmu.fetch_byte(self.sp, &self.state);
        self.sp = self.sp.wrapping_add(1);
        (high_byte as u16) << 8 | lower_byte as u16
    }

    fn log_to_file(&self, instruction: u8) {
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
