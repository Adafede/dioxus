// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Local 2D SMILES depiction via `chematic`.

/// Fallback inline SVG used when SMILES parsing or depiction fails.
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn empty_svg() -> String {
    "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 40 20\"><text x=\"20\" y=\"12\" fill=\"#94a3b8\" text-anchor=\"middle\" font-family=\"system-ui\" font-size=\"10\">no structure</text></svg>".to_string()
}

/// Render a SMILES string into an inline `<svg>` element locally via `chematic`.
///
/// Falls back to [`empty_svg`] when the SMILES string is empty, invalid, or
/// fails layout.
#[cfg(any(test, target_arch = "wasm32"))]
#[must_use]
pub(crate) fn render_svg(smiles: &str) -> String {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return empty_svg();
    }

    let depicted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        chematic::smiles::parse(trimmed)
            .ok()
            .map(|mol| chematic::depict::depict_svg(&mol))
    }));
    match depicted {
        Ok(Some(svg)) if svg.trim_start().starts_with("<svg") => svg,
        _ => empty_svg(),
    }
}

#[cfg(test)]
mod tests;
