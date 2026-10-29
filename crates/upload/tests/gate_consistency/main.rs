// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Repository checks the gate can run cheaply.
//!
//! That the local gate, the git hooks and CI are one gate is now structural:
//! CI and the hooks only call `./mk`, and the standard rusti-graben `mk` pins and
//! version-checks every tool (`./mk tools`). What remains here is what that
//! cannot see: the apps' landmarks and file sizes, ignored data files, the
//! mutants configuration, the task files cargo-make loads, and the workflows'
//! use of `./mk`.
//!
//! ## The scraping is textual, and why
//!
//! There is no YAML or TOML parser in this workspace, and adding one to read two
//! config files would be a dependency for a test. What these checks need is
//! "which names appear under this key", which is decided by indentation and by a
//! prefix. Every scraper below asserts on its own output being non-empty, so one
//! that silently finds nothing fails rather than passes everything.
//!
//! The task list is read out of `Makefile.toml` and `make/*.toml` as *source*,
//! not through `./mk --list-all-steps`. Asking the runner to confirm
//! itself would be circular, and it would fail open in exactly the case that
//! matters: cargo-make loads no task file at all when `extend` is not the first
//! key in `Makefile.toml`, and reports that as an empty list rather than an
//! error.
//!
//! ## Why this test lives in `crates/upload/tests/`
//!
//! The workspace has no root package, so there is no `tests/` directory that a
//! `--workspace` build is obliged to compile. `upload` is the one crate every
//! app depends on, so its `tests/` directory is compiled by `./mk test`,
//! by `./mk lint --all-targets` and by the CI test job alike, on the host
//! and on `wasm32`. Put the test anywhere else and one of those three stops
//! running it.
//!
//! The tests return `Result` rather than panicking, because the workspace denies
//! `clippy::unwrap_used` and `clippy::expect_used` and that reaches test targets.

#![allow(unused_crate_dependencies)] // reads files; links nothing it needs to name

use std::collections::BTreeSet;
use std::error::Error;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// The `id` every app's `<main>` must carry, because it is the `href`
/// `ui::skip_link` hardcodes. Duplicated from `crates/ui/src/common.rs` on
/// purpose: this test reads app source as text and cannot import the constant
/// out of the component that renders it.
const MAIN_LANDMARK_ID: &str = "id: \"main-content\"";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> Result<String> {
    let full = repo_root().join(path);
    std::fs::read_to_string(&full).map_err(|e| format!("{}: {e}", full.display()).into())
}

/// Every task file, concatenated: the root `Makefile.toml` and each file under
/// `make/`.
fn makefiles() -> Result<String> {
    let root = repo_root();
    let mut all = std::fs::read_to_string(root.join("Makefile.toml"))
        .map_err(|e| format!("Makefile.toml: {e}"))?;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(root.join("make"))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    // Sorted so a failure names the same files in the same order every time.
    entries.sort();
    for path in entries {
        all.push('\n');
        all.push_str(&std::fs::read_to_string(&path)?);
    }
    Ok(all)
}

/// Every task name defined in the makefiles.
fn all_tasks(makefiles: &str) -> BTreeSet<String> {
    makefiles
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("[tasks.")?;
            let name = rest.split(']').next()?.trim_matches('"');
            (!name.is_empty() && !name.contains(char::is_whitespace)).then(|| name.to_string())
        })
        .collect()
}

/// Data that must never be committed, and the reason each one exists.
///
/// This is a list of paths, not a list of rules: it is here so that deleting a
/// `.gitignore` line has to be done in two places, and so the two places can be
/// compared. The largest of these is 33 MB of third-party LIPID MAPS data, and a
/// licence is not something a later commit can take back.
const LOCAL_ONLY_DATA: &[(&str, &str)] = &[
    (
        "/apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/LMSD.sdf.zip",
        "downloaded from lipidmaps.org by `smarts-evoliposuction download`",
    ),
    (
        "/apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/LMSD.sdf.tsv",
        "the same records as the zip, converted to TSV",
    ),
    (
        "/apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/example_lipids.smi",
        "the scratch input used while developing a rule set",
    ),
    (
        "/smarts_results.csv",
        "the tool's default output name, so it lands in whatever directory it was run from",
    ),
    (
        "/example_lipids.smi",
        "a copy of the scratch input, left in the repository root",
    ),
    ("/smiles_sets/", "70 MB of SMILES split out of the TSV"),
    (
        "/smarts_results_tested_smarts/",
        "the tested SMARTS per class",
    ),
];

/// Reads the token pair rather than the structure around it, because the same
/// command is written once as a task's inline script and once as a folded YAML
/// scalar whose first line holds only `./mk` and whose arguments are on
/// the lines after it.
fn mutated_packages(text: &str) -> BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        while let Some(word) = words.next() {
            if word == "--package"
                && let Some(name) = words.next()
            {
                names.insert(name.to_string());
            }
        }
    }
    names
}

mod apps;
mod data_files;
mod mk_runner;
mod mutants;
mod task_files;
mod workflows;
