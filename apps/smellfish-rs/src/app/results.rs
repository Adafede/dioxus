// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Renderable result-view components for smellfish-rs.
//!
//! Each `#[component]` owns one responsibility so `app.rs` only wires
//! signals and composes them. Formatting helpers live in [`super::formatting`],
//! motif classification in [`super::motif_display`], and CSV export in
//! [`super::csv_export`].
use crate::literature::LITERATURE;
use crate::model::MoleculeRow;
use dioxus::prelude::*;

pub use super::csv_export::download_csv;
pub use super::formatting::{format_score, scaffold_emoji, verdict_color};
pub use super::motif_display::{
    motif_chip_class, motif_display_label, motif_is_natural, motif_is_synthetic,
    motif_is_unclassified, summary_chip_class, summary_display_label, summary_is_natural,
    summary_is_synthetic, summary_is_unclassified,
};

/// Hero banner.
#[component]
pub fn Hero() -> Element {
    rsx! {
        section { class: "hero",
            h1 { "\u{1f41f} Smellfish-rs" }
            p { "A natural-product originality screen for SMILES lists." }
        }
    }
}

/// Motif chip summary panel (only rendered when motifs were detected).
#[component]
pub fn MotifPanel(motifs: Signal<Vec<crate::model::MotifSummary>>) -> Element {
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
                " results \u{00b7} "
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
                div { class: "small muted smiles-display", "Row {row.index} \u{00b7} {row.num_atoms} heavy atoms" }
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
