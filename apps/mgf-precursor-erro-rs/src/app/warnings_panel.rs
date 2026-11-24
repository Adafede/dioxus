// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Memoized warnings panel displaying skipped-SMILES, unrecognized adducts, and high-error spectra.

use dioxus::prelude::*;
use mgf_analysis_core::{HighErrorSmilesDetail, PrecursorStats, WarningDetail};
use ui::theme::StyleBuilder;

use super::plots::format_value;

#[derive(Clone, PartialEq)]
struct SortedWarnings {
    skipped: usize,
    unparsed_count: usize,
    unparsed: Vec<(String, WarningDetail)>,
    adducts: Vec<(String, usize)>,
    high_error: Vec<(String, HighErrorSmilesDetail)>,
}

/// Displays memoized warning summaries without re-sorting on unrelated UI renders.
#[component]
pub(super) fn WarningsPanel(metrics: Signal<Option<PrecursorStats>>) -> Element {
    let sorted = use_memo(move || {
        let guard = metrics.read();
        let m = guard.as_ref()?;
        Some(SortedWarnings {
            skipped: m.skipped_spectra,
            unparsed_count: m.unparsed_smiles,
            unparsed: m
                .sorted_unparsed_warnings()
                .into_iter()
                .map(|(s, d)| (s.to_string(), d.clone()))
                .collect(),
            adducts: m
                .sorted_unrecognized_adducts()
                .into_iter()
                .map(|(a, c)| (a.to_string(), c))
                .collect(),
            high_error: m
                .sorted_high_error_smiles()
                .into_iter()
                .map(|(s, d)| (s.to_string(), d.clone()))
                .collect(),
        })
    });
    let guard = sorted.read();
    let Some(data) = guard.as_ref() else {
        return rsx! {};
    };

    rsx! {
        if data.skipped > 0 || !data.adducts.is_empty() {
            div {
                style: StyleBuilder::new().property("margin-top", "0.9rem").padding("0.8rem 0.9rem").border("1px solid #fcd34d").border_radius("12px").property("background", "#fffbeb").color("#92400e").build(),
                p { style: StyleBuilder::new().margin("0 0 0.35rem").font_weight("700").build(), "Warnings" }
                p { style: StyleBuilder::new().margin("0").font_size("0.9rem").build(), "{data.skipped} spectra were skipped because the adduct or reference mass could not be resolved." }
                if data.unparsed_count > 0 {
                    p { style: StyleBuilder::new().margin("0.45rem 0 0").font_size("0.88rem").build(), "{data.unparsed_count} spectra had SMILES that could not be parsed into a reference mass." }
                }
                if !data.unparsed.is_empty() {
                    div { style: StyleBuilder::new().property("margin-top", "0.6rem").padding("0.7rem 0.8rem").border("1px solid #fde68a").border_radius("10px").property("background", "#fffbeb").color("#92400e").build(),
                        p { style: StyleBuilder::new().margin("0 0 0.35rem").font_weight("700").font_size("0.86rem").build(), "Excluded unparsed SMILES" }
                        ul { style: StyleBuilder::new().margin("0.25rem 0 0 1.05rem").padding("0").font_size("0.84rem").property("max-height", "160px").property("overflow", "auto").build(),
                            for (smiles, detail) in &data.unparsed {
                                {
                                    let formula_display = detail.formula.as_deref().filter(|v| !v.trim().is_empty()).map_or_else(String::new, |f| format!(" [formula: {f}]"));
                                    let count = detail.count;
                                    rsx! { li { "{smiles} ({count}){formula_display}" } }
                                }
                            }
                        }
                    }
                }
                if !data.adducts.is_empty() {
                    ul { style: StyleBuilder::new().margin("0.45rem 0 0 1.1rem").padding("0").font_size("0.88rem").build(),
                        for (adduct, count) in &data.adducts {
                            li { "{adduct}: {count}" }
                        }
                    }
                }
            }
        }
        if !data.high_error.is_empty() {
            div {
                style: StyleBuilder::new().property("margin-top", "1rem").padding("0.8rem 0.9rem").border("1px solid #fecaca").border_radius("12px").property("background", "#fef2f2").color("#991b1b").build(),
                p { style: StyleBuilder::new().margin("0 0 0.35rem").font_weight("700").build(), "SMILES for spectra above 0.01 Da" }
                ul { style: StyleBuilder::new().margin("0.25rem 0 0 1.1rem").padding("0").font_size("0.88rem").property("max-height", "240px").property("overflow", "auto").build(),
                    for (smiles, detail) in &data.high_error {
                        {
                            let suffix = if detail.count > 1 { format!(" (x{})", detail.count) } else { String::new() };
                            let fmt_opt = |v: Option<f64>| v.map_or_else(|| "n/a".to_string(), format_value);
                            let calc = fmt_opt(detail.calculated_mass);
                            let exp = fmt_opt(detail.expected_mass);
                            let obs = fmt_opt(detail.observed_precursor_mz);
                            let err_da = fmt_opt(detail.max_abs_error_da);
                            let err_ppm = fmt_opt(detail.max_abs_error_ppm);
                            let formula = detail.formula.as_deref().filter(|v| !v.trim().is_empty()).map_or_else(String::new, |f| format!("; formula {f}"));
                            rsx! {
                                li {
                                    "{smiles}{suffix} — worst error {err_da} Da / {err_ppm} ppm (derived expected precursor {exp}; observed precursor {obs}; reference mass {calc}){formula}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
