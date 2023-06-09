use log::LevelFilter;
use log4rs::{append::file::FileAppender, Config, encode::pattern::PatternEncoder, config::{Appender, Root}};
 
use crate::{cpu, mbc::{self, no_mbc::NoMbc, mbc1::Mbc1}, mmu::Mmu};
use crate::custom_errors::UnableToOpenSelectedFileError;

pub struct GameBoyApp {
    cpu: Option<cpu::Cpu>,
    current_rom_path: Option<String>,
}

impl GameBoyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
	Self {current_rom_path: None}
    }

    // TODO make this use results for the various types of errors
    // Tries to load the selected rom
    fn load_rom(&mut self) -> Option<cpu::Cpu> {
	let mut total_rom = Vec::new();

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
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        #[cfg(not(target_arch = "wasm32"))]
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
		    // Open rom button
                    if ui.button("Open Rom").clicked() {
			if let Some(path) = rfd::FileDialog::new().add_filter("game-boy-filter", &["gb", "gbc"]).pick_file() {
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
        egui::CentralPanel::default().show(ctx, |ui| {
	    // Show the game window here

        });

        if true {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
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
