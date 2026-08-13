//! Dioxus UI for `cxsmiles-yoga`: a textarea, a "generate" button, and a results
//! panel that shows the produced CX-SMILES, the detected construct type, a
//! round-trip confidence indicator, and 2D depiction of the shared scaffold.
//!
//! The `app` entry point intentionally is **not** annotated with `#[component]`
//! so that `dioxus::launch(cxsmiles_yoga::app)` works exactly like the sibling
//! `lipid-selecto-rs` app.

use std::sync::Arc;

use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use ui::prelude::*;

use crate::cxsmiles::{Construct, CxResult, generate_cxsmiles};
use crate::depict;
use crate::examples;

/// Best-effort copy to the clipboard. On wasm32 uses the async clipboard API;
/// on native returns `false` (the app is browser-targeted).
#[cfg(target_arch = "wasm32")]
fn copy_to_clipboard(text: &str) -> bool {
    if let Some(win) = web_sys::window() {
        let nav = win.navigator();
        let cb = nav.clipboard();
        // `write_text` returns a JS Promise; fire-and-forget is fine for a
        // best-effort copy — the `copied` flash only reflects the attempt.
        let _ = cb.write_text(text);
        return true;
    }
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn copy_to_clipboard(_text: &str) -> bool {
    false
}

/// A small "Copy" button that flashes "Copied!".
#[component]
fn CopyCell(props: CopyCellProps) -> Element {
    let mut copied = use_signal(|| false);
    let text = props.text.clone();
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| {
                let ok = copy_to_clipboard(&text);
                copied.set(ok);
                if ok {
                    spawn(async move {
                        let _ = TimeoutFuture::new(1200).await;
                        copied.set(false);
                    });
                }
            },
            style: StyleBuilder::new()
                .background_color("#f1f5f9")
                .color("#334155")
                .border(&format!("1px solid {}", ColorScheme::LIGHT.border))
                .border_radius(Radius::MD)
                .padding("2px 0.7rem")
                .font_size(Typography::UI)
                .font_weight("600")
                .cursor("pointer")
                .build(),
            if *copied.read() {
                "Copied!"
            } else {
                "Copy"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq, Debug)]
struct CopyCellProps {
    text: Arc<str>,
}

/// Renders the shared scaffold with its floating groups drawn alongside.
fn results(res: &CxResult) -> Element {
    let frac = res.confidence.coverage.fraction() * 100.0;
    let coverage_pct = format!("{:.0}%", frac);
    let tone = if res.confidence.clean {
        NoticeTone::Success
    } else {
        NoticeTone::Warning
    };
    let construct_label = match res.construct {
        Construct::Positional => "Positional isomer (m: blocks)",
        Construct::Repeating => "Variable-length repeat (Sg:n: block)",
        Construct::BestEffort => "Best-effort (no clean shared scaffold)",
    };

    let n_enum = res.enumerated.len();
    let scaffold_svg = depict::render_smiles_svg(&res.scaffold_smiles);

    rsx! {
        Card {
            title: "Generated CX-SMILES".to_string(),
            div { style: "display: flex; flex-direction: column; gap: 0.75rem;",
                div {
                    style: "padding: 0.6rem 0.7rem; background: #0f172a0d; border-radius: 10px; border: 1px solid #e2e8f0; font-family: ui-monospace, monospace; font-size: 0.85rem; word-break: break-all; color: #0f172a;",
                    "{res.cx_smiles}"
                }

                div { style: "display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap;",
                    CopyCell { text: Arc::<str>::from(res.cx_smiles.as_str()) }
                    span { style: "font-size: 0.8rem; color: #64748b;", "{construct_label}" }
                }

                NoticeBar {
                    label: format!(
                        "Round-trip recall: {coverage_pct} ({}/{})",
                        res.confidence.coverage.covered,
                        res.confidence.coverage.total
                    ),
                    tone,
                }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem;",
                    div { style: "background: #fff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 0.5rem; min-height: 120px;",
                        div { style: "font-size: 0.72rem; font-weight: 600; color: #475569; margin-bottom: 0.25rem;", "Shared scaffold" }
                        div { style: "width: 100%; display: flex; justify-content: center; align-items: center;",
                            div { dangerous_inner_html: "{scaffold_svg}" }
                        }
                    }

                    if !res.enumerated.is_empty() {
                        div { style: "background: #fff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 0.5rem; min-height: 120px; display: flex; flex-direction: column; gap: 0.5rem;",
                            div { style: "font-size: 0.72rem; font-weight: 600; color: #475569;", "Enumerated candidates ({n_enum})" }
                            div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(110px, 1fr)); gap: 0.4rem; width: 100%; overflow-y: auto;",
                                for smi in &res.enumerated {
                                    {
                                        let s = smi.clone();
                                        let svg = depict::render_smiles_svg(&s);
                                        rsx! {
                                            div { style: "width: 100%; height: 90px; display: grid; place-items: center; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden;",
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
    }
}

/// The main `cxsmiles-yoga` UI. Entry point for `dioxus::launch(cxsmiles_yoga::app)`.
#[allow(clippy::too_many_lines)]
pub fn app() -> Element {
    let colors = ColorScheme::LIGHT;
    let mut input = use_signal(|| String::new());
    let mut result = use_signal(|| None::<CxResult>);
    let mut error = use_signal(|| None::<String>);
    let mut selected = use_signal(|| String::new());
    let mut aromatic = use_signal(|| false); // NYI: "enumerate equivalent aromatic positions"

    let on_generate = move |_| {
        error.set(None);
        result.set(None);
        let lines: Vec<String> = input
            .read()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        match generate_cxsmiles(&lines) {
            Ok(r) => result.set(Some(r)),
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    let on_example_select = move |value: String| {
        if let Some(sm) = examples::example_smiles(&value) {
            input.set(sm.to_string());
        }
        selected.set(value);
        result.set(None);
        error.set(None);
    };

    rsx! {
        DocumentHead {
            title: "🧘 CX-SMILES Yoga".to_string(),
            lang: "en".to_string(),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts: vec!["https://scripts.simpleanalyticscdn.com/latest.js".to_string()],
            inline_style: Some(
                "-webkit-text-size-adjust:100%;-moz-text-size-adjust:100%;text-size-adjust:100%;background:#f8fafc;color:#0f172a;font-family:ui-system,system-ui,sans-serif"
                    .to_string()
            ),
        }

        Header {
            title: "🧘 CX-SMILES Yoga".to_string(),
            subtitle: Some("Generate CX-SMILES from a list of related structures".to_string()),
        }

        main { style: "max-width: 1000px; margin: 0 auto; padding: 1.5rem 1rem 3rem;",
            p { style: "color: #475569; font-size: 0.95rem; max-width: 60rem;",
                "Paste a list of related SMILES (one per line). The tool finds the shared scaffold, "
                "classifies each variable region as a positional isomer (m:) or a variable-length "
                "repeat (Sg:n:), and emits a single CX-SMILES. It then re-expands the result and "
                "reports how many of your inputs round-trip — the higher the percentage, the more "
                "confident the output."
            }

            div { style: "margin: 1rem 0;",
                SegmentedControl {
                    aria_label: "Pick an example".to_string(),
                    selected_value: "{selected.read()}",
                    items: examples::example_items(),
                    on_select: on_example_select,
                }
            }

            div { style: "background: #fff; border: 1px solid #e2e8f0; border-radius: 20px; box-shadow: 0 12px 40px rgba(15,23,42,0.06); padding: 1.25rem; margin-bottom: 1.25rem;",
                div { style: "display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 0.5rem;",
                    label { style: "font-size: 0.85rem; font-weight: 600; color: #0f172a;", "SMILES list" }
                    span { style: "font-size: 0.72rem; color: #94a3b8;", "{input.read().lines().count()} line(s)" }
                }
                textarea {
                    style: "width: 100%; min-height: 140px; font-family: ui-monospace, monospace; font-size: 0.85rem; padding: 0.6rem 0.7rem; border: 1px solid #cbd5e1; border-radius: 12px; resize: vertical; color: #0f172a;",
                    value: "{input.read()}",
                    placeholder: "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1",
                    oninput: move |e| input.set(e.value()),
                }

                div { style: "display: flex; align-items: center; gap: 0.5rem; margin-top: 0.6rem; font-size: 0.82rem; color: #64748b;",
                    input {
                        r#type: "checkbox",
                        id: "aromatic-equiv",
                        checked: "{aromatic.read()}",
                        onchange: move |_| {
                        let cur = *aromatic.read();
                        aromatic.set(!cur);
                    },
                    }
                    label { r#for: "aromatic-equiv", "(NYI) Enumerate equivalent aromatic positions" }
                }
            }

            div { style: "margin-bottom: 1rem;",
                button {
                    r#type: "button",
                    disabled: input.read().trim().is_empty(),
                    onclick: on_generate,
                    style: StyleBuilder::new()
                        .background_color(colors.accent)
                        .color(colors.bg)
                        .border("none")
                        .border_radius(Radius::LG)
                        .padding(&format!("{} {} {} {}", Spacing::SM, Spacing::MD, Spacing::SM, Spacing::MD))
                        .font_size(Typography::UI)
                        .font_weight("600")
                        .cursor("pointer")
                        .build(),
                    "Generate"
                }
            }

            if let Some(err) = error.read().as_ref() {
                NoticeBar { label: err.clone(), tone: NoticeTone::Danger }
            }

            if let Some(res) = result.read().as_ref() {
                { results(res) }
            }
        }
        Footer {}
    }
}
