// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Top-level result summary components (`Hero`, `MotifPanel`, `ResultsView`).

#![expect(
    clippy::empty_enums,
    reason = "Dioxus #[component] macro generates empty Props enums for zero-arg components"
)]

use dioxus::prelude::*;
use smellfish_core::csv::format_csv_rows;
use smellfish_core::model::{MoleculeRow, MotifSummary, SourceClass};

use super::molecule_card::MoleculeCard;
use super::motif_display::{
    chip_class_for, display_label, is_natural_source, is_synthetic_source, is_unclassified_source,
};

pub(super) type MotifPredicate = fn(SourceClass) -> bool;

pub(super) const MOTIF_GROUPS: [(&str, &str, MotifPredicate); 3] = [
    ("Natural", "Natural-product motif", is_natural_source),
    (
        "Synthetic-leaning",
        "Synthetic-leaning functional group",
        is_synthetic_source,
    ),
    (
        "Unclassified",
        "Unclassified functional group",
        is_unclassified_source,
    ),
];

/// Hero banner.
#[component]
pub(super) fn Hero() -> Element {
    rsx! {
        section { class: "hero",
            h1 { "\u{1f41f} Smellfish-rs" }
            p { "A natural-product originality screen for SMILES lists." }
        }
    }
}

/// Motif chip summary panel rendered when structural motifs are present.
#[component]
pub(super) fn MotifPanel(motifs: Signal<Vec<MotifSummary>>) -> Element {
    let motifs_ref = motifs.read();
    rsx! {
        section { class: "panel",
            h2 { "Motifs" }
            div { class: "motif-groups",
                for (heading, title, matches_group) in MOTIF_GROUPS {
                    div { class: "motif-group",
                        h3 { class: "small", "{heading}" }
                        div { class: "chip-list",
                            for motif in motifs_ref.iter().filter(|m| matches_group(m.source_class)).take(12) {
                                span {
                                    class: "{chip_class_for(motif.source_class)}",
                                    title: "{title}",
                                    "{display_label(motif.source_class, &motif.label, &motif.kingdom, &motif.kingdoms)} ({motif.count})"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Header + card grid rendered once `rows` is non-empty.
#[component]
pub(super) fn ResultsView(rows: Signal<Vec<MoleculeRow>>) -> Element {
    let rows_ref = rows.read();
    rsx! {
        section { class: "panel",
            div { class: "small",
                strong { "{rows_ref.len()}" }
                " results \u{00b7} "
                a {
                    href: "#",
                    onclick: move |evt| {
                        evt.prevent_default();
                        let csv = format_csv_rows(&rows.read());
                        drop(upload::download_text(&csv, "smellfish-results.csv"));
                    },
                    "Download CSV"
                }
            }
        }
        section { class: "cards",
            for row in rows_ref.iter() {
                MoleculeCard { key: "{row.index}", row: row.clone() }
            }
        }
    }
}
