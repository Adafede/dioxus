// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Memoized result card showing CX-SMILES output, scaffold SVG, and candidates.

use std::sync::Arc;

use chem_depict::{render_smiles_svg, render_smiles_svg_highlighted};
use cxsmiles_core::{ConstructKind, CxResult};
use dioxus::prelude::*;
use ui::{Card, ColorScheme, NoticeBar, NoticeTone, Radius, StyleBuilder, Typography};

use super::clipboard::CopyCell;

const CANDIDATE_PAGE_SIZE: usize = 60;

fn depict_scaffold(res: &CxResult) -> String {
    let highlight_atoms: Vec<usize> = res
        .floating
        .iter()
        .flat_map(|f| f.equiv.iter().copied())
        .chain(res.repeat_unit.iter().flat_map(|u| u.atoms.iter().copied()))
        .collect();
    render_smiles_svg_highlighted(&res.scaffold_smiles, &highlight_atoms)
}

fn card_surface(flex: bool) -> String {
    let light = ColorScheme::LIGHT;
    let b = StyleBuilder::new()
        .background_color(light.surface)
        .property_fmt("border", format_args!("1px solid {}", light.border))
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
        .property_fmt("border", format_args!("1px solid {}", light.border))
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

fn recall_notice(res: &CxResult) -> (NoticeTone, String) {
    let frac = res.coverage.fraction() * 100.0;
    if res.coverage.is_clean() {
        (
            NoticeTone::Success,
            format!(
                "Round-trip recall: 100% ({}/{})",
                res.coverage.covered, res.coverage.total
            ),
        )
    } else {
        let floored = (frac * 10.0).floor() / 10.0;
        (
            NoticeTone::Warning,
            format!(
                "Round-trip recall: {floored:.1}% ({}/{})",
                res.coverage.covered, res.coverage.total
            ),
        )
    }
}

const fn construct_label(kind: ConstructKind) -> &'static str {
    match kind {
        ConstructKind::Positional => "Positional isomer (m: blocks)",
        ConstructKind::Repeating => "Variable-length repeat (Sg:n: block)",
        ConstructKind::BestEffort => "Best-effort (no clean shared scaffold)",
    }
}

fn candidates_section(
    res: &CxResult,
    candidates: &[(String, String)],
    mut show_all: Signal<bool>,
) -> Element {
    if candidates.is_empty() {
        return rsx! {};
    }
    let light = ColorScheme::LIGHT;
    let n_enum = res.enumerated.len();
    let all_enumerated = res.enumerated.join("\n");
    let cell_style = StyleBuilder::new()
        .width("100%")
        .height("90px")
        .display("grid")
        .property("place-items", "center")
        .background_color(light.bg)
        .property_fmt("border", format_args!("1px solid {}", light.border))
        .border_radius(Radius::MD)
        .property("overflow", "hidden")
        .build();
    rsx! {
        div { style: card_surface(true),
            div { style: StyleBuilder::new().display("flex").justify_content("space-between").align_items("center").build(),
                div { style: field_label(false), "Enumerated candidates ({n_enum})" }
                CopyCell { text: Arc::<str>::from(all_enumerated.as_str()) }
            }
            div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fill, minmax(110px, 1fr))").gap("0.4rem").width("100%").property("max-height", "420px").property("overflow-y", "auto").build(),
                for (idx, (smi, svg)) in candidates.iter().enumerate() {
                    div {
                        key: "{idx}-{smi}",
                        title: "{smi}",
                        style: "{cell_style}",
                        div { dangerous_inner_html: "{svg}" }
                    }
                }
            }
            if n_enum > CANDIDATE_PAGE_SIZE && !*show_all.read() {
                button {
                    r#type: "button",
                    onclick: move |_| show_all.set(true),
                    style: StyleBuilder::new().background_color(light.bg).color(light.text2).property_fmt("border", format_args!("1px solid {}", light.border)).border_radius(Radius::MD).padding("0.3rem 0.6rem").font_size(Typography::UI).cursor("pointer").build(),
                    "Show all ({n_enum})"
                }
            }
        }
    }
}

/// Renders the CX-SMILES output card when a result is present, lazily memoizing SVGs.
#[component]
pub(super) fn ResultsPanel(result: Signal<Option<CxResult>>, show_all: Signal<bool>) -> Element {
    let scaffold_svg = use_memo(move || result.read().as_ref().map(depict_scaffold));
    let candidates = use_memo(move || {
        let guard = result.read();
        let Some(res) = guard.as_ref() else {
            return Vec::new();
        };
        let limit = if *show_all.read() {
            res.enumerated.len()
        } else {
            CANDIDATE_PAGE_SIZE
        };
        res.enumerated
            .iter()
            .take(limit)
            .map(|smi| (smi.clone(), render_smiles_svg(smi)))
            .collect::<Vec<_>>()
    });

    let res_guard = result.read();
    let Some(res) = res_guard.as_ref() else {
        return rsx! {};
    };
    let svg_guard = scaffold_svg.read();
    let Some(scaffold) = svg_guard.as_deref() else {
        return rsx! {};
    };
    let cand_guard = candidates.read();
    let light = ColorScheme::LIGHT;
    let (tone, recall_text) = recall_notice(res);
    let construct_text = construct_label(res.construct);
    let cx_box_style = StyleBuilder::new()
        .padding("0.6rem 0.7rem")
        .background_color(light.bg)
        .border_radius(Radius::MD)
        .property_fmt("border", format_args!("1px solid {}", light.border))
        .font_family("ui-monospace, monospace")
        .font_size(Typography::UI)
        .property("word-break", "break-all")
        .color(light.text)
        .build();

    rsx! {
        Card {
            title: "Generated CX-SMILES".to_string(),
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.75rem").build(),
                div { style: cx_box_style, "{res.cx_smiles}" }
                div { style: StyleBuilder::new().display("flex").align_items("center").gap("0.5rem").flex_wrap("wrap").build(),
                    CopyCell { text: Arc::<str>::from(res.cx_smiles.as_str()) }
                    span { style: StyleBuilder::new().font_size(Typography::UI).color(light.text3).build(), "{construct_text}" }
                }
                NoticeBar {
                    label: recall_text,
                    tone,
                }
                div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "1fr 1fr").gap("0.75rem").build(),
                    div { style: card_surface(false),
                        div { style: field_label(true), "Shared scaffold" }
                        div { style: StyleBuilder::new().width("100%").display("flex").justify_content("center").align_items("center").build(),
                            div { dangerous_inner_html: "{scaffold}" }
                        }
                        { floating_badges(res) }
                    }
                    { candidates_section(res, &cand_guard, show_all) }
                }
            }
        }
    }
}
