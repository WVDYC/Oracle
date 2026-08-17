# Waddington-X BioTech Platform 🧬
### Epigenetic Landscape & Macrophage Phenotype Dynamics Research Workstation

[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-eframe%20%2F%20egui-blue)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Domain](https://img.shields.io/badge/Domain-Systems_Biology_%26_Immunology-purple)]()

> **Interactive In Silico Modeling Platform for Waddington Epigenetic Landscapes, Nonlinear Gene Regulatory Networks (GRN), Langevin Stochastic Dynamics, Flow Cytometry (FACS) Gating, and Pharmacological Drug Perturbations.**

---

[English Documentation](#-english-documentation) | [Русская документация](#-русская-документация)

---

# 🇬🇧 English Documentation

## 📌 Scientific Overview

Cellular differentiation and phenotypic plasticity (e.g., macrophage polarization across **M0, M1, M2, and M3/Hybrid** states) represent nonlinear dynamical systems governed by mutually repressive gene regulatory networks (GRNs).

**Waddington-X** provides a high-performance, real-time native desktop workstation written in **Rust** to explore:
1. **Waddington Quasi-Potential Landscapes $U(x, y)$:** Multistable energy topography with deep attractor valleys and transition barriers ($\Delta U$).
2. **Nonlinear GRN Toggle Switch with Hill Kinetics:** Mutual cross-inhibition between master transcription factors (STAT1/NF-$\kappa$B vs. STAT6/PPAR-$\gamma$).
3. **Single-Cell Stochastic Dynamics (Langevin Equations):** Real-time Euler-Maruyama simulation of single cells experiencing transcriptional noise ($\sigma$) and jumping across epigenetic barriers.
4. **Flow Cytometry (FACS) 4-Quadrant Gating:** Standard FlowJo-compatible quadrant analysis (Q1–Q4) with dynamic threshold dragging and Mean Fluorescence Intensity (MFI) computation.
5. **Epigenetic Hysteresis & Bifurcations:** S-shaped fold bifurcation curves explaining lineage commitment and epigenetic memory.
6. **Pharmacological Drug Screening:** In silico simulation of FDA-approved & experimental compounds (JAK1/2 inhibitors, anti-IL4R mAbs, TLR4 antagonists, HIF-1$\alpha$ inhibitors, HDAC inhibitors).
7. **Real-World Data Integration:** Ingestion of single-cell RNA-seq (scRNA-seq) and Flow Cytometry CSV datasets with automatic coordinate mapping.

---

## 🔬 Mathematical Formulations

### 1. Nonlinear GRN Differential Equations (Drift Vector $\vec{F}$)
$$\frac{dx}{dt} = \frac{\alpha_1 x^n + S_{\text{LPS}}^{\text{eff}} + 0.10}{1 + x^n + \gamma_1 y^n} + \text{Hypoxia}_x - \delta_1 x$$
$$\frac{dy}{dt} = \frac{\alpha_2 y^n + S_{\text{IL4}}^{\text{eff}} + 0.10}{1 + y^n + \gamma_2 x^n} + \text{Hypoxia}_y - \delta_2 y$$

* $x$: Activity of the Pro-inflammatory Master Axis (STAT1 / NF-$\kappa$B / iNOS $\to$ **M1**).
* $y$: Activity of the Pro-healing / Tissue Repair Axis (STAT6 / PPAR-$\gamma$ / Arg1 $\to$ **M2**).
* $n$: Hill cooperativity coefficient ($n \ge 2$).
* $\gamma$: Cross-inhibition strength.
* $\delta$: Degradation rate.

### 2. Stochastic Langevin Single-Cell Integration
$$d\mathbf{r}_i = \mathbf{F}(\mathbf{r}_i)\,dt + \sigma_{\text{eff}}\,\sqrt{dt}\,\mathbf{\xi}_i, \quad \mathbf{\xi}_i \sim \mathcal{N}(0, \mathbf{I})$$

### 3. Population Shannon Diversity Index (Entropy)
$$H = -\sum_{k \in \{\text{M0, M1, M2, M3}\}} p_k \ln(p_k)$$

---

## 🖥 Workstation Modules & Tabs

| Tab | Feature Description |
|---|---|
| **🌐 2D Waddington Landscape** | Full-canvas topographic heatmap, equipotential contour rings, drift vector field, phase nullclines, interactive mouse cell-injection, and hover probe tooltip. |
| **🔬 FACS Gating & Cytometry** | 4-quadrant flow cytometry scatter (Q1: M2, Q2: Double⁺, Q3: Double⁻, Q4: M1), draggable gate thresholds, cell counts, percentages, and MFI. |
| **📈 Bifurcations & Hysteresis** | S-shaped fold bifurcation diagram depicting stable/unstable fixed points across cytokine titration and the bistable memory window. |
| **⏱ Time-Series Kinetics** | Multi-channel real-time strip chart for STAT1, STAT6, and Shannon Entropy dynamics. |
| **💊 Drug Screening Assay** | Pharmacological titration of JAK inhibitors (Tofacitinib), anti-IL-4R antibodies (Dupilumab), TLR4 blockers (TAK-242), HIF-1α inhibitors, and HDAC epigenetic modulators. |
| **📚 Theory & Methodology** | Mathematical derivations, biophysical parameter tables, and seminal literature citations. |

---

## 🚀 Quickstart & Installation

### Prerequisites
* [Rust & Cargo](https://rustup.rs/) (version 1.70+ recommended).

### Build & Run
```bash
# Clone the repository
git clone https://github.com/altemirzhilkibayev/Oracle.git
cd Oracle

# Run automated test suite
cargo test

# Launch the native workstation
cargo run --release
```

---

## 📊 Scientific Data Import & Export

* **Custom CSV Loading:** Click `Load CSV` in the left panel to ingest any FlowJo or scRNA-seq expression matrix containing columns such as `STAT1`, `STAT6`, `CD80`, `CD206`, `iNOS`, `Arg1`, or `x`/`y`.
* **Export Utilities:**
  * `Export CSV` $\to$ Single-cell simulated population state matrix.
  * `Save PNG Figure` $\to$ Publication-ready $1000 \times 800$ figures.
  * `Save FACS Gating Report` $\to$ Quadrant statistics and MFI breakdown.
  * `Save Time-Series Kinetics CSV` $\to$ Kinetic trajectory history.
  * `Save Full Scientific Summary Report` $\to$ Comprehensive biophysical summary.

---

# 🇷🇺 Русская документация

## 📌 Научный обзор платформы

Дифференцировка и фенотипическая пластичность клеток (например, поляризация макрофагов по состояниям **M0, M1, M2, M3/Hybrid**) представляют собой нелинейные динамические системы, управляемые взаимно ингибирующими генно-регуляторными сетями (GRN).

**Waddington-X** — это высокопроизводительная исследовательская рабочая станция на **Rust**, созданная для:
1. **Моделирования эпигенетического ландшафта Уоддингтона $U(x, y)$:** Расчет топографии потенциальной энергии с долинами аттракторов и барьерами переходов ($\Delta U$).
2. **Нелинейных дифференциальных уравнений с кинетикой Хилла:** Взаимное подавление транскрипционных факторов STAT1/NF-$\kappa$B vs STAT6/PPAR-$\gamma$.
3. **Стохастической динамики единичных клеток (уравнения Ланжевена):** Симуляция в реальном времени сотен клеток со стохастическим шумом транскрипции ($\sigma$).
4. **Проточной цитометрии (FACS 4-Quadrant Gating):** Классический 4-квадрантный анализ проточной цитометрии (Q1–Q4) с интерактивным перетаскиванием порогов гейтирования и расчетом средней интенсивности флуоресценции (MFI).
5. **Бифуркаций и эпигенетического гистерезиса:** S-образные кривые, объясняющие феномен клеточной памяти и устойчивости фенотипа.
6. **Фармакологического тестирования (Drug Screening):** Тестирование ингибиторов JAK1/2, антител к IL-4R, антагонистов TLR4, блокаторов HIF-1$\alpha$ и HDAC-ингибиторов.
7. **Интеграции с лабораторными данными:** Загрузка таблиц scRNA-seq и проточной цитометрии из CSV с авто-масштабированием.

---

## 🖥 Вкладки и функционал рабочей станции

| Вкладка | Описание |
|---|---|
| **🌐 2D Waddington Landscape** | Топографическая тепловая карта, изолинии, векторное поле дрейфа, нульклины, сброс клеток кликом мыши и зонд с подсказками. |
| **🔬 FACS Gating & Cytometry** | Скаттерплот проточной цитометрии с 4 квадрантами (Q1: M2, Q2: Double⁺, Q3: Double⁻, Q4: M1), расчет MFI и процентов. |
| **📈 Bifurcations & Hysteresis** | Бифуркационная диаграмма состояний при титровании LPS с выделением окна эпигенетической памяти. |
| **⏱ Time-Series Kinetics** | Многоканальный самописец динамики экспрессии STAT1, STAT6 и энтропии во времени. |
| **💊 Drug Screening Assay** | Фармакологический скрининг таргетных ингибиторов и моноклональных антител. |
| **📚 Theory & Methodology** | Математические уравнения, значения параметров и список научной литературы. |

---

## 🚀 Установка и запуск

```bash
# Клонирование репозитория
git clone https://github.com/altemirzhilkibayev/Oracle.git
cd Oracle

# Запуск тестов
cargo test

# Запуск программы
cargo run --release
```

---

## 📚 Academic References / Научная литература

1. **Waddington, C. H.** (1957). *The Strategy of the Genes*. Allen & Unwin, London.
2. **Huang, S., et al.** (2005). Bifurcation dynamics in lineage-commitment in bipotent progenitor cells. *Developmental Biology*, 280(1), 40-58.
3. **Sica, A., & Mantovani, A.** (2012). Macrophage plasticity and polarization: in vivo veritas. *Journal of Clinical Investigation*, 122(3), 787-795.
4. **Murray, P. J., et al.** (2014). Macrophage activation and polarization: nomenclature and experimental guidelines. *Immunity*, 41(1), 14-20.
5. **Zhou, J. X., et al.** (2012). Quasi-potential landscape in complex dynamical systems. *Physical Review E*, 85(6), 061918.

---

## 📄 License
MIT License. Developed by **Altemir Zhilkibaev** (altemirzhilkibaev@gmail.com).
