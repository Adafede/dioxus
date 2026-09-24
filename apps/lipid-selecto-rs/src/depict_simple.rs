// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Depiction using simolecule `CDKdepict` API.
//!
//! Returns HTML `img` tag that loads from the working simolecule service.

/// Render a SMILES string into an SVG `<img>` tag via the simolecule `CDKdepict` service.
///
/// The returned HTML `<img>` tag fetches the structure from the remote
/// `simolecule.com` `CDKdepict` endpoint.
#[must_use]
#[cfg(target_arch = "wasm32")]
pub(crate) fn render_svg(smiles: &str) -> String {
    // URL encode the SMILES (simple percent encoding for special chars)
    let encoded = smiles
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect::<String>();

    let url = format!("https://www.simolecule.com/cdkdepict/depict/bow/svg?smi={encoded}");

    // Return HTML with an img tag that loads the remote depiction
    format!(
        r#"<img src="{url}" style="width: 100%; height: 100%; object-fit: contain;" alt="Depiction" loading="lazy" />"#
    )
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn test_depict_fatty_acid() {
        let smiles = "CCCCCCCCCCCCCCCC(=O)O";
        let content = render_svg(smiles);
        assert!(content.contains("img"));
        assert!(content.contains("simolecule"));
        assert!(content.contains("cdkdepict"));
    }
}
