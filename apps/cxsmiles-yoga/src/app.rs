//! Dioxus UI for `cxsmiles-yoga`.
//!
//! A textarea, a "generate" button, and a results panel show the produced
//! CX-SMILES, the detected construct type, a round-trip confidence indicator,
//! and 2D depiction of the shared scaffold.
//!
//! The `app` entry point intentionally is **not** annotated with `#[component]`
//! so that `dioxus::launch(cxsmiles_yoga::app)` works exactly like the sibling
//! `lipid-selecto-rs` app.
//!
//! Style convention: every inline `style:` is a `StyleBuilder` value (re-exported
//! by `ui::prelude`) — never a raw CSS string — and reused sub-patterns are
//! extracted as module-level style fns (`card_surface`, `field_label`). No
//! `clippy::too_many_lines` allows are patched around: the input card is split
//! into `input_card` and `StyleBuilder` chains are kept on single lines so
//! both `results` and `app` stay within the lint threshold.

use std::sync::Arc;

use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use ui::prelude::*;

use crate::cxsmiles::{Construct, CxResult, generate};
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
const fn copy_to_clipboard(_text: &str) -> bool {
    false
}

/// A small "Copy" button that flashes "Copied!".
#[component]
fn CopyCell(props: CopyCellProps) -> Element {
    let mut copied = use_signal(|| false);
    let text = props.text;
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| {
                let ok = copy_to_clipboard(&text);
                copied.set(ok);
                if ok {
                    spawn(async move {
                        TimeoutFuture::new(1200).await;
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

/// Shared card surface used by the scaffold and enumerated-candidate panels.
/// The `flex` variant extends the base with a flex column layout.
fn card_surface(flex: bool) -> String {
    let b = StyleBuilder::new()
        .background_color("#fff")
        .border("1px solid #e2e8f0")
        .border_radius("12px")
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

/// Reused field-label typography; `with_margin` appends the scaffold margin.
fn field_label(with_margin: bool) -> String {
    let b = StyleBuilder::new()
        .font_size("0.72rem")
        .font_weight("600")
        .color("#475569");
    if with_margin {
        b.property("margin-bottom", "0.25rem").build()
    } else {
        b.build()
    }
}

/// The SMILES-textarea input card (value, line count, NYI aromatic checkbox).
///
/// Split out of [`app`] so `app` stays small enough to satisfy
/// `clippy::too_many_lines` without an `#[allow]` attribute.
fn input_card(mut input: Signal<String>, mut aromatic: Signal<bool>) -> Element {
    let line_count = input.read().lines().count();
    rsx! {
        div { style: StyleBuilder::new().background_color("#fff").border("1px solid #e2e8f0").border_radius("20px").box_shadow("0 12px 40px rgba(15,23,42,0.06)").padding("1.25rem").property("margin-bottom", "1.25rem").build(),
            div { style: StyleBuilder::new().display("flex").justify_content("space-between").align_items("baseline").property("margin-bottom", "0.5rem").build(),
                label { style: StyleBuilder::new().font_size("0.85rem").font_weight("600").color("#0f172a").build(), "SMILES list" }
                span { style: StyleBuilder::new().font_size("0.72rem").color("#94a3b8").build(), "{line_count} line(s)" }
            }
            textarea {
                style: StyleBuilder::new().width("100%").property("min-height", "140px").font_family("ui-monospace, monospace").font_size("0.85rem").padding("0.6rem 0.7rem").border("1px solid #cbd5e1").border_radius("12px").property("resize", "vertical").color("#0f172a").build(),
                value: "{input.read()}",
                placeholder: "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1",
                oninput: move |e| input.set(e.value()),
            }
            div { style: StyleBuilder::new().display("flex").align_items("center").gap("0.5rem").property("margin-top", "0.6rem").font_size("0.82rem").color("#64748b").build(),
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
    }
}

/// Renders the shared scaffold with its floating groups drawn alongside.
fn results(res: &CxResult) -> Element {
    let frac = res.confidence.coverage.fraction() * 100.0;
    let coverage_pct = format!("{frac:.0}%");
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
            div { style: StyleBuilder::new().display("flex").flex_direction("column").gap("0.75rem").build(),
                div {
                    style: StyleBuilder::new().padding("0.6rem 0.7rem").background_color("#0f172a0d").border_radius("10px").border("1px solid #e2e8f0").font_family("ui-monospace, monospace").font_size("0.85rem").property("word-break", "break-all").color("#0f172a").build(),
                    "{res.cx_smiles}"
                }
                div { style: StyleBuilder::new().display("flex").align_items("center").gap("0.5rem").flex_wrap("wrap").build(),
                    CopyCell { text: Arc::<str>::from(res.cx_smiles.as_str()) }
                    span { style: StyleBuilder::new().font_size("0.8rem").color("#64748b").build(), "{construct_label}" }
                }
                NoticeBar {
                    label: format!("Round-trip recall: {coverage_pct} ({}/{})", res.confidence.coverage.covered, res.confidence.coverage.total),
                    tone,
                }
                div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "1fr 1fr").gap("0.75rem").build(),
                    div { style: card_surface(false),
                        div { style: field_label(true), "Shared scaffold" }
                        div { style: StyleBuilder::new().width("100%").display("flex").justify_content("center").align_items("center").build(),
                            div { dangerous_inner_html: "{scaffold_svg}" }
                        }
                    }
                    if !res.enumerated.is_empty() {
                        div { style: card_surface(true),
                            div { style: field_label(false), "Enumerated candidates ({n_enum})" }
                            div { style: StyleBuilder::new().display("grid").property("grid-template-columns", "repeat(auto-fill, minmax(110px, 1fr))").gap("0.4rem").width("100%").property("overflow-y", "auto").build(),
                                for smi in &res.enumerated {
                                    {
                                        let s = smi.clone();
                                        let svg = depict::render_smiles_svg(&s);
                                        rsx! {
                                            div { style: StyleBuilder::new().width("100%").height("90px").display("grid").property("place-items", "center").background_color("#f8fafc").border("1px solid #e2e8f0").border_radius("8px").property("overflow", "hidden").build(),
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
///
/// # Errors
///
/// Returns an `Element` (`Result<VNode, RenderError>`); a render error from
/// `rsx!` propagates to the nearest error boundary.
pub fn app() -> Element {
    let mut input = use_signal(String::new);
    let mut result = use_signal(|| None::<CxResult>);
    let mut error = use_signal(|| None::<String>);
    let mut selected = use_signal(String::new);
    let aromatic = use_signal(|| false); // NYI: "enumerate equivalent aromatic positions"

    let on_generate = move |_| {
        error.set(None);
        result.set(None);
        let lines: Vec<String> = input
            .read()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        match generate(&lines) {
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
            inline_style: Some("-webkit-text-size-adjust:100%;-moz-text-size-adjust:100%;text-size-adjust:100%;background:#f8fafc;color:#0f172a;font-family:ui-system,system-ui,sans-serif".to_string()),
        }

        Header {
            title: "🧘 CX-SMILES Yoga".to_string(),
            subtitle: Some("Generate CX-SMILES from a list of related structures".to_string()),
        }

        main { style: StyleBuilder::new().property("max-width", "1000px").margin("0 auto").padding("1.5rem 1rem 3rem").build(),
            p { style: StyleBuilder::new().color("#475569").font_size("0.95rem").property("max-width", "60rem").build(),
                "Paste a list of related SMILES (one per line). The tool finds the shared scaffold, "
                "classifies each variable region as a positional isomer (m:) or a variable-length "
                "repeat (Sg:n:), and emits a single CX-SMILES. It then re-expands the result and "
                "reports how many of your inputs round-trip — the higher the percentage, the more "
                "confident the output."
            }

            div { style: StyleBuilder::new().margin("1rem 0").build(),
                SegmentedControl {
                    aria_label: "Pick an example".to_string(),
                    selected_value: "{selected.read()}",
                    items: examples::example_items(),
                    on_select: on_example_select,
                }
            }

            { input_card(input, aromatic) }

            div { style: StyleBuilder::new().property("margin-bottom", "1rem").build(),
                Button {
                    label: "Generate",
                    variant: ButtonVariant::Primary,
                    disabled: input.read().trim().is_empty(),
                    onclick: Some(EventHandler::new(on_generate)),
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
