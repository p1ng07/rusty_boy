// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    use rusty_boy::game_app::GameBoyApp;

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


//     while !rl.window_should_close() {
//     }
// }
