use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|err| panic!("CARGO_MANIFEST_DIR is not set: {err}"));
    let manifest_dir = Path::new(&manifest_dir);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=public/ertl_source_vs_synthetic.txt");
    println!("cargo:rerun-if-changed=public/ertl_kingdom_enrichment.txt");
    println!("cargo:rerun-if-changed=public/group_names.txt");
    println!("cargo:rerun-if-changed=public/user_source_vs_synthetic.txt");
    println!("cargo:rerun-if-changed=public/user_kingdom_enrichment.txt");
    println!("cargo:rerun-if-changed=public/motif-library.js");
    download_with_header(
        &manifest_dir.join("public/ertl_npsubstituents.txt"),
        "https://peter-ertl.com/molecular/data/npsubstituents.txt",
        "# Source: Peter Ertl npsubstituents.txt from https://peter-ertl.com/molecular/data/npsubstituents.txt\n",
    );
    /* Download the LOTUS mortar scaffold-frequency CSV by Rutz et al. and
     * pre-filter to scaffolds with MoleculePercentage > 1 % (i.e. > 0.01
     * as a 0–1 fraction).  Only the surviving SMILES are written to
     * public/lotus_1percent_scaffolds.txt so the browser doesn't have to
     * ship or parse the full 100k-line file. */
    download_and_filter_lotus_scaffolds(
        &manifest_dir.join("public/lotus_1percent_scaffolds.txt"),
        "https://raw.githubusercontent.com/Adafede/marimo/refs/heads/main/apps/public/mortar/Fragments_Scaffold_Generator.csv",
    );
}

fn download_with_header(dst: &Path, url: &str, header: &str) {
    let output = Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .unwrap_or_else(|err| panic!("failed to run curl for {url}: {err}"));
    assert!(
        output.status.success(),
        "failed to download {url}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut out = String::new();
    out.push_str(header);
    if !header.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&String::from_utf8_lossy(&output.stdout));
    out.push('\n');
    fs::write(dst, out).unwrap_or_else(|err| panic!("failed to write {}: {err}", dst.display()));
}

/// Download the LOTUS `Fragments_Scaffold_Generator.csv` (mortar fragmentation
/// by Rutz et al.) and extract scaffold SMILES whose `MoleculePercentage`
/// column exceeds 1 % (0.01 as a 0–1 fraction).  The result is written as
/// one SMILES per line to `dst`.
fn download_and_filter_lotus_scaffolds(dst: &Path, url: &str) {
    let output = Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .unwrap_or_else(|err| panic!("failed to run curl for {url}: {err}"));
    assert!(
        output.status.success(),
        "failed to download {url}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    let mut kept: Vec<String> = Vec::new();
    let mut in_header = true;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if in_header {
            in_header = false;
            continue; // skip header row
        }
        if line.is_empty() {
            continue;
        }
        // Columns: SMILES,Frequency,Percentage,MoleculeFrequency,MoleculePercentage
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 5 {
            continue;
        }
        let smiles = parts[0].trim();
        let mol_pct: f64 = parts[4].trim().parse().unwrap_or(0.0);
        /* Reject scaffolds shorter than 5 characters — single atoms,
         * diatomic fragments, and very short chains are too
         * non-specific to be meaningful structural motifs. */
        if mol_pct > 0.01 && !smiles.is_empty() && smiles.len() >= 5 {
            kept.push(smiles.to_string());
        }
    }
    let mut out = String::new();
    out.push_str("# Source: Rutz et al. LOTUS mortar scaffold fragmentation\n");
    out.push_str("# https://github.com/Adafede/marimo (apps/public/mortar/Fragments_Scaffold_Generator.csv)\n");
    out.push_str("# Filtered to scaffolds with MoleculePercentage > 1 % (");
    out.push_str(&kept.len().to_string());
    out.push_str(" scaffolds)\n\n");
    for s in &kept {
        out.push_str(s);
        out.push('\n');
    }
    fs::write(dst, out).unwrap_or_else(|err| panic!("failed to write {}: {err}", dst.display()));
    eprintln!(
        "lotus_scaffolds: retained {} scaffolds above 1 %",
        kept.len()
    );
}
