// Flow Cytometry (FACS) 4-Quadrant Gating Analysis Module
// Implements Q1, Q2, Q3, Q4 gating, Mean Fluorescence Intensity (MFI), and single-cell population statistics.

use crate::model::SingleCell;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FacsGatingGates {
    pub gate_x_threshold: f32, // STAT1 / CD80 threshold (default ~1.0)
    pub gate_y_threshold: f32, // STAT6 / CD206 threshold (default ~1.0)
}

impl Default for FacsGatingGates {
    fn default() -> Self {
        Self {
            gate_x_threshold: 1.0,
            gate_y_threshold: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacsQuadrant {
    Q1, // Upper-Left: STAT1- / STAT6+ (M2 Pro-healing)
    Q2, // Upper-Right: STAT1+ / STAT6+ (Double Positive / M3 Hybrid Plastic)
    Q3, // Lower-Left: STAT1- / STAT6- (Double Negative / M0 Naive Baseline)
    Q4, // Lower-Right: STAT1+ / STAT6- (M1 Pro-inflammatory)
}

#[allow(dead_code)]
impl FacsQuadrant {
    pub fn label(&self) -> &'static str {
        match self {
            FacsQuadrant::Q1 => "Q1: STAT1⁻ / STAT6⁺ (M2 Repair)",
            FacsQuadrant::Q2 => "Q2: STAT1⁺ / STAT6⁺ (Double⁺ / M3 Hybrid)",
            FacsQuadrant::Q3 => "Q3: STAT1⁻ / STAT6⁻ (Double⁻ / M0 Naive)",
            FacsQuadrant::Q4 => "Q4: STAT1⁺ / STAT6⁻ (M1 Attack)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            FacsQuadrant::Q1 => "Q1 (M2)",
            FacsQuadrant::Q2 => "Q2 (Double⁺)",
            FacsQuadrant::Q3 => "Q3 (Double⁻)",
            FacsQuadrant::Q4 => "Q4 (M1)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FacsAnalysisReport {
    pub total_count: usize,
    pub count_q1: usize,
    pub count_q2: usize,
    pub count_q3: usize,
    pub count_q4: usize,
    pub pct_q1: f32,
    pub pct_q2: f32,
    pub pct_q3: f32,
    pub pct_q4: f32,
    pub mfi_stat1_total: f32,
    pub mfi_stat6_total: f32,
    pub mfi_stat1_q4: f32,
    pub mfi_stat6_q1: f32,
}

pub fn classify_quadrant(x: f32, y: f32, gates: &FacsGatingGates) -> FacsQuadrant {
    if x >= gates.gate_x_threshold && y >= gates.gate_y_threshold {
        FacsQuadrant::Q2
    } else if x < gates.gate_x_threshold && y >= gates.gate_y_threshold {
        FacsQuadrant::Q1
    } else if x < gates.gate_x_threshold && y < gates.gate_y_threshold {
        FacsQuadrant::Q3
    } else {
        FacsQuadrant::Q4
    }
}

pub fn analyze_facs_population(cells: &[SingleCell], gates: &FacsGatingGates) -> FacsAnalysisReport {
    let total = cells.len();
    if total == 0 {
        return FacsAnalysisReport {
            total_count: 0,
            count_q1: 0,
            count_q2: 0,
            count_q3: 0,
            count_q4: 0,
            pct_q1: 0.0,
            pct_q2: 0.0,
            pct_q3: 0.0,
            pct_q4: 0.0,
            mfi_stat1_total: 0.0,
            mfi_stat6_total: 0.0,
            mfi_stat1_q4: 0.0,
            mfi_stat6_q1: 0.0,
        };
    }

    let mut q1 = 0;
    let mut q2 = 0;
    let mut q3 = 0;
    let mut q4 = 0;

    let mut sum_x = 0.0f32;
    let mut sum_y = 0.0f32;
    let mut sum_x_q4 = 0.0f32;
    let mut sum_y_q1 = 0.0f32;

    for cell in cells {
        sum_x += cell.x;
        sum_y += cell.y;

        match classify_quadrant(cell.x, cell.y, gates) {
            FacsQuadrant::Q1 => {
                q1 += 1;
                sum_y_q1 += cell.y;
            }
            FacsQuadrant::Q2 => q2 += 1,
            FacsQuadrant::Q3 => q3 += 1,
            FacsQuadrant::Q4 => {
                q4 += 1;
                sum_x_q4 += cell.x;
            }
        }
    }

    let n = total as f32;
    FacsAnalysisReport {
        total_count: total,
        count_q1: q1,
        count_q2: q2,
        count_q3: q3,
        count_q4: q4,
        pct_q1: (q1 as f32 / n) * 100.0,
        pct_q2: (q2 as f32 / n) * 100.0,
        pct_q3: (q3 as f32 / n) * 100.0,
        pct_q4: (q4 as f32 / n) * 100.0,
        mfi_stat1_total: sum_x / n,
        mfi_stat6_total: sum_y / n,
        mfi_stat1_q4: if q4 > 0 { sum_x_q4 / q4 as f32 } else { 0.0 },
        mfi_stat6_q1: if q1 > 0 { sum_y_q1 / q1 as f32 } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facs_quadrant_classification() {
        let gates = FacsGatingGates {
            gate_x_threshold: 1.0,
            gate_y_threshold: 1.0,
        };
        assert_eq!(classify_quadrant(0.5, 1.5, &gates), FacsQuadrant::Q1);
        assert_eq!(classify_quadrant(1.8, 1.8, &gates), FacsQuadrant::Q2);
        assert_eq!(classify_quadrant(0.3, 0.4, &gates), FacsQuadrant::Q3);
        assert_eq!(classify_quadrant(2.0, 0.3, &gates), FacsQuadrant::Q4);
    }
}
