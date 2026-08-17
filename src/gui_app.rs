// Waddington-X BioTech Platform - Scientific Research Workstation
// High-Performance Desktop Interface for Epigenetic Landscapes, FACS Gating,
// Nonlinear Bifurcations, Time-Series Kinetics, and Drug Perturbation Assays.

use crate::data::{
    export_facs_gating_report, export_population_summary_report, export_simulation_csv,
    export_time_series_csv, generate_sample_dataset, parse_csv_data, ExperimentalDataset,
    SampleDatasetType,
};
use crate::facs::{analyze_facs_population, FacsAnalysisReport, FacsGatingGates};
use crate::model::{Phenotype, SimulationModel};
use crate::renderer::{CanvasAction, CanvasRenderer, RenderSettings};
use eframe::egui::{self, Color32, ProgressBar, RichText, ScrollArea};
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkstationTab {
    Landscape2D,
    FacsGating,
    Bifurcation,
    Kinetics,
    DrugAssay,
    Methodology,
}

pub struct WaddingtonGuiApp {
    pub model: SimulationModel,
    pub render_settings: RenderSettings,
    pub facs_gates: FacsGatingGates,
    pub facs_report: FacsAnalysisReport,
    pub experimental_dataset: Option<ExperimentalDataset>,
    pub selected_sample_type: SampleDatasetType,
    pub active_tab: WorkstationTab,
    pub custom_csv_path: String,
    pub status_message: String,
    pub population_target_count: usize,
    pub export_csv_name: String,
    pub export_png_name: String,
}

impl Default for WaddingtonGuiApp {
    fn default() -> Self {
        let model = SimulationModel::default();
        let default_dataset = generate_sample_dataset(SampleDatasetType::TumorMicroenvironment);
        let cell_count = model.cells.len();
        let facs_gates = FacsGatingGates::default();
        let facs_report = analyze_facs_population(&model.cells, &facs_gates);

        Self {
            model,
            render_settings: RenderSettings::default(),
            facs_gates,
            facs_report,
            experimental_dataset: Some(default_dataset),
            selected_sample_type: SampleDatasetType::TumorMicroenvironment,
            active_tab: WorkstationTab::Landscape2D,
            custom_csv_path: "data/single_cell_sample.csv".to_string(),
            status_message: "Waddington-X Research Workstation ready. Stochastic simulation active.".to_string(),
            population_target_count: cell_count,
            export_csv_name: "waddington_cell_population.csv".to_string(),
            export_png_name: "waddington_scientific_figure.png".to_string(),
        }
    }
}

impl eframe::App for WaddingtonGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.model.is_running {
            self.model.step(0.016);
            self.facs_report = analyze_facs_population(&self.model.cells, &self.facs_gates);
            ctx.request_repaint();
        }

        // Top Header and Tab Navigation
        self.render_top_panel(ctx);

        // Left Controls Panel
        self.render_left_panel(ctx);

        // Right Analytics Panel
        self.render_right_panel(ctx);

        // Central Main Workspace View
        self.render_central_workspace(ctx);
    }
}

impl WaddingtonGuiApp {
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_header_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("🧬 Waddington-X").strong().color(Color32::from_rgb(56, 189, 248)));
                ui.label(RichText::new("Epigenetic Landscape Research Workstation").color(Color32::from_rgb(148, 163, 184)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("↺ Reset M0").color(Color32::from_rgb(226, 232, 240))).clicked() {
                        self.model.reset_to_m0();
                        self.status_message = "Reset population to naive M0 baseline state.".to_string();
                    }

                    if self.model.is_running {
                        if ui.button(RichText::new("⏸ Pause").color(Color32::from_rgb(251, 146, 60))).clicked() {
                            self.model.is_running = false;
                        }
                    } else {
                        if ui.button(RichText::new("▶ Play").color(Color32::from_rgb(74, 222, 128))).clicked() {
                            self.model.is_running = true;
                        }
                    }

                    ui.label(format!("Time: {:.1}s", self.model.sim_time));
                    ui.separator();
                });
            });

            ui.add_space(3.0);
            // Tab navigation bar
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, WorkstationTab::Landscape2D, "🌐 2D Waddington Landscape");
                ui.selectable_value(&mut self.active_tab, WorkstationTab::FacsGating, "🔬 FACS Gating & Cytometry");
                ui.selectable_value(&mut self.active_tab, WorkstationTab::Bifurcation, "📈 Bifurcation & Hysteresis");
                ui.selectable_value(&mut self.active_tab, WorkstationTab::Kinetics, "⏱ Time-Series Kinetics");
                ui.selectable_value(&mut self.active_tab, WorkstationTab::DrugAssay, "💊 Drug Screening Assay");
                ui.selectable_value(&mut self.active_tab, WorkstationTab::Methodology, "📚 Theory & Methodology");
            });
            ui.add_space(3.0);
        });
    }

    fn render_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("controls_side_panel")
            .resizable(true)
            .default_width(310.0)
            .width_range(280.0..=400.0)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(5.0);

                    // 1. Microenvironment Cytokines
                    ui.collapsing(RichText::new("🧪 Microenvironment Cytokines").heading(), |ui| {
                        ui.label(RichText::new("Cytokine gradients deforming the Waddington surface:").small());
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("LPS / IFN-γ (M1):");
                            ui.add(egui::Slider::new(&mut self.model.params.s_lps, 0.0..=3.0).step_by(0.05));
                        });

                        ui.horizontal(|ui| {
                            ui.label("IL-4 / IL-13 (M2):");
                            ui.add(egui::Slider::new(&mut self.model.params.s_il4, 0.0..=3.0).step_by(0.05));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Hypoxia / M3 Signal:");
                            ui.add(egui::Slider::new(&mut self.model.params.s_hypoxia, 0.0..=3.0).step_by(0.05));
                        });

                        ui.add_space(6.0);
                        ui.label(RichText::new("Quick Cytokine Shocks:").strong().small());
                        ui.horizontal(|ui| {
                            if ui.button("⚡ +LPS").clicked() {
                                self.model.add_cytokine_shock(Phenotype::M1);
                                self.status_message = "Acute LPS/IFN-γ inflammatory shock applied.".to_string();
                            }
                            if ui.button("⚡ +IL-4").clicked() {
                                self.model.add_cytokine_shock(Phenotype::M2);
                                self.status_message = "IL-4/IL-13 regenerative stimulus applied.".to_string();
                            }
                            if ui.button("⚡ +Hypoxia").clicked() {
                                self.model.add_cytokine_shock(Phenotype::M3);
                                self.status_message = "Severe tissue hypoxia applied.".to_string();
                            }
                            if ui.button("Washout").clicked() {
                                self.model.add_cytokine_shock(Phenotype::M0);
                                self.status_message = "Cytokines washed out to basal resting level.".to_string();
                            }
                        });
                    });

                    ui.separator();

                    // 2. Biophysical GRN Parameters
                    ui.collapsing(RichText::new("⚙ Biophysical Parameters (GRN)").heading(), |ui| {
                        ui.label(RichText::new("Gene Regulatory Network mutual-inhibition toggle switch:").small());
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("Hill Exponent (n):");
                            ui.add(egui::Slider::new(&mut self.model.params.hill_n, 1.0..=5.0).step_by(0.5));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Cross-Inhibition (γ):");
                            ui.add(egui::Slider::new(&mut self.model.params.gamma, 0.0..=3.0).step_by(0.1));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Gene Noise (σ):");
                            ui.add(egui::Slider::new(&mut self.model.params.noise_sigma, 0.01..=0.50).step_by(0.01));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Degradation (δ):");
                            ui.add(egui::Slider::new(&mut self.model.params.delta, 0.2..=2.5).step_by(0.1));
                        });
                    });

                    ui.separator();

                    // 3. Population Dynamics
                    ui.collapsing(RichText::new("👥 Cell Population Dynamics").heading(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Population Size (N):");
                            let mut count = self.population_target_count;
                            if ui.add(egui::Slider::new(&mut count, 20..=500).step_by(10.0)).changed() {
                                self.population_target_count = count;
                                self.model.set_cell_count(count);
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Re-seed Population").clicked() {
                                self.model.init_population(self.population_target_count);
                                self.status_message = "Re-initialized cell population.".to_string();
                            }
                            if ui.button("Step +0.1s").clicked() {
                                self.model.step(0.1);
                            }
                        });
                    });

                    ui.separator();

                    // 4. Visualization Layers
                    ui.collapsing(RichText::new("👁 Visualization Layers").heading(), |ui| {
                        ui.checkbox(&mut self.render_settings.show_heatmap, "Potential Heatmap U(x,y)");
                        ui.checkbox(&mut self.render_settings.show_contours, "Equipotential Contours (Изолинии)");
                        ui.checkbox(&mut self.render_settings.show_vector_field, "Drift Force Vectors (Векторное поле)");
                        ui.checkbox(&mut self.render_settings.show_nullclines, "Phase Nullclines (dx/dt=0, dy/dt=0)");
                        ui.checkbox(&mut self.render_settings.show_simulated_cells, "Simulated Single Cells");
                        ui.checkbox(&mut self.render_settings.show_trails, "Cell Trajectory Trails");
                        ui.checkbox(&mut self.render_settings.show_experimental_data, "Experimental Data Overlay");
                        ui.checkbox(&mut self.render_settings.show_attractor_labels, "Attractor Fate Labels");
                    });

                    ui.separator();

                    // 5. Experimental Data (CSV)
                    ui.collapsing(RichText::new("📊 Experimental Data (CSV)").heading(), |ui| {
                        egui::ComboBox::from_label("Sample Dataset")
                            .selected_text(self.selected_sample_type.title())
                            .show_ui(ui, |ui| {
                                for st in SampleDatasetType::all() {
                                    if ui.selectable_value(&mut self.selected_sample_type, *st, st.title()).clicked() {
                                        self.experimental_dataset = Some(generate_sample_dataset(*st));
                                        self.status_message = format!("Loaded benchmark dataset: {}", st.title());
                                    }
                                }
                            });

                        if let Some(ds) = &self.experimental_dataset {
                            ui.label(RichText::new(format!("Active: {} ({} cells)", ds.name, ds.cells.len())).color(Color32::from_rgb(147, 197, 253)).small());
                            ui.label(RichText::new(&ds.description).italics().weak().small());
                        }

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.custom_csv_path);
                            if ui.button("Load CSV").clicked() {
                                match fs::read_to_string(&self.custom_csv_path) {
                                    Ok(content) => {
                                        match parse_csv_data(&content, "User Custom Dataset") {
                                            Ok(ds) => {
                                                let count = ds.cells.len();
                                                self.experimental_dataset = Some(ds);
                                                self.status_message = format!("Successfully loaded {} cells from CSV!", count);
                                            }
                                            Err(err) => {
                                                self.status_message = format!("CSV Parse Error: {}", err);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.status_message = format!("File Read Error: {}", e);
                                    }
                                }
                            }
                        });

                        if ui.button("Clear Experimental Overlay").clicked() {
                            self.experimental_dataset = None;
                            self.status_message = "Cleared experimental data overlay.".to_string();
                        }
                    });

                    ui.separator();

                    // 6. Scientific Export Tools
                    ui.collapsing(RichText::new("💾 Scientific Export & Reports").heading(), |ui| {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.export_csv_name);
                            if ui.button("Export CSV").clicked() {
                                let csv_data = export_simulation_csv(&self.model.cells, &self.model.params);
                                if fs::write(&self.export_csv_name, csv_data).is_ok() {
                                    self.status_message = format!("Exported population data to '{}'", self.export_csv_name);
                                } else {
                                    self.status_message = "Failed to write CSV file.".to_string();
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.export_png_name);
                            if ui.button("Save PNG Figure").clicked() {
                                match CanvasRenderer::generate_high_res_png(
                                    &self.model.params,
                                    &self.model.cells,
                                    self.experimental_dataset.as_ref(),
                                    &self.export_png_name,
                                ) {
                                    Ok(_) => {
                                        self.status_message = format!("High-resolution figure saved to '{}'", self.export_png_name);
                                    }
                                    Err(err) => {
                                        self.status_message = format!("PNG export error: {}", err);
                                    }
                                }
                            }
                        });

                        if ui.button("Save Full Scientific Summary Report").clicked() {
                            let report = export_population_summary_report(&self.model.stats, &self.model.params, self.model.sim_time);
                            if fs::write("waddington_scientific_report.txt", report).is_ok() {
                                self.status_message = "Saved 'waddington_scientific_report.txt'.".to_string();
                            }
                        }

                        if ui.button("Save FACS Gating Report").clicked() {
                            let facs_txt = export_facs_gating_report(&self.facs_report, &self.facs_gates);
                            if fs::write("waddington_facs_report.txt", facs_txt).is_ok() {
                                self.status_message = "Saved 'waddington_facs_report.txt'.".to_string();
                            }
                        }

                        if ui.button("Save Time-Series Kinetics CSV").clicked() {
                            let ts_csv = export_time_series_csv(&self.model.time_series_history);
                            if fs::write("waddington_kinetics_timeseries.csv", ts_csv).is_ok() {
                                self.status_message = "Saved 'waddington_kinetics_timeseries.csv'.".to_string();
                            }
                        }
                    });

                    ui.add_space(10.0);
                });
            });
    }

    fn render_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("analytics_side_panel")
            .resizable(true)
            .default_width(290.0)
            .width_range(260.0..=380.0)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(5.0);
                    ui.heading("📊 System Biology Analytics");
                    ui.separator();

                    let stats = &self.model.stats;

                    let (dom_name, dom_color) = if stats.pct_m1 >= stats.pct_m2 && stats.pct_m1 >= stats.pct_m3 && stats.pct_m1 >= stats.pct_m0 {
                        ("M1 Pro-inflammatory", Color32::from_rgb(239, 68, 68))
                    } else if stats.pct_m2 >= stats.pct_m1 && stats.pct_m2 >= stats.pct_m3 && stats.pct_m2 >= stats.pct_m0 {
                        ("M2 Pro-healing / Repair", Color32::from_rgb(34, 197, 94))
                    } else if stats.pct_m3 >= stats.pct_m1 && stats.pct_m3 >= stats.pct_m2 && stats.pct_m3 >= stats.pct_m0 {
                        ("M3 Alternative / Hybrid", Color32::from_rgb(234, 179, 8))
                    } else {
                        ("M0 Naive / Resting", Color32::from_rgb(148, 163, 184))
                    };

                    ui.label("Dominant Population State:");
                    ui.label(RichText::new(dom_name).color(dom_color).strong().size(15.0));
                    ui.add_space(8.0);

                    // Phenotype Distribution Breakdown
                    ui.label(RichText::new("Phenotype Fraction Distribution:").strong());

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("M1 (Attack):").color(Color32::from_rgb(248, 113, 113)));
                        ui.add(ProgressBar::new(stats.pct_m1 / 100.0).text(format!("{:.1}% ({})", stats.pct_m1, stats.count_m1)));
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("M2 (Repair):").color(Color32::from_rgb(74, 222, 128)));
                        ui.add(ProgressBar::new(stats.pct_m2 / 100.0).text(format!("{:.1}% ({})", stats.pct_m2, stats.count_m2)));
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("M3 (Hybrid):").color(Color32::from_rgb(250, 204, 21)));
                        ui.add(ProgressBar::new(stats.pct_m3 / 100.0).text(format!("{:.1}% ({})", stats.pct_m3, stats.count_m3)));
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("M0 (Naive):").color(Color32::from_rgb(148, 163, 184)));
                        ui.add(ProgressBar::new(stats.pct_m0 / 100.0).text(format!("{:.1}% ({})", stats.pct_m0, stats.count_m0)));
                    });

                    ui.add_space(10.0);
                    ui.separator();

                    // FACS Quadrant Live Summary
                    ui.heading("🔬 FACS Quadrant Summary");
                    ui.label(format!("Q1 (M2 Repair):        {:.1}% (MFI={:.2})", self.facs_report.pct_q1, self.facs_report.mfi_stat6_q1));
                    ui.label(format!("Q2 (Double⁺ Hybrid):   {:.1}%", self.facs_report.pct_q2));
                    ui.label(format!("Q3 (Double⁻ Naive):    {:.1}%", self.facs_report.pct_q3));
                    ui.label(format!("Q4 (M1 Attack):        {:.1}% (MFI={:.2})", self.facs_report.pct_q4, self.facs_report.mfi_stat1_q4));

                    ui.add_space(10.0);
                    ui.separator();

                    // Thermodynamic & Information Metrics
                    ui.heading("🌡 Thermodynamic Metrics");
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Shannon Diversity (H):");
                        ui.label(RichText::new(format!("{:.4}", stats.shannon_entropy)).strong().color(Color32::from_rgb(192, 132, 252)));
                    });
                    let entropy_desc = if stats.shannon_entropy > 1.1 {
                        "High Plasticity / Mixed Heterogeneous state"
                    } else if stats.shannon_entropy > 0.6 {
                        "Moderate Intermediate Specialization"
                    } else {
                        "Uniform Homogeneous Lineage Commitment"
                    };
                    ui.label(RichText::new(entropy_desc).small().color(Color32::from_rgb(148, 163, 184)));

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Barrier ΔU (M1 → M2):");
                        ui.label(RichText::new(format!("{:.3}", stats.barrier_m1_m2)).strong());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Barrier ΔU (M2 → M1):");
                        ui.label(RichText::new(format!("{:.3}", stats.barrier_m2_m1)).strong());
                    });

                    ui.add_space(10.0);
                    ui.separator();

                    // Biological Interpretation Card
                    ui.heading("💡 Biological Interpretation");
                    ui.add_space(4.0);

                    let interpretation_text = if self.model.params.s_lps > 1.2 && self.model.params.s_il4 < 0.5 {
                        "Pro-inflammatory Acute Response: High LPS/IFN-γ drives dominant STAT1/NF-κB activation. The M1 valley is the global minimum; energy barrier against M2 repolarization is high."
                    } else if self.model.params.s_il4 > 1.2 && self.model.params.s_lps < 0.5 {
                        "Pro-healing Tissue Remodeling: IL-4/IL-13 stimulation activates STAT6/PPAR-γ pathways. Macrophages commit to tissue regeneration and anti-inflammatory resolution (M2 valley)."
                    } else if self.model.params.s_hypoxia > 1.0 {
                        "Hypoxic Microenvironment Adaptation: Hypoxia stabilizes HIF-1α, promoting phenotypic plasticity and co-expression of M1/M2 markers (TAM/M3-like phenotype)."
                    } else if self.model.params.s_lps > 0.8 && self.model.params.s_il4 > 0.8 {
                        "Antagonistic Co-stimulation / Bimodal Bistability: Competing cytokine signals create a bistable landscape where single-cell noise determines stochastic fate choice."
                    } else {
                        "Homeostatic Basal Resting State: Macrophages remain predominantly uncommitted (M0) around the shallow central basin."
                    };

                    ui.label(RichText::new(interpretation_text).small());

                    ui.add_space(12.0);
                    ui.separator();

                    ui.label(RichText::new("System Status:").strong().small());
                    ui.label(RichText::new(&self.status_message).color(Color32::from_rgb(125, 211, 252)).small());
                });
            });
    }

    fn render_central_workspace(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                WorkstationTab::Landscape2D => {
                    let action = CanvasRenderer::render_landscape(
                        ui,
                        &self.model.params,
                        &self.model.cells,
                        self.experimental_dataset.as_ref(),
                        &self.render_settings,
                    );
                    if let CanvasAction::InjectCells { x, y, count } = action {
                        self.model.inject_cells_at(x, y, count);
                        self.status_message = format!("Injected {} single cells at ({:.2}, {:.2})!", count, x, y);
                    }
                }
                WorkstationTab::FacsGating => {
                    ui.horizontal(|ui| {
                        ui.heading("🔬 Flow Cytometry (FACS) 4-Quadrant Analysis");
                        ui.label(RichText::new("Drag gating intersection lines to adjust thresholds").italics().small());
                    });
                    ui.separator();
                    CanvasRenderer::render_facs_plot(
                        ui,
                        &self.model.cells,
                        &mut self.facs_gates,
                        &self.facs_report,
                    );
                }
                WorkstationTab::Bifurcation => {
                    ui.heading("📈 Bifurcation Diagram & Epigenetic Hysteresis");
                    ui.label(RichText::new("Fixed points (x*) of STAT1 expression across LPS titration showing fold bifurcation bistability").small());
                    ui.separator();
                    CanvasRenderer::render_bifurcation_diagram(ui, &self.model.params);
                }
                WorkstationTab::Kinetics => {
                    ui.heading("⏱ Time-Series Population Polarization Kinetics");
                    ui.label(RichText::new("Real-time multi-channel strip chart of STAT1, STAT6, and Shannon Entropy").small());
                    ui.separator();
                    CanvasRenderer::render_time_series_chart(ui, &self.model.time_series_history);
                }
                WorkstationTab::DrugAssay => {
                    self.render_drug_assay_tab(ui);
                }
                WorkstationTab::Methodology => {
                    self.render_methodology_tab(ui);
                }
            }
        });
    }

    fn render_drug_assay_tab(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.heading("💊 Pharmacological Perturbation & Drug Screening Assay");
            ui.label(RichText::new("Simulate the molecular impact of targeted inhibitors on macrophage polarization:").color(Color32::from_rgb(148, 163, 184)));
            ui.add_space(8.0);
            ui.separator();

            let drugs = &mut self.model.params.drugs;

            ui.group(|ui| {
                ui.heading("1. JAK1/2 Inhibitors (e.g. Tofacitinib / Ruxolitinib)");
                ui.label("Inhibits Janus kinase phosphorylation, suppressing both STAT1 (M1) and STAT6 (M2) downstream cascades.");
                ui.add(egui::Slider::new(&mut drugs.jak_inhibitor, 0.0..=1.0).text("Dose / Efficacy"));
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.heading("2. Anti-IL-4Rα Monoclonal Antibody (Dupilumab-like)");
                ui.label("Specifically blocks IL-4 and IL-13 signaling, preventing M2 polarization and tissue fibrosis.");
                ui.add(egui::Slider::new(&mut drugs.anti_il4r_mab, 0.0..=1.0).text("Dose / Efficacy"));
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.heading("3. TLR4 Small Molecule Antagonists (e.g. TAK-242 / Resatorvid)");
                ui.label("Blocks Toll-like receptor 4 activation by LPS, abolishing acute pro-inflammatory cytokine storms.");
                ui.add(egui::Slider::new(&mut drugs.tlr4_antagonist, 0.0..=1.0).text("Dose / Efficacy"));
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.heading("4. HIF-1α Transcription Inhibitors");
                ui.label("Suppresses hypoxia-induced phenotypic plasticity and tumor-associated macrophage (TAM) immunosuppression.");
                ui.add(egui::Slider::new(&mut drugs.hif1a_inhibitor, 0.0..=1.0).text("Dose / Efficacy"));
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.heading("5. HDAC Inhibitors (Epigenetic Remodeling)");
                ui.label("Promotes chromatin decondensation, increasing stochastic gene expression noise (σ) and transition rates.");
                ui.add(egui::Slider::new(&mut drugs.hdac_inhibitor, 0.0..=1.0).text("Dose / Efficacy"));
            });

            ui.add_space(10.0);
            if ui.button("Reset All Drug Dosages to Zero (Vehicles Only)").clicked() {
                self.model.params.drugs = crate::model::DrugAssaySettings::default();
                self.status_message = "Cleared all pharmacological agents.".to_string();
            }
        });
    }

    fn render_methodology_tab(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.heading("📚 Scientific Methodology, Mathematical Formulations & References");
            ui.separator();
            ui.add_space(6.0);

            ui.label(RichText::new("1. Nonlinear Gene Regulatory Network (GRN) Toggle Switch").strong().size(14.0));
            ui.label("Macrophage polarization between M1 and M2 states is modeled as a mutually repressive dynamical system governed by Hill kinetics:");
            ui.monospace("dx/dt = (α·xⁿ + S_LPS + 0.1) / (1 + xⁿ + γ·yⁿ) + Hypoxia_x - δ·x");
            ui.monospace("dy/dt = (α·yⁿ + S_IL4 + 0.1) / (1 + yⁿ + γ·xⁿ) + Hypoxia_y - δ·y");
            ui.label("where x is STAT1/NF-κB activity, y is STAT6/PPAR-γ activity, n is Hill cooperativity, γ is cross-inhibition, and δ is clearance rate.");

            ui.add_space(10.0);
            ui.label(RichText::new("2. Stochastic Langevin Dynamics (Euler-Maruyama)").strong().size(14.0));
            ui.label("Single-cell gene expression noise is simulated via stochastic differential equations (SDEs):");
            ui.monospace("dr_i = F(r_i)·dt + σ·dW_i");
            ui.label("where F(r_i) is the deterministic drift vector and σ represents biological noise amplitude.");

            ui.add_space(10.0);
            ui.label(RichText::new("3. Shannon Entropy of Cell Population").strong().size(14.0));
            ui.label("Population heterogeneity and lineage plasticity are quantified using information entropy:");
            ui.monospace("H = - ∑ p_k · ln(p_k)");

            ui.add_space(10.0);
            ui.label(RichText::new("4. Key Academic References & Literature Citations").strong().size(14.0));
            ui.label("• Waddington, C. H. (1957). The Strategy of the Genes. Allen & Unwin, London.");
            ui.label("• Huang, S., et al. (2005). Bifurcation dynamics in lineage-commitment in bipotent progenitor cells. Dev. Biol., 280(1), 40-58.");
            ui.label("• Sica, A., & Mantovani, A. (2012). Macrophage plasticity and polarization: in vivo veritas. J. Clin. Invest., 122(3), 787-795.");
            ui.label("• Murray, P. J., et al. (2014). Macrophage activation and polarization: nomenclature and experimental guidelines. Immunity, 41(1), 14-20.");
            ui.label("• Zhou, J. X., et al. (2012). Quasi-potential landscape in complex dynamical systems. Physical Review E, 85(6), 061918.");
        });
    }
}
