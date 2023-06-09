use log::LevelFilter;
use log4rs::{append::file::FileAppender, Config, encode::pattern::PatternEncoder, config::{Appender, Root}};
 
use crate::{cpu, mbc::{self, no_mbc::NoMbc, mbc1::Mbc1}, mmu::Mmu};

pub struct GameBoyApp {
    cpu: Option<cpu::Cpu>,
    current_rom_path: Option<String>,
}

impl GameBoyApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
	#[cfg(not(target_arch = "wasm32"))]
	init_file_logger();

	Self {
	    cpu: None,
	    current_rom_path: None
	}
    }

    // TODO make this use results for the various types of errors
    // Tries to load the selected rom
    fn load_rom(&mut self) -> Option<cpu::Cpu> {
	let total_rom: Vec<u8>;

	match std::fs::read(&self.current_rom_path.clone().unwrap()) {
	    Ok(byte_vec) => total_rom = byte_vec,
	    Err(_) => return None
	};

	let mbc_type_code = total_rom[0x147];
	
	let mbc = match mbc_type_code {
	    0 => Box::new(NoMbc::new(total_rom)) as Box<dyn mbc::Mbc>,
	    1 | 2 | 3 => Box::new(Mbc1::new(total_rom)) as Box<dyn mbc::Mbc>,
	    _ => {
		println!("Mbc with code {:X} is not yet implemented", mbc_type_code);
		return None
	    }
	};

	let mmu = Mmu::new(mbc);
	let cpu = cpu::Cpu::new(cpu::CpuState::NonBoot, mmu);

	Some(cpu)
    }
}

impl eframe::App for GameBoyApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

	#[cfg(not(target_arch = "wasm32"))]
	egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
	    egui::menu::bar(ui, |ui| {
		ui.menu_button("File", |ui| {
		    // Open rom button
		    if ui.button("Open Rom").clicked() {
			let picked_path = rfd::FileDialog::new().add_filter("*.gb, *.gbc", &["gb", "gbc"]).pick_file();
			if let Some(path) = picked_path {
			    self.current_rom_path = Some(path.display().to_string());
			    self.cpu = self.load_rom();
			}
		    }
		    if ui.button("Quit").clicked() {
			_frame.close();
		    }
		});
	    });
	});

	egui::SidePanel::left("side_panel").show(ctx, |ui| {
	    ui.heading("Side Panel");
	});

	// TODO: Game window
	// This is going to be the game window
	egui::CentralPanel::default().show(ctx, |_ui| {
	});

	egui::Window::new("Window").show(ctx, |ui| {
	    if self.cpu.is_none() { return; };

	    match self.cpu.as_mut() {
		Some(cpu) => run_frame(cpu),
		None => (),
	    };

	    if ui.input(|i| i.key_pressed(egui::Key::A)) {
		println!("Ohhh yap");
	    }

	});
    }
}

fn run_frame(cpu: &mut cpu::Cpu) {
    // cpu.mmu
    // 	.joypad
    // 	.update_input(&mut rl, &mut cpu.mmu.interrupt_handler);

    // run 69905 t-cycles of cpu work, equating to 4MHz of t-cycles per second
    let mut ran_cycles = 0;
    while ran_cycles < 69905 {
	ran_cycles += cpu.cycle();
    }
}

fn init_file_logger() {
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
}
