// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Repository checks the gate can run cheaply.
//!
//! That the local gate, the git hooks and CI are one gate is structural:
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
//! ## Why this lives in `crates/workspace-gates/tests/`
//!
//! The workspace has no root package, so there is no `tests/` directory that a
//! `--workspace` build is obliged to compile. Hosting these repository-wide
//! checks in a dedicated private crate keeps `upload` focused on streaming and
//! browser helpers while still compiling and running under `./mk test` and
//! `./mk lint --all-targets`.
//!
//! The tests return `Result` rather than panicking, because the workspace denies
//! `clippy::unwrap_used` and `clippy::expect_used` and that reaches test targets.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> Result<String> {
    let full = repo_root().join(path);
    std::fs::read_to_string(&full).map_err(|e| format!("{}: {e}", full.display()).into())
}

fn normalize_rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn has_extension(path: &Path, exts: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| exts.contains(&ext))
}

fn collect_dir(
    root: &Path,
    dir: &Path,
    recursive: bool,
    exts: &[&str],
    out: &mut BTreeMap<String, PathBuf>,
) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        if name == "target" || name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            if recursive {
                collect_dir(root, &path, true, exts, out)?;
            }
            continue;
        }
        if has_extension(&path, exts) {
            out.insert(normalize_rel(root, &path), path);
        }
    }
    Ok(())
}

/// Sorted list of `.toml` task files under `make/`.
fn make_toml_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(root.join("make"))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    // Sorted so a failure names the same files in the same order every time.
    entries.sort();
    Ok(entries)
}

/// Every task file, concatenated: the root `Makefile.toml` and each file under
/// `make/`.
fn makefiles() -> Result<String> {
    let root = repo_root();
    let mut all = std::fs::read_to_string(root.join("Makefile.toml"))
        .map_err(|e| format!("Makefile.toml: {e}"))?;
    for path in make_toml_paths(&root)? {
        all.push('\n');
        all.push_str(&std::fs::read_to_string(&path)?);
    }
    Ok(all)
}

mod apps;
mod data_files;
mod mk_runner;
mod mutants;
mod surfaces;
mod task_files;
mod workflows;
