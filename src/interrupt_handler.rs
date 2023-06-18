use strum_macros::EnumIter;

// This macro makes it possible to iterate through every variant of the interrupt
#[derive(EnumIter)]
pub enum Interrupt {
    Vblank,
    Stat,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    pub fn mask(&self) -> u8 {
        match *self {
            Self::Vblank => 0x1,
            Self::Stat => 0x2,
            Self::Timer => 0x4,
            Self::Serial => 0x8,
            Self::Joypad => 0x10,
        }
    }

    pub fn jump_vector(&self) -> u16 {
        match *self {
            Self::Vblank => 0x40,
            Self::Stat => 0x48,
            Self::Timer => 0x50,
            Self::Serial => 0x58,
            Self::Joypad => 0x60,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Default)]
pub struct InterruptHandler {
    pub enabled: bool,
    pub IF: u8, // Interrupts flags
    pub IE: u8, // Interrupts enable
}

#[allow(dead_code)]
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
        self.IF ^= interrupt_type.mask();
    }
}
