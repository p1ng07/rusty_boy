use crate::mmu::Mmu;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use mbc::mbc1::Mbc1;
use mbc::no_mbc::NoMbc;
use raylib::prelude::*;
use std::env;

mod mbc;
mod mmu;
mod cpu;
mod cpu_registers;
mod interrupt_handler;
mod joypad;
mod ppu;
mod serial;
mod timer;

use log::LevelFilter;

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

    // Construct memory bank controller of game
    let mut total_rom = Vec::new();
    if let Some(rom_path) = args.get(1){
	total_rom = std::fs::read(&rom_path).unwrap_or_else(|_err| panic!("Rom {} does not exist.", rom_path));
    }

    let mbc_type_code = total_rom[0x147];
    
    let mbc = match mbc_type_code {
	0 => Box::new(NoMbc::new(total_rom)) as Box<dyn mbc::Mbc>,
	1 | 2 | 3 => Box::new(Mbc1::new(total_rom)) as Box<dyn mbc::Mbc>,
	_ => panic!("Mbc with code {:X} is not yet implemented", mbc_type_code)
    };

    let mmu = Mmu::new(mbc);

    let mut cpu = cpu::Cpu::new(cpu::CpuState::NonBoot, mmu);

    while !rl.window_should_close() {
        cpu.mmu
            .joypad
            .update_input(&mut rl, &mut cpu.mmu.interrupt_handler);

        // run 69905 t-cycles of cpu work, equating to 4MHz of t-cycles per second
        let mut ran_cycles = 0;
        while ran_cycles < 69905 {
            ran_cycles += cpu.cycle();
        }
        let mut d = rl.begin_drawing(&thread);
	d.clear_background(Color::PINK);
    }
}
