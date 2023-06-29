use super::Cpu;
use crate::cpu::is_bit_set;

impl Cpu {
    pub(crate) fn execute_cb(&mut self) {
        let instruction = self.fetch_byte_pc();

        match instruction {
            0x00 => self.registers.b = self.rlc(self.registers.b),
            0x01 => self.registers.c = self.rlc(self.registers.c),
            0x02 => self.registers.d = self.rlc(self.registers.d),
            0x03 => self.registers.e = self.rlc(self.registers.e),
            0x04 => self.registers.h = self.rlc(self.registers.h),
            0x05 => self.registers.l = self.rlc(self.registers.l),
            0x06 => {
                let mut byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
                byte = self.rlc(byte);
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x07 => self.registers.a = self.rlc(self.registers.a),
            0x08 => self.registers.b = self.rrc(self.registers.b),
            0x09 => self.registers.c = self.rrc(self.registers.c),
            0x0A => self.registers.d = self.rrc(self.registers.d),
            0x0B => self.registers.e = self.rrc(self.registers.e),
            0x0C => self.registers.h = self.rrc(self.registers.h),
            0x0D => self.registers.l = self.rrc(self.registers.l),
            0x0E => {
                let mut byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
                byte = self.rrc(byte);
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x0F => self.registers.a = self.rrc(self.registers.a),
            0x10 => self.registers.b = self.rl(self.registers.b),
            0x11 => self.registers.c = self.rl(self.registers.c),
            0x12 => self.registers.d = self.rl(self.registers.d),
            0x13 => self.registers.e = self.rl(self.registers.e),
            0x14 => self.registers.h = self.rl(self.registers.h),
            0x15 => self.registers.l = self.rl(self.registers.l),
            0x16 => {
                let mut byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
                byte = self.rl(byte);
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x17 => self.registers.a = self.rl(self.registers.a),
            0x18 => self.registers.b = self.rr(self.registers.b),
            0x19 => self.registers.c = self.rr(self.registers.c),
            0x1A => self.registers.d = self.rr(self.registers.d),
            0x1B => self.registers.e = self.rr(self.registers.e),
            0x1C => self.registers.h = self.rr(self.registers.h),
            0x1D => self.registers.l = self.rr(self.registers.l),
            0x1E => {
                let mut byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
                byte = self.rr(byte);
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x1F => self.registers.a = self.rr(self.registers.a),
            0x20 => self.registers.b = self.sla(self.registers.b),
            0x21 => self.registers.c = self.sla(self.registers.c),
            0x22 => self.registers.d = self.sla(self.registers.d),
            0x23 => self.registers.e = self.sla(self.registers.e),
            0x24 => self.registers.h = self.sla(self.registers.h),
            0x25 => self.registers.l = self.sla(self.registers.l),
            0x26 => {
                let mut byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
                byte = self.sla(byte);
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x27 => self.registers.a = self.sla(self.registers.a),
            0x28 => self.registers.b = self.sra(self.registers.b),
            0x29 => self.registers.c = self.sra(self.registers.c),
            0x2A => self.registers.d = self.sra(self.registers.d),
            0x2B => self.registers.e = self.sra(self.registers.e),
            0x2C => self.registers.h = self.sra(self.registers.h),
            0x2D => self.registers.l = self.sra(self.registers.l),
            0x2E => {
                let mut byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
                byte = self.sra(byte);
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x2F => self.registers.a = self.sra(self.registers.a),
            0x30 => self.registers.b = self.swap(self.registers.b),
            0x31 => self.registers.c = self.swap(self.registers.c),
            0x32 => self.registers.d = self.swap(self.registers.d),
            0x33 => self.registers.e = self.swap(self.registers.e),
            0x34 => self.registers.h = self.swap(self.registers.h),
            0x35 => self.registers.l = self.swap(self.registers.l),
            0x36 => {
                let mut byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
                byte = self.swap(byte);
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x37 => self.registers.a = self.swap(self.registers.a),
            0x38 => self.registers.b = self.srl(self.registers.b),
            0x39 => self.registers.c = self.srl(self.registers.c),
            0x3A => self.registers.d = self.srl(self.registers.d),
            0x3B => self.registers.e = self.srl(self.registers.e),
            0x3C => self.registers.h = self.srl(self.registers.h),
            0x3D => self.registers.l = self.srl(self.registers.l),
            0x3E => {
                let mut byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
                byte = self.srl(byte);
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x3F => self.registers.a = self.srl(self.registers.a),
            0x40 => self.bit(self.registers.b, 0),
            0x41 => self.bit(self.registers.c, 0),
            0x42 => self.bit(self.registers.d, 0),
            0x43 => self.bit(self.registers.e, 0),
            0x44 => self.bit(self.registers.h, 0),
            0x45 => self.bit(self.registers.l, 0),
            0x46 => {
                let reg = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.bit(reg, 0);
                self.tick();
            }
            0x47 => self.bit(self.registers.a, 0),
            0x48 => self.bit(self.registers.b, 1),
            0x49 => self.bit(self.registers.c, 1),
            0x4A => self.bit(self.registers.d, 1),
            0x4B => self.bit(self.registers.e, 1),
            0x4C => self.bit(self.registers.h, 1),
            0x4D => self.bit(self.registers.l, 1),
            0x4E => {
                let reg = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.bit(reg, 1);
                self.tick();
            }
            0x4F => self.bit(self.registers.a, 1),
            0x50 => self.bit(self.registers.b, 2),
            0x51 => self.bit(self.registers.c, 2),
            0x52 => self.bit(self.registers.d, 2),
            0x53 => self.bit(self.registers.e, 2),
            0x54 => self.bit(self.registers.h, 2),
            0x55 => self.bit(self.registers.l, 2),
            0x56 => {
                let reg = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.bit(reg, 2);
                self.tick();
            }
            0x57 => self.bit(self.registers.a, 2),
            0x58 => self.bit(self.registers.b, 3),
            0x59 => self.bit(self.registers.c, 3),
            0x5A => self.bit(self.registers.d, 3),
            0x5B => self.bit(self.registers.e, 3),
            0x5C => self.bit(self.registers.h, 3),
            0x5D => self.bit(self.registers.l, 3),
            0x5E => {
                let reg = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.bit(reg, 3);
                self.tick();
            }
            0x5F => self.bit(self.registers.a, 3),
            0x60 => self.bit(self.registers.b, 4),
            0x61 => self.bit(self.registers.c, 4),
            0x62 => self.bit(self.registers.d, 4),
            0x63 => self.bit(self.registers.e, 4),
            0x64 => self.bit(self.registers.h, 4),
            0x65 => self.bit(self.registers.l, 4),
            0x66 => {
                let reg = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.bit(reg, 4);
                self.tick();
            }
            0x67 => self.bit(self.registers.a, 4),
            0x68 => self.bit(self.registers.b, 5),
            0x69 => self.bit(self.registers.c, 5),
            0x6A => self.bit(self.registers.d, 5),
            0x6B => self.bit(self.registers.e, 5),
            0x6C => self.bit(self.registers.h, 5),
            0x6D => self.bit(self.registers.l, 5),
            0x6E => {
                let reg = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.bit(reg, 5);
                self.tick();
            }
            0x6F => self.bit(self.registers.a, 5),
            0x70 => self.bit(self.registers.b, 6),
            0x71 => self.bit(self.registers.c, 6),
            0x72 => self.bit(self.registers.d, 6),
            0x73 => self.bit(self.registers.e, 6),
            0x74 => self.bit(self.registers.h, 6),
            0x75 => self.bit(self.registers.l, 6),
            0x76 => {
                let reg = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.bit(reg, 6);
                self.tick();
            }
            0x77 => self.bit(self.registers.a, 6),
            0x78 => self.bit(self.registers.b, 7),
            0x79 => self.bit(self.registers.c, 7),
            0x7A => self.bit(self.registers.d, 7),
            0x7B => self.bit(self.registers.e, 7),
            0x7C => self.bit(self.registers.h, 7),
            0x7D => self.bit(self.registers.l, 7),
            0x7E => {
                let reg = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                );
                self.bit(reg, 7);
                self.tick();
            }
            0x7F => self.bit(self.registers.a, 7),
            0x80 => self.registers.b &= !(1 << 0),
            0x81 => self.registers.c &= !(1 << 0),
            0x82 => self.registers.d &= !(1 << 0),
            0x83 => self.registers.e &= !(1 << 0),
            0x84 => self.registers.h &= !(1 << 0),
            0x85 => self.registers.l &= !(1 << 0),
            0x86 => {
                let byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) & !(1 << 0);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x87 => self.registers.a &= !(1 << 0),
            0x88 => self.registers.b &= !(1 << 1),
            0x89 => self.registers.c &= !(1 << 1),
            0x8A => self.registers.d &= !(1 << 1),
            0x8B => self.registers.e &= !(1 << 1),
            0x8C => self.registers.h &= !(1 << 1),
            0x8D => self.registers.l &= !(1 << 1),
            0x8E => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) & !(1 << 1);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x8F => self.registers.a &= !(1 << 1),
            0x90 => self.registers.b &= !(1 << 2),
            0x91 => self.registers.c &= !(1 << 2),
            0x92 => self.registers.d &= !(1 << 2),
            0x93 => self.registers.e &= !(1 << 2),
            0x94 => self.registers.h &= !(1 << 2),
            0x95 => self.registers.l &= !(1 << 2),
            0x96 => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) & !(1 << 2);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x97 => self.registers.a &= !(1 << 2),
            0x98 => self.registers.b &= !(1 << 3),
            0x99 => self.registers.c &= !(1 << 3),
            0x9A => self.registers.d &= !(1 << 3),
            0x9B => self.registers.e &= !(1 << 3),
            0x9C => self.registers.h &= !(1 << 3),
            0x9D => self.registers.l &= !(1 << 3),
            0x9E => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) & !(1 << 3);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0x9F => self.registers.a &= !(1 << 3),
            0xA0 => self.registers.b &= !(1 << 4),
            0xA1 => self.registers.c &= !(1 << 4),
            0xA2 => self.registers.d &= !(1 << 4),
            0xA3 => self.registers.e &= !(1 << 4),
            0xA4 => self.registers.h &= !(1 << 4),
            0xA5 => self.registers.l &= !(1 << 4),
            0xA6 => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) & !(1 << 4);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xA7 => self.registers.a &= !(1 << 4),
            0xA8 => self.registers.b &= !(1 << 5),
            0xA9 => self.registers.c &= !(1 << 5),
            0xAA => self.registers.d &= !(1 << 5),
            0xAB => self.registers.e &= !(1 << 5),
            0xAC => self.registers.h &= !(1 << 5),
            0xAD => self.registers.l &= !(1 << 5),
            0xAE => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) & !(1 << 5);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xAF => self.registers.a &= !(1 << 5),
            0xB0 => self.registers.b &= !(1 << 6),
            0xB1 => self.registers.c &= !(1 << 6),
            0xB2 => self.registers.d &= !(1 << 6),
            0xB3 => self.registers.e &= !(1 << 6),
            0xB4 => self.registers.h &= !(1 << 6),
            0xB5 => self.registers.l &= !(1 << 6),
            0xB6 => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) & !(1 << 6);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xB7 => self.registers.a &= !(1 << 6),
            0xB8 => self.registers.b &= !(1 << 7),
            0xB9 => self.registers.c &= !(1 << 7),
            0xBA => self.registers.d &= !(1 << 7),
            0xBB => self.registers.e &= !(1 << 7),
            0xBC => self.registers.h &= !(1 << 7),
            0xBD => self.registers.l &= !(1 << 7),
            0xBE => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) & !(1 << 7);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xBF => self.registers.a &= !(1 << 7),
            0xC0 => self.registers.b |= 1 << 0,
            0xC1 => self.registers.c |= 1 << 0,
            0xC2 => self.registers.d |= 1 << 0,
            0xC3 => self.registers.e |= 1 << 0,
            0xC4 => self.registers.h |= 1 << 0,
            0xC5 => self.registers.l |= 1 << 0,
            0xC6 => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) | (1 << 0);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xC7 => self.registers.a |= 1 << 0,
            0xC8 => self.registers.b |= 1 << 1,
            0xC9 => self.registers.c |= 1 << 1,
            0xCA => self.registers.d |= 1 << 1,
            0xCB => self.registers.e |= 1 << 1,
            0xCC => self.registers.h |= 1 << 1,
            0xCD => self.registers.l |= 1 << 1,
            0xCE => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) | (1 << 1);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xCF => self.registers.a |= 1 << 1,
            0xD0 => self.registers.b |= 1 << 2,
            0xD1 => self.registers.c |= 1 << 2,
            0xD2 => self.registers.d |= 1 << 2,
            0xD3 => self.registers.e |= 1 << 2,
            0xD4 => self.registers.h |= 1 << 2,
            0xD5 => self.registers.l |= 1 << 2,
            0xD6 => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) | (1 << 2);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xD7 => self.registers.a |= 1 << 2,
            0xD8 => self.registers.b |= 1 << 3,
            0xD9 => self.registers.c |= 1 << 3,
            0xDA => self.registers.d |= 1 << 3,
            0xDB => self.registers.e |= 1 << 3,
            0xDC => self.registers.h |= 1 << 3,
            0xDD => self.registers.l |= 1 << 3,
            0xDE => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) | (1 << 3);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xDF => self.registers.a |= 1 << 3,
            0xE0 => self.registers.b |= 1 << 4,
            0xE1 => self.registers.c |= 1 << 4,
            0xE2 => self.registers.d |= 1 << 4,
            0xE3 => self.registers.e |= 1 << 4,
            0xE4 => self.registers.h |= 1 << 4,
            0xE5 => self.registers.l |= 1 << 4,
            0xE6 => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) | (1 << 4);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xE7 => self.registers.a |= 1 << 4,
            0xE8 => self.registers.b |= 1 << 5,
            0xE9 => self.registers.c |= 1 << 5,
            0xEA => self.registers.d |= 1 << 5,
            0xEB => self.registers.e |= 1 << 5,
            0xEC => self.registers.h |= 1 << 5,
            0xED => self.registers.l |= 1 << 5,
            0xEE => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) | (1 << 5);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xEF => self.registers.a |= 1 << 5,
            0xF0 => self.registers.b |= 1 << 6,
            0xF1 => self.registers.c |= 1 << 6,
            0xF2 => self.registers.d |= 1 << 6,
            0xF3 => self.registers.e |= 1 << 6,
            0xF4 => self.registers.h |= 1 << 6,
            0xF5 => self.registers.l |= 1 << 6,
            0xF6 => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) | (1 << 6);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xF7 => self.registers.a |= 1 << 6,
            0xF8 => self.registers.b |= 1 << 7,
            0xF9 => self.registers.c |= 1 << 7,
            0xFA => self.registers.d |= 1 << 7,
            0xFB => self.registers.e |= 1 << 7,
            0xFC => self.registers.h |= 1 << 7,
            0xFD => self.registers.l |= 1 << 7,
            0xFE => {
                let received_byte = self.mmu.fetch_byte(
                    self.registers.get_hl(),
                    &self.state,
                    &mut self.interrupt_handler,
                ) | (1 << 7);
                self.tick();
                self.mmu.write_byte(
                    self.registers.get_hl(),
                    received_byte,
                    &mut self.state,
                    &mut self.interrupt_handler,
                );
                self.tick();
            }
            0xFF => self.registers.a |= 1 << 7,
        };
    }

    fn rr(&mut self, mut reg: u8) -> u8 {
        let old_carry = self.registers.is_carry_flag_high();
        let new_carry = is_bit_set(reg, 1);
        self.registers.set_carry_flag(new_carry);
        reg >>= 1;
        reg |= (old_carry as u8) << 7;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn rlc(&mut self, mut reg: u8) -> u8 {
        let carry = is_bit_set(reg, 7);
        self.registers.set_carry_flag(carry);
        reg = (reg << 1) | if carry { 1 } else { 0 };
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn rrc(&mut self, mut reg: u8) -> u8 {
        let carry = is_bit_set(reg, 1);
        self.registers.set_carry_flag(carry);
        reg = (reg >> 1) | (carry as u8) << 7;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn rl(&mut self, mut reg: u8) -> u8 {
        let carry = self.registers.is_carry_flag_high() as u8;
        self.registers.set_carry_flag(is_bit_set(reg, 7));

        reg <<= 1;
        reg &= 0xFE;
        reg |= carry as u8;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn sla(&mut self, mut reg: u8) -> u8 {
        let carry = is_bit_set(reg, 7);
        self.registers.set_carry_flag(carry);
        reg <<= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn sra(&mut self, mut reg: u8) -> u8 {
        let signal = reg & 0x80;
        self.registers.set_carry_flag(is_bit_set(reg, 1));
        reg >>= 1;
        reg &= 0b01111111;
        reg |= signal;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn srl(&mut self, mut reg: u8) -> u8 {
        self.registers.set_carry_flag(is_bit_set(reg, 1));
        reg >>= 1;
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_n_flag(false);
        reg
    }

    fn swap(&mut self, reg: u8) -> u8 {
        self.registers.set_zero_flag(reg == 0);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);
        self.registers.set_n_flag(false);
        let high = reg & 0xF0;
        let low = reg & 0x0F;
        low << 4 | high >> 4
    }

    fn bit(&mut self, reg: u8, bit_index: u8) {
        if (reg & (1 << bit_index)) == 0 {
            self.registers.set_zero_flag(true);
        } else {
            self.registers.set_zero_flag(false);
        }
        self.registers.set_n_flag(false);
        self.registers.set_half_carry_flag(true);
    }
}
