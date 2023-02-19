use crate::mmu::Mmu;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::env;

mod cpu;
mod timer;
mod cpu_registers;
mod interrupt_handler;
mod joypad;
mod mmu;
mod ppu;
mod serial;

use log::LevelFilter;
use raylib::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    rl.set_target_fps(60);

    // Configure logging
    let logfile = FileAppender::builder()
        .append(false)
        .encoder(Box::new(PatternEncoder::new("{m}\n")))
        .build("log/output.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();

    let mut cpu = cpu::Cpu::new(cpu::CpuState::NonBoot);

    if let Some(rom_path) = args.get(1) {
	// There was a rom path, try to load it
        let loading_was_sucessful = mmu.load_rom(rom_path.to_owned());

	if !loading_was_sucessful {
	    panic!("It wasn't possible to load the rom {}", rom_path);
	}
    } else {
	// There wasn't a loaded rom, do whatever you like
    };

    while !rl.window_should_close() {
        // Update joypad input state
        // TODO: Make this request a joypad interrupt
        cpu.mmu.joypad.update_input(&mut rl, &mut mmu.interrupt_handler);

        // run 69905 t-cycles of cpu work, equating to 4MHz of t-cycles per second
        let mut ran_cycles = 0;
        while ran_cycles < 69905 {
            ran_cycles += cpu.cycle();
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::PURPLE);
    }
}
