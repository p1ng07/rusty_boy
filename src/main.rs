use std::env;

mod cpu;
mod cpu_registers;
mod joypad;
mod mmu;
mod ppu;

use joypad::Joypad;
use raylib::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    rl.set_target_fps(60);

    let mut cpu = match args.get(1) {
        None => panic!("Não foi passada uma ROM para usar"),
        Some(path) => cpu::Cpu::new(path.to_owned()),
    };

    let mut joypad: Joypad = Default::default();

    while !rl.window_should_close() {
        // run 69905 t-cycles of cpu work, equating to 4MHz of t-cycles per second
        let mut ran_cycles = 0;

        cpu._update_input(&mut rl);
        while ran_cycles < 69905 {
            ran_cycles += cpu.cycle();
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::PURPLE);
    }
}
