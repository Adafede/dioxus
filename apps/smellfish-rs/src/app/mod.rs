//! Smellfish app shell — signal wiring + composition.
//!
//! The heavy rendering subtrees live in `results` (`Hero`, `MotifPanel`,
//! `ResultsView`, `MoleculeCard`, `Footer`) and the upload dispatch lives in
//! `browser` (`attempt_import` / `attempt_import_from_text`), mirroring the
//! `mgf-precursor-erro-rs` `app/` layout. `app()` keeps only signal declaration,
//! a handful of 2-line handler closures, and the inline `InputPanel` block
//! (kept inline because its `oninput`/`onfocus` closures need the local
//! `pasted_smiles`/`demo_cleared` signals).
use crate::document_head::SmellfishDocumentHead;
use crate::model::{EndpointStatus, MoleculeRow, MotifSummary};
use dioxus::events::{DragData, FormData};
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use ui::prelude::{Button, ButtonVariant};

mod browser;
mod results;

use self::results::{Footer, Hero, MotifPanel, ResultsView};

/// Ten representative natural-product SMILES, pre-loaded as the default
/// paste-buffer content so the tool is ready to run out of the box.
const DEMO_SMILES: &str = "\
COC1=CC(=CC2=C1OCO2)C3C4COC(C4CO3)C5=CC(=C(C(=C5)OC)OC)OC
CC1=C(C(CCC1)(C)C)C=CC(=CC=CC(=CC#CC=C(C)C=O)C)C
CC(=CO)C1CCC2(C1C3CCC4C5(CCC(C(C5CCC4(C3(CC2)C)C)(C)C)O)C)C(=O)O
CCCCCCCCC=CCCCCCCCCC(=O)N
CC1C(C(C(C(O1)OC2CCC3(C(C2(C)CO)CCC4(C3CC=C5C4(CCC6(C5CC(CC6)(C)C)C(=O)O)C)C)C)O)O)O
CC1=CCCC(=CC2C(C(C1)OC(=O)C(=CCO)CO)C(=C)C(=O)O2)CO
CC1(C2CCC3(C(C2(CCC1O)C)CCC4C3(CCC5(C4C(CC5)C(=C)C=O)C)C)C)C
COC1=CC(=CC(=C1O)OC)C2C3COC(C3CO2)C4=CC(=C(C(=C4)OC)OC)OC
C1=CC(=CC=C1CCC(=O)CC(CCC2=CC(=C(C=C2)O)O)OC3C(C(C(C(O3)CO)O)O)O)O
CCCCCCCC=CCCCCCCCC(N)=O";

/// The `app` entry point is intentionally **not** annotated with `#[component]`
/// because the function lives in a `pub mod app;` module that is re-exported
/// via `pub use app::app;` in `lib.rs`.  The `#[component]` macro generates a
/// type-level binding named `app` that conflicts with the module name in the
/// type namespace (E0255).  Since `app()` takes no props, `dioxus::launch`
/// accepts it directly as a plain function — same pattern used by
/// `lipid-selecto-rs` and `mgf-precursor-erro-rs`.
///
/// # Errors
///
/// Returns a rendering error only if the RSX tree cannot be constructed.
pub fn app() -> Element {
    ui::shared_signal!(file_name, String::new);

    let status = use_signal(String::new);
    let busy = use_signal(|| false);
    let mut drag_active = use_signal(|| false);
    let rows = use_signal(Vec::<MoleculeRow>::new);
    let motifs = use_signal(Vec::<MotifSummary>::new);
    let endpoints = use_signal(Vec::<EndpointStatus>::new);
    let warnings = use_signal(Vec::<String>::new);
    let mut pasted_smiles = use_signal(|| DEMO_SMILES.to_string());
    let mut demo_cleared = use_signal(|| false);

    let on_file_change = move |evt: Event<FormData>| {
        browser::attempt_import(
            &evt.data().files(),
            file_name,
            status,
            busy,
            drag_active,
            rows,
            motifs,
            endpoints,
            warnings,
        );
    };
    let on_drag_enter = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };
    let on_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };
    let on_drag_leave = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(false);
    };
    let on_drop = move |evt: Event<DragData>| {
        browser::attempt_import(
            &evt.data().files(),
            file_name,
            status,
            busy,
            drag_active,
            rows,
            motifs,
            endpoints,
            warnings,
        );
    };
    let submit_pasted_smiles = move |_| {
        browser::attempt_import_from_text(
            pasted_smiles.read().trim().to_string(),
            file_name,
            status,
            busy,
            drag_active,
            rows,
            motifs,
            endpoints,
            warnings,
        );
    };

    let file_name_value = file_name.read().clone();
    let warning_text = warnings.read().join(" • ");
    let pasted_smiles_value = pasted_smiles.read().clone();
    let ep_list = endpoints.read().clone();

    rsx! {
        SmellfishDocumentHead {}
        div { class: "shell",

            Hero {}

            section { class: "panel input-panel",
                div { class: "input-split",
                    label { class: if *drag_active.read() { "input-card dropzone dragging" } else { "input-card dropzone" },
                        r#for: "smiles-csv",
                        ondragenter: on_drag_enter,
                        ondragover: on_drag_over,
                        ondragleave: on_drag_leave,
                        ondrop: on_drop,

                        div { class: "input-card-body",
                            strong { "Drop a CSV or click to browse" }
                            div { class: "small muted", "CSV with a smiles column" }
                        }

                        input {
                            id: "smiles-csv",
                            r#type: "file",
                            accept: ".csv,text/csv",
                            disabled: *busy.read(),
                            onchange: on_file_change,
                        }
                    }

                    div { class: "input-card paste-card",
                        div { class: "input-card-body",
                            div { class: "paste-head",
                                strong { "Paste SMILES" }
                                span { class: "small muted", "One per line" }
                            }
                            label { r#for: "smiles-paste", class: "visually-hidden", "SMILES structures, one per line" }
                            textarea {
                                id: "smiles-paste",
                                class: "smiles-textarea",
                                placeholder: "CCO\nC1CCCCC1\nCOC1=CC=CC=C1",
                                disabled: *busy.read(),
                                value: "{pasted_smiles_value}",
                                onfocus: move |_| {
                                    if !*demo_cleared.read() {
                                        demo_cleared.set(true);
                                        pasted_smiles.set(String::new());
                                    }
                                },
                                oninput: move |evt| pasted_smiles.set(evt.value()),
                            }
                        }
                        div { class: "paste-actions",
                            Button {
                                label: "Analyze pasted SMILES",
                                variant: ButtonVariant::Primary,
                                disabled: *busy.read(),
                                onclick: Some(EventHandler::new(submit_pasted_smiles)),
                            }
                        }
                    }
                }

                if !status.read().is_empty() || *busy.read() {
                    p { class: "status", role: "status", aria_live: "polite", aria_atomic: "true",
                        if *busy.read() {
                            span { class: "spinner" }
                        }
                        "{status}"
                    }
                }
                if !ep_list.is_empty() {
                    div { class: "endpoint-status",
                        for ep in &ep_list {
                            span { class: "endpoint-chip", class: if ep.reachable { "ok" } else { "down" }, "{ep.name}: {ep.detail} ({ep.endpoint})" }
                        }
                    }
                }

                if !file_name_value.is_empty() {
                    p { class: "small muted", "Loaded: {file_name_value}" }
                }

                if !warning_text.is_empty() {
                    div { class: "alert", "{warning_text}" }
                }
            }

            if !motifs.read().is_empty() {
                MotifPanel { motifs }
            }

            if !rows.read().is_empty() {
                ResultsView { rows }
            }

            Footer {}
        }
    }
}
