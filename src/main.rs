// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    use rusty_boy::game_app::GameBoyApp;

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Rusty boy: Your favourite gameboy emulator",
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
