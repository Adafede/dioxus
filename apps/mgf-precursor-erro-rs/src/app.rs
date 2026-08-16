use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use ui::prelude::*;

mod browser;
mod diagnostics;
mod plots;
mod results;

use self::results::ResultsPanel;

use crate::diagnostics::RecalibrationStats;
use crate::metrics::PrecursorStats;
use crate::recalibration::CalibrationModel;

#[component]
fn skip_link() -> Element {
    rsx! {
        a {
            href: "#main",
            class: "skip-link",
            style: StyleBuilder::new().property("position", "absolute").property("top", "-100%").property("left", "0.5rem").property("z-index", "9999").padding("0.5rem 1rem").property("background", "transparent").color("#0b5cab").font_size("0.875rem").font_weight("600").border_radius("0 0 4px 4px").text_decoration("underline").build(),
            "Skip to main content"
        }
    }
}

/// Renders the MGF precursor-error analysis UI.
///
/// # Errors
///
/// Returns a rendering error only if the RSX tree cannot be constructed.
pub fn app() -> Element {
    ui::shared_signals! {
        file_name: String::new,
        metrics: || None::<PrecursorStats>,
    }
    ui::shared_signal!(status, || "Drop an MGF file to begin.".to_string());
    ui::shared_signal!(busy, || false);
    let drag_active = use_signal(|| false);
    let original_mgf_content = use_signal(String::new);

    // Recalibration control signals
    let calibration_model = use_signal(|| CalibrationModel::None);
    let lambda_value = use_signal(|| 0.5);
    let recalibration_diagnostics = use_signal(|| None::<RecalibrationStats>);
    let cumulative_dist_tab = use_signal(|| "mda"); // "mda" or "ppm"

    let on_file_change = move |evt: Event<FormData>| {
        browser::attempt_analysis_from_files(
            &evt.data().files(),
            file_name,
            status,
            metrics,
            busy,
            drag_active,
            original_mgf_content,
        );
    };
    let on_drop = move |evt: Event<DragData>| {
        browser::attempt_analysis_from_files(
            &evt.data().files(),
            file_name,
            status,
            metrics,
            busy,
            drag_active,
            original_mgf_content,
        );
    };

    rsx! {
        DocumentHead {
            title: "MGF Precursor Error".to_string(),
            lang: "en".to_string(),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts: vec!["https://scripts.simpleanalyticscdn.com/latest.js".to_string()],
            inline_style: Some(
                "body{font-family:ui-system,system-ui,sans-serif;-webkit-text-size-adjust:100%;\
                 -moz-text-size-adjust:100%;text-size-adjust:100%;}\
                 .skip-link:focus{top:0!important;outline:3px solid #0b5cab;outline-offset:2px}"
                    .to_string()
            ),
        }

        div {
            style: StyleBuilder::new().min_height("100vh").padding("2rem 1rem 3rem").property("background", "linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%)").color("#0f172a").build(),
            skip_link {}

            main { id: "main",
                style: StyleBuilder::new().property("max-width", "960px").margin("0 auto").build(),
                h1 { style: StyleBuilder::new().margin("0 0 0.35rem").font_size("1.7rem").property("letter-spacing", "-0.02em").build(), "MGF Precursor Error" }
                p {
                    style: StyleBuilder::new().margin("0 0 1.25rem").color("#475569").font_size("0.95rem").build(),
                    "Upload an MGF file and explore precursor mass errors in Da and ppm."
                }

                div {
                    style: StyleBuilder::new().property("background", "rgba(255,255,255,0.9)").border("1px solid rgba(148,163,184,0.22)").border_radius("20px").box_shadow("0 12px 40px rgba(15, 23, 42, 0.08)").padding("1.25rem").property("backdrop-filter", "blur(12px)").build(),
                    UploadZone {
                        file_name,
                        status,
                        busy,
                        drag_active,
                        on_file_change,
                        on_drop,
                        accept: ".mgf",
                        label: "Drop an MGF file here or click to browse",
                        hint: "Accepts .mgf files. Use drag and drop or browse.",
                        icon: "📁",
                    }
                    p {
                        style: StyleBuilder::new().margin("0.5rem 0 0").color("#475569").font_size("0.85rem").build(),
                        "Plots cap at 5 mDa / 10 ppm for the signed-error views"
                    }

                    example_load_button {
                        file_name,
                        status,
                        metrics,
                        busy,
                        original_mgf_content,
                    }

                    ResultsPanel {
                        metrics,
                        calibration_model,
                        lambda_value,
                        recalibration_diagnostics,
                        cumulative_dist_tab,
                        original_mgf_content,
                        file_name,
                        status,
                    }
                }
            }
        }
    }
}

/// Renders the "Load example MGF" button, shown only when no file has been
/// loaded yet. On wasm32 it fetches the example spectrum; on native (no
/// browser) it sets a "needs a browser" status message.
#[component]
fn example_load_button(
    file_name: Signal<String>,
    status: Signal<String>,
    metrics: Signal<Option<PrecursorStats>>,
    busy: Signal<bool>,
    original_mgf_content: Signal<String>,
) -> Element {
    rsx! {
        if file_name.read().is_empty() && metrics.read().is_none() && !(*busy.read()) {
            button {
                r#type: "button",
                style: StyleBuilder::new().property("margin-top", "0.8rem").border("1px solid #2563eb").border_radius("999px").property("background", "#eff6ff").color("#1d4ed8").font_size("0.84rem").font_weight("700").padding("0.45rem 0.8rem").cursor("pointer").build(),
                onclick: move |_| {
                    #[cfg(target_arch = "wasm32")]
                    browser::load_example_mgf(status, metrics, busy, file_name, original_mgf_content);
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        status.set("This app needs to run in a browser.".to_string());
                    }
                },
                "Load example MGF"
            }
        }
    }
}
