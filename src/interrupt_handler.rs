use strum_macros::EnumIter;

// This macro makes it possible to iterate through every variant of the interrupt
#[derive(EnumIter)]
pub enum Interrupt {
    VBLANK,
    STAT,
    TIMER,
    SERIAL,
    JOYPAD,
}

impl Interrupt {
    pub fn mask(&self) -> u8 {
        match *self {
            Self::VBLANK => 0x1,
            Self::STAT => 0x2,
            Self::TIMER => 0x4,
            Self::SERIAL => 0x8,
            Self::JOYPAD => 0x10,
        }
    }

    pub fn jump_vector(&self) -> u16 {
        match *self {
            Self::VBLANK => 0x40,
            Self::STAT => 0x48,
            Self::TIMER => 0x50,
            Self::SERIAL => 0x58,
            Self::JOYPAD => 0x60,
        }
    }
}

#[derive(Default)]
pub struct InterruptHandler {
    pub enabled: bool,
    pub IF: u8, // Interrupts flags
    pub IE: u8, // Interrupts enable
}

impl InterruptHandler {
    // Changes the IF register depending on which interrupt was requested
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.IF |= interrupt.mask();
    }

    // Enables the given interrupt in the IE register
    pub fn enable_interrupt(&mut self, interrupt: Interrupt) {
        self.IE |= interrupt.mask();
    }

    // Disables the interrupt in the IF register
    pub(crate) fn unrequest_interrupt(&mut self, interrupt_type: &Interrupt) {
        self.IF = self.IF ^ interrupt_type.mask();
    }
}
