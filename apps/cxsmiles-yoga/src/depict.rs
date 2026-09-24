// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Local 2D depiction — no remote rendering service required.
//!
//! Wraps `chematic::depict::{depict_svg, depict_svg_highlighted}`, which compute
//! a 2D layout and emit an inline SVG string. Used by the results panel to draw
//! the shared scaffold and every enumerated candidate.

use chematic::depict::depict_svg;
use chematic::smiles::parse;

/// Render a SMILES string to an inline SVG, parsing it with chematic.
#[must_use]
pub fn render_smiles_svg(smiles: &str) -> String {
    parse(smiles).map_or_else(|_| empty_svg(), |mol| depict_svg(&mol))
}

fn empty_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 60"><text x="50%" y="34" text-anchor="middle" fill="#94a3b8" font-size="13" font-family="ui-system,system-ui,sans-serif">could not render</text></svg>"##.to_string()
}
