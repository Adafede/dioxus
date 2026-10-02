// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Depiction using simolecule `CDKdepict` API.
//!
//! Returns HTML `img` tag that loads from the working simolecule service.

/// Render a SMILES string into an SVG `<img>` tag via the simolecule `CDKdepict` service.
///
/// The returned HTML `<img>` tag fetches the structure from the remote
/// `simolecule.com` `CDKdepict` endpoint.
///
/// `any(test, …)` rather than wasm-only, because `build_gallery` is the only
/// caller and the host test build needs both. It builds a string and nothing
/// else, and gating it wasm-only meant the host test build never compiled the
/// URL encoder below — which is where the bug in its previous form was.
#[cfg(any(test, target_arch = "wasm32"))]
#[must_use]
pub(crate) fn render_svg(smiles: &str) -> String {
    let url = format!(
        "https://www.simolecule.com/cdkdepict/depict/bow/svg?smi={}",
        encode(smiles)
    );

    format!(
        r#"<img src="{url}" style="width: 100%; height: 100%; object-fit: contain;" alt="Depiction" loading="lazy" />"#
    )
}

/// Percent-encode for a query-string value, leaving the unreserved set alone.
///
/// Over the *bytes*, not over `char`s. The previous version matched on `char`
/// and then wrote `c as u8`, which for any character above U+00FF produced the
/// encoding of its low byte rather than of the character — a SMILES with a
/// non-ASCII atom label or a CX-SMILES extension in it would have asked
/// simolecule for a different molecule than the one on screen. The unreserved
/// set is the same one `application/x-www-form-urlencoded` defines.
#[cfg(any(test, target_arch = "wasm32"))]
fn encode(value: &str) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(char::from(*byte));
            }
            other => {
                // `write!` to a `String` is infallible; the `let _` says so
                // rather than unwrapping, which is denied.
                let _ = write!(out, "%{other:02X}");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
