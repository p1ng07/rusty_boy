use std::{
    ops::{Sub, Mul},
    time::{Duration, Instant},
};

use egui::{Ui, Image};
use eframe::Frame;
use epaint::{ColorImage, Color32, ImageDelta, Vec2};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

use crate::cpu;
use crate::mbc::{mbc1::Mbc1, no_mbc::NoMbc, Mbc};
use crate::mmu::Mmu;

const IMAGE_WIDTH: usize = 160;
const IMAGE_HEIGHT: usize = 144;

pub struct GameBoyApp {
    cpu: Option<cpu::Cpu>,
    gameImage: ColorImage,
    paused: bool,
    current_rom_path: Option<String>,
}

impl GameBoyApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        init_file_logger();

        Self {
            paused: false,
            cpu: None,
            current_rom_path: None,
	    gameImage:ColorImage::new([IMAGE_WIDTH,IMAGE_HEIGHT], Color32::BLACK)
        }
    }

    // TODO make this use results for the various types of errors
    // Tries to load the selected rom
    fn load_rom(&mut self) -> Option<cpu::Cpu> {
        let total_rom: Vec<u8>;

        match std::fs::read(&self.current_rom_path.clone().unwrap()) {
            Ok(byte_vec) => total_rom = byte_vec,
            Err(_) => return None,
        };

        let mbc_type_code = total_rom[0x147];

        let mbc = match mbc_type_code {
            0 => Box::new(NoMbc::new(total_rom)) as Box<dyn Mbc>,
            1 | 2 | 3 => Box::new(Mbc1::new(total_rom)) as Box<dyn Mbc>,
            _ => {
                println!("Mbc with code {:X} is not yet implemented", mbc_type_code);
                return None;
            }
        };

        let mmu = Mmu::new(mbc);
        let cpu = cpu::Cpu::new(false, mmu);

	println!("loading rom");

        Some(cpu)
    }

    fn run_frame(&mut self, ui: &Ui) {
	let cpu = match self.cpu.as_mut() {
	    Some(x) => x,
	    None => return
	};

	cpu.mmu
	    .joypad
	    .update_input(ui, &mut cpu.interrupt_handler);

	// run 69905 t-cycles of cpu work per frame, equating to 4MHz of t-cycles per second
	let mut ran_cycles = 0;
	while ran_cycles <= 70224 {
	    ran_cycles += cpu.cycle();
	}

	// TODO Update current game image with the newly rendered ppu image inside self.cpu.mmu.ppu
    }
}

impl eframe::App for GameBoyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Get the time at which a game update should happen
        let deadline = std::time::Instant::now()
            .checked_add(Duration::from_micros(16600u64))
            .unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // Open rom button
                    if ui.button("Open Rom").clicked() {
                        let picked_path = rfd::FileDialog::new()
                            .add_filter("*.gb, *.gbc", &["gb", "gbc"])
                            .pick_file();
                        if let Some(path) = picked_path {
                            self.current_rom_path = Some(path.display().to_string());
                            self.cpu = self.load_rom();
                        }
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.toggle_value(&mut self.paused, "Pause");

            if self.paused {
                if ui.button("Step Frame").clicked() {
                    match self.cpu.as_mut() {
                        Some(cpu) => {
                            self.run_frame(ui);
                        }
                        None => (),
                    }
                }
                if ui.button("Step PC").clicked() {
                    if let Some(cpu) = self.cpu.as_mut() {
                        cpu.cycle();
                    }
                }
            }
        });

	egui::Window::new("Game window")
	    .default_size(Vec2::new(300f32, 100f32))
	    .show(ctx, |ui| {


            if self.paused || self.cpu.is_none() {
		render_game_window(ctx, ui);
                return;
            };

            if let Some(cpu) = self.cpu.as_mut() {
                self.run_frame(ui);
            };

	    render_game_window(ctx, ui);
            // TODO: render game window here
        });

        // Update the context after 16.6 ms (forcing the fps to be 60)
        ctx.request_repaint_after(deadline.sub(Instant::now()));
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
}

fn render_game_window(ctx: &egui::Context, ui: &mut Ui) {
    // Create the main black image
    let image = ColorImage::new([IMAGE_WIDTH,IMAGE_HEIGHT], Color32::BLACK);

    // Change a sub-square of that image
    let delta = ImageDelta::partial([0,0], ColorImage::new([144,144], Color32::WHITE), egui::TextureOptions::default());

    let tex = egui::Context::load_texture(ctx, "main_image", image, egui::TextureOptions::default());
    
    // Change the texture using the created imageDelta
    ctx.tex_manager().write().set(tex.id(), delta);
    ui.add(egui::Image::new(&tex, tex.size_vec2()));
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
