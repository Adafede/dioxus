//! Generate example_lipids.smi from LipidMaps LMSD data.
//!
//! Downloads LMSD.sdf.zip from LipidMaps, parses it, and generates
//! example SMILES for all major lipid classes.
//!
//! Also outputs a Rust constant that can be used in the lipid-selecto-rs app.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use lipidsdl::sdf::lipidmaps;

/// Family code to LIPID MAPS family code prefix mapping.
fn get_family_code(main_class: &str) -> Option<&'static str> {
    let main_class_str = main_class.trim();

    // Extract the class code from brackets, e.g., "[GP01]"
    if let Some(s) = main_class_str.split_whitespace().next() {
        if s.starts_with('[') && s.ends_with(']') {
            let code = &s[1..s.len() - 1];
            if code.starts_with("GP") {
                return Some("GP");
            }
            if code.starts_with("GL") {
                return Some("GL");
            }
            if code.starts_with("FA") {
                return Some("FA");
            }
            if code.starts_with("SP") {
                return Some("SP");
            }
            if code.starts_with("ST") {
                return Some("ST");
            }
            if code.starts_with("PR") {
                return Some("PR");
            }
            if code.starts_with("SL") {
                return Some("SL");
            }
            if code.starts_with("PK") {
                return Some("PK");
            }
            return None;
        }
    }

    // Map first word to family
    let first_word = main_class_str.split_whitespace().next().unwrap_or("");
    match first_word {
        "Fatty" => Some("FA"),
        "Glycerophosphocholines"
        | "Glycerophosphoethanolamines"
        | "Glycerophosphoglycerols"
        | "Glycerophosphoserines"
        | "Glycerophosphates"
        | "Glycerophosphoinositols"
        | "Glycerophosphoglycerophosphoglycerols" => Some("GP"),
        "Triradylglycerols" | "Diacylglycerols" | "Monoglycerides" => Some("GL"),
        "Ceramides"
        | "Sphingomyens"
        | "Neutral glycosphingolipids"
        | "Acidic glycosphingolipids"
        | "Amphoteric glycosphingolipids" => Some("SP"),
        "Sterols" | "Cholestanols" | "Bile acids and derivatives" => Some("ST"),
        "Isoprenoids" | "Diterpenoids" | "Squalene" => Some("PR"),
        "Monoglycosyl diacylglycerols" | "Acyltrehaloses" => Some("SL"),
        "Flavonoids" | "Polyketides" | "Poly" => Some("PK"),
        _ => None,
    }
}

/// Check if a SMILES string looks like a valid lipid (has carbon atoms)
fn is_valid_lipid(smiles: &str) -> bool {
    smiles.chars().filter(|&c| c == 'C').count() >= 6
}

/// Example with family info for Rust output.
#[derive(Debug, Clone)]
struct LipidExample {
    name: String,
    smiles: String,
    family: &'static str,
}

/// Sample examples from the TSV data using round-robin sampling.
///
/// For each MAIN_CLASS, we sample evenly across all entries to get diverse
/// representation rather than just taking the first N.
fn sample_examples_from_tsv(tsv_text: &str, max_per_main_class: usize) -> Vec<LipidExample> {
    type Example = (String, String, String); // LM_ID, SMILES, Name

    // Group by MAIN_CLASS
    let mut by_main_class: HashMap<String, Vec<Example>> = HashMap::new();

    for line in tsv_text.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < lipidmaps::COLUMNS.len() {
            continue;
        }

        let lm_id = fields[0].to_string();
        let name = fields[1].to_string();
        let smiles = fields[11].to_string();
        let main_class = fields[5].to_string();

        if smiles.is_empty() || lm_id.is_empty() || main_class.is_empty() {
            continue;
        }

        let main_class_str = main_class.trim().to_string();

        if let Some(family) = get_family_code(&main_class_str) {
            by_main_class
                .entry(main_class_str)
                .or_default()
                .push((lm_id, smiles, name));
        }
    }

    // For each MAIN_CLASS, sample max_per_main_class examples
    // Use round-robin to get diverse samples
    let mut all_examples: Vec<LipidExample> = Vec::new();

    for (main_class, entries) in by_main_class {
        let family = get_family_code(&main_class).unwrap_or("FA");

        // Sort by LM_ID for deterministic order
        let mut entries = entries;
        entries.sort();

        // Filter to valid lipids with non-empty names
        let valid_entries: Vec<_> = entries
            .into_iter()
            .filter(|(_, s, n)| is_valid_lipid(s) && !n.trim().is_empty())
            .collect();

        if valid_entries.is_empty() {
            continue;
        }

        let total = valid_entries.len();
        let max_to_take = max_per_main_class.min(total);

        // Round-robin sampling: take every Nth entry
        let step = (total + max_per_main_class - 1) / max_per_main_class;
        let step = step.max(1);

        for (idx, (lm_id, smiles, name)) in valid_entries
            .into_iter()
            .enumerate()
            .filter(|(i, _)| i % step == 0)
            .take(max_to_take)
        {
            let name = name.trim();
            if !name.is_empty() {
                all_examples.push(LipidExample {
                    name: name.to_string(),
                    smiles,
                    family,
                });
            }
        }
    }

    all_examples
}

fn extract_sdf_from_zip(data: &[u8]) -> Result<String, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))
        .map_err(|e| format!("Failed to read zip: {e}"))?;

    let mut entry_name: Option<String> = None;
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| format!("{e}"))?;
        let name = entry.name();
        if name.to_lowercase().ends_with(".sdf") {
            entry_name = Some(name.to_string());
            break;
        }
    }
    let entry_name = entry_name.ok_or("No SDF file found in zip")?;

    let mut entry = archive.by_name(&entry_name).map_err(|e| format!("{e}"))?;
    let mut sdf_text = String::new();
    entry
        .read_to_string(&mut sdf_text)
        .map_err(|e| format!("{e}"))?;

    Ok(sdf_text)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut assets_dir = Path::new("apps/lipid-selecto-rs/assets");
    let mut max_per_main_class: usize = 10;
    let mut write_rust_constant = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--rust" => {
                write_rust_constant = true;
            }
            "--max" | "-m" if i + 1 < args.len() => {
                max_per_main_class = args[i + 1].parse().unwrap_or(10);
                i += 1;
            }
            _ if !args[i].starts_with('-') => {
                assets_dir = Path::new(&args[i]);
            }
            _ => {}
        }
        i += 1;
    }

    eprintln!(
        "Generating example_lipids.smi from LipidMaps (max {} per MAIN_CLASS)...",
        max_per_main_class
    );

    let zip_path = std::env::temp_dir().join("LMSD.sdf.zip");
    let url = "https://www.lipidmaps.org/files/?file=LMSD&ext=sdf.zip";

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    if let Err(e) = rt.block_on(lipidsdl::download::download_to(
        url,
        &zip_path,
        lipidsdl::download::noop_progress,
    )) {
        eprintln!("Download failed: {e}");
        std::process::exit(1);
    }

    let data = std::fs::read(&zip_path).expect("Failed to read downloaded zip");
    let sdf_text = extract_sdf_from_zip(&data).expect("Failed to extract SDF");

    let (tsv, count) = lipidmaps::to_lmsd_tsv(&sdf_text);

    eprintln!("Processed {count} lipids from LMSD");

    let examples = sample_examples_from_tsv(&tsv, max_per_main_class);
    eprintln!("Selected {} total examples", examples.len());

    // Write SMI file
    let out_path = assets_dir.join("example_lipids.smi");
    std::fs::create_dir_all(assets_dir).expect("Failed to create output directory");
    let mut out = std::fs::File::create(&out_path).expect("Failed to create output file");

    use std::io::Write;
    writeln!(
        out,
        "# Example lipids covering all major LIPID MAPS classes"
    )
    .expect("Failed to write header");
    writeln!(out, "# Format: ID\tSMILES\tDescription").expect("Failed to write format");
    writeln!(out).expect("Failed to write blank line");

    for ex in &examples {
        writeln!(out, "{}\t{}", ex.name, ex.smiles).expect("Failed to write entry");
        writeln!(out, "# {} ({})", ex.name, ex.family).expect("Failed to write description");
    }

    eprintln!(
        "Wrote {} examples to {}",
        examples.len(),
        out_path.display()
    );

    // Also write Rust constant if requested
    if write_rust_constant {
        let rust_path = Path::new("apps/lipid-selecto-rs/src/examples.rs");
        let mut rust_out = std::fs::File::create(rust_path).expect("Failed to create examples.rs");

        writeln!(
            rust_out,
            "//! Collection of {} example SMILES covering all LIPID MAPS classes from real data.",
            examples.len()
        )
        .unwrap();
        writeln!(rust_out, "//! Generated from LipidMaps LMSD dataset.").unwrap();
        writeln!(
            rust_out,
            "//! Covers all 8 categories: FA, GL, GP, SP, ST, PR, SL, PK"
        )
        .unwrap();
        writeln!(rust_out).unwrap();
        writeln!(
            rust_out,
            "/// Real lipid examples from LipidMaps LMSD dataset."
        )
        .unwrap();
        writeln!(
            rust_out,
            "pub const EXAMPLE_LIPIDS: &[(&str, &str, &str)] = &[ "
        )
        .unwrap();

        // Group examples by family
        let mut by_family: HashMap<&str, Vec<_>> = HashMap::new();
        for ex in &examples {
            by_family.entry(ex.family).or_default().push(ex.clone());
        }

        let order = ["FA", "GL", "GP", "SP", "ST", "PR", "SL", "PK"];
        for fam in &order {
            if let Some(examples) = by_family.get(fam) {
                // Write comment header
                match *fam {
                    "FA" => writeln!(rust_out, "    // === Fatty Acyls (FA) ===").unwrap(),
                    "GL" => writeln!(rust_out, "    // === Glycerolipids (GL) ===").unwrap(),
                    "GP" => writeln!(rust_out, "    // === Glycerophospholipids (GP) ===").unwrap(),
                    "SP" => writeln!(rust_out, "    // === Sphingolipids (SP) ===").unwrap(),
                    "ST" => writeln!(rust_out, "    // === Sterol Lipids (ST) ===").unwrap(),
                    "PR" => writeln!(rust_out, "    // === Prenol Lipids (PR) ===").unwrap(),
                    "SL" => writeln!(rust_out, "    // === Saccharolipids (SL) ===").unwrap(),
                    "PK" => writeln!(rust_out, "    // === Polyketides (PK) ===").unwrap(),
                    _ => {}
                }

                for ex in examples {
                    let safe_name = ex.name.replace('\\', "").replace("\"", "");
                    let safe_smiles = ex.smiles.replace('\\', "\\\\").replace("\"", "\\\"");
                    writeln!(
                        rust_out,
                        "    (\"{}\", \"{}\", \"{}\"),",
                        safe_name, safe_smiles, safe_name
                    )
                    .unwrap();
                }
            }
        }

        writeln!(rust_out, "];").unwrap();

        // Write the example_smiles helper function
        writeln!(rust_out).unwrap();
        writeln!(
            rust_out,
            "/// Convert example list to query format (just SMILES + description lines separated by newlines)."
        ).unwrap();
        writeln!(rust_out, "#[must_use]").unwrap();
        writeln!(rust_out, "pub fn example_smiles() -> Vec<String> {{").unwrap();
        writeln!(rust_out, "    EXAMPLE_LIPIDS").unwrap();
        writeln!(rust_out, "        .iter()").unwrap();
        writeln!(
            rust_out,
            "        .map(|(id, smiles, _)| format!(\"{{id}}\\t{{smiles}}\"))"
        )
        .unwrap();
        writeln!(rust_out, "        .collect()").unwrap();
        writeln!(rust_out, "}}").unwrap();

        eprintln!("Wrote Rust constant to {}", rust_path.display());
    }
}
