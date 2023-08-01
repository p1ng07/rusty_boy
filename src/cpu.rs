use strum::IntoEnumIterator;
use serde::{Serialize, Deserialize};

use crate::cpu_registers::CpuRegisters;
use crate::interrupt_handler::*;
use crate::mmu::Mmu;
use crate::ppu::PpuModes;

#[derive(PartialEq, Serialize, Deserialize)]
pub enum CpuState {
    NonBoot,
    Stopped,
    DMA,
    Halt,
}

// Emulates the core cpu, is responsible for decoding instructions and executing them
// it "drives the whole system", basically the cpu is what ticks the other components
// and makes them do stuff, like the ppu or the timer
#[derive(Serialize, Deserialize)]
pub struct Cpu {
    state: CpuState,
    pub mmu: Mmu,
    pc: u16,
    sp: u16,
    pub interrupt_handler: InterruptHandler,
    registers: CpuRegisters,
    delta_t_cycles: i32, // t-cycles performed in the current instruction
    halt_bug: bool,
    enable_interrupts_next_tick: bool,

    /* Used to count 2 'fast' double speed cycles
    Every time we tick 1 'fast' m-cycle this gets incremented
    And every 2 increments of this we tick the ppu by 1 'normal' m-cycle
    Because the ppu does not run on double frequency like the cpu or timer
     */
    double_speed_delta_counter: u8,
    //
}

// Instructions and cb-prefixed instructions are on separate files
mod cb_instructions;
mod instructions;

impl Cpu {
    pub fn new(mmu: Mmu) -> Cpu {
        let mut cpu = Cpu {
            pc: 0,
            sp: 0,
            mmu,
            state: CpuState::NonBoot,
            delta_t_cycles: 0,
            registers: CpuRegisters::default(),
            interrupt_handler: InterruptHandler::new(),
            halt_bug: false,
            enable_interrupts_next_tick: false,
            double_speed_delta_counter: 0,
        };

        initialize_cpu_state_defaults(&mut cpu);
        cpu
    }

    // Cycle the cpu once, fetch an instruction and run it, returns the number of t-cycles it took to run it
    pub fn cycle(&mut self) -> i32 {
        // Print state of emulator to logger
        self.log_to_file();

        if self.state == CpuState::Halt {
            // Check for halt bug right after halt is executed and before the cpu ticks
            if !self.interrupt_handler.enabled && self.interrupt_handler.is_interrupt_pending() {
                self.state = CpuState::NonBoot;
                self.halt_bug = true;
            }

            self.tick();

            // If there are interrupts pending, and it is possible to service them, disable halt mode
            if self.interrupt_handler.is_interrupt_pending() {
                self.state = CpuState::NonBoot;
            }

            if self.interrupt_handler.enabled && self.interrupt_handler.IE > 0 {
                self.handle_interrupts();
            }

            let instruction_delta_t_cycles = self.delta_t_cycles;
            self.delta_t_cycles = 0;
            return instruction_delta_t_cycles;
        }

        let first_byte = self.fetch_byte_pc();

        // Cycle timing is done mid-instruction (i.e. inside the
        // instructions match statement using a self.tick() function
        // to tick the machine 1 m-cycle forward)

        self.execute(first_byte);

        // Service interrupts
        if self.interrupt_handler.enabled && self.interrupt_handler.IE > 0 {
            self.handle_interrupts();
        }

        let instruction_delta_t_cycles = self.delta_t_cycles;
        self.delta_t_cycles = 0;
        instruction_delta_t_cycles
    }

    // Transfers one byte of data if a OAM DMA is active
    fn tick_dma(&mut self) {
        if self.state != CpuState::DMA {
            return;
        }
        if self.mmu.dma_iterator > 159 {
            self.state = CpuState::NonBoot;
            self.mmu.dma_iterator = 0;
            return;
        }
        let address: u16 = ((self.mmu.dma_source as u16) << 8) | self.mmu.dma_iterator as u16;
        let dma_byte = self.mmu.fetch_byte(address, &mut self.interrupt_handler);
        let destination = self.mmu.dma_iterator as usize;
        self.mmu.ppu.oam_ram[destination] = dma_byte;
        self.mmu.dma_iterator += 1;
    }

    // Transfers 16 bytes of information if a HDMA is active
    fn tick_hdma(&mut self){
	if !self.mmu.hdma_controller.is_active {
	    return;
	}

	self.mmu.hdma_controller.length -= 1;

	for _ in 0..16 {
	    let byte = self.mmu.fetch_byte(self.mmu.hdma_controller.iterator_hdma, &mut self.interrupt_handler);
	    self.mmu.ppu.write_vram(self.mmu.hdma_controller.destination_hdma, byte);

	    self.mmu.hdma_controller.iterator_hdma += 1;
	    self.mmu.hdma_controller.destination_hdma += 1;
	}

	if self.mmu.hdma_controller.length == 0 {
	    // Transfer has terminated
	    self.mmu.hdma_controller.is_active = false;
	    self.mmu.hdma_controller.length = 0xFF;
	}
    }

    // Ticks every component by 4 t-cycles
    fn tick(&mut self) {
        self.delta_t_cycles += 4;

        // While in double speed, the ppu operates at it's normal frequency
        // If the cpu is in double speed mode, every 8 'fast' t-cycles, we cycle the ppu by 4 t-cycles
        // If the cpu is in normal speed mode, just tick 4 t-cycles for every 4 t-cycles
        if !is_bit_set(self.mmu.key1, 7)
            || (is_bit_set(self.mmu.key1, 7) && self.double_speed_delta_counter % 2 == 0)
        {
	    // Record if the ppu was already in hblank
	    let ppu_was_in_hblank = match self.mmu.ppu.mode {
		PpuModes::HBlank => true,
		_ => false
	    };

	    // Components that aren't affected by cpu double speed
            self.mmu.ppu.tick(&mut self.interrupt_handler);
            self.mmu.ppu.tick(&mut self.interrupt_handler);
            self.mmu.ppu.tick(&mut self.interrupt_handler);
            self.mmu.ppu.tick(&mut self.interrupt_handler);
	    
	    // self.mmu.mbc.tick();

	    if let PpuModes::HBlank = self.mmu.ppu.mode {
		// If the ppu wasn't in hblank at the start of the tick
		// and is in hblank now, perform a hdma tick
		if !ppu_was_in_hblank {
		    self.tick_hdma();
		}  
	    };
        }

	// Components that are affected by cpu double speed
        // Advance the double speed delta counter by 1 m-cycle
        self.double_speed_delta_counter = self.double_speed_delta_counter.wrapping_add(1);

	self.tick_dma();
	
        self.mmu
            .timer
            .tick(&mut self.interrupt_handler);

	// TODO take this out of here
        // Delayed EI instruction
        if self.enable_interrupts_next_tick {
            self.interrupt_handler.enabled = true;
            self.enable_interrupts_next_tick = false;
        }
    }

    fn fetch_byte_pc(&mut self) -> u8 {
        let byte = self.mmu.fetch_byte(self.pc, &mut self.interrupt_handler);
        self.tick();

        self.pc = self.pc.wrapping_add(1);
        if self.halt_bug {
            self.pc -= 1;
            self.halt_bug = false;
        }

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
        let lower_byte = self.mmu.fetch_byte(self.sp, &mut self.interrupt_handler);
        self.sp = self.sp.wrapping_add(1);
        self.tick();
        let high_byte = self.mmu.fetch_byte(self.sp, &mut self.interrupt_handler);
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
        // log::info!(
        //     "A:{} F:{} B:{} C:{} D:{} E:{} H:{} L:{} SP:{} PC:{} PCMEM:{},{},{},{}",
        //     format!("{:0>2X}", self.registers.a),
        //     format!("{:0>2X}", self.registers.f),
        //     format!("{:0>2X}", self.registers.b),
        //     format!("{:0>2X}", self.registers.c),
        //     format!("{:0>2X}", self.registers.d),
        //     format!("{:0>2X}", self.registers.e),
        //     format!("{:0>2X}", self.registers.h),
        //     format!("{:0>2X}", self.registers.l),
        //     format!("{:0>4X}", self.sp),
        //     format!("{:0>4X}", self.pc - 1),
        //     format!("{:02X}", instruction),
        //     format!("{:02X}", self.mmu.fetch_byte(self.pc, &self.state)),
        //     format!("{:02X}", self.mmu.fetch_byte(self.pc + 1, &self.state)),
        //     format!("{:02X}", self.mmu.fetch_byte(self.pc + 2, &self.state))
        // );
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
                self.mmu.fetch_byte(self.pc, &mut self.interrupt_handler)
            ),
            format!(
                "{:02X}",
                self.mmu
                    .fetch_byte(self.pc + 1, &mut self.interrupt_handler)
            ),
            format!(
                "{:02X}",
                self.mmu
                    .fetch_byte(self.pc + 2, &mut self.interrupt_handler)
            ),
            format!(
                "{:02X}",
                self.mmu
                    .fetch_byte(self.pc + 3, &mut self.interrupt_handler)
            ),
        );
    }
}

fn initialize_cpu_state_defaults(cpu: &mut Cpu) {
    cpu.registers.a = 0x11;
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

pub fn is_bit_set(num: u8, bit_index: u8) -> bool {
    ((num >> bit_index) & 1) > 0
}
