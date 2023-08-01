use serde::{Serialize, Deserialize};

use crate::cpu::CpuState;
use crate::interrupt_handler::{Interrupt, InterruptHandler};

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
#[derive(Default, Serialize, Deserialize)]
pub struct Timer {
    delta_cycles_tima: i32,
    pub divider: u16,
    pub tima: u8,
    tma: u8,
    tima_was_reloaded: bool,
    pub timer_control: u8,
    reload_tima_next_m_cycle: bool
}

impl Timer {
    pub fn tick(&mut self, interrupt_handler: &mut InterruptHandler) {
        // Define the frequency of the timer counter based on the lower 2 bits of tac
        let threshold = self.get_mask_of_timer_frequency();

	let threshold_bit_was_one = threshold & self.divider > 0;

	self.divider = self.divider.wrapping_add(4);

        // If the timer bit is not high, then the time_counter should not count
        if self.timer_control & 0x4 == 0 {
            return;
        }

	self.tima_was_reloaded ^= self.tima_was_reloaded;
	
	if self.reload_tima_next_m_cycle {
	    self.tima = self.tma;
	    self.tima_was_reloaded = true;
	    self.reload_tima_next_m_cycle = false;
	    interrupt_handler.request_interrupt(Interrupt::Timer);
	}


	if self.divider & threshold == 0 && threshold_bit_was_one {
	    let (added_timer_counter, overflow) = self.tima.overflowing_add(1);

	    self.reload_tima_next_m_cycle = overflow;
	    self.tima = added_timer_counter;
	};
    }

    fn get_mask_of_timer_frequency(&mut self) -> u16 {
        let threshold = match self.timer_control & 0b0000_0011 {
            0x0 => 1 << 9, // 4096 Hz
            0x1 => 1 << 3,   // 262144 Hz
            0x2 => 1 << 5,   // 65536 Hz
            0x3 => 1 << 7,  // 16384 Hz
            _ => 64,     // It won't get to this
        };
        threshold
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.divider >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.timer_control,
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0xFF04 => {
		// Write div
		self.divider = 0;
	    },
            0xFF05 => {
		// Write tima

		// If tima was reloaded on this cycle, ignore the write
		if self.tima_was_reloaded {
		    self.tima_was_reloaded = false;
		    return;
		}

		self.tima = byte;
		
		// If tima was te be reloaded and the interrupt fired, ignore both of those
		self.reload_tima_next_m_cycle ^= self.reload_tima_next_m_cycle;
	    },
            0xFF06 => {
		// Write tma
		self.tma = byte;

		// If tima was reloaded on this cycle, write to tima also
		if self.tima_was_reloaded {
		    self.tima = byte;
		}
	    },
            0xFF07 => {
		// Write tac
		self.timer_control = byte;
	    },
            _ => (),
        }
    }
}
