use egui::{TextureFilter, TextureOptions, Ui, RichText};
use epaint::{Color32, ColorImage};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use std::{fs::File, io::ErrorKind, time::{Duration, Instant}};
use std::io::prelude::*;

use crate::cpu::{self, Cpu};
use crate::mbc::{mbc1::Mbc1, no_mbc::NoMbc, Mbc};
use crate::mmu::Mmu;
use crate::{
    constants::{GAMEBOY_HEIGHT, GAMEBOY_WIDTH},
    cpu::is_bit_set,
    mbc::{mbc3::Mbc3, mbc5::Mbc5},
};

pub struct GameBoyApp {
    cpu: Option<cpu::Cpu>,
    paused: bool,
    current_rom_path: Option<String>,
    game_framebuffer: [Color32; GAMEBOY_HEIGHT * GAMEBOY_WIDTH],
    game_window_open: bool,
    game_is_in_double_speed: bool
}

impl GameBoyApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // #[cfg(not(target_arch = "wasm32"))]
        // init_file_logger();

        Self {
            paused: false,
            cpu: None,
            current_rom_path: None,
            game_framebuffer: [Color32::WHITE; GAMEBOY_HEIGHT * GAMEBOY_WIDTH],
            game_window_open: true,
	    game_is_in_double_speed: false
        }
    }

    // TODO make this use results for the various types of errors
    // Tries to load the selected rom
    fn load_rom(&mut self) -> Option<cpu::Cpu> {
        let total_rom = match std::fs::read(&self.current_rom_path.clone().unwrap()) {
            Ok(byte_vec) => byte_vec,
            Err(_) => return None,
        };

        // IF true, the game supports gbc enhancements
        // IF false, the game is DMG only and needs
        // a default palette
        let is_dmg_game = total_rom[0x143] & 0x80 == 0;

        // TODO make these code checks length safe
        let mbc_type_code = total_rom[0x147];

        let mbc = match mbc_type_code {
            0 => Box::new(NoMbc::new(total_rom)) as Box<dyn Mbc>,
            1 | 2 | 3 => Box::new(Mbc1::new(total_rom)) as Box<dyn Mbc>,
            0xF..=0x13 => Box::new(Mbc3::new(total_rom)) as Box<dyn Mbc>,
            0x19..=0x1E => Box::new(Mbc5::new(total_rom)) as Box<dyn Mbc>,
            _ => {
                println!("Mbc with code {:X} is not yet implemented", mbc_type_code);
                return None;
            }
        };

        let mmu = Mmu::new(mbc, is_dmg_game);
        let cpu = cpu::Cpu::new(mmu);

        Some(cpu)
    }

    fn run_frame(&mut self, ui: &Ui) {
        let cpu = match self.cpu.as_mut() {
            Some(x) => x,
            None => return,
        };

        cpu.mmu.joypad.update_input(ui, &mut cpu.interrupt_handler);

        // run 70225 t-cycles of cpu work per frame, equating to 4MHz of t-cycles per second
        let mut ran_cycles = 0;

        let cycle_limit = 70225 * if is_bit_set(cpu.mmu.key1, 7) { 2 } else { 1 };
        // Run a frame of cpu clocks, if the cpu is in double speed mode, run double those cycles
        while ran_cycles < cycle_limit {
            ran_cycles += cpu.cycle();
        }

        self.game_framebuffer = cpu.mmu.ppu.current_framebuffer;
    }

    // Returns an image containing the game frame
    fn render_game_frame(&self, ctx: &egui::Context, ui: &mut Ui) -> egui::Image {
        // Create the main black image
        let mut image = ColorImage::new([GAMEBOY_WIDTH, GAMEBOY_HEIGHT], Color32::BLUE);

        // Print the current framebuffer
        image.pixels = self.game_framebuffer.to_vec();

        // Change the texture using the created imageDelta
        // ctx.tex_manager().write().set(tex.id(), delta);
        let mut size = egui::Vec2::new(image.size[0] as f32, image.size[1] as f32);

        // Make the image sharper
        let mut texture_options = TextureOptions::default();
        texture_options.magnification = TextureFilter::Nearest;
        texture_options.minification = TextureFilter::Nearest;

        let tex = egui::Context::load_texture(ctx, "main_image", image, texture_options);

        size *= (ui.available_width() / size.x).max(0.4);
	size = ui.available_size();
        // ui.image(&tex, size);
	egui::Image::new(&tex, size)
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
	// Check if shift is pressed, if so, run in double speed
	self.game_is_in_double_speed = ctx.input(|i| {
	    i.modifiers.shift
	});

	if ctx.input(|ui| ui.key_pressed(egui::Key::Space)) {
	    self.paused = !self.paused;
	}

	if ctx.input(|ui| ui.modifiers.ctrl && ui.key_pressed(egui::Key::S)) {
	    save_state(&self.cpu);
	}
	if ctx.input(|ui| ui.modifiers.ctrl && ui.key_pressed(egui::Key::O)) {
	    self.open_rom();
	}

	if ctx.input(|ui| ui.modifiers.ctrl && ui.key_pressed(egui::Key::L)) {
	    match load_state() {
		Some(cpu) => self.cpu = Some(cpu),
		None => (),
	    }
	}
    }

    fn open_rom(&mut self) {
        let picked_path = rfd::FileDialog::new()
		        .set_title("Open rom")
		        .add_filter("*.gb, *.gbc", &["gb", "gbc"])
		        .pick_file();
        if let Some(path) = picked_path {
		        self.current_rom_path = Some(path.display().to_string());
		        self.cpu = self.load_rom();
	            }
    }

    // fn dump_vram(&self, vram: [u8; 0xA0]) {
    //     for index in (0..vram.len()).step_by(16) {
    //         let number = 0x8000u16 + index.to_owned() as u16;
    //         log::info!(
    // 		"{:X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
    // 		number,
    // 		vram[index.to_owned() as usize],
    // 		vram[1 + index.to_owned() as usize],
    // 		vram[2+ index.to_owned()  as usize],
    // 		vram[3+ index.to_owned()  as usize],
    // 		vram[4+ index.to_owned()  as usize],
    // 		vram[5+ index.to_owned()  as usize],
    // 		vram[6+ index.to_owned()  as usize],
    // 		vram[7+ index.to_owned()  as usize],
    // 		vram[8+ index.to_owned()  as usize],
    // 		vram[9+ index.to_owned()  as usize],
    // 		vram[10+ index.to_owned()  as usize],
    // 		vram[11+ index.to_owned()  as usize],
    // 		vram[12+ index.to_owned()  as usize],
    // 		vram[13+ index.to_owned()  as usize],
    // 		vram[14+ index.to_owned()  as usize],
    // 		vram[15+ index.to_owned()  as usize],
    // 	    );
    //     }
    // }
}

impl eframe::App for GameBoyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Get the time at which a game update should happen
        let deadline = std::time::Instant::now()
            .checked_add(Duration::from_micros(16600u64))
            .unwrap();

	// Handle input
	self.handle_input(ctx);

        #[cfg(not(target_arch = "wasm32"))]
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
		ui.menu_button("File", |ui| {
		    // Open rom button
		    if ui.add(egui::Button::new("Open rom").shortcut_text("Ctrl-O")).clicked() {
			self.open_rom();
		    }

		    // Show save state button if a cpu is loaded and button is clicked
		    if self.cpu.is_some() &&
			ui.add(egui::Button::new("Save State").shortcut_text("Ctrl-S")).clicked()
		    {
			save_state(&self.cpu);
		    }

		    if ui.add(egui::Button::new("Load State").shortcut_text("Ctrl-L")).clicked(){
			match load_state() {
			    Some(cpu) => self.cpu = Some(cpu),
			    None => (),
			}
		    }

		    ui.horizontal(|ui| {
			if ui.button("Quit").clicked() {
			    frame.close();
			}
		    })

		}); // End of "File" menu

		// Display pause menu 
		if self.cpu.is_some() {
		    ui.menu_button("Pause menu", |ui| {
			ui.toggle_value(&mut self.paused, "Pause");

			if self.paused {
			    if ui.button("Step Frame").clicked() {
				if let Some(_) = self.cpu {
				    self.run_frame(ui);
				}
			    }
			    if ui.button("Step PC").clicked() {
				if let Some(cpu) = self.cpu.as_mut() {
				    cpu.cycle();
				}
			    }
			}
		    });

		    // Display a helper to tell the player that the gme is in double speed
		    if self.game_is_in_double_speed {
		        ui.label(RichText::new("Speed: 2x").color(Color32::LIGHT_BLUE));
		    }
		    if self.paused {
		        ui.label(RichText::new("Paused").color(Color32::LIGHT_BLUE));
		    }

		}
		
            });
        });


        if self.cpu.is_some() {
            let frame = egui::Frame::default().inner_margin(egui::Margin::default());
	    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
		if self.cpu.is_some() {
		    if !self.paused {
			self.run_frame(ui);

			// If the game is in double speed, run two frames
			if self.game_is_in_double_speed {
			    self.run_frame(ui);
			}
		    }

		    let game_image = self.render_game_frame(ctx, ui);
		    ui.add(game_image);
		};
	    });
	}
        // Update the context after 16.6 ms (forcing the fps to be 60)
        ctx.request_repaint_after(deadline.duration_since(Instant::now()));
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
    }
}

fn load_state() -> Option<Cpu> {
    let picked_path = rfd::FileDialog::new()
	.add_filter("sav files", &["gbsave"])
	.pick_file();

    if let Some(path) = picked_path {
	let total_rom: Vec<u8>;

	match std::fs::read(path) {
	    Ok(byte_vec) => total_rom = byte_vec,
	    Err(_) => return None,
	};

	match bincode::deserialize(&total_rom){
	    Ok(cpu) => Some(cpu),
	    Err(_) => None
	}

    }else {
	return None;
    }
}

// TODO cleanup, this code is awful and it should use results or something, really hurts to look at it
fn save_state(cpu: &Option<cpu::Cpu>) {
    let cpu = match cpu.as_ref() {
        Some(x) => x,
	None => return
    };

    let save = bincode::serialize(cpu);

    let save_file_path = rfd::FileDialog::new()
	.set_file_name(".gbsave")
	.save_file();
    if save_file_path.is_none() {
	return;
    }

    // TODO handle existing files
    match File::create(save_file_path.unwrap()) {
	Ok(mut file) => file.write_all(&(save.unwrap())),
	Err(e) => return,
    };
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
