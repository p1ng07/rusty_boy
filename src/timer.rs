use crate::interrupt_handler::{InterruptHandler, Interrupt};
use crate::cpu::CpuState;

// Represents the gameboy timer
// delta_cycles: used to count the number of t-cycles elapsed since the last timer update, timer_counter updates every 4 delta_cycles
// divider: current timer counter, this always increments
// timer_counter: timer counter that increments at the rate specified in the tac register
// timer_module: reset value for timer_counter
// tac: timer control and timer_modulo speed specifier:
//     Bit  2   - Timer Enable
//     Bits 1-0 - Input Clock Select
//            00: CPU Clock / 1024 (DMG, SGB2, CGB Single Speed Mode:   4096 Hz, SGB1:   ~4194 Hz, CGB Double Speed Mode:   8192 Hz)
//            01: CPU Clock / 16   (DMG, SGB2, CGB Single Speed Mode: 262144 Hz, SGB1: ~268400 Hz, CGB Double Speed Mode: 524288 Hz)
//            10: CPU Clock / 64   (DMG, SGB2, CGB Single Speed Mode:  65536 Hz, SGB1:  ~67110 Hz, CGB Double Speed Mode: 131072 Hz)
//            11: CPU Clock / 256  (DMG, SGB2, CGB Single Speed Mode:  16384 Hz, SGB1:
#[derive(Default)]
pub struct Timer {
    delta_cycles_div: i32, 
    delta_cycles_tima : i32, 
    pub divider: u8,
    timer_counter: u8,
    timer_modulo: u8,
    timer_control: u8, 
}

impl Timer {
    pub fn step(&mut self, cpu_state: &CpuState, elapsed_cycles: i32, interrupt_handler: &mut InterruptHandler){
	// Div registers gets incremented every 256 t-cycles
	self.delta_cycles_div += elapsed_cycles;
	if self.delta_cycles_div >= 256 && *cpu_state != CpuState::Stopped {
	    self.divider = self.divider.wrapping_add(1);
	    self.delta_cycles_div -= 256;
	}

	// Define the frequency of the timer counter based on the lower 2 bits of tac
	let threshold = match self.timer_control & 0b0000_0011 {
	    0x0 => 1024,  // 4096 Hz
	    0x1 => 16,    // 262144 Hz
	    0x2 => 64,    // 65536 Hz
	    0x3 => 256,   // 16384 Hz
	    _ => 64, // It won't get to this
	};

	// If the timer bit is not high, then the time_counter should not count
	if self.timer_control & 0x4 != 1 {
	    return;
	}

	self.delta_cycles_tima += elapsed_cycles;
	if self.delta_cycles_tima >= threshold {
	    self.delta_cycles_tima -= threshold;
	    let added_timer_counter = self.timer_counter.wrapping_add(1);

	    // Check for overflow
	    if added_timer_counter < self.timer_counter {
		// There was an overflow, fire interrupt and reset to timer_modulo
		interrupt_handler.request_interrupt(Interrupt::Timer);
		self.timer_counter = self.timer_modulo;
	    } else {
		// There was no oveflow, just add normally
		self.timer_counter = added_timer_counter;
	    }

	}
    }

    pub fn read_byte(&self, address: u16) -> u8 {
	match address {
	    0xFF04 => self.divider,
	    0xFF05 => self.timer_counter,
	    0xFF06 => self.timer_modulo,
	    0xFF07 => self.timer_control,
	    _ => 0xFF,
	}
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
	match address {
	    0xFF04 => self.divider = 0,
	    0xFF05 => self.timer_counter = byte,
	    0xFF06 => self.timer_modulo = byte,
	    0xFF07 => self.timer_control = byte,
	    _ => (),
	}
    }
}

