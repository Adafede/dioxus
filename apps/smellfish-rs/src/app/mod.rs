// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Smellfish app shell — signal declaration and component composition.

use crate::document_head::SmellfishDocumentHead;
use dioxus::prelude::*;
use ui::prelude::{SimpleAnalyticsBadge, skip_link};

pub(crate) mod browser;
mod footer;
mod formatting;
mod input_panel;
mod molecule_card;
mod motif_display;
mod results;

use self::footer::Footer;
use self::input_panel::InputPanel;
use self::results::{Hero, MotifPanel, ResultsView};

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

/// Root Dioxus application component for `smellfish-rs`.
///
/// # Errors
///
/// Returns a rendering error only if the RSX tree fails to build.
pub fn app() -> Element {
    ui::shared_signal!(file_name, String::new);

    let signals = browser::ImportSignals {
        generation: use_signal(|| 0_u64),
        file_name,
        status: use_signal(String::new),
        busy: use_signal(|| false),
        drag_active: use_signal(|| false),
        rows: use_signal(Vec::new),
        motifs: use_signal(Vec::new),
        endpoints: use_signal(Vec::new),
        warnings: use_signal(Vec::new),
    };
    let pasted_smiles = use_signal(|| DEMO_SMILES.to_string());
    let demo_cleared = use_signal(|| false);

    rsx! {
        SmellfishDocumentHead {}
        skip_link {}
        div { class: "shell",
            Hero {}
            main { id: "main-content",
                InputPanel { signals, pasted_smiles, demo_cleared }
                if !signals.motifs.read().is_empty() {
                    MotifPanel { motifs: signals.motifs }
                }
                if !signals.rows.read().is_empty() {
                    ResultsView { rows: signals.rows }
                }
            }
            Footer {}
        }
        SimpleAnalyticsBadge {
            domain: "adafede.github.io".to_string(),
            path: "/dioxus/smellfish-rs/".to_string(),
        }
    }
}
