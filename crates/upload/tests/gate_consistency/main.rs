// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! The local gate, the git hooks and the CI gate must be the same gate.
//!
//! `./mk ci`, `prek.toml` and `.github/workflows/ci.yml` are three
//! hand-maintained lists of the same checks, and hand-maintained lists drift.
//! They had: the local gate ran checks CI did not, CI ran jobs the local gate
//! did not, the WASM clippy job never compiled `crates/upload`'s three
//! `#[cfg(wasm32)]` modules, and the doc task built the docs without failing on
//! a rustdoc warning.
//!
//! `MAPPING` below is the correspondence, written out once. These tests hold it
//! to four things: it covers every CI job, every task it names really exists,
//! `./mk ci` runs exactly the part of it meant to be local, and every
//! gated task has a hook. Adding a check to one file and forgetting the others
//! now fails the build.
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

/// The `dependencies` of a task, read from its `[tasks."<name>"]` block.
///
/// Parsed by finding the table header and reading until the next one, rather
/// than by looking for `./mk <name>` anywhere in the file: a task that
/// *calls* another is not the same as a task that *depends on* it, and the
/// difference is exactly the drift being looked for.
fn task_dependencies(makefiles: &str, task: &str) -> Result<BTreeSet<String>> {
    let header = format!("[tasks.\"{task}\"]");
    let body = makefiles
        .split_once(&header)
        .map(|(_, rest)| rest)
        .ok_or_else(|| format!("no task named `{task}` in the makefiles"))?;
    // Up to the next table header, which is a line starting with `[`.
    let body = body
        .lines()
        .take_while(|line| !line.trim_start().starts_with('['))
        .collect::<Vec<_>>()
        .join("\n");
    let deps = body
        .split_once("dependencies")
        .map(|(_, rest)| rest)
        .ok_or_else(|| format!("task `{task}` has no dependencies list"))?;
    // The list, or the first line of it: cargo-make accepts both a single string
    // and an array, and the gate uses the array form.
    let list = deps
        .split_once('[')
        .map(|(_, rest)| rest)
        .or_else(|| deps.split_once('"').map(|(_, rest)| rest))
        .ok_or_else(|| format!("task `{task}` has an unreadable dependencies list"))?;
    let list = list.split_once(']').map_or(list, |(head, _)| head);
    Ok(list
        .split(',')
        .filter_map(|d| d.split('"').nth(1))
        .map(str::to_string)
        .collect())
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

/// The tasks a `prek.toml` hook delegates to.
///
/// The delegation is `./mk <task>`, not `cargo make <task>`, and the difference is
/// load-bearing: `./mk` holds `--no-workspace`, which is the only supported way to
/// stop cargo-make re-running every task once per workspace member. A hook that
/// invoked `cargo make` directly would run its check eight times.
fn hooked_tasks(hooks: &str) -> BTreeSet<String> {
    let mut tasks = BTreeSet::new();
    for line in hooks.lines() {
        if let Some((_, rest)) = line.split_once("entry = \"./mk ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            if !name.is_empty() {
                tasks.insert(name);
            }
        }
    }
    tasks
}

/// Every task a hook covers, whether by `./mk` delegation or by prek
/// running the tool itself.
fn covered_by_a_hook(hooks: &str) -> BTreeSet<String> {
    let mut covered = hooked_tasks(hooks);
    // `typos` and `tombi` are covered by prek's own repo hooks rather than by a
    // `./mk` delegation: prek installs and runs those tools itself. The
    // gate still has a `./mk typos` task for CI, but the coverage comes
    // from the repo hook. Counting them here is what stops
    // `every_gated_task_has_a_hook` demanding a duplicate delegation for a task
    // that is already covered twice.
    if hooks.contains("crate-ci/typos") {
        covered.insert("typos".to_owned());
    }
    if hooks.contains("tombi-pre-commit") {
        covered.insert("tombi-check".to_owned());
        covered.insert("tombi-lint".to_owned());
        covered.insert("tombi-fmt".to_owned());
    }
    covered
}

/// The CI job names across every workflow, read from the `jobs:` block only.
fn ci_jobs(yaml: &str) -> Result<BTreeSet<String>> {
    let after = yaml
        .split_once("\njobs:\n")
        .map(|(_, rest)| rest)
        .ok_or("the workflow has no `jobs:` block")?;
    Ok(after
        .lines()
        .take_while(|line| line.trim().is_empty() || line.starts_with(' '))
        .filter_map(|line| {
            let indent = line.len() - line.trim_start().len();
            (indent == 2)
                .then(|| line.trim().strip_suffix(':'))
                .flatten()
                .map(str::to_string)
        })
        .filter(|name| !name.starts_with('#'))
        .collect())
}

/// The workflows `MAPPING` covers, as `(file, job)` pairs.
///
/// The `jobs:` scrape is per file, so the file has to be named. Two workflows
/// are in scope: `ci.yml` is the per-push gate, `scheduled.yml` is the weekly
/// one. The other workflows in `.github/workflows/` -- `CodeQL`, dependency
/// review and the Pages deploy -- run no `./mk` task and are not part of
/// this mapping; `codeql` and `dependency-review` are GitHub's own analyses of
/// the code rather than checks this repository defines.
const MAPPED_WORKFLOWS: &[&str] = &[
    ".github/workflows/ci.yml",
    ".github/workflows/scheduled.yml",
];

/// The workflows that deliberately run no `./mk` task, and why. Asserted
/// in both directions, so adding a fifth workflow without a row fails.
const UNMAPPED_WORKFLOWS: &[(&str, &str)] = &[
    (
        ".github/workflows/codeql.yml",
        "GitHub's own static analysis of the code; it has no cargo flags to drift",
    ),
    (
        ".github/workflows/dependency-review.yml",
        "GitHub's own advisory check on the dependency diff of a pull request",
    ),
    (
        ".github/workflows/deploy.yml",
        "builds the wasm bundles and publishes them to Pages; it needs a dx \
         build per app and writes to Pages, which is not a code check",
    ),
];

/// Every CI job across the mapped workflows, as `(file, job)`.
fn all_mapped_jobs() -> Result<Vec<(String, String)>> {
    let mut jobs = Vec::new();
    for file in MAPPED_WORKFLOWS {
        for job in ci_jobs(&read(file)?)? {
            jobs.push(((*file).to_owned(), job));
        }
    }
    Ok(jobs)
}

/// The `./mk <task>` commands a job runs, for one workflow file.
fn tasks_a_job_runs(yaml: &str, job: &str) -> BTreeSet<String> {
    let mut tasks = BTreeSet::new();
    let mut inside = false;
    for line in yaml.lines() {
        if line == format!("  {job}:") {
            inside = true;
            continue;
        }
        // A new job starts at column 2, so this ends the block.
        if inside && line.starts_with("  ") && !line.starts_with("   ") && line.ends_with(':') {
            break;
        }
        if !inside {
            continue;
        }
        // Both `run: ./mk x` and `- run: ./mk x` appear.
        if let Some((_, rest)) = line.split_once("./mk ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            if !name.is_empty() {
                tasks.insert(name);
            }
        }
    }
    tasks
}

/// One CI job, the local tasks that cover it, and whether it gates.
struct Job {
    /// The workflow file, so the same job name in two files is two rows.
    file: &'static str,
    name: &'static str,
    /// Tasks whose combined commands are what this job runs.
    tasks: &'static [&'static str],
    /// `false` for the jobs `./mk ci` deliberately leaves out, or that CI
    /// runs and the local gate cannot.
    in_local_gate: bool,
    /// Why, for a job that is not in the local gate. Asserted non-empty.
    note: &'static str,
}

/// The correspondence, in CI's order.
const MAPPING: &[Job] = &[
    Job {
        file: ".github/workflows/ci.yml",
        name: "ci",
        tasks: &["ci"],
        in_local_gate: true,
        note: "",
    },
    Job {
        file: ".github/workflows/ci.yml",
        name: "msrv",
        // Not a task: the point of the job is to build with the oldest
        // supported toolchain, which `./mk` cannot express because the
        // toolchain is pinned in `rust-toolchain.toml` and rustup reads that
        // file. Named as the two tasks it runs, so the "every named task exists"
        // check still holds and the job is not silently unmapped.
        tasks: &["check", "test"],
        in_local_gate: false,
        note: "builds on the MSRV rather than the pinned toolchain, which is a \
               different thing from anything the local gate can do",
    },
    Job {
        file: ".github/workflows/ci.yml",
        name: "wasm",
        tasks: &["web-build-all", "web-size"],
        in_local_gate: false,
        note: "the only job that links a .wasm; it needs a dx build and \
               binaryen, so it is minutes and it needs a toolchain the gate does \
               not install",
    },
    Job {
        file: ".github/workflows/scheduled.yml",
        name: "mutants",
        tasks: &["mutants"],
        in_local_gate: false,
        note: "non-blocking in CI: 13 of cxsmiles-yoga's 290 mutants survive \
               today, so a gate would be permanently red; 421 mutants also take \
               ~25 min",
    },
    Job {
        file: ".github/workflows/scheduled.yml",
        name: "coverage",
        tasks: &["cov"],
        in_local_gate: false,
        note: "rebuilds the whole crate graph under instrumentation, so it is \
               minutes; the report is wanted weekly rather than on every push",
    },
    Job {
        file: ".github/workflows/scheduled.yml",
        name: "dependencies",
        tasks: &["outdated", "udeps", "geiger", "msrv", "action-pins"],
        in_local_gate: false,
        note: "four questions about the dependency tree, three of which are \
               reports rather than gates and one of which bisects a build per \
               toolchain",
    },
];

/// The aggregate tasks: entry points rather than leaf checks.
///
/// `ci-fast`, `ci-slow` and `setup` are *run* rather than *depended on*, so they
/// belong in the gate's own vocabulary and not in the set of leaves `ci` depends
/// on. Asserted in `the_exceptions_are_all_real`.
const AGGREGATES: &[&str] = &["ci-fast", "ci-slow", "setup"];

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
mod ci_mapping;
mod data_files;
mod hooks;
mod mk_runner;
mod mutants;
mod task_files;
mod workflows;
