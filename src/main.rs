// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    use rusty_boy::game_app::GameBoyApp;

    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(GameBoyApp::new(cc))),
    )
}

// TODO: compiling to web
// #[cfg(target_arch = "wasm32")]
// fn main() {
//     // Make sure panics are logged using `console.error`.
//     console_error_panic_hook::set_once();

//     // Redirect tracing to console.log and friends:
//     tracing_wasm::set_as_global_default();

//     let web_options = eframe::WebOptions::default();

//     wasm_bindgen_futures::spawn_local(async {
//         eframe::start_web(
//             "game_id", // hardcode it
//             web_options,
//             Box::new(|cc| Box::new(eframe_template::GameApp::new(cc))),
//         )
//         .await
//         .expect("failed to start eframe");
//     });
// }

// fn main() {
//     let args: Vec<String> = env::args().collect();
// TODO: Get rid of raylib
//     let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
//     rl.set_target_fps(60);

//     // Construct memory bank controller of game
//     let mut total_rom = Vec::new();
//     if let Some(rom_path) = args.get(1){
// 	total_rom = std::fs::read(&rom_path).unwrap_or_else(|_err| panic!("Rom {} does not exist.", rom_path));
//     }

//     let mbc_type_code = total_rom[0x147];
    
//     let mbc = match mbc_type_code {
// 	0 => Box::new(NoMbc::new(total_rom)) as Box<dyn mbc::Mbc>,
// 	1 | 2 | 3 => Box::new(Mbc1::new(total_rom)) as Box<dyn mbc::Mbc>,
// 	_ => panic!("Mbc with code {:X} is not yet implemented", mbc_type_code)
//     };

//     let mmu = Mmu::new(mbc);

//     let mut cpu = cpu::Cpu::new(cpu::CpuState::NonBoot, mmu);

//     while !rl.window_should_close() {
//         cpu.mmu
//             .joypad
//             .update_input(&mut rl, &mut cpu.mmu.interrupt_handler);

//         // run 69905 t-cycles of cpu work, equating to 4MHz of t-cycles per second
//         let mut ran_cycles = 0;
//         while ran_cycles < 69905 {
//             ran_cycles += cpu.cycle();
//         }
//         let mut d = rl.begin_drawing(&thread);
// 	d.clear_background(Color::PINK);
//     }
// }
