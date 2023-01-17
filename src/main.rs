use std::env;

use cpu::cpu_registers::*;
use macroquad::prelude::*;

mod cpu;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut cpu = match args.get(1) {
	None => panic!("Não foi passada uma ROM para usar"),
	Some(path) => cpu::Cpu::new(path.to_owned())
    };

    for i in 0..0xFFF {
	println!("primeiro opcode {}", format!("{:X}", cpu.fetch()));
    }

    // loop {
    //     clear_background(RED);

    //     draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
    //     draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
    //     draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

    //     draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

    //     next_frame().await
    // }
}
