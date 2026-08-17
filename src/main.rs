// Oracle - Scientific Research Workstation Launcher
// High-performance modeling and interactive visualization of Epigenetic Landscapes

mod data;
mod facs;
mod gui_app;
mod model;
mod renderer;

use gui_app::WaddingtonGuiApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(1280.0, 820.0)),
        min_window_size: Some(eframe::egui::vec2(980.0, 620.0)),
        resizable: true,
        ..Default::default()
    };

    println!("Starting Oracle Scientific Research Workstation (eframe/egui)...");
    eframe::run_native(
        "Oracle | Epigenetic Landscape & Phenotype Dynamics Workstation",
        options,
        Box::new(|_cc| Box::new(WaddingtonGuiApp::default())),
    )
}
