#![allow(non_snake_case)]

//! Smellfish-rs categories.
//!
//! - **Shared decorations**: RDKit SMARTS motifs found across the uploaded
//!   molecules. These are the recurring substructures used to spot common
//!   chemical decorations.
//! - **Result**: a quick source check for each row. `PubChem` and `LOTUS` are
//!   marked by SPARQL hits; `Natural product` is positive whenever either source
//!   supports the molecule; `Synthetic score` is a coarse heuristic for rows
//!   that fail both checks.
//! - **Verdict**: the user-facing summary. The app leans toward
//!   "Looks legitimate" when both sources agree, "Citation available" when one
//!   source agrees, and "Citation needed" / "Smells fishy" when neither does.

use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use futures::future::join;
#[cfg(target_arch = "wasm32")]
use js_sys::{JSON, Promise, Reflect};
#[cfg(target_arch = "wasm32")]
use serde::Deserialize;
#[cfg(target_arch = "wasm32")]
use std::collections::{BTreeSet, HashMap, HashSet};
#[cfg(target_arch = "wasm32")]
use std::sync::OnceLock;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
const LOTUS_ENDPOINT: &str = "https://qlever.dev/api/wikidata";
#[cfg(target_arch = "wasm32")]
const PUBCHEM_ENDPOINT: &str = "https://qlever.cs.uni-freiburg.de/api/pubchem";
#[cfg(target_arch = "wasm32")]
const QUERY_CHUNK_SIZE: usize = 40;

const CSS: &str = r#"
.shell {
  min-height: 100vh;
  padding: 24px;
  background: #f5f7fb;
  color: #162033;
  font-family: Inter, system-ui, sans-serif;
}
.card, .panel {
  background: #fff;
  border: 1px solid #d7dee9;
  border-radius: 16px;
  box-shadow: 0 10px 35px rgba(25, 39, 62, 0.06);
}
.panel { padding: 20px; margin-bottom: 20px; }
.hero {
  display: grid;
  gap: 10px;
  margin-bottom: 20px;
}
.hero h1 {
  margin: 0;
  font-size: 2rem;
}
.hero p {
  margin: 0;
  color: #4d5b74;
  line-height: 1.5;
}
.dropzone {
  position: relative;
  display: grid;
  gap: 8px;
  place-items: center;
  min-height: 150px;
  border: 2px dashed #9eb0cc;
  border-radius: 16px;
  padding: 18px;
  cursor: pointer;
  background: #fbfcfe;
  text-align: center;
}
.dropzone.dragging { border-color: #0d6efd; background: #eef5ff; }
.dropzone input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}
.status {
  margin: 12px 0 0;
  font-weight: 600;
}
.summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px;
}
.summary-item {
  padding: 14px;
  border: 1px solid #e1e7f0;
  border-radius: 14px;
  background: #fcfdff;
}
.summary-item h3, .summary-item h4 {
  margin: 0 0 8px;
  font-size: 0.95rem;
}
.chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-width: 100%;
}
.chip {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  padding: 5px 9px;
  border-radius: 999px;
  background: #eef4ff;
  color: #21467a;
  font-size: 0.82rem;
}
.chip.alt { background: #f3f0ff; color: #5e44ad; }
.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}
.card {
  overflow: visible;
}
.card-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 14px 0;
}
.card-body {
  display: grid;
  gap: 10px;
  padding: 14px;
}
.svg-wrap {
  display: grid;
  place-items: center;
  min-height: 210px;
  background: #fff;
  border: 1px solid #eef2f7;
  border-radius: 14px;
  overflow: visible;
}
.svg-wrap svg {
  display: block;
  max-width: 100%;
  max-height: 260px;
  width: 100%;
  height: auto;
}
.svg-wrap > div {
  width: 100%;
}
.meta {
  display: grid;
  gap: 6px;
  font-size: 0.9rem;
  color: #384861;
  overflow-wrap: anywhere;
}
.result-box {
  display: grid;
  gap: 4px;
  padding: 12px;
  border: 1px solid #e3e9f2;
  border-radius: 12px;
  background: #f9fbff;
}
.result-grid {
  display: grid;
  gap: 4px;
  font-size: 0.88rem;
}
.result-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 12px;
}
.result-row span:first-child {
  overflow-wrap: anywhere;
}
.result-badge {
  font-weight: 700;
  white-space: nowrap;
}
.verdict {
  margin-top: 4px;
  font-weight: 700;
}
.muted { color: #63738d; }
.error { color: #b42318; font-weight: 600; }
.small { font-size: 0.82rem; }
"#;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
struct RawRow {
    index: usize,
    label: String,
    smiles: String,
}

#[derive(Clone, Debug)]
struct MoleculeRow {
    index: usize,
    label: String,
    smiles: String,
    canonical_smiles: String,
    inchikey: String,
    svg: Option<String>,
    motifs: Vec<String>,
    lotus_taxa: Vec<String>,
    lotus_compounds: Vec<String>,
    pubchem_cids: Vec<String>,
    pubchem_names: Vec<String>,
    pubchem_taxa: Vec<String>,
    verdict: String,
    error: Option<String>,
}

#[derive(Clone, Debug)]
struct MotifSummary {
    label: String,
    smarts: String,
    count: usize,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug, Default)]
struct SourceSummary {
    taxa: BTreeSet<String>,
    compounds: BTreeSet<String>,
    names: BTreeSet<String>,
    cids: BTreeSet<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug, Default)]
struct Enrichment {
    lotus: HashMap<String, SourceSummary>,
    pubchem: HashMap<String, SourceSummary>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
struct RdkitInspectResponse {
    canonicalsmiles: Option<String>,
    inchikey: Option<String>,
    svg: Option<String>,
    motifs: Option<Vec<RdkitMotifHit>>,
    error: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
struct RdkitMotifHit {
    label: String,
    smarts: String,
}

fn main() {
    launch(app);
}

#[component]
fn app() -> Element {
    #[cfg(target_arch = "wasm32")]
    let file_name = use_signal(String::new);
    #[cfg(not(target_arch = "wasm32"))]
    let mut file_name = use_signal(String::new);
    let mut status = use_signal(|| "Drop a CSV with a SMILES column to begin.".to_string());
    let busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);
    let rows = use_signal(Vec::<MoleculeRow>::new);
    let motifs = use_signal(Vec::<MotifSummary>::new);

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
        begin_import(
            web_file.clone(),
            file.name(),
            file_name,
            status,
            busy,
            drag_active,
            rows,
            motifs,
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            status.set("This app needs to run in a browser.".to_string());
        }
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
        begin_import(
            web_file.clone(),
            file.name(),
            file_name,
            status,
            busy,
            drag_active,
            rows,
            motifs,
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            status.set("This app needs to run in a browser.".to_string());
        }
    };

    rsx! {
        div { class: "shell",
            style { "{CSS}" }

            section { class: "hero",
                h1 { "🐟 Smellfish-rs" }
                p { "Because something smells fishy. Drop a CSV with SMILES, render every molecule with RDKit.js, find the most common decorations, and stamp Citation needed when the evidence looks thin." }
                p { class: "small muted", "Uses rdkit.js for structure rendering and QLever SPARQL for lookup — no API calls." }
            }

            section { class: "panel",
                h2 { "How the labels work" }
                div { class: "summary-grid",
                    div { class: "summary-item",
                        h3 { "Shared decorations" }
                        div { class: "small muted", "Common RDKit SMARTS motifs found across the uploaded molecules. These are the recurring substructures that get highlighted in the grid." }
                    }
                    div { class: "summary-item",
                        h3 { "Result" }
                        div { class: "small muted", "A quick source check for each row: PubChem and LOTUS show SPARQL hits, Natural product is positive when either source agrees, and Synthetic score is the fallback when both checks fail." }
                    }
                    div { class: "summary-item",
                        h3 { "Verdict" }
                        div { class: "small muted", "The human-readable call. Both sources agreeing means Looks legitimate; one source gives Citation available; no source means Citation needed or Smells fishy." }
                    }
                }
            }

            section { class: "panel",
                label { class: if *drag_active.read() { "dropzone dragging" } else { "dropzone" },
                    r#for: "smiles-csv",
                    ondragenter: on_drag_enter,
                    ondragover: on_drag_over,
                    ondragleave: on_drag_leave,
                    ondrop: on_drop,

                    div {
                        strong { "Drop CSV here or click to browse" }
                        div { class: "small muted", "Expect a column named smiles (or smile / structure)." }
                    }

                    input {
                        id: "smiles-csv",
                        r#type: "file",
                        accept: ".csv,text/csv",
                        disabled: *busy.read(),
                        onchange: on_file_change,
                    }
                }

                p { class: "status", role: "status", aria_live: "polite", aria_atomic: "true", "{status}" }

                if !file_name.read().is_empty() {
                    p { class: "small muted", "Loaded: {file_name}" }
                }
            }

            if !motifs.read().is_empty() || !rows.read().is_empty() {
                section { class: "panel",
                    h2 { "Shared decorations" }
                    div { class: "summary-grid",
                        for motif in motifs.read().iter() {
                            div { class: "summary-item",
                                h3 { "{motif.label}" }
                                div { class: "small muted", "{motif.smarts}" }
                                div { class: "chip", "{motif.count} molecules" }
                            }
                        }
                    }
                }
            }

            if !rows.read().is_empty() {
                section { class: "cards",
                    for row in rows.read().iter() {
                        article { class: "card",
                            div { class: "card-head",
                                div {
                                    strong { "{row.label}" }
                                    div { class: "small muted", "Row {row.index}" }
                                }
                                if let Some(err) = row.error.as_deref() {
                                    div { class: "error small", "{err}" }
                                }
                            }
                            div { class: "card-body",
                                div { class: "svg-wrap",
                                    if let Some(svg) = row.svg.as_deref() {
                                        div { dangerous_inner_html: "{svg}" }
                                    } else {
                                        div { class: "small muted", "No SVG available." }
                                    }
                                }
                                div { class: "meta",
                                    div { "SMILES: " span { class: "muted", "{row.smiles}" } }
                                    if !row.canonical_smiles.is_empty() {
                                        div { "Canonical: " span { class: "muted", "{row.canonical_smiles}" } }
                                    }
                                    div { "InChIKey: " span { class: "muted", "{row.inchikey}" } }
                                    div {
                                        "Motifs: "
                                        if row.motifs.is_empty() {
                                            span { class: "muted", "none" }
                                        } else {
                                            span { class: "chip-list",
                                                for motif in row.motifs.iter() {
                                                    span { class: "chip alt", "{motif}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "meta",
                                    strong { "LOTUS" }
                                    if row.lotus_taxa.is_empty() {
                                        div { class: "muted", "No taxa found." }
                                    } else {
                                        div { class: "chip-list",
                                            for taxon in row.lotus_taxa.iter().take(4) {
                                                span { class: "chip", "{taxon}" }
                                            }
                                        }
                                    }
                                    if !row.lotus_compounds.is_empty() {
                                        div { class: "small muted", "Compounds: {row.lotus_compounds.len()}" }
                                    }
                                }
                                div { class: "meta",
                                    strong { "PubChem" }
                                    if row.pubchem_cids.is_empty() {
                                        div { class: "muted", "No records found." }
                                    } else {
                                        div { class: "chip-list",
                                            for cid in row.pubchem_cids.iter().take(4) {
                                                span { class: "chip alt", "CID {cid}" }
                                            }
                                        }
                                    }
                                    if !row.pubchem_names.is_empty() {
                                        div { class: "small muted", "Names: {row.pubchem_names.len()}" }
                                    }
                                    if !row.pubchem_taxa.is_empty() {
                                        div { class: "small muted", "Taxa: {row.pubchem_taxa.len()}" }
                                    }
                                }
                                div { class: "result-box",
                                    strong { "Result" }
                                    div { class: "result-grid",
                                        div { class: "result-row",
                                            span { "PubChem" }
                                            span { class: "result-badge", if row.pubchem_cids.is_empty() { "✗" } else { "✓" } }
                                        }
                                        div { class: "result-row",
                                            span { "LOTUS" }
                                            span { class: "result-badge", if row.lotus_taxa.is_empty() { "✗" } else { "✓" } }
                                        }
                                        div { class: "result-row",
                                            span { "Natural product" }
                                            span { class: "result-badge", if row.lotus_taxa.is_empty() && row.pubchem_cids.is_empty() { "✗" } else { "✓" } }
                                        }
                                        div { class: "result-row",
                                            span { "Synthetic score" }
                                            span { class: "result-badge", if row.lotus_taxa.is_empty() && row.pubchem_cids.is_empty() { "high" } else { "low" } }
                                        }
                                    }
                                    div { class: "verdict", "{row.verdict}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn begin_import(
    file: web_sys::File,
    file_name_value: String,
    mut file_name: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
    mut rows: Signal<Vec<MoleculeRow>>,
    mut motifs: Signal<Vec<MotifSummary>>,
) {
    file_name.set(file_name_value);
    busy.set(true);
    drag_active.set(false);
    status.set("Reading CSV...".to_string());
    rows.set(Vec::new());
    motifs.set(Vec::new());

    spawn(async move {
        let result: Result<(Vec<MoleculeRow>, Vec<MotifSummary>, usize, Option<String>), String> =
            async move {
                let text = read_file_text(&file).await?;
                let raw_rows = parse_csv_rows(&text)?;
                let mut parsed_rows = Vec::with_capacity(raw_rows.len());
                let mut motif_counts: HashMap<String, (String, HashSet<usize>)> = HashMap::new();
                let mut inchikeys = BTreeSet::new();

                for raw in raw_rows {
                    match rdkit_inspect(&raw.smiles).await {
                        Ok(inspect) => {
                            if let Some(err) = inspect.error {
                                parsed_rows.push(MoleculeRow {
                                    index: raw.index,
                                    label: raw.label,
                                    smiles: raw.smiles,
                                    canonical_smiles: String::new(),
                                    inchikey: String::new(),
                                    svg: None,
                                    motifs: Vec::new(),
                                    lotus_taxa: Vec::new(),
                                    lotus_compounds: Vec::new(),
                                    pubchem_cids: Vec::new(),
                                    pubchem_names: Vec::new(),
                                    pubchem_taxa: Vec::new(),
                                    verdict: String::new(),
                                    error: Some(err),
                                });
                                continue;
                            }
                            let motifs_list = inspect.motifs.unwrap_or_default();
                            let motif_labels = motifs_list
                                .iter()
                                .map(|hit| hit.label.clone())
                                .collect::<Vec<_>>();
                            for hit in &motifs_list {
                                let entry = motif_counts
                                    .entry(hit.label.clone())
                                    .or_insert_with(|| (hit.smarts.clone(), HashSet::new()));
                                entry.1.insert(raw.index);
                            }
                            let inchikey = inspect.inchikey.unwrap_or_default();
                            if !inchikey.is_empty() {
                                inchikeys.insert(inchikey.clone());
                            }
                            parsed_rows.push(MoleculeRow {
                                index: raw.index,
                                label: raw.label,
                                smiles: raw.smiles,
                                canonical_smiles: inspect.canonicalsmiles.unwrap_or_default(),
                                inchikey,
                                svg: inspect.svg,
                                motifs: motif_labels,
                                lotus_taxa: Vec::new(),
                                lotus_compounds: Vec::new(),
                                pubchem_cids: Vec::new(),
                                pubchem_names: Vec::new(),
                                pubchem_taxa: Vec::new(),
                                verdict: String::new(),
                                error: None,
                            });
                        }
                        Err(err) => {
                            parsed_rows.push(MoleculeRow {
                                index: raw.index,
                                label: raw.label,
                                smiles: raw.smiles,
                                canonical_smiles: String::new(),
                                inchikey: String::new(),
                                svg: None,
                                motifs: Vec::new(),
                                lotus_taxa: Vec::new(),
                                lotus_compounds: Vec::new(),
                                pubchem_cids: Vec::new(),
                                pubchem_names: Vec::new(),
                                pubchem_taxa: Vec::new(),
                                verdict: String::new(),
                                error: Some(err),
                            });
                        }
                    }
                }

                let motif_summary = motif_counts
                    .into_iter()
                    .map(|(label, (smarts, rows))| MotifSummary {
                        label,
                        smarts,
                        count: rows.len(),
                    })
                    .collect::<Vec<_>>();

                let unique_keys = inchikeys.into_iter().collect::<Vec<_>>();
                let mut enrichment_warning = None;
                let enrichment = if unique_keys.is_empty() {
                    Enrichment::default()
                } else {
                    match enrich_sources(&unique_keys).await {
                        Ok(data) => data,
                        Err(err) => {
                            enrichment_warning =
                                Some(format!("Parsed CSV, but enrichment failed: {err}"));
                            Enrichment::default()
                        }
                    }
                };

                let parsed_rows = merge_enrichment(parsed_rows, &enrichment);
                Ok((
                    parsed_rows,
                    motif_summary,
                    unique_keys.len(),
                    enrichment_warning,
                ))
            }
            .await;

        match result {
            Ok((parsed_rows, motif_summary, key_count, warning)) => {
                let row_count = parsed_rows.len();
                let motif_count = motif_summary.len();
                rows.set(parsed_rows);
                motifs.set(sorted_motifs(motif_summary));
                status.set(warning.unwrap_or_else(|| {
                    format!("Done — {row_count} rows, {motif_count} motifs, {key_count} unique InChIKeys")
                }));
            }
            Err(err) => {
                status.set(format!("Error reading CSV: {err}"));
            }
        }

        busy.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
fn sorted_motifs(mut motifs: Vec<MotifSummary>) -> Vec<MotifSummary> {
    motifs.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.label.cmp(&right.label))
    });
    motifs
}

#[cfg(target_arch = "wasm32")]
fn merge_enrichment(rows: Vec<MoleculeRow>, enrichment: &Enrichment) -> Vec<MoleculeRow> {
    rows.into_iter()
        .map(|mut row| {
            if !row.inchikey.is_empty() {
                if let Some(summary) = enrichment.lotus.get(&row.inchikey) {
                    row.lotus_taxa = summary.taxa.iter().cloned().collect();
                    row.lotus_compounds = summary.compounds.iter().cloned().collect();
                }
                if let Some(summary) = enrichment.pubchem.get(&row.inchikey) {
                    row.pubchem_cids = summary.cids.iter().cloned().collect();
                    row.pubchem_names = summary.names.iter().cloned().collect();
                    row.pubchem_taxa = summary.taxa.iter().cloned().collect();
                }
            }
            row.verdict = verdict_for_row(&row);
            row
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn verdict_for_row(row: &MoleculeRow) -> String {
    if row.error.is_some() {
        return "⚠ Citation needed.".to_string();
    }
    let has_lotus = !row.lotus_taxa.is_empty();
    let has_pubchem = !row.pubchem_cids.is_empty();
    let is_natural = has_lotus || has_pubchem;
    let suspicious = !has_lotus && !has_pubchem;

    if suspicious {
        return "👃 Smells fishy. Citation needed.".to_string();
    }
    if has_lotus && has_pubchem {
        return "Looks legitimate.".to_string();
    }
    if is_natural {
        return "📚 Citation available.".to_string();
    }
    "🤨 Citation needed.".to_string()
}

#[cfg(target_arch = "wasm32")]
fn parse_csv_rows(text: &str) -> Result<Vec<RawRow>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    let smiles_idx = detect_column(
        &headers,
        &["smiles", "smile", "structure", "canonical_smiles"],
    )
    .unwrap_or(0);
    let label_idx = detect_column(&headers, &["name", "label", "id", "compound"]);

    let mut rows = Vec::new();
    for (line, record) in rdr.records().enumerate() {
        let record = record.map_err(|e| e.to_string())?;
        let smiles = record.get(smiles_idx).unwrap_or("").trim().to_string();
        if smiles.is_empty() {
            rows.push(RawRow {
                index: line + 1,
                label: label_for_record(&record, label_idx, line + 1),
                smiles: String::new(),
            });
            continue;
        }
        rows.push(RawRow {
            index: line + 1,
            label: label_for_record(&record, label_idx, line + 1),
            smiles,
        });
    }

    if rows.is_empty() {
        return Err("CSV does not contain any data rows".to_string());
    }
    Ok(rows)
}

#[cfg(target_arch = "wasm32")]
fn detect_column(headers: &csv::StringRecord, names: &[&str]) -> Option<usize> {
    headers.iter().enumerate().find_map(|(idx, header)| {
        let normalized = header.trim().to_ascii_lowercase();
        names
            .iter()
            .any(|needle| normalized == *needle)
            .then_some(idx)
    })
}

#[cfg(target_arch = "wasm32")]
fn label_for_record(
    record: &csv::StringRecord,
    label_idx: Option<usize>,
    line_no: usize,
) -> String {
    let label = label_idx
        .and_then(|idx| record.get(idx))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("");
    if label.is_empty() {
        format!("Molecule {line_no}")
    } else {
        label.to_string()
    }
}

#[cfg(target_arch = "wasm32")]
async fn read_file_text(file: &web_sys::File) -> Result<String, String> {
    let promise = file.text();
    let value = JsFuture::from(promise)
        .await
        .map_err(|err| format!("failed to read file: {err:?}"))?;
    value
        .as_string()
        .ok_or_else(|| "file did not resolve to text".to_string())
}

#[cfg(target_arch = "wasm32")]
async fn rdkit_inspect(smiles: &str) -> Result<RdkitInspectResponse, String> {
    let value = rdkit_bridge_call("inspect", smiles).await?;
    let json = js_value_to_json(value)?;
    serde_json::from_value(json).map_err(|err| err.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn rdkit_bridge_call(method: &str, smiles: &str) -> Result<JsValue, String> {
    let window = web_sys::window().ok_or_else(|| "window is unavailable".to_string())?;
    let window_value = JsValue::from(window);
    let bridge = Reflect::get(&window_value, &JsValue::from_str("__smilesRdkit"))
        .map_err(|_| "rdkit.js bridge lookup failed".to_string())?;
    if bridge.is_null() || bridge.is_undefined() {
        return Err("rdkit.js bridge is unavailable".to_string());
    }

    let ready = Reflect::get(&bridge, &JsValue::from_str("ready"))
        .map_err(|_| "rdkit.js readiness promise missing".to_string())?;
    if let Ok(promise) = ready.dyn_into::<Promise>() {
        JsFuture::from(promise)
            .await
            .map_err(|err| format!("rdkit.js failed to initialize: {err:?}"))?;
    }

    let function = Reflect::get(&bridge, &JsValue::from_str(method))
        .map_err(|_| format!("rdkit.js method '{method}' not found"))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| format!("rdkit.js method '{method}' is not callable"))?;

    let result = function
        .call1(&bridge, &JsValue::from_str(smiles))
        .map_err(|err| format!("rdkit.js {method} call failed: {err:?}"))?;

    match result.dyn_into::<Promise>() {
        Ok(promise) => JsFuture::from(promise)
            .await
            .map_err(|err| format!("rdkit.js {method} failed: {err:?}")),
        Err(val) => Ok(val),
    }
}

#[cfg(target_arch = "wasm32")]
fn js_value_to_json(value: JsValue) -> Result<serde_json::Value, String> {
    let text = JSON::stringify(&value)
        .ok()
        .and_then(|value| value.as_string())
        .ok_or_else(|| "rdkit.js returned a non-serializable value".to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

#[cfg(target_arch = "wasm32")]
fn http_client() -> Result<&'static reqwest::Client, String> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    match CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .build()
            .map_err(|e| e.to_string())
    }) {
        Ok(client) => Ok(client),
        Err(message) => Err(format!("failed to initialize HTTP client: {message}")),
    }
}

#[cfg(target_arch = "wasm32")]
async fn enrich_sources(inchikeys: &[String]) -> Result<Enrichment, String> {
    let lotus = fetch_lotus_hits(inchikeys);
    let pubchem = fetch_pubchem_hits(inchikeys);
    let (lotus, pubchem) = join(lotus, pubchem).await;
    Ok(Enrichment {
        lotus: lotus?,
        pubchem: pubchem?,
    })
}

#[cfg(target_arch = "wasm32")]
async fn fetch_lotus_hits(inchikeys: &[String]) -> Result<HashMap<String, SourceSummary>, String> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = build_lotus_query(chunk);
        for binding in sparql_bindings(LOTUS_ENDPOINT, &query).await? {
            let inchikey = binding_value(&binding, "inchikey");
            if inchikey.is_empty() {
                continue;
            }
            let entry = summary.entry(inchikey).or_default();
            let compound = binding_value(&binding, "compoundLabel");
            let taxon = binding_value(&binding, "taxonLabel");
            if !compound.is_empty() {
                entry.compounds.insert(compound);
            }
            if !taxon.is_empty() {
                entry.taxa.insert(taxon);
            }
        }
    }
    Ok(summary)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_pubchem_hits(
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, String> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = build_pubchem_query(chunk);
        for binding in sparql_bindings(PUBCHEM_ENDPOINT, &query).await? {
            let inchikey = binding_value(&binding, "inchikey");
            if inchikey.is_empty() {
                continue;
            }
            let entry = summary.entry(inchikey).or_default();
            let cid = binding_value(&binding, "cid");
            let label = binding_value(&binding, "label");
            let iupac = binding_value(&binding, "iupac");
            let taxon = binding_value(&binding, "taxonLabel");
            if !cid.is_empty() {
                entry.cids.insert(cid);
            }
            if !label.is_empty() {
                entry.names.insert(label);
            }
            if !iupac.is_empty() {
                entry.names.insert(iupac);
            }
            if !taxon.is_empty() {
                entry.taxa.insert(taxon);
            }
        }
    }
    Ok(summary)
}

#[cfg(target_arch = "wasm32")]
async fn sparql_bindings(
    endpoint: &str,
    query: &str,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, String> {
    let response = post_sparql(endpoint, query).await?;
    let bindings = response
        .get("results")
        .and_then(|value| value.get("bindings"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "SPARQL response missing bindings".to_string())?;
    Ok(bindings
        .iter()
        .filter_map(serde_json::Value::as_object)
        .cloned()
        .collect())
}

#[cfg(target_arch = "wasm32")]
async fn post_sparql(endpoint: &str, query: &str) -> Result<serde_json::Value, String> {
    let client = http_client()?;
    let response = client
        .post(endpoint)
        .header("Accept", "application/sparql-results+json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("query={}", urlencoding::encode(query)))
        .send()
        .await
        .map_err(|err| err.to_string())?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), compact_text(&body)));
    }

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|err| err.to_string())
}

#[cfg(target_arch = "wasm32")]
fn compact_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len().min(200));
    for ch in text.chars() {
        if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t') {
            continue;
        }
        out.push(ch);
        if out.len() >= 200 {
            out.push_str("...");
            break;
        }
    }
    out.trim().to_string()
}

#[cfg(target_arch = "wasm32")]
fn binding_value(binding: &serde_json::Map<String, serde_json::Value>, key: &str) -> String {
    binding
        .get(key)
        .and_then(|value| value.get("value"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(target_arch = "wasm32")]
fn build_lotus_query(chunk: &[String]) -> String {
    let values = chunk
        .iter()
        .map(|value| format!("\"{}\"", escape_sparql_literal(value)))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
PREFIX p: <http://www.wikidata.org/prop/>
PREFIX ps: <http://www.wikidata.org/prop/statement/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT DISTINCT ?inchikey ?compound ?compoundLabel ?taxonLabel WHERE {{
  VALUES ?inchikey {{ {values} }}
  ?compound wdt:P235 ?inchikey ;
            p:P703 ?statement .
  ?statement ps:P703 ?taxon .
  OPTIONAL {{ ?compound rdfs:label ?compoundLabel FILTER(LANG(?compoundLabel) = "en") }}
  OPTIONAL {{ ?taxon rdfs:label ?taxonLabel FILTER(LANG(?taxonLabel) = "en") }}
}}
"#
    )
}

#[cfg(target_arch = "wasm32")]
fn build_pubchem_query(chunk: &[String]) -> String {
    let values = chunk
        .iter()
        .map(|value| format!("\"{}\"", escape_sparql_literal(value)))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX vocab: <http://rdf.ncbi.nlm.nih.gov/pubchem/vocabulary#>
PREFIX skos: <http://www.w3.org/2004/02/skos/core#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT DISTINCT ?inchikey ?cid ?label ?iupac ?taxonLabel WHERE {{
  VALUES ?inchikey {{ {values} }}
  ?compound dcterms:identifier ?cid ;
            vocab:inchikey ?inchikey .
  OPTIONAL {{ ?compound skos:prefLabel ?label }}
  OPTIONAL {{ ?compound vocab:preferred_iupac_name ?iupac }}
  OPTIONAL {{
    ?compound <http://purl.obolibrary.org/obo/RO_0002162> ?taxon .
    OPTIONAL {{ ?taxon rdfs:label ?taxonLabel FILTER(LANG(?taxonLabel) = "en") }}
  }}
}}
"#
    )
}

#[cfg(target_arch = "wasm32")]
fn escape_sparql_literal(value: &str) -> String {
    value.replace('\\', r"\\").replace('"', r#"\""#)
}
