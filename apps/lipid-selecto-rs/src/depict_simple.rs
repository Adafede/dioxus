//! Depiction using simolecule CDKdepict API.
//!
//! Returns HTML img tag that loads from the working simolecule service.

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
        "https://www.simolecule.com/cdkdepict/depict/bow/svg?smi={}",
        encoded
    );

    // Return HTML with an img tag that loads the remote depiction
    Some(format!(
        r#"<img src="{}" style="width: 100%; height: 100%; object-fit: contain;" alt="Depiction" loading="lazy" />"#,
        url
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depict_fatty_acid() {
        let smiles = "CCCCCCCCCCCCCCCC(=O)O";
        let html = render_svg(smiles);
        assert!(html.is_some());
        let content = html.unwrap();
        assert!(content.contains("img"));
        assert!(content.contains("simolecule"));
        assert!(content.contains("cdkdepict"));
    }
}

