// Main Desktop Native GUI Application Launcher

mod gui_app;
use gui_app::WaddingtonGuiApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(900.0, 600.0)),
        resizable: true,
        ..Default::default()
    };

    println!("Starting Native Desktop GUI Application (eframe/egui)...");
    eframe::run_native(
        "Waddington-X BioTech Platform",
        options,
        Box::new(|_cc| Box::new(WaddingtonGuiApp::default())),
    )
}
