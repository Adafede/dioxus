/// Depiction using CDKdepict HTTP API.
/// Returns an HTML img tag that loads from the public CDKdepict service.

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

    // Return an SVG wrapper that embeds the remote image
    // The browser will fetch and render it asynchronously
    Some(format!(
        r#"<!-- CDKdepict remote rendering -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 300" width="100%" height="100%">
  <image href="{}" width="100%" height="100%" preserveAspectRatio="xMidYMid meet" />
</svg>"#,
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
        let svg_text = svg.unwrap();
        assert!(svg_text.contains("svg"));
        assert!(svg_text.contains("cdkdepict"));
    }
}

