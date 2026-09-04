<p align="center">
  <img src="assets/app_icon.png" width="130" height="130" alt="Oracle Logo" />
</p>

<h1 align="center">🔮 Oracle</h1>
<h3 align="center">Epigenetic Landscape & Macrophage Phenotype Dynamics Research Workstation</h3>

<p align="center">
  <a href="https://github.com/WVDYC/Oracle/releases/latest">
    <img src="https://img.shields.io/badge/Download_for_Windows-Setup.exe-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Download Windows" />
  </a>
  <a href="https://github.com/WVDYC/Oracle/releases/latest">
    <img src="https://img.shields.io/badge/Download_for_macOS-Apple_Silicon-000000?style=for-the-badge&logo=apple&logoColor=white" alt="Download macOS" />
  </a>
  <a href="https://github.com/WVDYC/Oracle/releases/latest">
    <img src="https://img.shields.io/badge/Download_for_Linux-x86__64-FCC624?style=for-the-badge&logo=linux&logoColor=black" alt="Download Linux" />
  </a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/GUI-eframe%20%2F%20egui-blue" alt="GUI" />
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License" />
  <img src="https://img.shields.io/badge/Domain-Systems_Biology_%26_Immunology-purple" alt="Domain" />
</p>

> **Interactive In Silico Modeling Platform for Waddington Epigenetic Landscapes, Nonlinear Gene Regulatory Networks (GRN), Langevin Stochastic Dynamics, Flow Cytometry (FACS) Gating, and Pharmacological Drug Perturbations.**

---

[English Documentation](#-english-documentation) | [Русская документация](#-русская-документация)

---

# 🇬🇧 English Documentation

## 📌 Scientific Overview

Cellular differentiation, phenotypic plasticity, and metabolic reprogramming (e.g., macrophage polarization across **M0, M1, M2a, M2b, M2c, M2d/TAM, and M-Hybrid** states per Murray et al., *Immunity 2014*) represent nonlinear dynamical systems governed by mutually repressive gene regulatory networks (GRNs) and metabolic rewiring (*O'Neill & Pearce, Nature Reviews Immunology 2016*).

**Oracle** provides a high-performance, real-time native desktop workstation written in **Rust** to explore:
1. **Waddington Quasi-Potential Landscapes U(x, y):** Multistable energy topography with deep attractor valleys and transition barriers (ΔU).
2. **Macrophage Polarization Spectrum (Murray et al. 2014):** Full 7-state resolution (M0 resting, M1 inflammatory, M2a wound healing, M2b immune-complex regulatory, M2c efferocytosis/deactivated, M2d tumor-associated, M-Hybrid plastic).
3. **Immunometabolic Reprogramming:** Warburg aerobic glycolysis flux ($J_{\text{glyc}}$) vs. mitochondrial OXPHOS & fatty acid oxidation ($J_{\text{oxphos}}$), itaconate/succinate TCA cycle break index, and ATP generation efficiency.
4. **Diagnostic Biomarker Matrix:** Continuous simulation of surface & intracellular markers (**CD80, CD86, iNOS, TNF-α, CD206/MRC1, Arg1, CD163, MerTK, VEGF, HIF-1α**).
5. **Nonlinear GRN Toggle Switch with Hill Kinetics:** Mutual cross-inhibition between master transcription factors (STAT1/NF-κB vs. STAT6/PPAR-γ).
6. **Single-Cell Stochastic Dynamics (Langevin Equations):** Real-time Euler-Maruyama simulation of single cells experiencing transcriptional noise (σ) and jumping across epigenetic barriers.
7. **Flow Cytometry (FACS) 4-Quadrant Gating:** Standard FlowJo-compatible quadrant analysis (Q1–Q4) with dynamic threshold dragging and Mean Fluorescence Intensity (MFI) computation.
8. **Epigenetic Hysteresis & Bifurcations:** S-shaped fold bifurcation curves explaining lineage commitment and epigenetic memory.
9. **Pharmacological Drug Screening:** In silico simulation of FDA-approved & experimental compounds (JAK1/2 inhibitors, anti-IL4R mAbs, TLR4 antagonists, HIF-1α inhibitors, HDAC inhibitors).
10. **Real-World Data Integration:** Ingestion of single-cell RNA-seq (scRNA-seq) and Flow Cytometry CSV datasets with automatic coordinate mapping.

---

## 🔬 Mathematical Formulations

### 1. Nonlinear GRN Differential Equations (Drift Vector F)
$$\frac{dx}{dt} = \frac{\alpha_1 x^n + S_{\text{LPS}}^{\text{eff}} + 0.3 S_{\text{IC}} + 0.10}{1 + x^n + \gamma_1 y^n} + \text{Hypoxia}_x - \delta_1 x$$
$$\frac{dy}{dt} = \frac{\alpha_2 y^n + S_{\text{IL4}}^{\text{eff}} + 0.4 S_{\text{IL10}} + 0.10}{1 + y^n + \gamma_2 x^n} + \text{Hypoxia}_y - \delta_2 y$$

* **x**: Activity of the Pro-inflammatory Master Axis (STAT1 / NF-κB / iNOS / CD80 → **M1**).
* **y**: Activity of the Pro-healing / Tissue Repair Axis (STAT6 / PPAR-γ / Arg1 / CD206 → **M2a**).
* **n**: Hill cooperativity coefficient ($n \ge 2$).
* **γ**: Cross-inhibition strength.
* **δ**: Degradation rate.

### 2. Immunometabolic Flux Coupling (O'Neill & Pearce)
$$J_{\text{glyc}} = \frac{0.30 + 0.80 x^{1.4} + 0.65 S_{\text{hypoxia}} + 0.30 S_{\text{LPS}}}{1 + 0.35 y}$$
$$J_{\text{oxphos}} = \frac{0.35 + 0.90 y^{1.4} + 0.40 S_{\text{IL4}}}{1 + 0.60 x}$$

### 3. Stochastic Langevin Single-Cell Integration
$$d\mathbf{r}_i = \mathbf{F}(\mathbf{r}_i)\,dt + \sigma_{\text{eff}}\,\sqrt{dt}\,\mathbf{\xi}_i, \quad \mathbf{\xi}_i \sim \mathcal{N}(0, \mathbf{I})$$

### 4. Population Shannon Diversity Index (Entropy)
$$H = -\sum_{k} p_k \ln(p_k)$$

---

## 🖥 Workstation Modules & Tabs

| Tab | Feature Description |
|---|---|
| **🌐 2D Waddington Landscape** | Topographic heatmap, equipotential contour rings, drift vector field, phase nullclines, mouse cell injection, and hover probe displaying metabolism & markers. |
| **🔬 FACS Gating & Cytometry** | 4-quadrant flow cytometry scatter (Q1: M2, Q2: Double⁺, Q3: Double⁻, Q4: M1), draggable gate thresholds, cell counts, percentages, and MFI. |
| **📈 Bifurcations & Hysteresis** | S-shaped fold bifurcation diagram depicting stable/unstable fixed points across cytokine titration and the bistable memory window. |
| **⏱ Time-Series Kinetics** | Multi-channel real-time strip chart for STAT1, STAT6, Shannon Entropy, and metabolic fluxes. |
| **💊 Drug Screening Assay** | Pharmacological titration of JAK inhibitors (Tofacitinib), anti-IL-4R antibodies (Dupilumab), TLR4 blockers (TAK-242), HIF-1α inhibitors, and HDAC epigenetic modulators. |
| **📚 Theory & Methodology** | Mathematical derivations, biophysical parameter tables, and seminal literature citations (Murray 2014, O'Neill & Pearce 2016, Mills 2016). |

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

Дифференцировка, фенотипическая пластичность и метаболическое перепрограммирование клеток (например, поляризация макрофагов по 7 состояниям: **M0, M1, M2a, M2b, M2c, M2d/TAM, M-Hybrid** по стандартам Murray et al., *Immunity 2014*) представляют собой нелинейные динамические системы, управляемые взаимно ингибирующими генно-регуляторными сетями (GRN) и перестройкой метаболизма (*O'Neill & Pearce, Nature Reviews Immunology 2016*).

**Oracle** — это высокопроизводительная исследовательская рабочая станция на **Rust**, созданная для:
1. **Моделирования эпигенетического ландшафта Уоддингтона U(x, y):** Расчет топографии потенциальной энергии с долинами аттракторов и барьерами переходов (ΔU).
2. **Спектра поляризации макрофагов (Murray et al. 2014):** Моделирование 7 состояний (M0 покоящиеся, M1 воспалительные, M2a заживление ран, M2b регуляторные, M2c фагоцитоз/деактивация, M2d ассоциированные с опухолью, M-Hybrid пластичные).
3. **Иммунометаболического перепрограммирования:** Расчет аэробного гликолиза Варбурга ($J_{\text{glyc}}$) vs митохондриального дыхания OXPHOS & окисления жирных кислот ($J_{\text{oxphos}}$), индекса разрыва цикла Кребса (накопление итаконата/сукцината) и эффективности генерации АТФ.
4. **Матрицы диагностических биомаркеров:** Моделирование уровней CD80, CD86, iNOS, TNF-α, CD206 (MRC1), Arg1, CD163, MerTK, VEGF, HIF-1α в реальном времени.
5. **Нелинейных дифференциальных уравнений с кинетикой Хилла:** Взаимное подавление транскрипционных факторов STAT1/NF-κB vs STAT6/PPAR-γ.
6. **Стохастической динамики единичных клеток (уравнения Ланжевена):** Симуляция сотен клеток со стохастическим шумом транскрипции (σ) по схеме Эйлера-Маруямы.
7. **Проточной цитометрии (FACS 4-Quadrant Gating):** Классический 4-квадрантный анализ проточной цитометрии (Q1–Q4) с интерактивным перетаскиванием порогов гейтирования и расчетом средней интенсивности флуоресценции (MFI).
8. **Бифуркаций и эпигенетического гистерезиса:** S-образные кривые, объясняющие феномен клеточной памяти и устойчивости фенотипа.
9. **Фармакологического тестирования (Drug Screening):** Тестирование ингибиторов JAK1/2, антител к IL-4R, антагонистов TLR4, блокаторов HIF-1α и HDAC-ингибиторов.
10. **Интеграции с лабораторными данными:** Загрузка таблиц scRNA-seq и проточной цитометрии из CSV с авто-масштабированием.

---

## 🖥 Вкладки и функционал рабочей станции

| Вкладка | Описание |
|---|---|
| **🌐 2D Waddington Landscape** | Топографическая тепловая карта, изолинии, векторное поле дрейфа, нульклины, сброс клеток кликом мыши и зонд с отображением метаболизма и маркеров. |
| **🔬 FACS Gating & Cytometry** | Скаттерплот проточной цитометрии с 4 квадрантами (Q1: M2, Q2: Double⁺, Q3: Double⁻, Q4: M1), расчет MFI и процентов. |
| **📈 Bifurcations & Hysteresis** | Бифуркационная диаграмма состояний при титровании LPS с выделением окна эпигенетической памяти. |
| **⏱ Time-Series Kinetics** | Многоканальный самописец динамики экспрессии STAT1, STAT6, энтропии и метаболических потоков во времени. |
| **💊 Drug Screening Assay** | Фармакологический скрининг таргетных ингибиторов и моноклональных антител. |
| **📚 Theory & Methodology** | Математические уравнения, значения параметров и список научной литературы (Murray 2014, O'Neill 2016, Mills 2016). |

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

1. **Murray, P. J., et al.** (2014). Macrophage activation and polarization: nomenclature and experimental guidelines. *Immunity*, 41(1), 14-20.
2. **O'Neill, L. A. J., & Pearce, E. J.** (2016). Immunometabolism governs dendritic cell and macrophage function. *Nature Reviews Immunology*, 16(9), 553-565.
3. **Mills, E. L., et al.** (2016). Succinate dehydrogenase and itaconate metabolic remodeling in M1 macrophages. *Cell*, 167(2), 457-470.
4. **Waddington, C. H.** (1957). *The Strategy of the Genes*. Allen & Unwin, London.
5. **Huang, S., et al.** (2005). Bifurcation dynamics in lineage-commitment in bipotent progenitor cells. *Developmental Biology*, 280(1), 40-58.
6. **Sica, A., & Mantovani, A.** (2012). Macrophage plasticity and polarization: in vivo veritas. *Journal of Clinical Investigation*, 122(3), 787-795.
7. **Zhou, J. X., et al.** (2012). Quasi-potential landscape in complex dynamical systems. *Physical Review E*, 85(6), 061918.

---

## 📄 License
MIT License. Developed by **Altemir Zhilkibaev** (altemirzhilkibaev@gmail.com).
