// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::models::DataStats;
use dioxus::prelude::*;

#[component]
pub fn StatsGrid(stats: DataStats) -> Element {
    rsx! {
        div { class: "grid three",
            StatCard { value: format_number(stats.n_compounds), label: "Unique Compounds" }
            StatCard { value: format_number(stats.n_taxa), label: "Unique Taxa" }
            StatCard {
                value: format_number(stats.n_compound_taxon_pairs),
                label: "Compound-Taxon Pairs"
            }
            StatCard { value: format_number(stats.n_taxa_with_ncbi), label: "Taxa with NCBI ID" }
            StatCard {
                value: format_number(stats.n_taxon_parent_pairs),
                label: "Taxon-Parent Pairs"
            }
            StatCard { value: format_number(stats.n_taxa_with_names), label: "Taxa with Names" }
            StatCard {
                value: format_number(stats.n_compound_parent_pairs),
                label: "Compound-Parent Pairs"
            }
            StatCard {
                value: format_number(stats.n_compounds_with_labels),
                label: "Compounds with Labels"
            }
        }
    }
}

#[component]
fn StatCard(value: String, label: &'static str) -> Element {
    rsx! {
        div { class: "card",
            div { class: "stat-value", "{value}" }
            div { class: "stat-label", "{label}" }
        }
    }
}

fn format_number(value: usize) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}
