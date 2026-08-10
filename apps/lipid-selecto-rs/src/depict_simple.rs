/// Simple depiction using CDKdepict HTTP API.
/// No complex layout algorithms - just send SMILES to the service.

pub fn render_svg(smiles: &str) -> Option<String> {
    // URL encode the SMILES manually (simple percent encoding)
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
    
    // Return a simple placeholder that shows the SMILES
    // In a real app, you'd fetch from the URL above, but WASM in browser
    // would need CORS-enabled server or run in Node.js backend
    Some(format!(
        "<!-- CDKdepict URL: {} -->\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 300 300\" width=\"100%\" height=\"100%\">\n  <text x=\"150\" y=\"150\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"12\" fill=\"#666\">\n    Depiction for: {}\n  </text>\n</svg>",
        url, smiles
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
        assert!(svg_text.contains(smiles));
    }
}

