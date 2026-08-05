use crate::evidence::{is_known_np_motif, is_scaffold_motif};
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
    let mut file_name = use_signal(String::new);
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
                p { class: "small muted",
                    "Ertl NP-likeness score (Ertl et al., J. Chem. Inf. Model. 2008) + checklist of structural flags."
                }
                p { class: "small",
                    a { class: "footer-link blue", href: "https://doi.org/10.1021/ci700286x", target: "_blank", rel: "noreferrer", "DOI 10.1021/ci700286x" }
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

                p { class: "status", role: "status", aria_live: "polite", aria_atomic: "true",
                    if *busy.read() {
                        span { class: "spinner" }
                    }
                    "{status}"
                }

                if !file_name_value.is_empty() {
                    p { class: "small muted", "Loaded: {file_name_value}" }
                }

                if !warning_text.is_empty() {
                    div { class: "alert", "{warning_text}" }
                }

                {
                    #[cfg(target_arch = "wasm32")]
                    {
                        rsx! { }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        rsx! { }
                    }
                }
            }

            if !motifs.read().is_empty() {
                section { class: "panel",
                    h2 { "Dataset motifs" }
                    div { class: "chip-list",
                        for motif in motifs.read().iter().filter(|m| m.kind == "ring").take(12) {
                            span { class: "{dataset_motif_class(&motif.label)}", "{motif.label} ({motif.count})" }
                        }
                        for motif in motifs.read().iter().filter(|m| m.kind == "decoration").take(12) {
                            span { class: "chip alt", "{motif.label} ({motif.count})" }
                        }
                    }
                }
            }

            if !rows.read().is_empty() {
                section { class: "panel",
                    div { class: "small",
                        strong { "{rows.read().len()}" }
                        " results · "
                        a { href: "#", onclick: move |_| download_csv(&rows.read()), "Download CSV" }
                    }
                }
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
                                    div { class: "small", "{scaffold_emoji(&row.ring_family)} {row.ring_family}" }
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

                                if !row.lotus_compounds.is_empty() {
                                    div { class: "meta small",
                                        strong { class: "blue", "LOTUS" }
                                        div { class: "chip-list",
                                            for qid in row.lotus_compounds.iter() {
                                                a {
                                                    class: if row.lotus_compounds_with_taxa.contains(qid) {
                                                        "cid-link green"
                                                    } else {
                                                        "cid-link red"
                                                    },
                                                    href: "https://www.wikidata.org/wiki/{qid}",
                                                    target: "_blank",
                                                    rel: "noreferrer",
                                                    "{qid}"
                                                }
                                            }
                                        }
                                    }
                                }
                                if !row.pubchem_cids.is_empty() {
                                    div { class: "meta small",
                                        strong { class: "blue", "PubChem" }
                                        div { class: "chip-list",
                                            for cid in row.pubchem_cids.iter() {
                                                a { class: "cid-link",
                                                    href: "https://pubchem.ncbi.nlm.nih.gov/compound/{cid}",
                                                    target: "_blank",
                                                    rel: "noreferrer",
                                                    "CID {cid}"
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
                                                span { class: "{row_motif_class(motif)}",
                                                    "{motif_emoji(motif)} {motif}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if !row.evidence_notes.is_empty() {
                                div { class: "evidence",
                                    details { open: true,
                                        summary { class: "small", "Evidence" }
                                        ul { style: "margin: 6px 0; padding-left: 20px; font-size: 0.8rem;",
                                            for note in row.evidence_notes.iter() {
                                                li { "{note}" }
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

            footer { class: "app-footer",
                div { class: "footer-line",
                    div { class: "footer-row",
                        span { class: "footer-label", "Citation" }
                        ul { class: "footer-links", role: "list",
                            li {
                                a { class: "footer-link red", href: "https://doi.org/10.7554/eLife.70780", target: "_blank", rel: "noreferrer", "LOTUS paper (eLife)" }
                            }
                        }
                    }
                }
                div { class: "footer-line",
                    div { class: "footer-row",
                        span { class: "footer-label", "Data" }
                        ul { class: "footer-links", role: "list",
                            li { a { class: "footer-link green", href: "https://www.wikidata.org/wiki/Q104225190", target: "_blank", rel: "noreferrer", "LOTUS Initiative" } }
                            li { a { class: "footer-link green", href: "https://www.wikidata.org/", target: "_blank", rel: "noreferrer", "Wikidata" } }
                            li { a { class: "footer-link green", href: "https://pubchem.ncbi.nlm.nih.gov/", target: "_blank", rel: "noreferrer", "PubChem" } }
                        }
                    }
                    div { class: "footer-row",
                        span { class: "footer-label", "Code" }
                        ul { class: "footer-links", role: "list",
                            li { a { class: "footer-link blue", href: "https://github.com/Adafede/dioxus/tree/main/apps/smellfish-rs", target: "_blank", rel: "noreferrer", "smellfish-rs" } }
                        }
                    }
                }
                div { class: "footer-line",
                    div { class: "footer-row",
                        span { class: "footer-label", "References" }
                        ul { class: "footer-links", role: "list",
                            li {
                                button { class: "ertl-work-btn",
                                    onclick: move |_| {
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            for paper in LITERATURE {
                                                let _ = web_sys::window()
                                                    .unwrap()
                                                    .open_with_url_and_target(
                                                        &format!("https://doi.org/{}", paper.doi),
                                                        "_blank"
                                                    );
                                            }
                                        }
                                        #[cfg(not(target_arch = "wasm32"))]
                                        let _ = LITERATURE;
                                    },
                                    "Ertl work"
                                }
                            }
                        }
                    }
                }
                div { class: "footer-line",
                    div { class: "footer-row",
                        span { class: "footer-label", "Programs" }
                        ul { class: "footer-links", role: "list",
                            li { a { class: "footer-link blue", href: "https://qlever.dev/wikidata", target: "_blank", rel: "noreferrer", "QLever" } }
                            li { a { class: "footer-link blue", href: "https://www.rdkitjs.com", target: "_blank", rel: "noreferrer", "RDKit.js" } }
                        }
                    }
                    div { class: "footer-row",
                        span { class: "footer-label", "License" }
                        ul { class: "footer-links", role: "list",
                            li { a { class: "footer-link blue", href: "https://www.gnu.org/licenses/agpl-3.0.html", target: "_blank", rel: "noreferrer", "AGPL-3.0" } }
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
    if l.contains("smells fishy") || l.contains("highly synthetic") {
        "verdict-fishy"
    } else if l.contains("likely") || l.contains("strong natural") || l.contains("lotus") {
        "verdict-likely"
    } else if l.contains("citation needed") {
        "verdict-skeptical"
    } else if l.contains("weak np signals") || l.contains("ertl") && l.contains("−1") {
        "verdict-caution"
    } else {
        "verdict-neutral"
    }
}

/// Emoji prefix for the scaffold family — 🌿 for NP-typical scaffolds,
/// ⚠ for polyaromatic (synthetic-typical).
fn scaffold_emoji(family: &str) -> &'static str {
    let l = family.to_ascii_lowercase();
    if l.contains("polyaromatic") {
        "⚠"
    } else if l.contains("polycyclic")
        || l.contains("steroid")
        || l.contains("sugar")
        || l.contains("macrolide")
        || l.contains("flavonoid")
        || l.contains("heteroaromatic")
    {
        "🌿"
    } else {
        "—"
    }
}

/// Chip CSS class for dataset-level motif labels — NP-known scaffolds get
/// a green highlight, other scaffolds get blue, decorations are neutral.
fn dataset_motif_class(label: &str) -> String {
    if is_known_np_motif(label) {
        "chip chip-np".to_string()
    } else if is_scaffold_motif(label) {
        "chip chip-scaffold".to_string()
    } else {
        "chip alt".to_string()
    }
}

/// Chip CSS class for per-molecule scaffold vs decoration highlights.
fn row_motif_class(label: &str) -> String {
    if is_known_np_motif(label) {
        "chip chip-np".to_string()
    } else if is_scaffold_motif(label) {
        "chip chip-scaffold".to_string()
    } else {
        "chip".to_string()
    }
}

fn motif_emoji(label: &str) -> String {
    if is_known_np_motif(label) {
        "🌿".to_string()
    } else {
        String::new()
    }
}

/// Escape a field for CSV output.
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Build a CSV string from molecule rows.
fn build_csv(rows: &[MoleculeRow]) -> String {
    let mut csv = String::from(
        "label,smiles,np_score,np_label,np_confidence,ring_family,substituents,locus,verdict_category,chemist_checks\n",
    );
    for r in rows {
        let checks = r
            .chemist_checks
            .iter()
            .map(|c| format!("{}:{}", c.name, c.status))
            .collect::<Vec<_>>()
            .join(";");
        let substituents = r.substituents.join(";");
        let locus = r
            .lotus_compounds
            .iter()
            .chain(r.pubchem_cids.iter())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(";");
        csv.push_str(&format!(
            "{},{},{:.3},{},{}%,{},{},{},{},{}\n",
            escape_csv(&r.label),
            escape_csv(&r.smiles),
            r.np_likeness,
            r.np_label,
            (r.np_confidence * 100.0).round(),
            escape_csv(&r.ring_family),
            escape_csv(&substituents),
            escape_csv(&locus),
            crate::evidence::verdict_category(&r.verdict),
            escape_csv(&checks),
        ));
    }
    csv
}

/// Build a CSV string from molecule rows and trigger a download via a
/// data: URI injected through `eval`.
#[cfg(target_arch = "wasm32")]
fn download_csv(rows: &[MoleculeRow]) {
    let csv = build_csv(rows);
    let url = format!("data:text/csv;charset=utf-8,{}", urlencoding::encode(&csv));
    let script = format!(
        r#"(function(){{var a=document.createElement('a');a.href='{}';a.download='smellfish-results.csv';a.click();}})()"#,
        url
    );
    let _ = js_sys::eval(&script);
}

#[cfg(not(target_arch = "wasm32"))]
fn download_csv(_rows: &[MoleculeRow]) {}
