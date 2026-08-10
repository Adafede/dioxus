/// Depiction using CDKdepict HTTP API.
/// Returns HTML with an image that loads from the public CDKdepict service.

pub fn render_svg(smiles: &str) -> Option<String> {
    // URL encode the SMILES (simple percent encoding for special chars)
    let encoded = smiles
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect::<String>();

    let url = format!(
        "https://cdkdepict.sourceforge.io/depict/bot/{}/svg",
        encoded
    );

    // Return HTML with an img tag that loads the remote depiction
    // This avoids CORS issues with embedded SVG images
    Some(format!(
        r#"<img src="{}" style="width: 100%; height: 100%; object-fit: contain;" alt="Depiction" />"#,
        url
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depict_fatty_acid() {
        let smiles = "CCCCCCCCCCCCCCCC(=O)O";
        let svg = render_svg(smiles);
        assert!(svg.is_some());
        let html = svg.unwrap();
        assert!(html.contains("img"));
        assert!(html.contains("cdkdepict"));
    }
}

