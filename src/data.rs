// Experimental Data Handling, CSV Parsing, Sample Research Datasets, and Export

use crate::facs::{FacsAnalysisReport, FacsGatingGates};
use crate::model::{
    classify_cell_phenotype, BiophysicalParams, Phenotype, PopulationStats, SingleCell, TimeSeriesPoint,
};
use rand::Rng;
use std::collections::VecDeque;
use std::io::Cursor;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExperimentalCell {
    pub id: String,
    pub x: f32, // STAT1 / CD80 / M1 marker level
    pub y: f32, // STAT6 / CD206 / M2 marker level
    pub phenotype: Phenotype,
    pub original_label: String,
}

#[derive(Debug, Clone)]
pub struct ExperimentalDataset {
    pub name: String,
    pub description: String,
    pub cells: Vec<ExperimentalCell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleDatasetType {
    ControlM0,
    LpsStimulated,
    Il4Stimulated,
    TumorMicroenvironment,
}

impl SampleDatasetType {
    pub fn all() -> &'static [SampleDatasetType] {
        &[
            SampleDatasetType::ControlM0,
            SampleDatasetType::LpsStimulated,
            SampleDatasetType::Il4Stimulated,
            SampleDatasetType::TumorMicroenvironment,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            SampleDatasetType::ControlM0 => "Control (M0 Naive Baseline, N=150)",
            SampleDatasetType::LpsStimulated => "LPS + IFN-γ Stimulated 24h (M1 Polarized, N=200)",
            SampleDatasetType::Il4Stimulated => "IL-4 Stimulated 24h (M2 Polarized, N=200)",
            SampleDatasetType::TumorMicroenvironment => "Tumor Microenvironment (Heterogeneous TAMs, N=250)",
        }
    }
}

pub fn generate_sample_dataset(dataset_type: SampleDatasetType) -> ExperimentalDataset {
    let mut rng = rand::thread_rng();
    let mut cells = Vec::new();

    match dataset_type {
        SampleDatasetType::ControlM0 => {
            for i in 0..150 {
                let x = (0.45f32 + rng.gen_range(-0.18f32..0.18f32)).clamp(0.05, 2.8);
                let y = (0.45f32 + rng.gen_range(-0.18f32..0.18f32)).clamp(0.05, 2.8);
                cells.push(ExperimentalCell {
                    id: format!("CTRL_CELL_{:03}", i + 1),
                    x,
                    y,
                    phenotype: classify_cell_phenotype(x, y),
                    original_label: "M0_Control".to_string(),
                });
            }
            ExperimentalDataset {
                name: "Naive M0 Control".to_string(),
                description: "Single-cell RNA-seq baseline profiling of resting, unstimulated murine bone marrow-derived macrophages (BMDMs).".to_string(),
                cells,
            }
        }
        SampleDatasetType::LpsStimulated => {
            for i in 0..200 {
                let is_responder = rng.gen_bool(0.88);
                let (x, y): (f32, f32) = if is_responder {
                    (
                        (2.1f32 + rng.gen_range(-0.35f32..0.35f32)).clamp(0.1, 2.9),
                        (0.3f32 + rng.gen_range(-0.15f32..0.20f32)).clamp(0.05, 2.9),
                    )
                } else {
                    (
                        (0.9f32 + rng.gen_range(-0.25f32..0.35f32)).clamp(0.1, 2.9),
                        (0.5f32 + rng.gen_range(-0.20f32..0.25f32)).clamp(0.05, 2.9),
                    )
                };
                cells.push(ExperimentalCell {
                    id: format!("LPS_CELL_{:03}", i + 1),
                    x,
                    y,
                    phenotype: classify_cell_phenotype(x, y),
                    original_label: if is_responder { "M1_Polarized" } else { "Intermediate" }.to_string(),
                });
            }
            ExperimentalDataset {
                name: "LPS+IFN-γ 24h (M1 Cohort)".to_string(),
                description: "Flow Cytometry (FACS) profiling of BMDMs treated with 100 ng/mL LPS and 20 ng/mL IFN-γ for 24 hours. High CD80/iNOS expression.".to_string(),
                cells,
            }
        }
        SampleDatasetType::Il4Stimulated => {
            for i in 0..200 {
                let is_responder = rng.gen_bool(0.85);
                let (x, y): (f32, f32) = if is_responder {
                    (
                        (0.3f32 + rng.gen_range(-0.15f32..0.20f32)).clamp(0.05, 2.9),
                        (2.1f32 + rng.gen_range(-0.35f32..0.35f32)).clamp(0.1, 2.9),
                    )
                } else {
                    (
                        (0.5f32 + rng.gen_range(-0.20f32..0.25f32)).clamp(0.05, 2.9),
                        (0.9f32 + rng.gen_range(-0.25f32..0.35f32)).clamp(0.1, 2.9),
                    )
                };
                cells.push(ExperimentalCell {
                    id: format!("IL4_CELL_{:03}", i + 1),
                    x,
                    y,
                    phenotype: classify_cell_phenotype(x, y),
                    original_label: if is_responder { "M2_Polarized" } else { "Intermediate" }.to_string(),
                });
            }
            ExperimentalDataset {
                name: "IL-4 24h (M2 Cohort)".to_string(),
                description: "Single-cell profiling of BMDMs stimulated with 20 ng/mL IL-4 for 24 hours. Strong induction of Arg1, CD206, and STAT6 phosphorylation.".to_string(),
                cells,
            }
        }
        SampleDatasetType::TumorMicroenvironment => {
            for i in 0..250 {
                let subtype = rng.gen_range(0..100);
                let (x, y, label): (f32, f32, &'static str) = if subtype < 50 {
                    (
                        (0.4f32 + rng.gen_range(-0.2f32..0.3f32)).clamp(0.05, 2.9),
                        (1.9f32 + rng.gen_range(-0.35f32..0.35f32)).clamp(0.1, 2.9),
                        "TAM_M2_Immunosuppressive",
                    )
                } else if subtype < 80 {
                    (
                        (1.6f32 + rng.gen_range(-0.3f32..0.3f32)).clamp(0.1, 2.9),
                        (1.6f32 + rng.gen_range(-0.3f32..0.3f32)).clamp(0.1, 2.9),
                        "TAM_Hypoxic_Hybrid",
                    )
                } else {
                    (
                        (1.8f32 + rng.gen_range(-0.3f32..0.3f32)).clamp(0.1, 2.9),
                        (0.5f32 + rng.gen_range(-0.2f32..0.3f32)).clamp(0.05, 2.9),
                        "TAM_M1_Perivascular",
                    )
                };
                cells.push(ExperimentalCell {
                    id: format!("TME_TAM_{:03}", i + 1),
                    x,
                    y,
                    phenotype: classify_cell_phenotype(x, y),
                    original_label: label.to_string(),
                });
            }
            ExperimentalDataset {
                name: "Tumor Microenvironment TAMs".to_string(),
                description: "Tumor-Associated Macrophage (TAM) single-cell suspension from primary solid tumor core, exhibiting high phenotypic plasticity and hypoxia adaptation.".to_string(),
                cells,
            }
        }
    }
}

pub fn parse_csv_data(content: &str, dataset_name: &str) -> Result<ExperimentalDataset, String> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(Cursor::new(content.as_bytes()));

    let headers = reader.headers().map_err(|e| format!("CSV Header Error: {}", e))?.clone();

    let mut x_col = None;
    let mut y_col = None;
    let mut id_col = None;
    let mut label_col = None;

    for (idx, header) in headers.iter().enumerate() {
        let h = header.to_lowercase();
        if h.contains("stat1") || h.contains("cd80") || h.contains("cd86") || h.contains("inos") || h == "m1" || h == "x" {
            if x_col.is_none() {
                x_col = Some(idx);
            }
        } else if h.contains("stat6") || h.contains("cd206") || h.contains("cd163") || h.contains("arg1") || h == "m2" || h == "y" {
            if y_col.is_none() {
                y_col = Some(idx);
            }
        } else if h.contains("id") || h.contains("cell") || h.contains("barcode") {
            if id_col.is_none() {
                id_col = Some(idx);
            }
        } else if h.contains("phenotype") || h.contains("cluster") || h.contains("label") || h.contains("class") {
            if label_col.is_none() {
                label_col = Some(idx);
            }
        }
    }

    let x_idx = x_col.unwrap_or(0);
    let y_idx = y_col.unwrap_or(if headers.len() > 1 { 1 } else { 0 });

    let mut raw_cells = Vec::new();
    let mut max_x = 0.0f32;
    let mut max_y = 0.0f32;

    for (row_idx, result) in reader.records().enumerate() {
        let record = result.map_err(|e| format!("Row {} Read Error: {}", row_idx + 1, e))?;
        if record.len() <= x_idx || record.len() <= y_idx {
            continue;
        }

        let x_val: f32 = record.get(x_idx).unwrap_or("0").parse().unwrap_or(0.0);
        let y_val: f32 = record.get(y_idx).unwrap_or("0").parse().unwrap_or(0.0);
        let id_val = id_col.and_then(|c| record.get(c)).unwrap_or(&format!("Cell_{}", row_idx + 1)).to_string();
        let label_val = label_col.and_then(|c| record.get(c)).unwrap_or("Experimental").to_string();

        if x_val > max_x { max_x = x_val; }
        if y_val > max_y { max_y = y_val; }

        raw_cells.push((id_val, x_val, y_val, label_val));
    }

    if raw_cells.is_empty() {
        return Err("No valid numerical cell data found in CSV.".to_string());
    }

    let scale_x = if max_x > 4.0 { 2.6 / max_x } else { 1.0 };
    let scale_y = if max_y > 4.0 { 2.6 / max_y } else { 1.0 };

    let mut cells = Vec::with_capacity(raw_cells.len());
    for (id, rx, ry, label) in raw_cells {
        let x = (rx * scale_x).clamp(0.02, 3.0);
        let y = (ry * scale_y).clamp(0.02, 3.0);
        let phenotype = classify_cell_phenotype(x, y);
        cells.push(ExperimentalCell {
            id,
            x,
            y,
            phenotype,
            original_label: label,
        });
    }

    Ok(ExperimentalDataset {
        name: dataset_name.to_string(),
        description: format!("Loaded {} experimental cells from custom CSV. Auto-scaled to Waddington coordinate space.", cells.len()),
        cells,
    })
}

pub fn export_simulation_csv(cells: &[SingleCell], params: &BiophysicalParams) -> String {
    let mut wtr = csv::Writer::from_writer(vec![]);

    let _ = wtr.write_record(&[
        "Cell_ID",
        "STAT1_Activity_M1",
        "STAT6_Activity_M2",
        "Velocity_X",
        "Velocity_Y",
        "Predicted_Phenotype",
        "Glycolysis_Flux",
        "OXPHOS_Flux",
        "Metabolic_Ratio",
        "CD80",
        "CD86",
        "iNOS",
        "CD206",
        "Arg1",
        "CD163",
        "VEGF",
        "LPS_Signal",
        "IL4_Signal",
        "Hypoxia_Signal",
    ]);

    for cell in cells {
        let _ = wtr.write_record(&[
            cell.id.to_string(),
            format!("{:.4}", cell.x),
            format!("{:.4}", cell.y),
            format!("{:.4}", cell.vx),
            format!("{:.4}", cell.vy),
            cell.phenotype.short_name().to_string(),
            format!("{:.3}", cell.metabolic.glycolysis_flux),
            format!("{:.3}", cell.metabolic.oxphos_flux),
            format!("{:.3}", cell.metabolic.metabolic_ratio),
            format!("{:.2}", cell.markers.cd80),
            format!("{:.2}", cell.markers.cd86),
            format!("{:.2}", cell.markers.inos),
            format!("{:.2}", cell.markers.cd206),
            format!("{:.2}", cell.markers.arg1),
            format!("{:.2}", cell.markers.cd163),
            format!("{:.2}", cell.markers.vegf),
            format!("{:.3}", params.s_lps),
            format!("{:.3}", params.s_il4),
            format!("{:.3}", params.s_hypoxia),
        ]);
    }

    let bytes = wtr.into_inner().unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

pub fn export_time_series_csv(history: &VecDeque<TimeSeriesPoint>) -> String {
    let mut wtr = csv::Writer::from_writer(vec![]);

    let _ = wtr.write_record(&[
        "Time_Seconds",
        "Mean_STAT1_M1",
        "Mean_STAT6_M2",
        "Shannon_Entropy",
        "Percent_M1",
        "Percent_M2a",
        "Percent_M2d",
        "Mean_Glycolysis",
        "Mean_OXPHOS",
    ]);

    for pt in history {
        let _ = wtr.write_record(&[
            format!("{:.2}", pt.time),
            format!("{:.4}", pt.mean_stat1),
            format!("{:.4}", pt.mean_stat6),
            format!("{:.4}", pt.entropy),
            format!("{:.2}", pt.pct_m1),
            format!("{:.2}", pt.pct_m2a),
            format!("{:.2}", pt.pct_m2d),
            format!("{:.3}", pt.mean_glycolysis),
            format!("{:.3}", pt.mean_oxphos),
        ]);
    }

    let bytes = wtr.into_inner().unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

pub fn export_facs_gating_report(report: &FacsAnalysisReport, gates: &FacsGatingGates) -> String {
    format!(
        "# Oracle Platform - Flow Cytometry (FACS) Gating Report\n\
        ========================================================\n\
        Total Gated Events:    {}\n\
        Gate X Threshold:      {:.3} (STAT1 / CD80)\n\
        Gate Y Threshold:      {:.3} (STAT6 / CD206)\n\n\
        --- Quadrant Statistics ---\n\
        Q1 (M2 Repair):        {} events ({:.2}%) | MFI={:.3}\n\
        Q2 (Double+ / Hybrid): {} events ({:.2}%)\n\
        Q3 (Double- / Naive):  {} events ({:.2}%)\n\
        Q4 (M1 Attack):        {} events ({:.2}%) | MFI={:.3}\n\n\
        --- Overall Mean Fluorescence Intensity (MFI) ---\n\
        Total Mean STAT1:      {:.4}\n\
        Total Mean STAT6:      {:.4}\n",
        report.total_count,
        gates.gate_x_threshold,
        gates.gate_y_threshold,
        report.count_q1,
        report.pct_q1,
        report.mfi_stat6_q1,
        report.count_q2,
        report.pct_q2,
        report.count_q3,
        report.pct_q3,
        report.count_q4,
        report.pct_q4,
        report.mfi_stat1_q4,
        report.mfi_stat1_total,
        report.mfi_stat6_total,
    )
}

pub fn export_population_summary_report(stats: &PopulationStats, params: &BiophysicalParams, sim_time: f32) -> String {
    format!(
        "# Oracle Platform - Simulation Report\n\
        ======================================\n\
        Simulation Time Elapsed: {:.2}s\n\
        Total Simulated Cells:   {}\n\n\
        --- Microenvironmental Inputs ---\n\
        LPS / IFN-γ Signal:      {:.3}\n\
        IL-4 / IL-13 Signal:     {:.3}\n\
        Immune Complexes (M2b):  {:.3}\n\
        IL-10 / TGF-β (M2c):     {:.3}\n\
        Hypoxia / TAM (M2d):     {:.3}\n\
        Hill Cooperativity (n):  {:.1}\n\
        Mutual Cross-Repression: {:.2}\n\
        Gene Expression Noise:   {:.3}\n\n\
        --- Drug Perturbations ---\n\
        JAK1/2 Inhibitor:        {:.2}\n\
        Anti-IL4R mAb:           {:.2}\n\
        TLR4 Antagonist:         {:.2}\n\
        HIF-1α Inhibitor:        {:.2}\n\
        HDAC Inhibitor:          {:.2}\n\n\
        --- Macrophage Subtypes Distribution (Murray et al. 2014) ---\n\
        M0 (Naive / Baseline):   {} ({:.1}%)\n\
        M1 (Pro-inflammatory):   {} ({:.1}%)\n\
        M2a (Wound Healing):     {} ({:.1}%)\n\
        M2b (Regulatory):        {} ({:.1}%)\n\
        M2c (Deactivated):       {} ({:.1}%)\n\
        M2d (Tumor-Associated):  {} ({:.1}%)\n\
        M-Hybrid (Plastic):      {} ({:.1}%)\n\n\
        --- Immunometabolic Flux (O'Neill & Pearce) ---\n\
        Mean Glycolytic Flux:    {:.3} (Warburg effect)\n\
        Mean OXPHOS Flux:        {:.3} (Mitochondrial respiration)\n\
        Metabolic Ratio (G/O):   {:.3}\n\
        Itaconate/Succinate Idx: {:.3} (Krebs cycle break)\n\
        ATP Energy Efficiency:   {:.1}%\n\n\
        --- System Biology & Thermodynamic Metrics ---\n\
        Population Center:       STAT1={:.3}, STAT6={:.3}\n\
        Shannon Diversity Index: {:.4} (Heterogeneity)\n\
        Barrier M1 -> M2 (ΔU):   {:.4}\n\
        Barrier M2 -> M1 (ΔU):   {:.4}\n",
        sim_time,
        stats.total_cells,
        params.s_lps,
        params.s_il4,
        params.s_immune_complexes,
        params.s_il10,
        params.s_hypoxia,
        params.hill_n,
        params.gamma,
        params.noise_sigma,
        params.drugs.jak_inhibitor,
        params.drugs.anti_il4r_mab,
        params.drugs.tlr4_antagonist,
        params.drugs.hif1a_inhibitor,
        params.drugs.hdac_inhibitor,
        stats.count_m0,
        stats.pct_m0,
        stats.count_m1,
        stats.pct_m1,
        stats.count_m2a,
        stats.pct_m2a,
        stats.count_m2b,
        stats.pct_m2b,
        stats.count_m2c,
        stats.pct_m2c,
        stats.count_m2d,
        stats.pct_m2d,
        stats.count_hybrid,
        stats.pct_hybrid,
        stats.mean_glycolysis,
        stats.mean_oxphos,
        stats.mean_metabolic_ratio,
        stats.mean_itaconate_succinate,
        stats.mean_atp_efficiency * 100.0,
        stats.mean_x,
        stats.mean_y,
        stats.shannon_entropy,
        stats.barrier_m1_m2,
        stats.barrier_m2_m1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_dataset_generation() {
        for st in SampleDatasetType::all() {
            let ds = generate_sample_dataset(*st);
            assert!(!ds.cells.is_empty());
            assert!(!ds.name.is_empty());
        }
    }

    #[test]
    fn test_csv_parser_and_export() {
        let csv_content = "Cell_ID,STAT1,STAT6,Phenotype\nC1,2.1,0.2,M1\nC2,0.3,2.2,M2\nC3,0.4,0.4,M0\n";
        let ds = parse_csv_data(csv_content, "Test Dataset").expect("Failed to parse CSV");
        assert_eq!(ds.cells.len(), 3);
        assert_eq!(ds.cells[0].phenotype, Phenotype::M1);
        assert_eq!(ds.cells[1].phenotype, Phenotype::M2a);
        assert_eq!(ds.cells[2].phenotype, Phenotype::M0);
    }
}
