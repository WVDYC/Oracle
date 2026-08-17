// Mathematical & Biophysical Model for Macrophage Epigenetic Landscape
// Implements nonlinear Gene Regulatory Networks (GRN), Hill kinetics,
// Langevin stochastic dynamics, Waddington quasi-potential, Shannon entropy,
// Pharmacological drug assays, time-series kinetics, and bifurcation solver.

use rand::Rng;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Phenotype {
    M0, // Naive / Uncommitted
    M1, // Pro-inflammatory (STAT1 / NF-kB high)
    M2, // Pro-healing / Tissue repair (STAT6 / PPAR-g high)
    M3, // Alternative / Hypoxia-induced / Hybrid plastic state
}

impl Phenotype {
    pub fn name(&self) -> &'static str {
        match self {
            Phenotype::M0 => "M0 (Naive / Baseline)",
            Phenotype::M1 => "M1 (Pro-inflammatory / Attack)",
            Phenotype::M2 => "M2 (Pro-healing / Repair)",
            Phenotype::M3 => "M3 (Alternative / Hybrid Plastic)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Phenotype::M0 => "M0",
            Phenotype::M1 => "M1",
            Phenotype::M2 => "M2",
            Phenotype::M3 => "M3",
        }
    }

    pub fn color_rgba(&self) -> [u8; 4] {
        match self {
            Phenotype::M0 => [148, 163, 184, 230], // Slate / Gray
            Phenotype::M1 => [239, 68, 68, 230],   // Crimson / Red
            Phenotype::M2 => [34, 197, 94, 230],   // Emerald / Green
            Phenotype::M3 => [234, 179, 8, 230],   // Amber / Yellow-Gold
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DrugAssaySettings {
    pub jak_inhibitor: f32,     // JAK1/2 inhibitor (Tofacitinib/Ruxolitinib) [0.0 .. 1.0]
    pub anti_il4r_mab: f32,     // Anti-IL-4R antibody (Dupilumab-like) [0.0 .. 1.0]
    pub tlr4_antagonist: f32,   // TLR4 antagonist [0.0 .. 1.0]
    pub hif1a_inhibitor: f32,   // HIF-1α transcription inhibitor [0.0 .. 1.0]
    pub hdac_inhibitor: f32,    // HDAC inhibitor (increases epigenetic plasticity) [0.0 .. 1.0]
}

impl Default for DrugAssaySettings {
    fn default() -> Self {
        Self {
            jak_inhibitor: 0.0,
            anti_il4r_mab: 0.0,
            tlr4_antagonist: 0.0,
            hif1a_inhibitor: 0.0,
            hdac_inhibitor: 0.0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BiophysicalParams {
    pub s_lps: f32,      // LPS / IFN-g inflammatory signal [0.0 .. 3.0]
    pub s_il4: f32,      // IL-4 / IL-13 anti-inflammatory signal [0.0 .. 3.0]
    pub s_hypoxia: f32,  // Hypoxia / M3 alternative polarization signal [0.0 .. 3.0]
    pub hill_n: f32,     // Hill cooperativity coefficient n (usually 2.0 - 4.0)
    pub gamma: f32,      // Mutual cross-repression strength (STAT1 <-> STAT6)
    pub alpha: f32,      // Basal self-activation rate
    pub delta: f32,      // Degradation / clearance rate
    pub noise_sigma: f32,// Stochastic gene expression noise amplitude
    pub drugs: DrugAssaySettings,
}

impl Default for BiophysicalParams {
    fn default() -> Self {
        Self {
            s_lps: 0.70,
            s_il4: 0.20,
            s_hypoxia: 0.10,
            hill_n: 3.0,
            gamma: 1.5,
            alpha: 1.8,
            delta: 1.0,
            noise_sigma: 0.18,
            drugs: DrugAssaySettings::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SingleCell {
    pub id: usize,
    pub x: f32, // STAT1 / M1 master regulator activity [0.0 .. 3.0]
    pub y: f32, // STAT6 / M2 master regulator activity [0.0 .. 3.0]
    pub vx: f32,
    pub vy: f32,
    pub phenotype: Phenotype,
    pub trail: VecDeque<[f32; 2]>,
}

impl SingleCell {
    pub fn new(id: usize, x: f32, y: f32) -> Self {
        let phenotype = classify_cell_phenotype(x, y);
        let mut trail = VecDeque::with_capacity(16);
        trail.push_back([x, y]);
        Self {
            id,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            phenotype,
            trail,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PopulationStats {
    pub total_cells: usize,
    pub count_m0: usize,
    pub count_m1: usize,
    pub count_m2: usize,
    pub count_m3: usize,
    pub pct_m0: f32,
    pub pct_m1: f32,
    pub pct_m2: f32,
    pub pct_m3: f32,
    pub mean_x: f32,
    pub mean_y: f32,
    pub shannon_entropy: f32,
    pub barrier_m1_m2: f32,
    pub barrier_m2_m1: f32,
}

#[derive(Debug, Clone)]
pub struct TimeSeriesPoint {
    pub time: f32,
    pub mean_stat1: f32,
    pub mean_stat6: f32,
    pub entropy: f32,
    pub pct_m1: f32,
    pub pct_m2: f32,
}

#[derive(Debug, Clone)]
pub struct BifurcationBranchPoint {
    pub input_val: f32,
    pub x_val: f32,
    pub is_stable: bool,
}

pub struct SimulationModel {
    pub params: BiophysicalParams,
    pub cells: Vec<SingleCell>,
    pub stats: PopulationStats,
    pub time_series_history: VecDeque<TimeSeriesPoint>,
    pub sim_time: f32,
    pub is_running: bool,
    pub next_cell_id: usize,
    pub history_record_timer: f32,
}

impl Default for SimulationModel {
    fn default() -> Self {
        let params = BiophysicalParams::default();
        let mut model = Self {
            params,
            cells: Vec::new(),
            stats: PopulationStats {
                total_cells: 0,
                count_m0: 0,
                count_m1: 0,
                count_m2: 0,
                count_m3: 0,
                pct_m0: 0.0,
                pct_m1: 0.0,
                pct_m2: 0.0,
                pct_m3: 0.0,
                mean_x: 0.0,
                mean_y: 0.0,
                shannon_entropy: 0.0,
                barrier_m1_m2: 0.0,
                barrier_m2_m1: 0.0,
            },
            time_series_history: VecDeque::with_capacity(320),
            sim_time: 0.0,
            is_running: true,
            next_cell_id: 1,
            history_record_timer: 0.0,
        };
        model.init_population(200);
        model.update_stats();
        model
    }
}

impl SimulationModel {
    pub fn init_population(&mut self, count: usize) {
        let mut rng = rand::thread_rng();
        self.cells.clear();
        self.next_cell_id = 1;
        for _ in 0..count {
            let x = (0.4f32 + rng.gen_range(-0.2f32..0.2f32)).clamp(0.05, 3.0);
            let y = (0.4f32 + rng.gen_range(-0.2f32..0.2f32)).clamp(0.05, 3.0);
            self.cells.push(SingleCell::new(self.next_cell_id, x, y));
            self.next_cell_id += 1;
        }
        self.time_series_history.clear();
        self.update_stats();
        self.record_history_point();
    }

    pub fn inject_cells_at(&mut self, x: f32, y: f32, count: usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let cx = (x + rng.gen_range(-0.1f32..0.1f32)).clamp(0.05, 3.0);
            let cy = (y + rng.gen_range(-0.1f32..0.1f32)).clamp(0.05, 3.0);
            self.cells.push(SingleCell::new(self.next_cell_id, cx, cy));
            self.next_cell_id += 1;
        }
        self.update_stats();
    }

    pub fn set_cell_count(&mut self, target_count: usize) {
        let current = self.cells.len();
        if target_count > current {
            let mut rng = rand::thread_rng();
            for _ in current..target_count {
                let x = (0.5f32 + rng.gen_range(-0.3f32..0.3f32)).clamp(0.05, 3.0);
                let y = (0.5f32 + rng.gen_range(-0.3f32..0.3f32)).clamp(0.05, 3.0);
                self.cells.push(SingleCell::new(self.next_cell_id, x, y));
                self.next_cell_id += 1;
            }
        } else if target_count < current {
            self.cells.truncate(target_count);
        }
        self.update_stats();
    }

    pub fn add_cytokine_shock(&mut self, shock_type: Phenotype) {
        match shock_type {
            Phenotype::M1 => {
                self.params.s_lps = (self.params.s_lps + 1.2).min(3.0);
                self.params.s_il4 = (self.params.s_il4 - 0.5).max(0.0);
            }
            Phenotype::M2 => {
                self.params.s_il4 = (self.params.s_il4 + 1.2).min(3.0);
                self.params.s_lps = (self.params.s_lps - 0.5).max(0.0);
            }
            Phenotype::M3 => {
                self.params.s_hypoxia = (self.params.s_hypoxia + 1.5).min(3.0);
            }
            Phenotype::M0 => {
                self.params.s_lps = 0.1;
                self.params.s_il4 = 0.1;
                self.params.s_hypoxia = 0.05;
            }
        }
        self.update_stats();
    }

    pub fn reset_to_m0(&mut self) {
        let mut rng = rand::thread_rng();
        for cell in &mut self.cells {
            cell.x = (0.4f32 + rng.gen_range(-0.15f32..0.15f32)).clamp(0.05, 3.0);
            cell.y = (0.4f32 + rng.gen_range(-0.15f32..0.15f32)).clamp(0.05, 3.0);
            cell.vx = 0.0;
            cell.vy = 0.0;
            cell.phenotype = Phenotype::M0;
            cell.trail.clear();
            cell.trail.push_back([cell.x, cell.y]);
        }
        self.update_stats();
        self.record_history_point();
    }

    pub fn step(&mut self, dt: f32) {
        if !self.is_running || dt <= 0.0 {
            return;
        }

        let mut rng = rand::thread_rng();
        let sqrt_dt = dt.sqrt();
        // Effective noise increases under HDAC inhibitors
        let sigma = self.params.noise_sigma * (1.0 + 1.5 * self.params.drugs.hdac_inhibitor);

        for cell in &mut self.cells {
            let (fx, fy) = compute_drift_vector(cell.x, cell.y, &self.params);

            // Euler-Maruyama stochastic integration
            let u1: f32 = rng.gen_range(0.0001..1.0);
            let u2: f32 = rng.gen_range(0.0001..1.0);
            let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
            let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).sin();

            let dx = fx * dt + sigma * sqrt_dt * z0;
            let dy = fy * dt + sigma * sqrt_dt * z1;

            cell.vx = dx / dt;
            cell.vy = dy / dt;

            cell.x = (cell.x + dx).clamp(0.02, 3.0);
            cell.y = (cell.y + dy).clamp(0.02, 3.0);

            cell.phenotype = classify_cell_phenotype(cell.x, cell.y);

            if cell.trail.len() >= 14 {
                cell.trail.pop_front();
            }
            cell.trail.push_back([cell.x, cell.y]);
        }

        self.sim_time += dt;
        self.history_record_timer += dt;

        self.update_stats();

        // Record time series point every 0.1s
        if self.history_record_timer >= 0.1 {
            self.history_record_timer = 0.0;
            self.record_history_point();
        }
    }

    fn record_history_point(&mut self) {
        if self.time_series_history.len() >= 300 {
            self.time_series_history.pop_front();
        }
        self.time_series_history.push_back(TimeSeriesPoint {
            time: self.sim_time,
            mean_stat1: self.stats.mean_x,
            mean_stat6: self.stats.mean_y,
            entropy: self.stats.shannon_entropy,
            pct_m1: self.stats.pct_m1,
            pct_m2: self.stats.pct_m2,
        });
    }

    pub fn update_stats(&mut self) {
        let total = self.cells.len();
        if total == 0 {
            return;
        }

        let mut m0 = 0;
        let mut m1 = 0;
        let mut m2 = 0;
        let mut m3 = 0;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;

        for cell in &self.cells {
            sum_x += cell.x;
            sum_y += cell.y;
            match cell.phenotype {
                Phenotype::M0 => m0 += 1,
                Phenotype::M1 => m1 += 1,
                Phenotype::M2 => m2 += 1,
                Phenotype::M3 => m3 += 1,
            }
        }

        let n = total as f32;
        let p0 = m0 as f32 / n;
        let p1 = m1 as f32 / n;
        let p2 = m2 as f32 / n;
        let p3 = m3 as f32 / n;

        // Shannon entropy H = - sum(p * ln(p))
        let mut entropy = 0.0;
        for p in [p0, p1, p2, p3] {
            if p > 1e-6 {
                entropy -= p * p.ln();
            }
        }

        let u_m1 = compute_waddington_potential(2.0, 0.3, &self.params);
        let u_m2 = compute_waddington_potential(0.3, 2.0, &self.params);
        let u_saddle = compute_waddington_potential(1.0, 1.0, &self.params);

        let barrier_m1_m2 = (u_saddle - u_m1).max(0.0);
        let barrier_m2_m1 = (u_saddle - u_m2).max(0.0);

        self.stats = PopulationStats {
            total_cells: total,
            count_m0: m0,
            count_m1: m1,
            count_m2: m2,
            count_m3: m3,
            pct_m0: p0 * 100.0,
            pct_m1: p1 * 100.0,
            pct_m2: p2 * 100.0,
            pct_m3: p3 * 100.0,
            mean_x: sum_x / n,
            mean_y: sum_y / n,
            shannon_entropy: entropy,
            barrier_m1_m2,
            barrier_m2_m1,
        };
    }
}

/// Computes the deterministic drift vector field (dx/dt, dy/dt) with pharmacological interventions
pub fn compute_drift_vector(x: f32, y: f32, params: &BiophysicalParams) -> (f32, f32) {
    let n = params.hill_n;
    let x_n = x.powf(n);
    let y_n = y.powf(n);

    // Apply drug modifications
    let eff_lps = params.s_lps * (1.0 - 0.85 * params.drugs.tlr4_antagonist) * (1.0 - 0.70 * params.drugs.jak_inhibitor);
    let eff_il4 = params.s_il4 * (1.0 - 0.90 * params.drugs.anti_il4r_mab) * (1.0 - 0.70 * params.drugs.jak_inhibitor);
    let eff_hypoxia = params.s_hypoxia * (1.0 - 0.90 * params.drugs.hif1a_inhibitor);

    let denom_x = 1.0 + x_n + params.gamma * y_n;
    let denom_y = 1.0 + y_n + params.gamma * x_n;

    let prod_x = (params.alpha * x_n + eff_lps + 0.10) / denom_x;
    let prod_y = (params.alpha * y_n + eff_il4 + 0.10) / denom_y;

    let hypoxia_x = 0.35 * eff_hypoxia * (y / (1.0 + y));
    let hypoxia_y = 0.35 * eff_hypoxia * (x / (1.0 + x));

    let dx = prod_x + hypoxia_x - params.delta * x;
    let dy = prod_y + hypoxia_y - params.delta * y;

    (dx, dy)
}

/// Calculates the Waddington Quasi-Potential U(x, y) representing the epigenetic landscape
pub fn compute_waddington_potential(x: f32, y: f32, params: &BiophysicalParams) -> f32 {
    let x0 = 0.5;
    let y0 = 0.5;

    let v_m0 = 1.8 * ((x - x0).powi(2) + (y - y0).powi(2));
    let v_m1 = 1.2 * ((x - 2.0).powi(2) + (y - 0.3).powi(2));
    let v_m2 = 1.2 * ((x - 0.3).powi(2) + (y - 2.0).powi(2));
    let v_m3 = 1.5 * ((x - 1.8).powi(2) + (y - 1.8).powi(2));

    let ridge = 1.6 * (-(x - y).powi(2) / 0.8).exp() * ((x + y - 1.2).max(0.0)).min(2.0);

    let beta = 2.0;
    let coupled = -(((-beta * v_m0).exp() + (-beta * v_m1).exp() + (-beta * v_m2).exp() + (-beta * v_m3).exp()).ln()) / beta;

    let eff_lps = params.s_lps * (1.0 - 0.85 * params.drugs.tlr4_antagonist) * (1.0 - 0.70 * params.drugs.jak_inhibitor);
    let eff_il4 = params.s_il4 * (1.0 - 0.90 * params.drugs.anti_il4r_mab) * (1.0 - 0.70 * params.drugs.jak_inhibitor);
    let eff_hypoxia = params.s_hypoxia * (1.0 - 0.90 * params.drugs.hif1a_inhibitor);

    let tilt_lps = -eff_lps * (1.2 * x - 0.4 * y);
    let tilt_il4 = -eff_il4 * (1.2 * y - 0.4 * x);
    let tilt_hypoxia = -eff_hypoxia * 1.1 * (x * y).sqrt();

    let boundary = 0.8 * ((x - 1.5).powi(4) + (y - 1.5).powi(4)) / 3.0;

    coupled + ridge + tilt_lps + tilt_il4 + tilt_hypoxia + boundary
}

/// Classifies single-cell biological phenotype based on STAT1 (x) and STAT6 (y) activation levels
pub fn classify_cell_phenotype(x: f32, y: f32) -> Phenotype {
    if x < 0.75 && y < 0.75 {
        Phenotype::M0
    } else if x >= 0.75 && x > 1.25 * y {
        Phenotype::M1
    } else if y >= 0.75 && y > 1.25 * x {
        Phenotype::M2
    } else {
        Phenotype::M3
    }
}

/// Computes the bifurcation curve of fixed points (stable attractors and unstable saddles) as LPS varies
pub fn compute_lps_bifurcation_curve(params: &BiophysicalParams) -> Vec<BifurcationBranchPoint> {
    let mut points = Vec::new();
    let steps = 45;

    for i in 0..=steps {
        let lps = (i as f32 / steps as f32) * 3.0;
        let mut test_params = params.clone();
        test_params.s_lps = lps;

        // Scan along 1D profile for steady states (dx/dt = 0 when y is near baseline or steady state)
        for x_step in 0..120 {
            let x = (x_step as f32 / 119.0) * 3.0;
            let y = 0.35 / (1.0 + x * x); // quasi-steady state for y
            let (fx, _) = compute_drift_vector(x, y, &test_params);

            if fx.abs() < 0.08 {
                // Check stability by perturbation
                let (fx_p, _) = compute_drift_vector(x + 0.02, y, &test_params);
                let is_stable = fx_p < fx; // negative slope df/dx < 0 means stable
                points.push(BifurcationBranchPoint {
                    input_val: lps,
                    x_val: x,
                    is_stable,
                });
            }
        }
    }

    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phenotype_classification() {
        assert_eq!(classify_cell_phenotype(0.3, 0.3), Phenotype::M0);
        assert_eq!(classify_cell_phenotype(2.1, 0.3), Phenotype::M1);
        assert_eq!(classify_cell_phenotype(0.3, 2.1), Phenotype::M2);
        assert_eq!(classify_cell_phenotype(1.8, 1.8), Phenotype::M3);
    }

    #[test]
    fn test_drug_inhibition() {
        let mut params = BiophysicalParams {
            s_lps: 2.0,
            s_il4: 0.1,
            ..BiophysicalParams::default()
        };
        let (fx_nodrug, _) = compute_drift_vector(1.0, 0.5, &params);

        params.drugs.tlr4_antagonist = 1.0;
        let (fx_withdrug, _) = compute_drift_vector(1.0, 0.5, &params);
        assert!(fx_withdrug < fx_nodrug, "TLR4 antagonist should decrease forward M1 drift");
    }

    #[test]
    fn test_bifurcation_curve_generation() {
        let params = BiophysicalParams::default();
        let curve = compute_lps_bifurcation_curve(&params);
        assert!(!curve.is_empty(), "Bifurcation solver should return branch points");
    }
}
