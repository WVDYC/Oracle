// High-Performance 2D Canvas Renderer for Waddington Epigenetic Landscape,
// Flow Cytometry (FACS) 4-Quadrant Scatter, Bifurcation Hysteresis Curves,
// Time-Series Strip Chart Kinetics, and High-Res PNG Generation.

use crate::data::ExperimentalDataset;
use crate::facs::{classify_quadrant, FacsAnalysisReport, FacsGatingGates, FacsQuadrant};
use crate::model::{
    compute_drift_vector, compute_lps_bifurcation_curve, compute_waddington_potential,
    BifurcationBranchPoint, BiophysicalParams, Phenotype, SingleCell, TimeSeriesPoint,
};
use eframe::egui::{
    self, Color32, Painter, Pos2, Rect, Stroke, Vec2,
};
use image::{ImageBuffer, Rgb};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub show_heatmap: bool,
    pub show_contours: bool,
    pub show_vector_field: bool,
    pub show_nullclines: bool,
    pub show_simulated_cells: bool,
    pub show_trails: bool,
    pub show_experimental_data: bool,
    pub show_attractor_labels: bool,
    pub grid_resolution: usize,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            show_heatmap: true,
            show_contours: true,
            show_vector_field: true,
            show_nullclines: true,
            show_simulated_cells: true,
            show_trails: true,
            show_experimental_data: true,
            show_attractor_labels: true,
            grid_resolution: 36,
        }
    }
}

pub enum CanvasAction {
    None,
    InjectCells { x: f32, y: f32, count: usize },
}

pub struct CanvasRenderer;

impl CanvasRenderer {
    /// Renders the main 2D Waddington Epigenetic Landscape Canvas
    pub fn render_landscape(
        ui: &mut egui::Ui,
        params: &BiophysicalParams,
        cells: &[SingleCell],
        exp_dataset: Option<&ExperimentalDataset>,
        settings: &RenderSettings,
    ) -> CanvasAction {
        let available_size = ui.available_size();
        let min_dim = available_size.x.min(available_size.y).max(380.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(min_dim), egui::Sense::click_and_drag());

        let painter = ui.painter_at(rect);

        let margin_left = 55.0;
        let margin_bottom = 50.0;
        let margin_top = 25.0;
        let margin_right = 65.0;

        let plot_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + margin_left, rect.min.y + margin_top),
            Pos2::new(rect.max.x - margin_right, rect.max.y - margin_bottom),
        );

        painter.rect_filled(rect, 8.0, Color32::from_rgb(15, 18, 26));
        painter.rect_filled(plot_rect, 0.0, Color32::from_rgb(10, 13, 20));

        let to_screen = |x: f32, y: f32| -> Pos2 {
            let px = plot_rect.min.x + (x / 3.0) * plot_rect.width();
            let py = plot_rect.max.y - (y / 3.0) * plot_rect.height();
            Pos2::new(px, py)
        };

        let to_bio = |pos: Pos2| -> (f32, f32) {
            let x = ((pos.x - plot_rect.min.x) / plot_rect.width()) * 3.0;
            let y = ((plot_rect.max.y - pos.y) / plot_rect.height()) * 3.0;
            (x.clamp(0.0, 3.0), y.clamp(0.0, 3.0))
        };

        // 1. Heatmap
        if settings.show_heatmap {
            Self::draw_heatmap(&painter, plot_rect, params, settings.grid_resolution);
        }

        // 2. Contours
        if settings.show_contours {
            Self::draw_contours(&painter, params, to_screen);
        }

        // 3. Vector Field
        if settings.show_vector_field {
            Self::draw_vector_field(&painter, params, to_screen);
        }

        // 4. Nullclines
        if settings.show_nullclines {
            Self::draw_nullclines(&painter, params, to_screen);
        }

        // 5. Attractor Badges
        if settings.show_attractor_labels {
            Self::draw_attractor_badges(&painter, to_screen);
        }

        // 6. Experimental Overlay
        if settings.show_experimental_data {
            if let Some(dataset) = exp_dataset {
                for cell in &dataset.cells {
                    let pos = to_screen(cell.x, cell.y);
                    let color = match cell.phenotype {
                        Phenotype::M0 => Color32::from_rgb(180, 200, 220),
                        Phenotype::M1 => Color32::from_rgb(255, 120, 120),
                        Phenotype::M2a => Color32::from_rgb(100, 240, 150),
                        Phenotype::M2b => Color32::from_rgb(192, 132, 252),
                        Phenotype::M2c => Color32::from_rgb(45, 212, 191),
                        Phenotype::M2d => Color32::from_rgb(251, 146, 60),
                        Phenotype::MHybrid => Color32::from_rgb(250, 204, 21),
                    };

                    let s = 4.5;
                    let pts = vec![
                        Pos2::new(pos.x, pos.y - s),
                        Pos2::new(pos.x + s, pos.y),
                        Pos2::new(pos.x, pos.y + s),
                        Pos2::new(pos.x - s, pos.y),
                    ];
                    painter.add(egui::Shape::convex_polygon(
                        pts,
                        color,
                        Stroke::new(1.0, Color32::from_black_alpha(180)),
                    ));
                }
            }
        }

        // 7. Live Simulated Cells
        if settings.show_simulated_cells {
            for cell in cells {
                if settings.show_trails && cell.trail.len() > 1 {
                    let points: Vec<Pos2> = cell.trail.iter().map(|p| to_screen(p[0], p[1])).collect();
                    let [r, g, b, _] = cell.phenotype.color_rgba();
                    for i in 0..points.len() - 1 {
                        let alpha = ((i + 1) as f32 / points.len() as f32) * 120.0;
                        painter.line_segment(
                            [points[i], points[i + 1]],
                            Stroke::new(1.2, Color32::from_rgba_unmultiplied(r, g, b, alpha as u8)),
                        );
                    }
                }

                let pos = to_screen(cell.x, cell.y);
                let [r, g, b, a] = cell.phenotype.color_rgba();
                let color = Color32::from_rgba_unmultiplied(r, g, b, a);

                painter.circle_filled(pos, 5.5, Color32::from_rgba_unmultiplied(r, g, b, 60));
                painter.circle_filled(pos, 3.5, color);
                painter.circle_stroke(pos, 3.5, Stroke::new(1.0, Color32::from_rgb(255, 255, 255)));
            }
        }

        // 8. Axes and labels
        Self::draw_axes_and_labels(&painter, rect, plot_rect, to_screen);

        // 9. Colorbar
        Self::draw_colorbar(&painter, rect, plot_rect);

        // 10. Click & Drag Interaction (Drop cell pulses)
        let mut action = CanvasAction::None;
        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if plot_rect.contains(pos) {
                    let (bx, by) = to_bio(pos);
                    action = CanvasAction::InjectCells {
                        x: bx,
                        y: by,
                        count: if response.clicked() { 15 } else { 3 },
                    };
                }
            }
        }

        // 11. Interactive Hover Probe
        if let Some(mouse_pos) = response.hover_pos() {
            if plot_rect.contains(mouse_pos) {
                let (bio_x, bio_y) = to_bio(mouse_pos);
                let potential = compute_waddington_potential(bio_x, bio_y, params);
                let (fx, fy) = compute_drift_vector(bio_x, bio_y, params);
                let force_mag = (fx * fx + fy * fy).sqrt();
                let phenotype = crate::model::classify_cell_phenotype(bio_x, bio_y);

                let meta = crate::model::compute_cell_metabolism(bio_x, bio_y, params);
                let markers = crate::model::compute_cell_markers(bio_x, bio_y, params);

                painter.line_segment(
                    [Pos2::new(plot_rect.min.x, mouse_pos.y), Pos2::new(plot_rect.max.x, mouse_pos.y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
                );
                painter.line_segment(
                    [Pos2::new(mouse_pos.x, plot_rect.min.y), Pos2::new(mouse_pos.x, plot_rect.max.y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
                );

                egui::show_tooltip(ui.ctx(), response.id, |ui| {
                    ui.label(egui::RichText::new("🔬 Epigenetic Landscape & Metabolism Probe").strong());
                    ui.label(format!("STAT1 (M1 Axis): {:.3}", bio_x));
                    ui.label(format!("STAT6 (M2 Axis): {:.3}", bio_y));
                    ui.label(format!("Potential U(x,y): {:.3}", potential));
                    ui.label(format!("Drift Force ‖F‖: {:.3}", force_mag));
                    ui.label(format!("Predicted State: {}", phenotype.name()));
                    ui.separator();
                    ui.label(egui::RichText::new("⚡ Immunometabolic Flux:").small().strong());
                    ui.label(format!("Glycolytic Flux (Warburg): {:.2}", meta.glycolysis_flux));
                    ui.label(format!("Mitochondrial OXPHOS:     {:.2}", meta.oxphos_flux));
                    ui.label(format!("Metabolic Ratio (Glyc/OX): {:.2}", meta.metabolic_ratio));
                    ui.label(format!("Itaconate/Succinate Index: {:.2}", meta.itaconate_succinate_index));
                    ui.separator();
                    ui.label(egui::RichText::new("🏷 Key Markers:").small().strong());
                    ui.label(format!("CD80: {:.2} | iNOS: {:.2} | TNF-α: {:.2}", markers.cd80, markers.inos, markers.tnf_alpha));
                    ui.label(format!("CD206: {:.2} | Arg1: {:.2} | CD163: {:.2} | VEGF: {:.2}", markers.cd206, markers.arg1, markers.cd163, markers.vegf));
                    ui.add_space(3.0);
                    ui.label(egui::RichText::new("💡 Click/drag on map to drop new cells").italics().small());
                });
            }
        }

        action
    }

    /// Renders Flow Cytometry (FACS) 4-Quadrant Scatter Plot
    pub fn render_facs_plot(
        ui: &mut egui::Ui,
        cells: &[SingleCell],
        gates: &mut FacsGatingGates,
        report: &FacsAnalysisReport,
    ) {
        let available_size = ui.available_size();
        let min_dim = available_size.x.min(available_size.y).max(380.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(min_dim), egui::Sense::click_and_drag());

        let painter = ui.painter_at(rect);

        let margin = 50.0;
        let plot_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + margin, rect.min.y + 20.0),
            Pos2::new(rect.max.x - 30.0, rect.max.y - margin),
        );

        painter.rect_filled(rect, 8.0, Color32::from_rgb(15, 18, 26));
        painter.rect_filled(plot_rect, 0.0, Color32::from_rgb(8, 10, 16));

        let to_screen = |x: f32, y: f32| -> Pos2 {
            let px = plot_rect.min.x + (x / 3.0) * plot_rect.width();
            let py = plot_rect.max.y - (y / 3.0) * plot_rect.height();
            Pos2::new(px, py)
        };

        let to_bio = |pos: Pos2| -> (f32, f32) {
            let x = ((pos.x - plot_rect.min.x) / plot_rect.width()) * 3.0;
            let y = ((plot_rect.max.y - pos.y) / plot_rect.height()) * 3.0;
            (x.clamp(0.1, 2.9), y.clamp(0.1, 2.9))
        };

        // Gating lines (Crosshair dividing Q1, Q2, Q3, Q4)
        let gate_screen = to_screen(gates.gate_x_threshold, gates.gate_y_threshold);
        let gate_stroke = Stroke::new(1.8, Color32::from_rgb(56, 189, 248));

        painter.line_segment(
            [Pos2::new(plot_rect.min.x, gate_screen.y), Pos2::new(plot_rect.max.x, gate_screen.y)],
            gate_stroke,
        );
        painter.line_segment(
            [Pos2::new(gate_screen.x, plot_rect.min.y), Pos2::new(gate_screen.x, plot_rect.max.y)],
            gate_stroke,
        );

        // Handle dragging the gating intersection
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if plot_rect.contains(pos) {
                    let (bx, by) = to_bio(pos);
                    gates.gate_x_threshold = bx;
                    gates.gate_y_threshold = by;
                }
            }
        }

        // Draw quadrant percentage overlays (FlowJo style)
        let q1_pos = Pos2::new(plot_rect.min.x + 12.0, plot_rect.min.y + 12.0);
        let q2_pos = Pos2::new(plot_rect.max.x - 12.0, plot_rect.min.y + 12.0);
        let q3_pos = Pos2::new(plot_rect.min.x + 12.0, plot_rect.max.y - 12.0);
        let q4_pos = Pos2::new(plot_rect.max.x - 12.0, plot_rect.max.y - 12.0);

        painter.text(q1_pos, egui::Align2::LEFT_TOP, format!("Q1 (M2 Repair)\n{:.1}% ({})", report.pct_q1, report.count_q1), egui::FontId::proportional(12.0), Color32::from_rgb(74, 222, 128));
        painter.text(q2_pos, egui::Align2::RIGHT_TOP, format!("Q2 (Double⁺ / M3)\n{:.1}% ({})", report.pct_q2, report.count_q2), egui::FontId::proportional(12.0), Color32::from_rgb(250, 204, 21));
        painter.text(q3_pos, egui::Align2::LEFT_BOTTOM, format!("Q3 (Double⁻ / M0)\n{:.1}% ({})", report.pct_q3, report.count_q3), egui::FontId::proportional(12.0), Color32::from_rgb(148, 163, 184));
        painter.text(q4_pos, egui::Align2::RIGHT_BOTTOM, format!("Q4 (M1 Attack)\n{:.1}% ({})", report.pct_q4, report.count_q4), egui::FontId::proportional(12.0), Color32::from_rgb(248, 113, 113));

        // Draw scatter dots for all cells
        for cell in cells {
            let pos = to_screen(cell.x, cell.y);
            let quad = classify_quadrant(cell.x, cell.y, gates);
            let color = match quad {
                FacsQuadrant::Q1 => Color32::from_rgba_unmultiplied(74, 222, 128, 200),
                FacsQuadrant::Q2 => Color32::from_rgba_unmultiplied(250, 204, 21, 200),
                FacsQuadrant::Q3 => Color32::from_rgba_unmultiplied(148, 163, 184, 200),
                FacsQuadrant::Q4 => Color32::from_rgba_unmultiplied(248, 113, 113, 200),
            };
            painter.circle_filled(pos, 3.2, color);
        }

        // Draw axes
        let axis_stroke = Stroke::new(1.2, Color32::from_rgb(100, 116, 139));
        painter.rect_stroke(plot_rect, 0.0, axis_stroke);

        for i in 0..=3 {
            let val = i as f32;
            let px = to_screen(val, 0.0);
            let py = to_screen(0.0, val);
            painter.text(Pos2::new(px.x, plot_rect.max.y + 6.0), egui::Align2::CENTER_TOP, format!("{:.1}", val), egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(plot_rect.min.x - 6.0, py.y), egui::Align2::RIGHT_CENTER, format!("{:.1}", val), egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        }

        painter.text(Pos2::new(plot_rect.center().x, rect.max.y - 12.0), egui::Align2::CENTER_BOTTOM, "STAT1 / CD80 Fluorescence (M1 Marker Axis)", egui::FontId::proportional(12.0), Color32::from_rgb(248, 113, 113));
        painter.text(Pos2::new(plot_rect.min.x, rect.min.y + 4.0), egui::Align2::LEFT_TOP, "STAT6 / CD206 Fluorescence (M2 Marker Axis)", egui::FontId::proportional(12.0), Color32::from_rgb(74, 222, 128));
    }

    /// Renders Bifurcation and Epigenetic Hysteresis Diagram
    pub fn render_bifurcation_diagram(
        ui: &mut egui::Ui,
        params: &BiophysicalParams,
    ) {
        let available_size = ui.available_size();
        let min_dim = available_size.x.min(available_size.y).max(380.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(min_dim), egui::Sense::hover());

        let painter = ui.painter_at(rect);

        let margin = 50.0;
        let plot_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + margin, rect.min.y + 25.0),
            Pos2::new(rect.max.x - 30.0, rect.max.y - margin),
        );

        painter.rect_filled(rect, 8.0, Color32::from_rgb(15, 18, 26));
        painter.rect_filled(plot_rect, 0.0, Color32::from_rgb(8, 10, 16));

        let to_screen = |lps: f32, stat1: f32| -> Pos2 {
            let px = plot_rect.min.x + (lps / 3.0) * plot_rect.width();
            let py = plot_rect.max.y - (stat1 / 3.0) * plot_rect.height();
            Pos2::new(px, py)
        };

        // Shaded bistable region (Hysteresis window between ~0.6 and ~1.6 LPS)
        let p_bistable_min = to_screen(0.65, 0.0).x;
        let p_bistable_max = to_screen(1.65, 0.0).x;
        let bistable_rect = Rect::from_min_max(
            Pos2::new(p_bistable_min, plot_rect.min.y),
            Pos2::new(p_bistable_max, plot_rect.max.y),
        );
        painter.rect_filled(bistable_rect, 0.0, Color32::from_rgba_unmultiplied(168, 85, 247, 30));
        painter.text(
            Pos2::new(bistable_rect.center().x, plot_rect.min.y + 12.0),
            egui::Align2::CENTER_TOP,
            "Bistable Hysteresis Window\n(Epigenetic Memory)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(192, 132, 252),
        );

        // Compute bifurcation fixed points
        let branch_points: Vec<BifurcationBranchPoint> = compute_lps_bifurcation_curve(params);

        for pt in &branch_points {
            let pos = to_screen(pt.input_val, pt.x_val);
            if pt.is_stable {
                painter.circle_filled(pos, 2.2, Color32::from_rgb(56, 189, 248));
            } else {
                painter.circle_filled(pos, 1.6, Color32::from_rgb(244, 63, 94));
            }
        }

        // Current operating state marker (Vertical line at current s_lps)
        let cur_lps_x = to_screen(params.s_lps, 0.0).x;
        painter.line_segment(
            [Pos2::new(cur_lps_x, plot_rect.min.y), Pos2::new(cur_lps_x, plot_rect.max.y)],
            Stroke::new(1.5, Color32::from_rgb(234, 179, 8)),
        );
        painter.text(
            Pos2::new(cur_lps_x, plot_rect.max.y + 16.0),
            egui::Align2::CENTER_TOP,
            format!("Current LPS: {:.2}", params.s_lps),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(234, 179, 8),
        );

        // Axes and Labels
        let axis_stroke = Stroke::new(1.2, Color32::from_rgb(100, 116, 139));
        painter.rect_stroke(plot_rect, 0.0, axis_stroke);

        for i in 0..=3 {
            let val = i as f32;
            let px = to_screen(val, 0.0);
            let py = to_screen(0.0, val);
            painter.text(Pos2::new(px.x, plot_rect.max.y + 4.0), egui::Align2::CENTER_TOP, format!("{:.1}", val), egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(plot_rect.min.x - 6.0, py.y), egui::Align2::RIGHT_CENTER, format!("{:.1}", val), egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        }

        painter.text(Pos2::new(plot_rect.center().x, rect.max.y - 10.0), egui::Align2::CENTER_BOTTOM, "Bifurcation Parameter: LPS / IFN-γ Cytokine Input", egui::FontId::proportional(12.0), Color32::from_rgb(248, 113, 113));
        painter.text(Pos2::new(plot_rect.min.x, rect.min.y + 4.0), egui::Align2::LEFT_TOP, "Steady State STAT1 Expression (x*)", egui::FontId::proportional(12.0), Color32::from_rgb(56, 189, 248));
    }

    /// Renders Time-Series Trajectory Kinetics Strip Chart
    pub fn render_time_series_chart(
        ui: &mut egui::Ui,
        history: &VecDeque<TimeSeriesPoint>,
    ) {
        let available_size = ui.available_size();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(available_size.x, 320.0), egui::Sense::hover());

        let painter = ui.painter_at(rect);

        let margin_left = 50.0;
        let margin_bottom = 35.0;
        let plot_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + margin_left, rect.min.y + 25.0),
            Pos2::new(rect.max.x - 20.0, rect.max.y - margin_bottom),
        );

        painter.rect_filled(rect, 8.0, Color32::from_rgb(15, 18, 26));
        painter.rect_filled(plot_rect, 0.0, Color32::from_rgb(8, 10, 16));

        if history.len() < 2 {
            painter.text(
                plot_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Accumulating kinetics data points...",
                egui::FontId::proportional(14.0),
                Color32::from_rgb(148, 163, 184),
            );
            return;
        }

        let min_t = history.front().unwrap().time;
        let max_t = history.back().unwrap().time.max(min_t + 0.1);

        let to_screen = |t: f32, val: f32| -> Pos2 {
            let px = plot_rect.min.x + ((t - min_t) / (max_t - min_t)) * plot_rect.width();
            let py = plot_rect.max.y - (val / 3.0) * plot_rect.height();
            Pos2::new(px, py)
        };

        // Draw Curves: STAT1 (Red), STAT6 (Green), Entropy (Purple)
        let stroke_stat1 = Stroke::new(2.0, Color32::from_rgb(239, 68, 68));
        let stroke_stat6 = Stroke::new(2.0, Color32::from_rgb(34, 197, 94));
        let stroke_entropy = Stroke::new(1.8, Color32::from_rgb(192, 132, 252));

        for i in 0..history.len() - 1 {
            let p1 = &history[i];
            let p2 = &history[i + 1];

            painter.line_segment([to_screen(p1.time, p1.mean_stat1), to_screen(p2.time, p2.mean_stat1)], stroke_stat1);
            painter.line_segment([to_screen(p1.time, p1.mean_stat6), to_screen(p2.time, p2.mean_stat6)], stroke_stat6);
            painter.line_segment([to_screen(p1.time, p1.entropy), to_screen(p2.time, p2.entropy)], stroke_entropy);
        }

        // Legend at top
        painter.text(Pos2::new(plot_rect.min.x + 10.0, rect.min.y + 6.0), egui::Align2::LEFT_TOP, "— Mean STAT1 (M1)", egui::FontId::proportional(11.0), Color32::from_rgb(239, 68, 68));
        painter.text(Pos2::new(plot_rect.min.x + 140.0, rect.min.y + 6.0), egui::Align2::LEFT_TOP, "— Mean STAT6 (M2)", egui::FontId::proportional(11.0), Color32::from_rgb(34, 197, 94));
        painter.text(Pos2::new(plot_rect.min.x + 270.0, rect.min.y + 6.0), egui::Align2::LEFT_TOP, "— Shannon Entropy (H)", egui::FontId::proportional(11.0), Color32::from_rgb(192, 132, 252));

        // Border and axes
        let axis_stroke = Stroke::new(1.2, Color32::from_rgb(100, 116, 139));
        painter.rect_stroke(plot_rect, 0.0, axis_stroke);

        painter.text(Pos2::new(plot_rect.min.x, plot_rect.max.y + 4.0), egui::Align2::LEFT_TOP, format!("{:.1}s", min_t), egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(plot_rect.max.x, plot_rect.max.y + 4.0), egui::Align2::RIGHT_TOP, format!("{:.1}s", max_t), egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(plot_rect.center().x, rect.max.y - 4.0), egui::Align2::CENTER_BOTTOM, "Simulation Time (s)", egui::FontId::proportional(11.0), Color32::from_rgb(203, 213, 225));
    }

    fn draw_heatmap(painter: &Painter, plot_rect: Rect, params: &BiophysicalParams, res: usize) {
        let step_x = plot_rect.width() / res as f32;
        let step_y = plot_rect.height() / res as f32;

        let mut potentials = Vec::with_capacity(res * res);
        let mut min_u = f32::MAX;
        let mut max_u = f32::MIN;

        for j in 0..res {
            for i in 0..res {
                let bx = (i as f32 + 0.5) / res as f32 * 3.0;
                let by = (j as f32 + 0.5) / res as f32 * 3.0;
                let u = compute_waddington_potential(bx, by, params);
                if u < min_u { min_u = u; }
                if u > max_u { max_u = u; }
                potentials.push(u);
            }
        }

        let range = (max_u - min_u).max(0.001);

        for j in 0..res {
            for i in 0..res {
                let u = potentials[j * res + i];
                let norm = ((u - min_u) / range).clamp(0.0, 1.0);
                let color = potential_to_color(norm);

                let x0 = plot_rect.min.x + i as f32 * step_x;
                let y0 = plot_rect.max.y - (j as f32 + 1.0) * step_y;
                let cell_rect = Rect::from_min_size(Pos2::new(x0, y0), Vec2::new(step_x + 0.5, step_y + 0.5));
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }
    }

    fn draw_contours<F>(painter: &Painter, params: &BiophysicalParams, to_screen: F)
    where
        F: Fn(f32, f32) -> Pos2,
    {
        let steps = 30;
        let mut grid = vec![vec![0.0f32; steps]; steps];
        let mut min_u = f32::MAX;
        let mut max_u = f32::MIN;

        for j in 0..steps {
            for i in 0..steps {
                let bx = (i as f32) / (steps - 1) as f32 * 3.0;
                let by = (j as f32) / (steps - 1) as f32 * 3.0;
                let u = compute_waddington_potential(bx, by, params);
                if u < min_u { min_u = u; }
                if u > max_u { max_u = u; }
                grid[j][i] = u;
            }
        }

        let num_levels = 8;
        let stroke = Stroke::new(0.8, Color32::from_rgba_unmultiplied(255, 255, 255, 75));

        for l in 1..num_levels {
            let level_val = min_u + (l as f32 / num_levels as f32) * (max_u - min_u);

            for j in 0..steps - 1 {
                for i in 0..steps - 1 {
                    let v00 = grid[j][i];
                    let v10 = grid[j][i + 1];
                    let v01 = grid[j + 1][i];
                    let v11 = grid[j + 1][i + 1];

                    let bx0 = (i as f32) / (steps - 1) as f32 * 3.0;
                    let bx1 = ((i + 1) as f32) / (steps - 1) as f32 * 3.0;
                    let by0 = (j as f32) / (steps - 1) as f32 * 3.0;
                    let by1 = ((j + 1) as f32) / (steps - 1) as f32 * 3.0;

                    let mut pts = Vec::new();

                    if (v00 <= level_val && v10 >= level_val) || (v00 >= level_val && v10 <= level_val) {
                        let t = (level_val - v00) / (v10 - v00 + 1e-6);
                        pts.push(to_screen(bx0 + t * (bx1 - bx0), by0));
                    }
                    if (v01 <= level_val && v11 >= level_val) || (v01 >= level_val && v11 <= level_val) {
                        let t = (level_val - v01) / (v11 - v01 + 1e-6);
                        pts.push(to_screen(bx0 + t * (bx1 - bx0), by1));
                    }
                    if (v00 <= level_val && v01 >= level_val) || (v00 >= level_val && v01 <= level_val) {
                        let t = (level_val - v00) / (v01 - v00 + 1e-6);
                        pts.push(to_screen(bx0, by0 + t * (by1 - by0)));
                    }
                    if (v10 <= level_val && v11 >= level_val) || (v10 >= level_val && v11 <= level_val) {
                        let t = (level_val - v10) / (v11 - v10 + 1e-6);
                        pts.push(to_screen(bx1, by0 + t * (by1 - by0)));
                    }

                    if pts.len() == 2 {
                        painter.line_segment([pts[0], pts[1]], stroke);
                    }
                }
            }
        }
    }

    fn draw_vector_field<F>(painter: &Painter, params: &BiophysicalParams, to_screen: F)
    where
        F: Fn(f32, f32) -> Pos2,
    {
        let grid_n = 11;
        for j in 0..grid_n {
            for i in 0..grid_n {
                let bx = 0.2 + (i as f32 / (grid_n - 1) as f32) * 2.6;
                let by = 0.2 + (j as f32 / (grid_n - 1) as f32) * 2.6;

                let (fx, fy) = compute_drift_vector(bx, by, params);
                let mag = (fx * fx + fy * fy).sqrt();
                if mag < 0.01 {
                    continue;
                }

                let scale = (mag.min(2.0) / 2.0) * 0.12 + 0.04;
                let end_bx = bx + (fx / mag) * scale;
                let end_by = by + (fy / mag) * scale;

                let p1 = to_screen(bx, by);
                let p2 = to_screen(end_bx, end_by);

                let stroke = Stroke::new(1.1, Color32::from_rgba_unmultiplied(220, 230, 255, 140));
                painter.line_segment([p1, p2], stroke);

                let dir = (p2 - p1).normalized();
                let normal = Vec2::new(-dir.y, dir.x);
                let arrow_head_len = 3.5;
                let arrow_head_w = 2.5;
                let p_left = p2 - dir * arrow_head_len + normal * arrow_head_w;
                let p_right = p2 - dir * arrow_head_len - normal * arrow_head_w;

                painter.line_segment([p2, p_left], stroke);
                painter.line_segment([p2, p_right], stroke);
            }
        }
    }

    fn draw_nullclines<F>(painter: &Painter, params: &BiophysicalParams, to_screen: F)
    where
        F: Fn(f32, f32) -> Pos2,
    {
        let steps = 40;
        let stroke_x = Stroke::new(1.5, Color32::from_rgba_unmultiplied(248, 113, 113, 200));
        let stroke_y = Stroke::new(1.5, Color32::from_rgba_unmultiplied(96, 165, 250, 200));

        let mut pts_x = Vec::new();
        for i in 0..steps {
            let by = (i as f32 / (steps - 1) as f32) * 3.0;
            let mut best_x = 0.0;
            let mut min_diff = f32::MAX;
            for k in 0..60 {
                let bx = (k as f32 / 59.0) * 3.0;
                let (fx, _) = compute_drift_vector(bx, by, params);
                if fx.abs() < min_diff {
                    min_diff = fx.abs();
                    best_x = bx;
                }
            }
            if min_diff < 0.15 {
                pts_x.push(to_screen(best_x, by));
            }
        }
        for i in 0..pts_x.len().saturating_sub(1) {
            painter.line_segment([pts_x[i], pts_x[i + 1]], stroke_x);
        }

        let mut pts_y = Vec::new();
        for i in 0..steps {
            let bx = (i as f32 / (steps - 1) as f32) * 3.0;
            let mut best_y = 0.0;
            let mut min_diff = f32::MAX;
            for k in 0..60 {
                let by = (k as f32 / 59.0) * 3.0;
                let (_, fy) = compute_drift_vector(bx, by, params);
                if fy.abs() < min_diff {
                    min_diff = fy.abs();
                    best_y = by;
                }
            }
            if min_diff < 0.15 {
                pts_y.push(to_screen(bx, best_y));
            }
        }
        for i in 0..pts_y.len().saturating_sub(1) {
            painter.line_segment([pts_y[i], pts_y[i + 1]], stroke_y);
        }
    }

    fn draw_attractor_badges<F>(painter: &Painter, to_screen: F)
    where
        F: Fn(f32, f32) -> Pos2,
    {
        let attractors = [
            (0.45, 0.45, "M0 (Naive)", Color32::from_rgb(160, 174, 192)),
            (2.20, 0.30, "M1 (Pro-inflammatory / Glycolysis)", Color32::from_rgb(239, 68, 68)),
            (0.30, 2.20, "M2a (Wound Healing / OXPHOS)", Color32::from_rgb(34, 197, 94)),
            (1.80, 1.80, "M2d (TAM) / M-Hybrid", Color32::from_rgb(249, 115, 22)),
        ];

        for (bx, by, text, color) in attractors {
            let pos = to_screen(bx, by);
            painter.circle_stroke(pos, 8.0, Stroke::new(1.5, color));
            painter.circle_filled(pos, 3.0, color);
            painter.text(
                Pos2::new(pos.x, pos.y + 12.0),
                egui::Align2::CENTER_TOP,
                text,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(240, 240, 240),
            );
        }
    }

    fn draw_axes_and_labels<F>(painter: &Painter, rect: Rect, plot_rect: Rect, to_screen: F)
    where
        F: Fn(f32, f32) -> Pos2,
    {
        let axis_stroke = Stroke::new(1.2, Color32::from_rgb(100, 116, 139));
        let grid_stroke = Stroke::new(0.5, Color32::from_rgba_unmultiplied(100, 116, 139, 45));

        painter.rect_stroke(plot_rect, 0.0, axis_stroke);

        for i in 0..=3 {
            let val = i as f32;
            let p_x = to_screen(val, 0.0);
            let p_y = to_screen(0.0, val);

            painter.line_segment(
                [Pos2::new(p_x.x, plot_rect.max.y), Pos2::new(p_x.x, plot_rect.max.y + 5.0)],
                axis_stroke,
            );
            if i > 0 && i < 3 {
                painter.line_segment([Pos2::new(p_x.x, plot_rect.min.y), Pos2::new(p_x.x, plot_rect.max.y)], grid_stroke);
            }
            painter.text(
                Pos2::new(p_x.x, plot_rect.max.y + 8.0),
                egui::Align2::CENTER_TOP,
                format!("{:.1}", val),
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 174, 192),
            );

            painter.line_segment(
                [Pos2::new(plot_rect.min.x - 5.0, p_y.y), Pos2::new(plot_rect.min.x, p_y.y)],
                axis_stroke,
            );
            if i > 0 && i < 3 {
                painter.line_segment([Pos2::new(plot_rect.min.x, p_y.y), Pos2::new(plot_rect.max.x, p_y.y)], grid_stroke);
            }
            painter.text(
                Pos2::new(plot_rect.min.x - 8.0, p_y.y),
                egui::Align2::RIGHT_CENTER,
                format!("{:.1}", val),
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 174, 192),
            );
        }

        painter.text(
            Pos2::new(plot_rect.center().x, rect.max.y - 12.0),
            egui::Align2::CENTER_BOTTOM,
            "STAT1 / NF-κB / Pro-inflammatory Master Axis (M1)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(248, 113, 113),
        );

        painter.text(
            Pos2::new(plot_rect.min.x, rect.min.y + 6.0),
            egui::Align2::LEFT_TOP,
            "STAT6 / PPAR-γ / Pro-healing Master Axis (M2)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(74, 222, 128),
        );
    }

    fn draw_colorbar(painter: &Painter, rect: Rect, plot_rect: Rect) {
        let bar_width = 12.0;
        let bar_rect = Rect::from_min_max(
            Pos2::new(rect.max.x - 48.0, plot_rect.min.y + 10.0),
            Pos2::new(rect.max.x - 48.0 + bar_width, plot_rect.max.y - 10.0),
        );

        let steps = 24;
        let step_h = bar_rect.height() / steps as f32;

        for i in 0..steps {
            let norm = 1.0 - (i as f32 / steps as f32);
            let color = potential_to_color(norm);
            let cell = Rect::from_min_size(
                Pos2::new(bar_rect.min.x, bar_rect.min.y + i as f32 * step_h),
                Vec2::new(bar_width, step_h + 0.5),
            );
            painter.rect_filled(cell, 0.0, color);
        }
        painter.rect_stroke(bar_rect, 0.0, Stroke::new(1.0, Color32::from_rgb(100, 116, 139)));

        painter.text(
            Pos2::new(bar_rect.max.x + 4.0, bar_rect.min.y),
            egui::Align2::LEFT_CENTER,
            "High (Barrier)",
            egui::FontId::proportional(9.0),
            Color32::from_rgb(248, 113, 113),
        );
        painter.text(
            Pos2::new(bar_rect.max.x + 4.0, bar_rect.max.y),
            egui::Align2::LEFT_CENTER,
            "Low (Valley)",
            egui::FontId::proportional(9.0),
            Color32::from_rgb(56, 189, 248),
        );
        painter.text(
            Pos2::new(bar_rect.center().x, bar_rect.min.y - 12.0),
            egui::Align2::CENTER_BOTTOM,
            "U(x,y)",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(203, 213, 225),
        );
    }

    pub fn generate_high_res_png(
        params: &BiophysicalParams,
        cells: &[SingleCell],
        exp_dataset: Option<&ExperimentalDataset>,
        file_path: &str,
    ) -> Result<(), String> {
        let width = 1000;
        let height = 800;
        let mut img = ImageBuffer::new(width, height);

        let plot_x0 = 80;
        let plot_y0 = 60;
        let plot_w = 780;
        let plot_h = 660;

        let mut min_u = f32::MAX;
        let mut max_u = f32::MIN;

        for j in 0..100 {
            for i in 0..100 {
                let bx = (i as f32 / 99.0) * 3.0;
                let by = (j as f32 / 99.0) * 3.0;
                let u = compute_waddington_potential(bx, by, params);
                if u < min_u { min_u = u; }
                if u > max_u { max_u = u; }
            }
        }
        let range_u = (max_u - min_u).max(0.001);

        for y in 0..height {
            for x in 0..width {
                if x >= plot_x0 && x < plot_x0 + plot_w && y >= plot_y0 && y < plot_y0 + plot_h {
                    let bx = ((x - plot_x0) as f32 / plot_w as f32) * 3.0;
                    let by = ((plot_y0 + plot_h - y) as f32 / plot_h as f32) * 3.0;
                    let u = compute_waddington_potential(bx, by, params);
                    let norm = ((u - min_u) / range_u).clamp(0.0, 1.0);
                    let [r, g, b] = potential_to_rgb(norm);
                    img.put_pixel(x, y, Rgb([r, g, b]));
                } else {
                    img.put_pixel(x, y, Rgb([16, 20, 28]));
                }
            }
        }

        for cell in cells {
            let cx = plot_x0 as f32 + (cell.x / 3.0) * plot_w as f32;
            let cy = (plot_y0 + plot_h) as f32 - (cell.y / 3.0) * plot_h as f32;
            let [r, g, b, _] = cell.phenotype.color_rgba();

            for dy in -3i32..=3i32 {
                for dx in -3i32..=3i32 {
                    if dx * dx + dy * dy <= 9 {
                        let px = (cx as i32 + dx) as u32;
                        let py = (cy as i32 + dy) as u32;
                        if px < width && py < height {
                            img.put_pixel(px, py, Rgb([r, g, b]));
                        }
                    }
                }
            }
        }

        if let Some(dataset) = exp_dataset {
            for cell in &dataset.cells {
                let cx = plot_x0 as f32 + (cell.x / 3.0) * plot_w as f32;
                let cy = (plot_y0 + plot_h) as f32 - (cell.y / 3.0) * plot_h as f32;

                for dy in -4i32..=4i32 {
                    for dx in -4i32..=4i32 {
                        if (dx.abs() + dy.abs()) <= 4 {
                            let px = (cx as i32 + dx) as u32;
                            let py = (cy as i32 + dy) as u32;
                            if px < width && py < height {
                                img.put_pixel(px, py, Rgb([240, 240, 255]));
                            }
                        }
                    }
                }
            }
        }

        img.save(file_path).map_err(|e| format!("Failed to save PNG: {}", e))?;
        Ok(())
    }
}

fn potential_to_color(norm: f32) -> Color32 {
    let [r, g, b] = potential_to_rgb(norm);
    Color32::from_rgb(r, g, b)
}

fn potential_to_rgb(norm: f32) -> [u8; 3] {
    if norm < 0.25 {
        let t = norm / 0.25;
        let r = (15.0 + t * 15.0) as u8;
        let g = (25.0 + t * 65.0) as u8;
        let b = (70.0 + t * 100.0) as u8;
        [r, g, b]
    } else if norm < 0.50 {
        let t = (norm - 0.25) / 0.25;
        let r = (30.0 + t * 20.0) as u8;
        let g = (90.0 + t * 80.0) as u8;
        let b = (170.0 - t * 40.0) as u8;
        [r, g, b]
    } else if norm < 0.75 {
        let t = (norm - 0.50) / 0.25;
        let r = (50.0 + t * 150.0) as u8;
        let g = (170.0 + t * 30.0) as u8;
        let b = (130.0 - t * 100.0) as u8;
        [r, g, b]
    } else {
        let t = (norm - 0.75) / 0.25;
        let r = (200.0 + t * 45.0) as u8;
        let g = (200.0 - t * 140.0) as u8;
        let b = (30.0 - t * 10.0) as u8;
        [r, g, b]
    }
}
