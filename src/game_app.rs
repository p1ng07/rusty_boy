use log::LevelFilter;
use log4rs::{append::file::FileAppender, Config, encode::pattern::PatternEncoder, config::{Appender, Root}};

pub struct GameBoyApp {
    current_rom_path: Option<String>,
}

impl GameBoyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
	// This is also where you can customize the look and feel of egui using
	// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.


	// Log to file only when running on native and debug mode
	// #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
	// init_file_logger();

	Self { current_rom_path: Default::default() }
    }

    fn load_rom(&self) {
	if let Some(x) = &self.current_rom_path {
	    println!("{} file was chosen", x)
	}
    }
}

impl eframe::App for GameBoyApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

	// TODO Add file picker for choosing which rom is going to be loaded
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Rom").clicked() {
			if let Some(path) = rfd::FileDialog::new().pick_file() {
			    self.current_rom_path = Some(path.display().to_string());
			    self.load_rom();
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
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
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
