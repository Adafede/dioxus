// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

// A build script has no `Result`-returning `main`: `panic!` (or `assert!`)
// is the only way to fail the build with a diagnostic, which is exactly the
// desired behavior here. Not reachable in the shipped WASM bundle.
#![expect(clippy::panic)]

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|err| panic!("CARGO_MANIFEST_DIR is not set: {err}"));
    let manifest_dir = Path::new(&manifest_dir);

    for path in [
        "build.rs",
        "assets/smellfish.css",
        "assets/rdkit_bridge.js",
        "public/ertl_source_vs_synthetic.txt",
        "public/ertl_kingdom_enrichment.txt",
        "public/group_names.txt",
        "public/user_source_vs_synthetic.txt",
        "public/user_kingdom_enrichment.txt",
        "public/motif-library.js",
        "public/ertl_npsubstituents.txt",
        "public/lotus_1percent_scaffolds.txt",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }

    download_with_header(
        &manifest_dir.join("public/ertl_npsubstituents.txt"),
        "https://peter-ertl.com/molecular/data/npsubstituents.txt",
        "# Source: Peter Ertl npsubstituents.txt from https://peter-ertl.com/molecular/data/npsubstituents.txt\n",
    );
    download_and_filter_lotus_scaffolds(
        &manifest_dir.join("public/lotus_1percent_scaffolds.txt"),
        "https://raw.githubusercontent.com/Adafede/marimo/refs/heads/main/apps/public/mortar/Fragments_Scaffold_Generator.csv",
    );
}

fn fetch_url(url: &str) -> String {
    let output = Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .unwrap_or_else(|err| panic!("failed to run curl for {url}: {err}"));
    assert!(
        output.status.success(),
        "failed to download {url}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn download_with_header(dst: &Path, url: &str, header: &str) {
    if fs::metadata(dst).is_ok_and(|m| m.len() > 0) {
        return;
    }
    let body = fetch_url(url);
    let mut out = String::new();
    out.push_str(header);
    if !header.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&body);
    out.push('\n');
    fs::write(dst, out).unwrap_or_else(|err| panic!("failed to write {}: {err}", dst.display()));
}

fn download_and_filter_lotus_scaffolds(dst: &Path, url: &str) {
    if fs::metadata(dst).is_ok_and(|m| m.len() > 0) {
        eprintln!(
            "lotus_scaffolds: {} already exists, skipping download",
            dst.display()
        );
        return;
    }
    let text = fetch_url(url);
    let mut kept: Vec<String> = Vec::new();
    for raw_line in text.lines().skip(1) {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let parts = parse_csv_line(line);
        let [smiles, _, _, _, mol_pct, ..] = parts.as_slice() else {
            continue;
        };
        let smiles = smiles.trim();
        let mol_pct: f64 = mol_pct.trim().parse().unwrap_or(0.0);
        if mol_pct > 0.01 && smiles.len() >= 5 {
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

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' && chars.peek() == Some(&'"') {
                let _ = chars.next();
                current.push('"');
            } else if ch == '"' {
                in_quotes = false;
            } else {
                current.push(ch);
            }
        } else if ch == '"' && current.trim().is_empty() {
            current.clear();
            in_quotes = true;
        } else if ch == ',' {
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    fields.push(current);
    fields
}
