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
    println!("cargo:rerun-if-changed=public/user_scaffolds.txt");
    println!("cargo:rerun-if-changed=public/motif-library.js");
    download_with_header(
        &manifest_dir.join("public/ertl_npsubstituents.txt"),
        "https://peter-ertl.com/molecular/data/npsubstituents.txt",
        "# Source: Peter Ertl npsubstituents.txt from https://peter-ertl.com/molecular/data/npsubstituents.txt\n",
    );
}

fn download_with_header(dst: &Path, url: &str, header: &str) {
    let output = Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .unwrap_or_else(|err| panic!("failed to run curl for {url}: {err}"));
    if !output.status.success() {
        panic!(
            "failed to download {url}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut out = String::new();
    out.push_str(header);
    if !header.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&String::from_utf8_lossy(&output.stdout));
    out.push('\n');
    fs::write(dst, out).unwrap_or_else(|err| panic!("failed to write {}: {err}", dst.display()));
}
