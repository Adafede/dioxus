use crate::literature::LITERATURE;
use crate::model::{EndpointStatus, MoleculeRow, MotifSummary};
use crate::styles::CSS;
use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::pipeline::begin_import;

#[component]
pub fn app() -> Element {
    #[cfg(target_arch = "wasm32")]
    let file_name = use_signal(String::new);
    #[cfg(not(target_arch = "wasm32"))]
    let mut file_name = use_signal(String::new);

    let mut status = use_signal(|| "Drop a CSV with a SMILES column to begin.".to_string());
    let busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);
    let rows = use_signal(Vec::<MoleculeRow>::new);
    let motifs = use_signal(Vec::<MotifSummary>::new);
    let endpoints = use_signal(Vec::<EndpointStatus>::new);
    let warnings = use_signal(Vec::<String>::new);

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
            endpoints,
            warnings,
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
            endpoints,
            warnings,
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            file_name.set(file.name());
            status.set("This app needs to run in a browser.".to_string());
        }
    };

    let file_name_value = file_name.read().clone();
    let warning_text = warnings.read().join(" • ");

    rsx! {
        div { class: "shell",
            style { "{CSS}" }

            section { class: "hero",
                h1 { "🐟 Smellfish-rs" }
                p { "A literature-backed natural-product originality screen for SMILES lists." }
                p { "It combines Ertl-style NP-likeness, scaffold complexity, stereo richness, and LOTUS / PubChem evidence." }
            }

            section { class: "panel",
                details {
                    summary { h2 { "Evidence basis" } }
                    div { class: "literature-list",
                        for paper in LITERATURE {
                            div { class: "literature-item",
                                strong { "{paper.title}" }
                                div { class: "small muted", "{paper.note}" }
                                div { class: "small", a { href: "https://doi.org/{paper.doi}", target: "_blank", rel: "noreferrer", "{paper.doi}" } }
                            }
                        }
                    }
                }
            }

            section { class: "panel",
                h2 { "How the labels work" }
                div { class: "summary-grid",
                    div { class: "summary-item",
                        h3 { "NP-likeness" }
                        div { class: "small muted", "Ertl-inspired score from ring complexity, sp3 richness, polarity, stereochemistry, and natural-product motifs." }
                    }
                    div { class: "summary-item",
                        h3 { "Scaffold family" }
                        div { class: "small muted", "Natural-product scaffold classes like steroid-like, sugar-like, fused heteroaromatic, or macrocyclic are called out explicitly." }
                    }
                    div { class: "summary-item",
                        h3 { "QLever connectivity" }
                        div { class: "small muted", "LOTUS and PubChem are checked independently; QLever problems are surfaced in the UI instead of being hidden." }
                    }
                    div { class: "summary-item",
                        h3 { "Common motifs" }
                        div { class: "small muted", "The app first finds motifs that recur across the uploaded set, then separates them into scaffold and decoration buckets." }
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
                        div { class: "small muted", "Expect a smiles column (or smile / structure / canonical_smiles)." }
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

                if !file_name_value.is_empty() {
                    p { class: "small muted", "Loaded: {file_name_value}" }
                }

                if !warning_text.is_empty() {
                    div { class: "alert", "{warning_text}" }
                }
            }

            if !endpoints.read().is_empty() {
                section { class: "panel",
                    h2 { "QLever status" }
                    div { class: "summary-grid",
                        for endpoint in endpoints.read().iter() {
                            div { class: "summary-item",
                                h3 { "{endpoint.name}" }
                                div { class: if endpoint.reachable { "chip good" } else { "chip warn" },
                                    if endpoint.reachable { "reachable" } else { "unreachable" }
                                }
                                div { class: "small muted", "{endpoint.endpoint}" }
                                if endpoint.reachable {
                                    div { class: "small", "{endpoint.detail}" }
                                } else {
                                    div { class: "small error", "{endpoint.detail}" }
                                }
                            }
                        }
                    }
                }
            }

            if !motifs.read().is_empty() {
                section { class: "panel",
                    h2 { "Dataset motifs" }
                    div { class: "summary-grid",
                        div { class: "summary-item",
                            h3 { "Scaffolds" }
                            div { class: "chip-list",
                                for motif in motifs.read().iter().filter(|motif| motif.kind == "ring") {
                                    span { class: "chip", "{motif.label} ({motif.count})" }
                                    div { class: "small muted", "{motif.smarts}" }
                                }
                            }
                        }
                        div { class: "summary-item",
                            h3 { "Decorations" }
                            div { class: "chip-list",
                                for motif in motifs.read().iter().filter(|motif| motif.kind == "decoration") {
                                    span { class: "chip alt", "{motif.label} ({motif.count})" }
                                    div { class: "small muted", "{motif.smarts}" }
                                }
                            }
                        }
                        div { class: "summary-item",
                            h3 { "Other motifs" }
                            div { class: "chip-list",
                                for motif in motifs.read().iter().filter(|motif| motif.kind != "ring" && motif.kind != "decoration") {
                                    span { class: "chip alt", "{motif.label} ({motif.count})" }
                                    div { class: "small muted", "{motif.smarts}" }
                                }
                            }
                        }
                    }
                }
            }

            if !rows.read().is_empty() {
                section { class: "panel",
                    h2 { "Per-row motif profile" }
                    div { class: "summary-grid",
                        for row in rows.read().iter() {
                            div { class: "summary-item",
                                h3 { "{row.label}" }
                                div { class: "small muted", "{row.ring_family}" }
                                div { class: "chip-list",
                                    span { class: "chip good", "{row.np_label}" }
                                    if row.evidence_notes.iter().any(|note| note.contains("scaffold-heavy")) {
                                        span { class: "chip", "scaffold-heavy" }
                                    }
                                    if row.evidence_notes.iter().any(|note| note.contains("decoration-heavy")) {
                                        span { class: "chip alt", "decoration-heavy" }
                                    }
                                }
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
                                }
                                div { class: "meta",
                                    strong { "Motifs" }
                                    if row.motifs.is_empty() {
                                        div { class: "muted", "none" }
                                    } else {
                                        div { class: "chip-list",
                                            for motif in row.motifs.iter() {
                                                span { class: "chip alt", "{motif}" }
                                            }
                                        }
                                    }
                                    if !row.evidence_notes.is_empty() {
                                        div { class: "chip-list",
                                            for note in row.evidence_notes.iter().take(3) {
                                                span { class: "chip", "{note}" }
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
                                        div { class: "small muted", "Same connectivity compounds: {row.lotus_compounds.len()}" }
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
                                        div { class: "small muted", "Same connectivity names: {row.pubchem_names.len()}" }
                                    }
                                    if !row.pubchem_taxa.is_empty() {
                                        div { class: "chip-list",
                                            for taxon in row.pubchem_taxa.iter().take(4) {
                                                span { class: "chip", "{taxon}" }
                                            }
                                        }
                                    }
                                }
                                div { class: "meta",
                                    strong { "NP-likeness" }
                                    div { class: "chip-list",
                                        span { class: "chip good", "{format_score(row.np_likeness)}" }
                                        span { class: "chip alt", "{row.np_label}" }
                                    }
                                    div { "{row.ring_family}" }
                                    if !row.evidence_notes.is_empty() {
                                        div { class: "chip-list",
                                            for note in row.evidence_notes.iter().take(3) {
                                                span { class: "chip", "{note}" }
                                            }
                                        }
                                    }
                                }
                                div { class: "result-box",
                                    strong { "Result" }
                                    div { class: "result-grid",
                                        div { class: "result-row",
                                            span { "LOTUS high evidence" }
                                            span { class: "result-badge", if row.lotus_taxa.is_empty() { "✗" } else { "✓" } }
                                        }
                                        div { class: "result-row",
                                            span { "PubChem low evidence" }
                                            span { class: "result-badge", if row.pubchem_cids.is_empty() { "✗" } else { "✓" } }
                                        }
                                        div { class: "result-row",
                                            span { "NP-likeness is positive" }
                                            span { class: "result-badge", if row.np_likeness > 0.0 { "✓" } else { "✗" } }
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

fn format_score(score: f64) -> String {
    format!("{score:+.2}")
}
