//! Renderable sub-trees + pure formatting helpers extracted from `app.rs`.
//!
//! Each `#[component]` owns one responsibility so `app.rs` only wires signals
//! and composes them. The helpers below are pure formatting / CSV-download
//! utilities used by [`MoleculeCard`] / [`ResultsView`].
use crate::literature::LITERATURE;
use crate::model::{MoleculeRow, MotifSummary, RdkitMotifHit, normalized_source_class};
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use std::fmt::Write;

/// Hero banner.
#[component]
pub fn Hero() -> Element {
    rsx! {
        section { class: "hero",
            h1 { "🐟 Smellfish-rs" }
            p { "A natural-product originality screen for SMILES lists." }
        }
    }
}

/// Motif chip summary panel (only rendered when motifs were detected).
#[component]
pub fn MotifPanel(motifs: Signal<Vec<MotifSummary>>) -> Element {
    rsx! {
        section { class: "panel",
            h2 { "Motifs" }
            div { class: "motif-groups",
                div { class: "motif-group",
                    h3 { class: "small", "Natural" }
                    div { class: "chip-list",
                        for motif in motifs.read().iter().filter(|m| summary_is_natural(m)).take(12) {
                            span { class: "{summary_chip_class(motif)}", title: "Natural-product motif", "{summary_display_label(motif)} ({motif.count})" }
                        }
                    }
                }
                div { class: "motif-group",
                    h3 { class: "small", "Synthetic-leaning" }
                    div { class: "chip-list",
                        for motif in motifs.read().iter().filter(|m| summary_is_synthetic(m)).take(12) {
                            span { class: "{summary_chip_class(motif)}", title: "Synthetic-leaning functional group", "{summary_display_label(motif)} ({motif.count})" }
                        }
                    }
                }
                div { class: "motif-group",
                    h3 { class: "small", "Unclassified" }
                    div { class: "chip-list",
                        for motif in motifs.read().iter().filter(|m| summary_is_unclassified(m)).take(12) {
                            span { class: "{summary_chip_class(motif)}", title: "Unclassified functional group", "{summary_display_label(motif)} ({motif.count})" }
                        }
                    }
                }
            }
        }
    }
}

/// Header + card list (rendered when `rows` is non-empty).
#[component]
pub fn ResultsView(rows: Signal<Vec<MoleculeRow>>) -> Element {
    rsx! {
        section { class: "panel",
            div { class: "small",
                strong { "{rows.read().len()}" }
                " results · "
                a { href: "#", onclick: move |_| download_csv(&rows.read()), "Download CSV" }
            }
        }
        section { class: "cards",
            for row in rows.read().iter() {
                MoleculeCard { row: row.clone() }
            }
        }
    }
}

/// A single molecule result card.
#[component]
pub fn MoleculeCard(row: MoleculeRow) -> Element {
    rsx! {
        article { class: "card",
            div { class: "card-head",
                div {
                    strong { "{row.label}" }
                }
                div { class: "small muted smiles-display", "Row {row.index} · {row.num_atoms} heavy atoms" }
                div { class: "small-muted smiles-small", "SMILES: {row.smiles}" }
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
                                    rel: "noopener noreferrer",
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
                                    rel: "noopener noreferrer",
                                    "CID {cid}"
                                }
                            }
                        }
                    }
                }
                if !row.motifs.is_empty() {
                    div { class: "meta",
                        strong { "Functional groups" }
                        div { class: "motif-groups",
                            div { class: "motif-group",
                                h4 { class: "small", "Natural" }
                                div { class: "chip-list",
                                    for motif in row.motif_hits.iter().filter(|m| motif_is_natural(m)) {
                                        span { class: "{motif_chip_class(motif)}", title: "Natural-product motif", "{motif_display_label(motif)}" }
                                    }
                                }
                            }
                            div { class: "motif-group",
                                h4 { class: "small", "Synthetic-leaning" }
                                div { class: "chip-list",
                                    for motif in row.motif_hits.iter().filter(|m| motif_is_synthetic(m)) {
                                        span { class: "{motif_chip_class(motif)}", title: "Synthetic-leaning functional group", "{motif_display_label(motif)}" }
                                    }
                                }
                            }
                            div { class: "motif-group",
                                h4 { class: "small", "Unclassified" }
                                div { class: "chip-list",
                                    for motif in row.motif_hits.iter().filter(|m| motif_is_unclassified(m)) {
                                        span { class: "{motif_chip_class(motif)}", title: "Unclassified functional group", "{motif_display_label(motif)}" }
                                    }
                                }
                            }
                        }
                    }
                }
                if !row.substituents_counts.is_empty() {
                    div { class: "meta",
                        strong { "Ertl substituents" }
                        div { class: "chip-list",
                            for (substituent, count) in row.substituents_counts.iter().take(6) {
                                span { class: "chip alt", title: "Occurrence count: {count}", "{substituent}" }
                            }
                            if row.substituents_counts.len() > 6 {
                                span { class: "chip alt", "+{row.substituents_counts.len() - 6} more" }
                            }
                        }
                    }
                }
                if !row.lotus_scaffolds.is_empty() {
                    div { class: "meta",
                        strong { "LOTUS 1% scaffolds" }
                        div { class: "chip-list",
                            for scaffold in row.lotus_scaffolds.iter() {
                                span { class: "chip lotus", title: "LOTUS scaffold (Rutz et al.) present above 1% frequency", "{scaffold}" }
                            }
                        }
                    }
                }
            }
            if !row.evidence_notes.is_empty() {
                div { class: "evidence",
                    details { open: true,
                        summary { class: "small", "Evidence" }
                        ul { class: "evidence-list",
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

/// Static footer with citation / data / code / references / program links.
#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "app-footer",
            div { class: "footer-line",
                div { class: "footer-row",
                    span { class: "footer-label", "Citation" }
                    ul { class: "footer-links", role: "list",
                        li {
                            a { class: "footer-link red", href: "https://doi.org/10.7554/eLife.70780", target: "_blank", rel: "noopener noreferrer", "LOTUS Article" }
                        }
                    }
                }
            }
            div { class: "footer-line",
                div { class: "footer-row",
                    span { class: "footer-label", "Data" }
                    ul { class: "footer-links", role: "list",
                        li { a { class: "footer-link green", href: "https://www.wikidata.org/wiki/Q104225190", target: "_blank", rel: "noopener noreferrer", "LOTUS Initiative" } }
                        li { a { class: "footer-link green", href: "https://www.wikidata.org/", target: "_blank", rel: "noopener noreferrer", "Wikidata" } }
                        li { a { class: "footer-link green", href: "https://pubchem.ncbi.nlm.nih.gov/", target: "_blank", rel: "noopener noreferrer", "PubChem" } }
                    }
                }
                div { class: "footer-row",
                    span { class: "footer-label", "Code" }
                    ul { class: "footer-links", role: "list",
                        li { a { class: "footer-link blue", href: "https://github.com/Adafede/dioxus/tree/main/apps/smellfish-rs", target: "_blank", rel: "noopener noreferrer", "smellfish-rs" } }
                    }
                }
            }
            div { class: "footer-line",
                div { class: "footer-row",
                    span { class: "footer-label", "References" }
                    ul { class: "footer-links", role: "list",
                        for paper in LITERATURE {
                            li {
                                a { class: "footer-link purple", href: "https://doi.org/{paper.doi}", target: "_blank", rel: "noopener noreferrer", title: "{paper.note}", "{paper.title}" }
                            }
                        }
                    }
                }
            }
            div { class: "footer-line",
                div { class: "footer-row",
                    span { class: "footer-label", "Programs" }
                    ul { class: "footer-links", role: "list",
                        li { a { class: "footer-link blue", href: "https://qlever.dev/wikidata", target: "_blank", rel: "noopener noreferrer", "QLever" } }
                        li { a { class: "footer-link blue", href: "https://www.rdkitjs.com", target: "_blank", rel: "noopener noreferrer", "RDKit.js" } }
                    }
                }
                div { class: "footer-row",
                    span { class: "footer-label", "License" }
                    ul { class: "footer-links", role: "list",
                        li { a { class: "footer-link blue", href: "https://www.gnu.org/licenses/agpl-3.0.html", target: "_blank", rel: "noopener noreferrer", "AGPL-3.0" } }
                    }
                }
            }
        }
    }
}

fn format_score(score: f64) -> String {
    format!("{score:+.2}")
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

fn summary_is_natural(motif: &MotifSummary) -> bool {
    normalized_source_class(&motif.source_class) == "natural"
}

fn summary_is_synthetic(motif: &MotifSummary) -> bool {
    normalized_source_class(&motif.source_class) == "synthetic"
}

fn summary_is_unclassified(motif: &MotifSummary) -> bool {
    normalized_source_class(&motif.source_class) == "unknown"
}

fn summary_chip_class(motif: &MotifSummary) -> &'static str {
    if summary_is_natural(motif) {
        "chip chip-np"
    } else {
        "chip alt"
    }
}

fn summary_display_label(motif: &MotifSummary) -> String {
    if summary_is_natural(motif) {
        let kingdoms = if motif.kingdoms.is_empty() {
            motif.kingdom.clone()
        } else if motif.kingdoms.len() == 1 {
            motif.kingdoms[0].clone()
        } else {
            motif.kingdoms.join(" + ")
        };
        format!("{} · {}", kingdoms, motif.label)
    } else if summary_is_synthetic(motif) {
        format!("synthetic · {}", motif.label)
    } else {
        format!("unclassified · {}", motif.label)
    }
}

fn motif_is_natural(motif: &RdkitMotifHit) -> bool {
    normalized_source_class(&motif.source_class) == "natural"
}

fn motif_is_synthetic(motif: &RdkitMotifHit) -> bool {
    normalized_source_class(&motif.source_class) == "synthetic"
}

fn motif_is_unclassified(motif: &RdkitMotifHit) -> bool {
    normalized_source_class(&motif.source_class) == "unknown"
}

fn motif_chip_class(motif: &RdkitMotifHit) -> &'static str {
    if motif_is_natural(motif) {
        "chip chip-np"
    } else {
        "chip alt"
    }
}

fn motif_display_label(motif: &RdkitMotifHit) -> String {
    if motif_is_natural(motif) {
        let kingdoms = if motif.kingdoms.is_empty() {
            motif.kingdom.clone()
        } else if motif.kingdoms.len() == 1 {
            motif.kingdoms[0].clone()
        } else {
            motif.kingdoms.join(" + ")
        };
        format!("{} · {}", kingdoms, motif.label)
    } else if motif_is_synthetic(motif) {
        format!("synthetic · {}", motif.label)
    } else {
        format!("unclassified · {}", motif.label)
    }
}

/// Escape a field for CSV output.
#[cfg(target_arch = "wasm32")]
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

/// Build a CSV string from molecule rows.
#[cfg(target_arch = "wasm32")]
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
        // Convert substituents counts to "label(count);" format
        let substituents: String = r
            .substituents_counts
            .iter()
            .map(|(label, count)| format!("{}({})", label, count))
            .collect::<Vec<_>>()
            .join(";");
        let locus = r
            .lotus_compounds
            .iter()
            .chain(r.pubchem_cids.iter())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(";");
        let _ = writeln!(
            csv,
            "{},{},{:.3},{},{}%,{},{},{},{},{}",
            escape_csv(&r.label),
            escape_csv(&r.smiles),
            r.np_likeness,
            r.np_label,
            (r.np_confidence * 100.0).round(),
            escape_csv(&r.ring_family),
            escape_csv(&substituents),
            escape_csv(&locus),
            crate::evidence::category(&r.verdict),
            escape_csv(&checks),
        );
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
const fn download_csv(_rows: &[MoleculeRow]) {}
