// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Memoized result card showing CX-SMILES output, scaffold SVG, and candidates.

use std::sync::Arc;

use chem_depict::{render_smiles_svg, render_smiles_svg_highlighted};
use cxsmiles_core::{ConstructKind, CxResult};
use dioxus::prelude::*;
use ui::{Card, ColorScheme, NoticeBar, NoticeTone, Radius, StyleBuilder, Typography};

use super::clipboard::CopyCell;

#[derive(Clone, PartialEq, Eq)]
struct DepictedResult {
    scaffold_svg: String,
    candidates: Vec<(String, String)>,
}

fn depict_result(res: &CxResult) -> DepictedResult {
    let highlight_atoms: Vec<usize> = res
        .floating
        .iter()
        .flat_map(|f| f.equiv.iter().copied())
        .chain(res.repeat_unit.iter().flat_map(|u| u.atoms.iter().copied()))
        .collect();
    let scaffold_svg = render_smiles_svg_highlighted(&res.scaffold_smiles, &highlight_atoms);
    let candidates = res
        .enumerated
        .iter()
        .map(|smi| (smi.clone(), render_smiles_svg(smi)))
        .collect();
    DepictedResult {
        scaffold_svg,
        candidates,
    }
}

fn card_surface(flex: bool) -> String {
    let light = ColorScheme::LIGHT;
    let b = StyleBuilder::new()
        .background_color(light.surface)
        .border(&format!("1px solid {}", light.border))
        .border_radius(Radius::MD)
        .padding("0.5rem")
        .min_height("120px");
    if flex {
        b.display("flex")
            .flex_direction("column")
            .gap("0.5rem")
            .build()
    } else {
        b.build()
    }
}

fn field_label(with_margin: bool) -> String {
    let b = StyleBuilder::new()
        .font_size(Typography::LABEL)
        .font_weight("600")
        .color(ColorScheme::LIGHT.text2);
    if with_margin {
        b.property("margin-bottom", "0.25rem").build()
    } else {
        b.build()
    }
}

fn floating_badges(res: &CxResult) -> Element {
    if res.floating.is_empty() && res.repeat_unit.is_none() {
        return rsx! {};
    }
    let light = ColorScheme::LIGHT;
    let badge_style = StyleBuilder::new()
        .font_family("ui-monospace, monospace")
        .font_size(Typography::LABEL)
        .padding("2px 0.5rem")
        .border_radius("999px")
        .background_color(light.bg)
        .border(&format!("1px solid {}", light.border))
        .color(light.text2)
        .build();
    rsx! {
        div { style: StyleBuilder::new().display("flex").flex_wrap("wrap").gap("0.4rem").property("margin-top", "0.4rem").build(),
            for part in &res.floating {
                {
                    let target_desc = if part.split {
                        "attached to parent group".to_string()
                    } else {
                        let idx = part.equiv.iter().map(ToString::to_string).collect::<Vec<_>>().join(".");
                        format!("atoms {idx}")
                    };
                    rsx! {
                        span { style: "{badge_style}", "{part.fragment_smiles} → {target_desc}" }
                    }
                }
            }
            if let Some(unit) = &res.repeat_unit {
                {
                    let atoms = unit.atoms.iter().map(ToString::to_string).collect::<Vec<_>>().join(",");
                    rsx! {
                        span { style: "{badge_style}", "Sg:n → atoms {atoms} ({unit.min}–{unit.max} repeats)" }
                    }
                }
            }
        }
    }
}

/// Renders the CX-SMILES output card when a result is present, memoizing SVGs.
#[component]
pub(super) fn ResultsPanel(result: Signal<Option<CxResult>>) -> Element {
    let depicted = use_memo(move || result.read().as_ref().map(depict_result));
    let res_guard = result.read();
    let Some(res) = res_guard.as_ref() else {
        return rsx! {};
    };
    let dep_guard = depicted.read();
    let Some(dep) = dep_guard.as_ref() else {
        return rsx! {};
    };
    let light = ColorScheme::LIGHT;
    let frac = res.confidence.coverage.fraction() * 100.0;
    let tone = if res.confidence.clean {
        NoticeTone::Success
    } else {
        NoticeTone::Warning
    };
    let construct_label = match res.construct {
        ConstructKind::Positional => "Positional isomer (m: blocks)",
        ConstructKind::Repeating => "Variable-length repeat (Sg:n: block)",
        ConstructKind::BestEffort => "Best-effort (no clean shared scaffold)",
    };
    let n_enum = res.enumerated.len();
    let all_enumerated = res.enumerated.join("\n");
    let cx_box_style = StyleBuilder::new()
        .padding("0.6rem 0.7rem")
        .background_color(light.bg)
        .border_radius(Radius::MD)
        .border(&format!("1px solid {}", light.border))
        .font_family("ui-monospace, monospace")
        .font_size(Typography::UI)
        .property("word-break", "break-all")
        .color(light.text)
        .build();
    let candidate_cell_style = StyleBuilder::new()
        .width("100%")
        .height("90px")
        .display("grid")
        .property("place-items", "center")
        .background_color(light.bg)
        .border(&format!("1px solid {}", light.border))
        .border_radius(Radius::MD)
        .property("overflow", "hidden")
        .build();

    rsx! {
        Card {
            title: "Generated CX-SMILES".to_string(),
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.75rem").build(),
                div { style: cx_box_style, "{res.cx_smiles}" }
                div { style: StyleBuilder::new().display("flex").align_items("center").gap("0.5rem").flex_wrap("wrap").build(),
                    CopyCell { text: Arc::<str>::from(res.cx_smiles.as_str()) }
                    span { style: StyleBuilder::new().font_size(Typography::UI).color(light.text3).build(), "{construct_label}" }
                }
                NoticeBar {
                    label: format!("Round-trip recall: {frac:.0}% ({}/{})", res.confidence.coverage.covered, res.confidence.coverage.total),
                    tone,
                }
                div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "1fr 1fr").gap("0.75rem").build(),
                    div { style: card_surface(false),
                        div { style: field_label(true), "Shared scaffold" }
                        div { style: StyleBuilder::new().width("100%").display("flex").justify_content("center").align_items("center").build(),
                            div { dangerous_inner_html: "{dep.scaffold_svg}" }
                        }
                        { floating_badges(res) }
                    }
                    if !dep.candidates.is_empty() {
                        div { style: card_surface(true),
                            div { style: StyleBuilder::new().display("flex").justify_content("space-between").align_items("center").build(),
                                div { style: field_label(false), "Enumerated candidates ({n_enum})" }
                                CopyCell { text: Arc::<str>::from(all_enumerated.as_str()) }
                            }
                            div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fill, minmax(110px, 1fr))").gap("0.4rem").width("100%").property("overflow-y", "auto").build(),
                                for (smi, svg) in &dep.candidates {
                                    div {
                                        title: "{smi}",
                                        style: "{candidate_cell_style}",
                                        div { dangerous_inner_html: "{svg}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
