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
                p { "A natural-product originality screen for SMILES lists." }
                p { class: "small muted", "Real Ertl NP-likeness score (Ertl et al., J. Chem. Inf. Model. 2008, DOI 10.1021/ci700286x) + a chemist's checklist of structural red flags." }
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
                        div { class: "small muted", "Expect a smiles column." }
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

            if !motifs.read().is_empty() {
                section { class: "panel",
                    h2 { "Dataset motifs" }
                    div { class: "chip-list",
                        for motif in motifs.read().iter().filter(|m| m.kind == "ring").take(12) {
                            span { class: "chip", "{motif.label} ({motif.count})" }
                        }
                        for motif in motifs.read().iter().filter(|m| m.kind == "decoration").take(12) {
                            span { class: "chip alt", "{motif.label} ({motif.count})" }
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
                                    div { class: "small muted", "Row {row.index} · {row.num_atoms} heavy atoms" }
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
                                        div { class: "small muted", "No structure." }
                                    }
                                }

                                div { class: "meta",
                                    strong { "Ertl NP-likeness" }
                                    div { class: "chip-list",
                                        if row.np_score_available {
                                            span { class: "chip good", "{format_score(row.np_likeness)}" }
                                        } else {
                                            span { class: "chip warn", "model unloaded" }
                                        }
                                        span { class: "chip alt", "{row.np_label}" }
                                        if row.np_score_available {
                                            span { class: "chip", "conf {format_confidence(row.np_confidence)}" }
                                        }
                                    }
                                    div { class: "small", "{row.ring_family}" }
                                    if !row.motif_context.is_empty() && row.motif_context != "no motif signal" {
                                        div { class: "small muted", "{row.motif_context}" }
                                    }
                                }

                                div { class: "meta",
                                    strong { "Chemist's checklist" }
                                    if row.chemist_checks.is_empty() {
                                        div { class: "small muted", "No checks available." }
                                    } else {
                                        div { class: "checklist",
                                            for check in row.chemist_checks.iter() {
                                                div { class: "check-row",
                                                    span { class: "check-status {check.status}", "{check.name}" }
                                                    span { class: "small muted", "{check.detail}" }
                                                }
                                            }
                                        }
                                    }
                                }

                                if !row.motifs.is_empty() {
                                    div { class: "meta",
                                        strong { "Motifs" }
                                        div { class: "chip-list",
                                            for motif in row.motifs.iter() {
                                                span { class: "chip alt", "{motif}" }
                                            }
                                        }
                                    }
                                }

                                if !row.lotus_taxa.is_empty() || !row.pubchem_cids.is_empty() {
                                    div { class: "meta",
                                        strong { "Database evidence" }
                                        div { class: "chip-list",
                                            if !row.lotus_taxa.is_empty() {
                                                span { class: "chip good", "LOTUS" }
                                            }
                                            if !row.pubchem_cids.is_empty() {
                                                span { class: "chip alt", "PubChem" }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "verdict {verdict_color(&row.verdict)}", "{row.verdict}" }
                        }
                    }
                }
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
        }
    }
}

fn format_score(score: f64) -> String {
    format!("{score:+.2}")
}

fn format_confidence(conf: f64) -> String {
    format!("{:.0}%", conf * 100.0)
}

/// CSS-safe verdict color class based on the verdict text.
fn verdict_color(verdict: &str) -> &'static str {
    let l = verdict.to_ascii_lowercase();
    if l.contains("smells fishy") {
        "verdict-fishy"
    } else if l.contains("looks legitimate")
        || l.contains("strong natural")
        || l.contains("lotus-backed")
    {
        "verdict-likely"
    } else if l.contains("not loaded") || l.contains("warning") || l.contains("⚠") {
        "verdict-caution"
    } else {
        "verdict-neutral"
    }
}
