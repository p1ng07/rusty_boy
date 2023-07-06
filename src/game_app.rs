use egui::{TextureFilter, TextureOptions, Ui};
use epaint::{Color32, ColorImage};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

use crate::{constants::{GAMEBOY_HEIGHT, GAMEBOY_WIDTH}, mbc::{mbc3::Mbc3, mbc5::Mbc5}};
use crate::cpu;
use crate::mbc::{mbc1::Mbc1, no_mbc::NoMbc, Mbc};
use crate::mmu::Mmu;

pub struct GameBoyApp {
    cpu: Option<cpu::Cpu>,
    paused: bool,
    current_rom_path: Option<String>,
    game_framebuffer: [Color32; GAMEBOY_HEIGHT * GAMEBOY_WIDTH],
    game_window_open: bool,
    tile_viewer_open: bool,
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
            tile_viewer_open: false,
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
	    15..=19 => Box::new(Mbc3::new(total_rom)) as Box<dyn Mbc>,
	    0x19..=0x1E => Box::new(Mbc5::new(total_rom)) as Box<dyn Mbc>,
            _ => {
                println!("Mbc with code {:X} is not yet implemented", mbc_type_code);
                return None;
            }
        };

        let mmu = Mmu::new(mbc);
        let cpu = cpu::Cpu::new(false, mmu);

        Some(cpu)
    }

    fn run_frame(&mut self, ui: &Ui) {
        let cpu = match self.cpu.as_mut() {
            Some(x) => x,
            None => return,
        };

        cpu.mmu.joypad.update_input(ui, &mut cpu.interrupt_handler);

        // TODO Fix this timing, games run too fast
        // run 70225 t-cycles of cpu work per frame, equating to 4MHz of t-cycles per second
        let mut ran_cycles = 0;
        while ran_cycles < 70225 {
            ran_cycles += cpu.cycle();
        }

        self.game_framebuffer = cpu.mmu.ppu.current_framebuffer;
    }

    fn render_game_window(&self, ctx: &egui::Context, ui: &mut Ui) {
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
        // ui.image(&tex, size);
        ui.image(&tex, size);
    }

    fn dump_vram(&self, vram: [u8; 0xA0]) {
        for index in (0..vram.len()).step_by(16) {
            let number = 0x8000u16 + index.to_owned() as u16;
            log::info!(
		"{:X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
		number,
		vram[index.to_owned() as usize],
		vram[1 + index.to_owned() as usize],
		vram[2+ index.to_owned()  as usize],
		vram[3+ index.to_owned()  as usize],
		vram[4+ index.to_owned()  as usize],
		vram[5+ index.to_owned()  as usize],
		vram[6+ index.to_owned()  as usize],
		vram[7+ index.to_owned()  as usize],
		vram[8+ index.to_owned()  as usize],
		vram[9+ index.to_owned()  as usize],
		vram[10+ index.to_owned()  as usize],
		vram[11+ index.to_owned()  as usize],
		vram[12+ index.to_owned()  as usize],
		vram[13+ index.to_owned()  as usize],
		vram[14+ index.to_owned()  as usize],
		vram[15+ index.to_owned()  as usize],
	    );
        }
    }
}

impl eframe::App for GameBoyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Get the time at which a game update should happen
        // let _deadline = std::time::Instant::now()
        //     .checked_add(Duration::from_micros(16600u64))
        //     .unwrap();

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
                ui.toggle_value(&mut self.game_window_open, "Game window");
                ui.toggle_value(&mut self.tile_viewer_open, "Tile viewer");
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
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

        if self.game_window_open {
            egui::Window::new("Game window")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    if let Some(_) = self.cpu {
                        if !self.paused {
                            self.run_frame(ui);
                        }

                        self.render_game_window(ctx, ui);
                    };
                });
        }
        // Update the context after 16.6 ms (forcing the fps to be 60)
        // ctx.request_repaint_after(deadline.sub(Instant::now()));
        ctx.request_repaint();
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
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
