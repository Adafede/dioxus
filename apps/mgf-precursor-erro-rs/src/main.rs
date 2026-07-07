use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;
use molecular_formulas::{MolecularFormula, prelude::ChemicalFormula};
use smiles_parser::chain;
use smiles_parser::graph::{Atom, MoleculeGraph};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::Blob;

#[cfg(target_arch = "wasm32")]
const CHUNK_SIZE: usize = 1 << 20;
#[cfg(target_arch = "wasm32")]
const PROGRESS_INTERVAL: usize = 1 << 20;
#[cfg(target_arch = "wasm32")]
const PROTON_MASS: f64 = 1.007_276_466_621;
#[cfg(target_arch = "wasm32")]
const SODIUM_MASS: f64 = 22.989_769_67;
#[cfg(target_arch = "wasm32")]
const POTASSIUM_MASS: f64 = 38.963_707;
#[cfg(target_arch = "wasm32")]
const AMMONIUM_MASS: f64 = 18.033_823;

#[cfg(target_arch = "wasm32")]
type ScanError = JsValue;

fn main() {
    dioxus::launch(app);
}

#[derive(Clone, Debug, PartialEq)]
struct HistogramData {
    bins: Vec<usize>,
    min: f64,
    max: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct PlotPoint {
    precursor_mz: f64,
    ppm_error: f64,
    da_error: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct PrecursorMetrics {
    spectra: usize,
    spectra_with_reference_mass: usize,
    reference_mass_source: String,
    observed_precursor_min: f64,
    observed_precursor_max: f64,
    observed_precursor_mean: f64,
    abs_error_da_min: f64,
    abs_error_da_max: f64,
    abs_error_da_mean: f64,
    abs_error_da_rms: f64,
    abs_error_ppm_min: f64,
    abs_error_ppm_max: f64,
    abs_error_ppm_mean: f64,
    abs_error_ppm_rms: f64,
    signed_error_da_mean: f64,
    signed_error_ppm_mean: f64,
    within_0_01_da: usize,
    within_0_005_da: usize,
    within_5_ppm: usize,
    within_10_ppm: usize,
    da_error_histogram: HistogramData,
    ppm_error_histogram: HistogramData,
    plot_points: Vec<PlotPoint>,
}

impl Default for PrecursorMetrics {
    fn default() -> Self {
        Self {
            spectra: 0,
            spectra_with_reference_mass: 0,
            reference_mass_source: "none".to_string(),
            observed_precursor_min: 0.0,
            observed_precursor_max: 0.0,
            observed_precursor_mean: 0.0,
            abs_error_da_min: 0.0,
            abs_error_da_max: 0.0,
            abs_error_da_mean: 0.0,
            abs_error_da_rms: 0.0,
            abs_error_ppm_min: 0.0,
            abs_error_ppm_max: 0.0,
            abs_error_ppm_mean: 0.0,
            abs_error_ppm_rms: 0.0,
            signed_error_da_mean: 0.0,
            signed_error_ppm_mean: 0.0,
            within_0_01_da: 0,
            within_0_005_da: 0,
            within_5_ppm: 0,
            within_10_ppm: 0,
            da_error_histogram: HistogramData::new(48, 0.0, 0.5),
            ppm_error_histogram: HistogramData::new(48, 0.0, 50.0),
            plot_points: Vec::new(),
        }
    }
}

impl HistogramData {
    fn new(bin_count: usize, min: f64, max: f64) -> Self {
        Self {
            bins: vec![0; bin_count],
            min,
            max,
        }
    }

    fn add_value(&mut self, value: f64) {
        if self.bins.is_empty() || !value.is_finite() {
            return;
        }
        let clamped = value.clamp(self.min, self.max);
        let idx = if (self.max - self.min).abs() < f64::EPSILON {
            0
        } else {
            ((clamped - self.min) / (self.max - self.min) * (self.bins.len() as f64 - 1.0)).floor()
                as usize
        };
        let idx = idx.min(self.bins.len().saturating_sub(1));
        self.bins[idx] = self.bins[idx].saturating_add(1);
    }
}

impl PrecursorMetrics {
    fn record_error(&mut self, abs_error_da: f64, abs_ppm: f64, precursor_mz: f64, ppm_error: f64) {
        self.da_error_histogram.add_value(abs_error_da);
        self.ppm_error_histogram.add_value(abs_ppm);

        if self.plot_points.len() < 260 {
            self.plot_points.push(PlotPoint {
                precursor_mz,
                ppm_error,
                da_error: abs_error_da,
            });
        } else {
            let len = self.plot_points.len();
            let stride = (self.spectra / 260).max(1);
            if self.spectra % stride == 0 {
                if let Some(point) = self.plot_points.get_mut(len / 2) {
                    *point = PlotPoint {
                        precursor_mz,
                        ppm_error,
                        da_error: abs_error_da,
                    };
                }
            }
        }
    }
}

fn exact_mass_from_smiles(smiles: &str) -> Option<f64> {
    let parsed = catch_unwind(AssertUnwindSafe(|| {
        let (rest, parsed_chain) = chain(smiles.as_bytes()).ok()?;
        if !rest.is_empty() {
            return None;
        }

        let graph = MoleculeGraph::from_chain(parsed_chain);
        let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
        for node_index in graph.node_indices() {
            let atom = graph.node_weight(node_index)?;
            let symbol = match atom {
                Atom::AliphaticOrganic(atom) => atom.element.get_symbol(),
                Atom::Element(element) => element.get_symbol(),
            };
            *counts.entry(symbol).or_default() += 1;
        }

        let mut formula_parts = counts.into_iter().collect::<Vec<_>>();
        formula_parts.sort_by(|(left, _), (right, _)| hill_order(left, right));

        let mut formula = String::new();
        for (symbol, count) in formula_parts {
            formula.push_str(symbol);
            if count > 1 {
                formula.push_str(&count.to_string());
            }
        }

        let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from_str(&formula).ok()?;
        Some(formula.isotopologue_mass())
    }));

    match parsed {
        Ok(mass) => mass,
        Err(_) => None,
    }
}

fn exact_mass_from_formula(formula: &str) -> Option<f64> {
    let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from_str(formula).ok()?;
    Some(formula.isotopologue_mass())
}

fn hill_order(left: &str, right: &str) -> std::cmp::Ordering {
    match (left, right) {
        ("C", "C") => std::cmp::Ordering::Equal,
        ("C", _) => std::cmp::Ordering::Less,
        (_, "C") => std::cmp::Ordering::Greater,
        ("H", "H") => std::cmp::Ordering::Equal,
        ("H", _) => std::cmp::Ordering::Less,
        (_, "H") => std::cmp::Ordering::Greater,
        (left, right) => left.cmp(right),
    }
}

#[cfg(target_arch = "wasm32")]
fn expected_precursor_mz(
    neutral_mass: f64,
    adduct: Option<&str>,
    charge: Option<&str>,
    ion_mode: Option<&str>,
) -> Option<f64> {
    let normalized_adduct = adduct.unwrap_or("").trim();
    let normalized_ion_mode = ion_mode.unwrap_or("").trim().to_ascii_lowercase();
    let charge_is_negative = charge.unwrap_or("").trim().ends_with('-');
    let charge_is_positive = charge.unwrap_or("").trim().ends_with('+');

    let shift = if normalized_adduct.is_empty() {
        if normalized_ion_mode == "negative" || charge_is_negative {
            -PROTON_MASS
        } else if normalized_ion_mode == "positive" || charge_is_positive {
            PROTON_MASS
        } else {
            0.0
        }
    } else {
        match normalized_adduct {
            "[M+H]+" => PROTON_MASS,
            "[M-H]-" => -PROTON_MASS,
            "[M+Na]+" => SODIUM_MASS,
            "[M+K]+" => POTASSIUM_MASS,
            "[M+NH4]+" => AMMONIUM_MASS,
            _ => 0.0,
        }
    };

    Some(neutral_mass + shift)
}

#[component]
fn app() -> Element {
    let mut file_name = use_signal(String::new);
    let mut metrics = use_signal(|| None::<PrecursorMetrics>);
    let mut status = use_signal(|| "Drop an MGF file to begin.".to_string());
    let mut busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);

    let on_file_change = move |evt: Event<FormData>| {
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        file_name.set(file.name());
        busy.set(true);
        drag_active.set(false);
        status.set("Reading MGF...".to_string());
        metrics.set(None);

        let mut status_for_progress = status;
        let mut metrics_for_results = metrics;

        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let total_bytes = blob.size() as u64;
                status_for_progress.set(format!("Scanning {total_bytes} bytes..."));
                let result = match scan_blob_with_progress(&blob, move |processed, total| {
                    let safe_total = total.max(1);
                    let displayed_processed = processed.min(safe_total);
                    let percent = (displayed_processed * 100 / safe_total).min(100);
                    status_for_progress.set(format!(
                        "Scanning {displayed_processed}/{safe_total} bytes ({percent}%)..."
                    ));
                })
                .await
                {
                    Ok(metrics) => metrics,
                    Err(error) => {
                        status_for_progress.set(format!("Error reading file: {error:?}"));
                        PrecursorMetrics::default()
                    }
                };
                metrics_for_results.set(Some(result));
                busy.set(false);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                status_for_progress.set("This app needs to run in a browser.".to_string());
                busy.set(false);
            }
        });
    };

    let on_drag_enter = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    let on_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    let on_drag_leave = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
    };

    let on_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
        let Some(file) = evt.data().files().into_iter().next() else {
            status.set("No file selected.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            status.set("This file type is not supported in the browser.".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        let Ok(blob) = web_file.clone().dyn_into::<Blob>() else {
            status.set("Unable to read the selected file as a blob.".to_string());
            return;
        };

        file_name.set(file.name());
        busy.set(true);
        status.set("Reading MGF...".to_string());
        metrics.set(None);

        let mut status_for_progress = status;
        let mut metrics_for_results = metrics;

        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let total_bytes = blob.size() as u64;
                status_for_progress.set(format!("Scanning {total_bytes} bytes..."));
                let result = match scan_blob_with_progress(&blob, move |processed, total| {
                    let safe_total = total.max(1);
                    let displayed_processed = processed.min(safe_total);
                    let percent = (displayed_processed * 100 / safe_total).min(100);
                    status_for_progress.set(format!(
                        "Scanning {displayed_processed}/{safe_total} bytes ({percent}%)..."
                    ));
                })
                .await
                {
                    Ok(metrics) => metrics,
                    Err(error) => {
                        status_for_progress.set(format!("Error reading file: {error:?}"));
                        PrecursorMetrics::default()
                    }
                };
                metrics_for_results.set(Some(result));
                busy.set(false);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                status_for_progress.set("This app needs to run in a browser.".to_string());
                busy.set(false);
            }
        });
    };

    rsx! {
        div {
            style: "min-height: 100vh; padding: 2rem 1rem 3rem; background: linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%); color: #0f172a;",
            div {
                style: "max-width: 960px; margin: 0 auto;",
                div {
                    style: "display: flex; align-items: center; gap: 1rem; margin-bottom: 1.25rem;",
                    img {
                        src: "assets/favicon.svg",
                        alt: "MGF precursor error icon",
                        style: "width: 56px; height: 56px; border-radius: 16px; box-shadow: 0 8px 24px rgba(15, 23, 42, 0.12);",
                    }
                    div {
                        h2 { style: "margin: 0; font-size: 1.7rem; letter-spacing: -0.02em;", "MGF Precursor Error Metrics" }
                        p {
                            style: "margin: 0.2rem 0 0; color: #475569; font-size: 0.95rem;",
                            "Upload an MGF file and summarize precursor mass errors in Da and ppm."
                        }
                    }
                }

                div {
                    style: "background: rgba(255,255,255,0.9); border: 1px solid rgba(148,163,184,0.22); border-radius: 20px; box-shadow: 0 12px 40px rgba(15, 23, 42, 0.08); padding: 1.25rem; backdrop-filter: blur(12px);",
                    label {
                        r#for: "mgf-upload",
                        style: format!(
                            "display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.6rem; min-height: 140px; width: 100%; box-sizing: border-box; position: relative; isolation: isolate; border: 2px dashed {}; border-radius: 18px; padding: 1.1rem; cursor: pointer; background: {}; color: #334155; font-weight: 600; text-align: center; transition: border-color 160ms ease, background 160ms ease, transform 160ms ease;",
                            if *drag_active.read() { "#2563eb" } else { "#94a3b8" },
                            if *drag_active.read() { "linear-gradient(135deg, rgba(219,234,254,0.96), rgba(239,246,255,0.94))" } else { "linear-gradient(135deg, rgba(248,250,252,0.95), rgba(239,246,255,0.95))" }
                        ),
                        ondragenter: on_drag_enter,
                        ondragover: on_drag_over,
                        ondragleave: on_drag_leave,
                        ondrop: on_drop,
                        span { style: "font-size: 1rem;", "Drop an MGF file here or click to browse" }
                        span { style: "font-size: 0.85rem; font-weight: 500; color: #64748b;", ".mgf files only" }
                        input {
                            id: "mgf-upload",
                            r#type: "file",
                            accept: ".mgf",
                            disabled: *busy.read(),
                            onchange: on_file_change,
                            style: "position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer;",
                        }
                    }

                    p {
                        style: "margin: 0.7rem 0 0; color: #475569; font-size: 0.9rem;",
                        if !file_name.read().is_empty() {
                            "Selected file: {file_name}"
                        }
                    }

                    p { style: "margin: 0.7rem 0 0; font-weight: 600; color: #334155;", "{status}" }

                    if let Some(metrics) = metrics.read().as_ref() {
                        div {
                            style: "margin-top: 1rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
                            h3 { style: "margin: 0 0 0.4rem; font-size: 1rem;", "Summary" }
                            p { style: "margin: 0.35rem 0; color: #475569;", "Processed {metrics.spectra} spectra using {metrics.reference_mass_source} as the reference mass." }
                            p { style: "margin: 0.35rem 0; color: #475569;", "{metrics.spectra_with_reference_mass} spectra had a usable reference mass." }

                            div {
                                style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; margin-top: 1rem;",
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Observed precursor m/z" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "min: {format_value(metrics.observed_precursor_min)}" }
                                        li { "max: {format_value(metrics.observed_precursor_max)}" }
                                        li { "mean: {format_value(metrics.observed_precursor_mean)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Absolute error (Da)" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "min: {format_value(metrics.abs_error_da_min)}" }
                                        li { "mean: {format_value(metrics.abs_error_da_mean)}" }
                                        li { "RMS: {format_value(metrics.abs_error_da_rms)}" }
                                        li { "max: {format_value(metrics.abs_error_da_max)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Absolute error (ppm)" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "min: {format_value(metrics.abs_error_ppm_min)}" }
                                        li { "mean: {format_value(metrics.abs_error_ppm_mean)}" }
                                        li { "RMS: {format_value(metrics.abs_error_ppm_rms)}" }
                                        li { "max: {format_value(metrics.abs_error_ppm_max)}" }
                                    }
                                }
                                div { style: "background: white; padding: 0.8rem; border-radius: 12px; border: 1px solid #e2e8f0; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);",
                                    h4 { style: "margin: 0 0 0.45rem; font-size: 0.95rem; color: #0f172a;", "Signed mean" }
                                    ul { style: "padding-left: 1.1rem; margin: 0.25rem 0 0; color: #475569;",
                                        li { "Da: {format_value(metrics.signed_error_da_mean)}" }
                                        li { "ppm: {format_value(metrics.signed_error_ppm_mean)}" }
                                    }
                                }
                            }

                            div {
                                style: "margin-top: 1rem; display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1rem;",
                                histogram_plot {
                                    title: "Absolute precursor error (Da)".to_string(),
                                    subtitle: "Distribution across spectra, with tolerance bands highlighted".to_string(),
                                    histogram: metrics.da_error_histogram.clone(),
                                    thresholds: vec![0.005, 0.01, 0.5],
                                    unit: "Da".to_string(),
                                }
                                histogram_plot {
                                    title: "Absolute precursor error (ppm)".to_string(),
                                    subtitle: "Distribution across spectra, with common tolerance bands".to_string(),
                                    histogram: metrics.ppm_error_histogram.clone(),
                                    thresholds: vec![5.0, 10.0],
                                    unit: "ppm".to_string(),
                                }
                                scatter_plot {
                                    title: "Error versus precursor mass".to_string(),
                                    subtitle: "Signed ppm error as a function of precursor m/z".to_string(),
                                    points: metrics.plot_points.clone(),
                                }
                            }

                            div {
                                style: "margin-top: 1rem; display: flex; flex-wrap: wrap; gap: 0.55rem;",
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #e0f2fe; color: #0369a1; border-radius: 999px; font-weight: 600;",
                                    "≤ 0.01 Da: {metrics.within_0_01_da}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #f5f3ff; color: #7c3aed; border-radius: 999px; font-weight: 600;",
                                    "< 0.005 Da: {metrics.within_0_005_da}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #fef3c7; color: #b45309; border-radius: 999px; font-weight: 600;",
                                    "≤ 5 ppm: {metrics.within_5_ppm}"
                                }
                                span { style: "display: inline-block; padding: 0.4rem 0.7rem; background: #fdf2f8; color: #be185d; border-radius: 999px; font-weight: 600;",
                                    "≤ 10 ppm: {metrics.within_10_ppm}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_value(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "n/a".to_string()
    }
}

fn scientific_palette(index: usize) -> &'static str {
    [
        "#4477AA", "#66CCEE", "#228833", "#CCBB44", "#EE6677", "#AA3377", "#BBBBBB", "#004488",
    ][index % 8]
}

#[component]
fn histogram_plot(
    title: String,
    subtitle: String,
    histogram: HistogramData,
    thresholds: Vec<f64>,
    unit: String,
) -> Element {
    let max_count = histogram.bins.iter().copied().max().unwrap_or(1).max(1);
    let width = 360.0f64;
    let height = 220.0f64;
    let padding = 24.0f64;
    let plot_width = width - padding * 2.0;
    let plot_height = height - padding * 2.0;
    let bars = histogram
        .bins
        .iter()
        .enumerate()
        .map(|(index, count)| {
            let bar_width = plot_width / histogram.bins.len() as f64 - 2.0;
            let x = padding + (index as f64 * (plot_width / histogram.bins.len() as f64));
            let bar_height = if *count == 0 {
                0.0
            } else {
                (*count as f64 / max_count as f64) * plot_height
            };
            let y = height - padding - bar_height;
            rsx! {
                rect {
                    x: x as i32,
                    y: y as i32,
                    width: bar_width as i32,
                    height: bar_height as i32,
                    fill: scientific_palette(index),
                    opacity: "0.95"
                }
            }
        })
        .collect::<Vec<_>>();

    let threshold_lines = thresholds
        .iter()
        .enumerate()
        .map(|(index, threshold)| {
            let normalized = ((threshold - histogram.min)
                / (histogram.max - histogram.min).max(1e-9))
            .clamp(0.0, 1.0);
            let x = padding + normalized * plot_width;
            rsx! {
                line {
                    x1: x as i32,
                    y1: padding as i32,
                    x2: x as i32,
                    y2: (height - padding) as i32,
                    stroke: scientific_palette(index + 2),
                    stroke_width: "1.25",
                    stroke_dasharray: "4 3"
                }
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        div {
            style: "padding: 0.9rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
            h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
            p { style: "margin: 0 0 0.65rem; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
            svg {
                width: "100%",
                height: "220px",
                view_box: "0 0 360 220",
                role: "img",
               style: "display: block;",
               title { "{title}" }
                rect { x: 0, y: 0, width: 360, height: 220, fill: "#f8fafc" }
                line { x1: padding as i32, y1: (height - padding) as i32, x2: (width - padding) as i32, y2: (height - padding) as i32, stroke: "#64748b", stroke_width: "1" }
                line { x1: padding as i32, y1: padding as i32, x2: padding as i32, y2: (height - padding) as i32, stroke: "#64748b", stroke_width: "1" }
                for bar in bars {
                    {bar}
                }
                for line in threshold_lines {
                    {line}
                }
                text { x: (padding + 4.0) as i32, y: (padding + 12.0) as i32, fill: "#64748b", font_size: "11", "0" }
                text { x: (width - padding - 20.0) as i32, y: (height - padding + 16.0) as i32, fill: "#64748b", font_size: "11", "{unit}" }
            }
        }
    }
}

#[component]
fn scatter_plot(title: String, subtitle: String, points: Vec<PlotPoint>) -> Element {
    let width = 360.0f64;
    let height = 220.0f64;
    let padding = 24.0f64;
    let plot_width = width - padding * 2.0;
    let plot_height = height - padding * 2.0;

    let (x_min, x_max) = if points.is_empty() {
        (0.0, 1.0)
    } else {
        let min = points
            .iter()
            .map(|point| point.precursor_mz)
            .fold(f64::INFINITY, f64::min);
        let max = points
            .iter()
            .map(|point| point.precursor_mz)
            .fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    };
    let x_span = (x_max - x_min).max(1e-9);

    let (y_min, y_max) = if points.is_empty() {
        (-10.0, 10.0)
    } else {
        let min = points
            .iter()
            .map(|point| point.ppm_error)
            .fold(f64::INFINITY, f64::min);
        let max = points
            .iter()
            .map(|point| point.ppm_error)
            .fold(f64::NEG_INFINITY, f64::max);
        let span = (max - min).max(20.0);
        (min - span * 0.05, max + span * 0.05)
    };
    let y_span = (y_max - y_min).max(1e-9);

    let circles = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let x = padding + ((point.precursor_mz - x_min) / x_span) * plot_width;
            let y = height - padding - ((point.ppm_error - y_min) / y_span) * plot_height;
            let color_index = ((point.ppm_error.abs() / y_span.max(1.0)) * 7.0) as usize;
            rsx! {
                circle {
                    cx: x as i32,
                    cy: y as i32,
                    r: "2.2",
                    fill: scientific_palette(index + color_index),
                    opacity: "0.95"
                }
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        div {
            style: "padding: 0.9rem; border: 1px solid #e2e8f0; border-radius: 16px; background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);",
            h4 { style: "margin: 0 0 0.2rem; font-size: 0.95rem; color: #0f172a;", "{title}" }
            p { style: "margin: 0 0 0.65rem; color: #64748b; font-size: 0.84rem;", "{subtitle}" }
            svg {
                width: "100%",
                height: "220px",
                view_box: "0 0 360 220",
                role: "img",
                style: "display: block;",
                title { "{title}" }
                rect { x: 0, y: 0, width: 360, height: 220, fill: "#f8fafc" }
                line { x1: padding as i32, y1: (height - padding) as i32, x2: (width - padding) as i32, y2: (height - padding) as i32, stroke: "#64748b", stroke_width: "1" }
                line { x1: padding as i32, y1: padding as i32, x2: padding as i32, y2: (height - padding) as i32, stroke: "#64748b", stroke_width: "1" }
                for circle in circles {
                    {circle}
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct ProgressReporter<'a> {
    last_reported: u64,
    callback: Box<dyn FnMut(u64, u64) + 'a>,
}

#[cfg(target_arch = "wasm32")]
impl<'a> ProgressReporter<'a> {
    fn new<F>(callback: F) -> Self
    where
        F: FnMut(u64, u64) + 'a,
    {
        Self {
            last_reported: 0,
            callback: Box::new(callback),
        }
    }

    fn report_now(&mut self, processed: u64, total: u64) {
        (self.callback)(processed, total);
        self.last_reported = processed;
    }

    fn maybe_report(&mut self, processed: u64, total: u64) -> bool {
        if processed.saturating_sub(self.last_reported) >= PROGRESS_INTERVAL as u64 {
            self.report_now(processed, total);
            true
        } else {
            false
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct BlobLineReader<'a> {
    blob: Blob,
    offset: u64,
    buffer: Vec<u8>,
    processed: u64,
    progress: ProgressReporter<'a>,
}

#[cfg(target_arch = "wasm32")]
impl<'a> BlobLineReader<'a> {
    fn new<F>(blob: &Blob, on_progress: F) -> Self
    where
        F: FnMut(u64, u64) + 'a,
    {
        Self {
            blob: blob.clone(),
            offset: 0,
            buffer: Vec::new(),
            processed: 0,
            progress: ProgressReporter::new(on_progress),
        }
    }

    fn total_bytes(&self) -> u64 {
        self.blob.size() as u64
    }

    async fn next_line(&mut self) -> Result<Option<String>, ScanError> {
        loop {
            if let Some(line) = self.take_line_from_buffer() {
                return Ok(Some(line));
            }

            if self.offset >= self.total_bytes() {
                if self.buffer.is_empty() {
                    return Ok(None);
                }
                let line = std::mem::take(&mut self.buffer);
                return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
            }

            self.load_next_chunk().await?;
        }
    }

    fn take_line_from_buffer(&mut self) -> Option<String> {
        if let Some(pos) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let mut line_bytes = self.buffer[..pos].to_vec();
            self.buffer.drain(..=pos);
            if line_bytes.last() == Some(&b'\r') {
                line_bytes.pop();
            }
            Some(String::from_utf8_lossy(&line_bytes).into_owned())
        } else {
            None
        }
    }

    async fn load_next_chunk(&mut self) -> Result<(), ScanError> {
        let start = self.offset;
        let end = (self.offset + CHUNK_SIZE as u64).min(self.total_bytes());
        let start_f64 = start as f64;
        let end_f64 = end as f64;
        let chunk = self
            .blob
            .slice_with_f64_and_f64(start_f64, end_f64)
            .map_err(JsValue::from)?;
        let promise = chunk.array_buffer();
        let bytes = JsFuture::from(promise).await?;
        let array = Uint8Array::new(&bytes);
        self.buffer.extend_from_slice(&array.to_vec());
        self.offset = end;
        self.processed = self.processed.saturating_add((end - start).max(1));
        if self
            .progress
            .maybe_report(self.processed, self.total_bytes())
        {
            TimeoutFuture::new(0).await;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
async fn scan_blob_with_progress(
    blob: &Blob,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<PrecursorMetrics, ScanError> {
    let mut reader = BlobLineReader::new(blob, move |processed, total| {
        on_progress(processed, total);
    });

    let mut current_block = Vec::new();
    let mut current_is_in_block = false;
    let mut metrics = PrecursorMetrics::default();

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "BEGIN IONS" {
            current_block.clear();
            current_is_in_block = true;
            current_block.push(trimmed.to_string());
            continue;
        }

        if !current_is_in_block {
            continue;
        }

        if trimmed == "END IONS" {
            current_block.push(trimmed.to_string());
            if let Some(result) = process_block(&mut current_block).await? {
                metrics = merge_metrics(metrics, result);
            }
            current_block.clear();
            current_is_in_block = false;
            continue;
        }

        current_block.push(trimmed.to_string());
    }

    Ok(metrics)
}

#[cfg(target_arch = "wasm32")]
async fn process_block(block_lines: &mut [String]) -> Result<Option<PrecursorMetrics>, ScanError> {
    let mut headers = std::collections::BTreeMap::new();
    let mut observed_precursor = None;
    let mut reference_mass = None;
    let mut reference_mass_source = None;
    let mut charge = None;
    let mut adduct = None;
    let mut ion_mode = None;
    let mut smiles = None;
    let mut formula = None;
    let mut feature_id = None;
    let mut scans = None;
    let mut spectra_lines = Vec::new();

    for line in block_lines.iter() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "BEGIN IONS" || trimmed == "END IONS" {
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("PRECURSOR_MZ=") {
            if let Ok(value) = stripped.parse::<f64>() {
                observed_precursor = Some(value);
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("PEPMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                observed_precursor = Some(value);
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXACTMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                reference_mass = Some(value);
                reference_mass_source = Some("EXACTMASS");
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("MOLECULEMASS=") {
            if let Ok(value) = stripped.parse::<f64>() {
                reference_mass = Some(value);
                reference_mass_source = Some("MOLECULEMASS");
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("CHARGE=") {
            charge = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("SMILES=") {
            smiles = Some(stripped.trim().to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("FORMULA=") {
            formula = Some(stripped.trim().to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("ADDUCT=") {
            adduct = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("IONMODE=") {
            ion_mode = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("FEATURE_ID=") {
            feature_id = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("SCANS=") {
            scans = Some(stripped.to_string());
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("EXTRACTSCAN=") {
            if feature_id.is_none() {
                feature_id = Some(stripped.to_string());
            }
            if scans.is_none() {
                scans = Some(stripped.to_string());
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("FILENAME=") {
            headers.insert("FILENAME".to_string(), stripped.to_string());
            continue;
        }

        spectra_lines.push(trimmed.to_string());
    }

    let Some(observed_precursor) = observed_precursor else {
        return Ok(None);
    };

    let reference_mass = reference_mass.or_else(|| {
        smiles
            .as_deref()
            .and_then(exact_mass_from_smiles)
            .map(|mass| {
                reference_mass_source = Some("SMILES");
                mass
            })
            .or_else(|| {
                formula
                    .as_deref()
                    .and_then(exact_mass_from_formula)
                    .map(|mass| {
                        reference_mass_source = Some("FORMULA");
                        mass
                    })
            })
    });

    let Some(reference_mass) = reference_mass else {
        return Ok(None);
    };

    let reference_mass_source = reference_mass_source.unwrap_or("unknown");
    let reference_mass_label = adduct.as_deref().map_or_else(
        || reference_mass_source.to_string(),
        |adduct| format!("{reference_mass_source} + {adduct}"),
    );

    let mut normalized = Vec::new();
    normalized.push("BEGIN IONS".to_string());

    if let Some(filename) = headers.get("FILENAME") {
        normalized.push(format!("FILENAME={filename}"));
    }

    if let Some(feature_id) = feature_id.as_deref() {
        normalized.push(format!("FEATURE_ID={feature_id}"));
    }

    normalized.push(format!("PEPMASS={observed_precursor}"));
    if let Some(charge) = charge.as_deref() {
        normalized.push(format!("CHARGE={charge}"));
    }
    if let Some(scans) = scans.as_deref() {
        normalized.push(format!("SCANS={scans}"));
    }
    normalized.push("RTINSECONDS=1.0".to_string());
    normalized.extend(spectra_lines.clone());
    normalized.push("END IONS".to_string());

    let block_text = normalized.join("\n");
    let mut parsed_lines = Vec::new();
    for raw_line in block_text.lines().filter(|line| !line.is_empty()) {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed == "BEGIN IONS" || trimmed == "END IONS" {
            continue;
        }

        if trimmed.starts_with("SCANS=") {
            parsed_lines.push(trimmed.to_string());
            continue;
        }

        if trimmed.starts_with("FEATURE_ID=")
            || trimmed.starts_with("PEPMASS=")
            || trimmed.starts_with("CHARGE=")
            || trimmed.starts_with("RTINSECONDS=")
            || trimmed.starts_with("FILENAME=")
        {
            parsed_lines.push(trimmed.to_string());
        }
    }

    if parsed_lines.is_empty() {
        return Ok(None);
    }

    let expected_precursor_mz = expected_precursor_mz(
        reference_mass,
        adduct.as_deref(),
        charge.as_deref(),
        ion_mode.as_deref(),
    )
    .unwrap_or(reference_mass);
    let error_da = observed_precursor - expected_precursor_mz;
    let abs_error_da = error_da.abs();
    let ppm = if expected_precursor_mz.abs() > f64::EPSILON {
        error_da / expected_precursor_mz * 1_000_000.0
    } else {
        f64::NAN
    };
    let abs_ppm = ppm.abs();

    let mut metrics = PrecursorMetrics::default();
    metrics.spectra = 1;
    metrics.spectra_with_reference_mass = 1;
    metrics.reference_mass_source = reference_mass_label;
    metrics.observed_precursor_min = observed_precursor;
    metrics.observed_precursor_max = observed_precursor;
    metrics.observed_precursor_mean = observed_precursor;
    metrics.abs_error_da_min = abs_error_da;
    metrics.abs_error_da_max = abs_error_da;
    metrics.abs_error_da_mean = abs_error_da;
    metrics.abs_error_da_rms = abs_error_da;
    metrics.abs_error_ppm_min = abs_ppm;
    metrics.abs_error_ppm_max = abs_ppm;
    metrics.abs_error_ppm_mean = abs_ppm;
    metrics.abs_error_ppm_rms = abs_ppm;
    metrics.signed_error_da_mean = error_da;
    metrics.signed_error_ppm_mean = ppm;
    metrics.record_error(abs_error_da, abs_ppm, observed_precursor, ppm);
    if abs_error_da <= 0.01 {
        metrics.within_0_01_da = 1;
    }
    if abs_error_da < 0.005 {
        metrics.within_0_005_da = 1;
    }
    if abs_ppm <= 5.0 {
        metrics.within_5_ppm = 1;
    }
    if abs_ppm <= 10.0 {
        metrics.within_10_ppm = 1;
    }

    Ok(Some(metrics))
}

#[cfg(test)]
mod tests {
    use super::exact_mass_from_smiles;

    #[test]
    fn computes_exact_mass_from_smiles() {
        let mass = exact_mass_from_smiles("CCO").expect("valid SMILES should parse");
        assert!((mass - 46.041864814).abs() < 1e-4);
    }
}

#[cfg(target_arch = "wasm32")]
fn merge_metrics(mut current: PrecursorMetrics, next: PrecursorMetrics) -> PrecursorMetrics {
    let current_spectra = current.spectra as f64;
    let next_spectra = next.spectra as f64;
    let total_spectra = current_spectra + next_spectra;

    current.spectra += next.spectra;
    current.spectra_with_reference_mass += next.spectra_with_reference_mass;
    if current.reference_mass_source == "none" {
        current.reference_mass_source = next.reference_mass_source;
    } else if current.reference_mass_source != next.reference_mass_source
        && !next.reference_mass_source.is_empty()
    {
        current.reference_mass_source = "mixed".to_string();
    }

    current.observed_precursor_min = current
        .observed_precursor_min
        .min(next.observed_precursor_min);
    current.observed_precursor_max = current
        .observed_precursor_max
        .max(next.observed_precursor_max);
    current.observed_precursor_mean = ((current.observed_precursor_mean * current_spectra)
        + (next.observed_precursor_mean * next_spectra))
        / total_spectra;

    current.abs_error_da_min = current.abs_error_da_min.min(next.abs_error_da_min);
    current.abs_error_da_max = current.abs_error_da_max.max(next.abs_error_da_max);
    current.abs_error_da_mean = ((current.abs_error_da_mean * current_spectra)
        + (next.abs_error_da_mean * next_spectra))
        / total_spectra;
    let current_da_rms_sq = current.abs_error_da_rms * current.abs_error_da_rms;
    let next_da_rms_sq = next.abs_error_da_rms * next.abs_error_da_rms;
    current.abs_error_da_rms =
        ((current_da_rms_sq * current_spectra) + (next_da_rms_sq * next_spectra)) / total_spectra;
    current.abs_error_da_rms = current.abs_error_da_rms.sqrt();

    current.abs_error_ppm_min = current.abs_error_ppm_min.min(next.abs_error_ppm_min);
    current.abs_error_ppm_max = current.abs_error_ppm_max.max(next.abs_error_ppm_max);
    current.abs_error_ppm_mean = ((current.abs_error_ppm_mean * current_spectra)
        + (next.abs_error_ppm_mean * next_spectra))
        / total_spectra;
    let current_ppm_rms_sq = current.abs_error_ppm_rms * current.abs_error_ppm_rms;
    let next_ppm_rms_sq = next.abs_error_ppm_rms * next.abs_error_ppm_rms;
    current.abs_error_ppm_rms =
        ((current_ppm_rms_sq * current_spectra) + (next_ppm_rms_sq * next_spectra)) / total_spectra;
    current.abs_error_ppm_rms = current.abs_error_ppm_rms.sqrt();

    current.signed_error_da_mean = ((current.signed_error_da_mean * current_spectra)
        + (next.signed_error_da_mean * next_spectra))
        / total_spectra;
    current.signed_error_ppm_mean = ((current.signed_error_ppm_mean * current_spectra)
        + (next.signed_error_ppm_mean * next_spectra))
        / total_spectra;

    current.within_0_01_da += next.within_0_01_da;
    current.within_0_005_da += next.within_0_005_da;
    current.within_5_ppm += next.within_5_ppm;
    current.within_10_ppm += next.within_10_ppm;

    for (idx, count) in next.da_error_histogram.bins.iter().enumerate() {
        if let Some(current_count) = current.da_error_histogram.bins.get_mut(idx) {
            *current_count += count;
        }
    }
    for (idx, count) in next.ppm_error_histogram.bins.iter().enumerate() {
        if let Some(current_count) = current.ppm_error_histogram.bins.get_mut(idx) {
            *current_count += count;
        }
    }

    current.plot_points.extend(next.plot_points);
    if current.plot_points.len() > 260 {
        current.plot_points = current
            .plot_points
            .iter()
            .enumerate()
            .filter(|(idx, _)| idx % (current.plot_points.len() / 260 + 1) == 0)
            .map(|(_, point)| point.clone())
            .collect();
    }

    current
}
