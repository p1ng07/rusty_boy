use crate::mmu::Mmu;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::env;

mod cpu;
mod cpu_registers;
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

    let mut mmu = match args.get(1) {
        Some(path_to_rom) => Mmu::new(path_to_rom.clone()),
        None => Mmu::new(String::from("")),
    };

    while !rl.window_should_close() {
        // Update input register
        mmu.joypad.update_input(&mut rl);

        // run 69905 t-cycles of cpu work, equating to 4MHz of t-cycles per second
        let mut ran_cycles = 0;
        while ran_cycles < 69905 {
            ran_cycles += cpu.cycle(&mut mmu);
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::PURPLE);
    }
}
