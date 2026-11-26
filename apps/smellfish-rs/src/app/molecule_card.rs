// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Per-molecule assessment card component.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus #[component] macro generates empty Props builder helper enum"
)]

use dioxus::prelude::*;
use smellfish_core::model::MoleculeRow;

use super::motif_display::{chip_class_for, display_label};
use super::results::MOTIF_GROUPS;

/// A single molecule result card displaying depiction, NP score, checks, DB hits, and verdict.
#[component]
pub(crate) fn MoleculeCard(row: MoleculeRow) -> Element {
    rsx! {
        article { class: "card",
            div { class: "card-head",
                div { strong { "{row.label}" } }
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
                        if let Some(np) = row.np_score {
                            span {
                                class: if np.score >= 1.0 { "chip good" } else if np.score >= 0.0 { "chip warn" } else { "chip fail" },
                                "{np.score:+.2}"
                            }
                        } else {
                            span { class: "chip warn", "model unloaded" }
                        }
                        span { class: "chip alt", "{row.np_label}" }
                    }
                    div { class: "small", "{row.ring_family}" }
                    if row.motif_context.is_detected() {
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

                if !row.lotus_compounds.is_empty() || !row.lotus_taxa.is_empty() {
                    div { class: "meta small",
                        strong { class: "blue", "LOTUS" }
                        div { class: "chip-list",
                            for compound in row.lotus_compounds.iter() {
                                a {
                                    class: if compound.has_taxa { "cid-link green" } else { "cid-link red" },
                                    href: "https://www.wikidata.org/wiki/{compound.qid}",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    "{compound.qid}"
                                }
                            }
                            for taxon in row.lotus_taxa.iter() {
                                span { class: "cid-link green", "{taxon}" }
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
                            for (heading, title, matches_group) in MOTIF_GROUPS {
                                div { class: "motif-group",
                                    h4 { class: "small", "{heading}" }
                                    div { class: "chip-list",
                                        for motif in row.motif_hits.iter().filter(|m| matches_group(m.source_class)) {
                                            span {
                                                class: "{chip_class_for(motif.source_class)}",
                                                title: "{title}",
                                                "{display_label(motif.source_class, &motif.label, &motif.kingdom, &motif.kingdoms)}"
                                            }
                                        }
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

            div { class: "verdict {row.verdict.category.css_class()}", "{row.verdict}" }
        }
    }
}
