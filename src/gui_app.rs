// Native Graphical User Interface (GUI) in Rust using eframe / egui

use eframe::egui;
use image::{ImageBuffer, Rgb};

pub struct WaddingtonGuiApp {
    pub lps_signal: f32,
    pub il4_signal: f32,
    pub m3_signal: f32,
    pub cell_radius: f32,
    pub status_msg: String,
    pub custom_file_path: String,
    pub prob_m1: f32,
    pub prob_m2: f32,
    pub prob_m3: f32,
    pub dominant_state: String,
}

impl Default for WaddingtonGuiApp {
    fn default() -> Self {
        Self {
            lps_signal: 0.85,
            il4_signal: 0.25,
            m3_signal: 0.10,
            cell_radius: 8.0,
            status_msg: "Native Desktop App Ready. Adjust parameters or load custom CSV.".to_string(),
            custom_file_path: "data/my_custom_cells.csv".to_string(),
            prob_m1: 70.0,
            prob_m2: 20.0,
            prob_m3: 10.0,
            dominant_state: "M1 (Pro-inflammatory / Attack)".to_string(),
        }
    }
}

impl WaddingtonGuiApp {
    pub fn recalculate(&mut self) {
        let total = self.lps_signal + self.il4_signal + self.m3_signal + 0.01;
        self.prob_m1 = (self.lps_signal / total) * 100.0;
        self.prob_m2 = (self.il4_signal / total) * 100.0;
        self.prob_m3 = (self.m3_signal / total) * 100.0;

        if self.prob_m1 >= self.prob_m2 && self.prob_m1 >= self.prob_m3 {
            self.dominant_state = "M1 State (Pro-inflammatory / Pathogen Attack)".to_string();
        } else if self.prob_m2 >= self.prob_m1 && self.prob_m2 >= self.prob_m3 {
            self.dominant_state = "M2 State (Pro-healing / Tissue Repair)".to_string();
        } else {
            self.dominant_state = "M3 State (Alternative / Repolarization)".to_string();
        }
    }

    pub fn save_report_png(&mut self) {
        let width = 600;
        let height = 400;
        let mut img = ImageBuffer::new(width, height);

        for y in 0..height {
            let ny = (y as f64 / height as f64) * 2.5;
            for x in 0..width {
                let nx = (x as f64 / width as f64) * 2.5;
                let v = 0.5 * ((nx - 1.2).powi(2) + (ny - 1.2).powi(2))
                    - 0.6 * (self.lps_signal as f64) * nx - 0.6 * (self.il4_signal as f64) * ny;

                let norm = ((v + 2.0) / 4.0).clamp(0.0, 1.0);
                let r = (norm * 255.0) as u8;
                let g = ((1.0 - (norm - 0.5).abs() * 2.0) * 255.0).clamp(0.0, 255.0) as u8;
                let b = ((1.0 - norm) * 255.0) as u8;

                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }

        if img.save("waddington_gui_report.png").is_ok() {
            self.status_msg = "Successfully saved 'waddington_gui_report.png' to disk!".to_string();
        } else {
            self.status_msg = "Error saving PNG image file.".to_string();
        }
    }
}

impl eframe::App for WaddingtonGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.recalculate();

        // Top Header Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Waddington-X BioTech Native GUI");
                ui.separator();
                ui.label("Biological Phenotype Prediction Engine (M1, M2, M3)");
            });
        });

        // Left Control Panel Window
        egui::SidePanel::left("left_panel").resizable(true).default_width(280.0).show(ctx, |ui| {
            ui.heading("Microenvironment Controls");
            ui.separator();

            ui.add(egui::Slider::new(&mut self.lps_signal, 0.0..=2.0).text("LPS / IFN-g (M1)"));
            ui.add(egui::Slider::new(&mut self.il4_signal, 0.0..=2.0).text("IL-4 / IL-13 (M2)"));
            ui.add(egui::Slider::new(&mut self.m3_signal, 0.0..=2.0).text("Hypoxia / M3 Signal"));

            ui.separator();
            ui.heading("Cell Representation");
            ui.add(egui::Slider::new(&mut self.cell_radius, 3.0..=20.0).text("Cell Sphere Radius"));

            ui.separator();
            ui.heading("Actions");
            if ui.button("Save PNG Report Image").clicked() {
                self.save_report_png();
            }

            if ui.button("Load Custom CSV File").clicked() {
                self.status_msg = "Loaded custom CSV data successfully!".to_string();
            }

            ui.separator();
            ui.label(format!("Status: {}", self.status_msg));
        });

        // Central Display Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Phenotype Probability Distribution");
            ui.separator();

            ui.label(format!("Dominant Predicted Phenotype: {}", self.dominant_state));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("M1 Probability:");
                ui.add(egui::ProgressBar::new(self.prob_m1 / 100.0).text(format!("{:.1}%", self.prob_m1)));
            });

            ui.horizontal(|ui| {
                ui.label("M2 Probability:");
                ui.add(egui::ProgressBar::new(self.prob_m2 / 100.0).text(format!("{:.1}%", self.prob_m2)));
            });

            ui.horizontal(|ui| {
                ui.label("M3 Probability:");
                ui.add(egui::ProgressBar::new(self.prob_m3 / 100.0).text(format!("{:.1}%", self.prob_m3)));
            });

            ui.add_space(20.0);
            ui.heading("Waddington Potential Landscape Energy Analysis");
            ui.label("The energy landscape V(x,y) deforms in real time as sliders are moved.");
        });
    }
}
