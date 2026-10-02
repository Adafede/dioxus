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

/// One CI job, the local tasks that cover it, and whether it gates.
#[test]
fn the_mapping_covers_every_ci_job() -> Result<()> {
    let jobs = all_mapped_jobs()?;
    let mut unmapped = Vec::new();
    for (file, job) in &jobs {
        let covered = MAPPING
            .iter()
            .any(|m| m.file == file && m.name == job.as_str());
        if !covered {
            unmapped.push(format!("{file}: {job}"));
        }
    }

    assert!(
        unmapped.is_empty(),
        "CI has jobs {unmapped:?} that `MAPPING` does not cover. Add a row, or \
         the gate is silently unenforced locally."
    );
    assert!(
        !MAPPING.is_empty(),
        "MAPPING is empty, so this test proves nothing"
    );
    Ok(())
}

#[test]
fn every_mapped_task_exists() -> Result<()> {
    let defined = all_tasks(&makefiles()?);
    let mut missing = Vec::new();
    for job in MAPPING {
        for task in job.tasks {
            if !defined.contains(*task) {
                missing.push(format!("{} -> {task}", job.name));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "MAPPING names tasks the makefiles do not define: {missing:?}"
    );
    Ok(())
}

/// Every CI job runs at least one task that exists, and runs at least one task
/// at all.
///
/// A job that names a task which does not exist is a red build nobody can
/// explain, because the job fails with "task not found" rather than with anything
/// about the change. A job that runs no task is a job that nothing holds to the
/// local gate, whatever its name says.
#[test]
fn every_ci_job_runs_a_task_that_exists() -> Result<()> {
    let makefiles = makefiles()?;
    let defined = all_tasks(&makefiles);
    let mut bad = Vec::new();
    let mut silent = Vec::new();

    for file in MAPPED_WORKFLOWS {
        let yaml = read(file)?;
        for job in ci_jobs(&yaml)? {
            let tasks = tasks_a_job_runs(&yaml, &job);
            if tasks.is_empty() {
                silent.push(format!("{file}: {job}"));
            }
            for task in tasks {
                if !defined.contains(&task) {
                    bad.push(format!(
                        "{file}: {job} runs `./mk {task}`, which does not exist"
                    ));
                }
            }
        }
    }

    assert!(bad.is_empty(), "{}", bad.join("\n"));
    assert!(
        silent.is_empty(),
        "these CI jobs run no `./mk` task, so nothing holds them to the \
         local gate: {silent:?}"
    );
    Ok(())
}

#[test]
fn the_local_gate_runs_exactly_the_mapped_local_jobs() -> Result<()> {
    // The `ci` job runs `./mk ci`, and `ci` is a fan-out over the leaf
    // checks. So the comparison is between what `ci` *depends on* and the union
    // of the local rows: the leaf tasks for the `ci` row (which is the gate
    // itself), and the named tasks for every other local row.
    let makefiles = makefiles()?;
    let mut expected: BTreeSet<String> = MAPPING
        .iter()
        .filter(|job| job.in_local_gate)
        .flat_map(|job| {
            // A row whose single task is the gate expands to that gate's
            // dependencies; anything else is a literal task name.
            if job.tasks == ["ci"] {
                task_dependencies(&makefiles, "ci")
                    .unwrap_or_default()
                    .into_iter()
                    .collect::<Vec<_>>()
            } else {
                job.tasks.iter().map(|t| (*t).to_string()).collect()
            }
        })
        .collect();
    // The aggregates are the pre-push and one-off entry points, not leaf checks,
    // and they are *run* rather than *depended on*.
    for aggregate in AGGREGATES {
        expected.remove(*aggregate);
    }

    let actual = task_dependencies(&makefiles, "ci")?;
    let missing: Vec<&String> = expected.difference(&actual).collect();
    let extra: Vec<&String> = actual.difference(&expected).collect();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "`./mk ci` and MAPPING disagree. In MAPPING but not in `ci`: \
         {missing:?}. In `ci` but not accounted for: {extra:?}. Add a row to \
         MAPPING rather than editing the task alone."
    );
    Ok(())
}

/// CI is the superset: a check in the pre-push gate but not in CI means a
/// contributor is paying for something CI never checks.
#[test]
fn ci_fast_is_a_subset_of_ci() -> Result<()> {
    let makefiles = makefiles()?;
    let full = task_dependencies(&makefiles, "ci")?;
    let fast = task_dependencies(&makefiles, "ci-fast")?;
    let extra: Vec<&String> = fast.difference(&full).collect();
    assert!(
        extra.is_empty(),
        "`ci-fast` runs {extra:?}, which `ci` does not. CI is meant to be the \
         superset; a check only in the pre-push gate is one CI never runs."
    );
    Ok(())
}

#[test]
fn the_slow_jobs_are_reachable_and_documented() -> Result<()> {
    let makefiles = makefiles()?;
    let slow = task_dependencies(&makefiles, "ci-slow")?;
    for job in MAPPING.iter().filter(|job| !job.in_local_gate) {
        assert!(
            !job.note.is_empty(),
            "{}:{} is out of the local gate with no stated reason",
            job.file,
            job.name
        );
    }
    // `ci-slow` is referenced by the comments and by the developer docs, so it
    // has to exist; whether a given slow job is in it is a judgement about cost
    // rather than drift, so it is asserted non-empty rather than to match.
    assert!(
        !slow.is_empty(),
        "`ci-slow` is referenced by the comments and must run something"
    );
    assert!(
        all_tasks(&makefiles).contains("ci-slow"),
        "`ci-slow` must exist: it is what the comments and CONTRIBUTING point at"
    );
    Ok(())
}

#[test]
fn every_hook_points_at_a_task_that_exists() -> Result<()> {
    let defined = all_tasks(&makefiles()?);
    let missing: Vec<String> = hooked_tasks(&read("prek.toml")?)
        .into_iter()
        .filter(|t| !defined.contains(t))
        .collect();
    assert!(
        missing.is_empty(),
        "prek.toml delegates to {missing:?}, which the makefiles do not define. \
         Every hook fails on push until the task is restored or the hook is cut."
    );
    Ok(())
}

#[test]
fn every_gated_task_has_a_hook() -> Result<()> {
    let makefiles = makefiles()?;
    let hooks = read("prek.toml")?;
    let covered = covered_by_a_hook(&hooks);
    let ci = task_dependencies(&makefiles, "ci")?;
    let missing: Vec<String> = ci.into_iter().filter(|t| !covered.contains(t)).collect();
    assert!(
        missing.is_empty(),
        "`./mk ci` runs {missing:?} with no prek hook. Add one at the \
         stage matching its cost, so the failure happens before the push."
    );
    Ok(())
}

/// The workflows that run no `./mk` task must be listed as deliberate.
///
/// Asserted in both directions: a workflow in the repository that is in neither
/// `MAPPED_WORKFLOWS` nor `UNMAPPED_WORKFLOWS` has no statement about whether it
/// is covered, and a stale entry in `UNMAPPED_WORKFLOWS` is how the next
/// genuine drift gets waved through.
#[test]
fn every_workflow_is_accounted_for() -> Result<()> {
    let mut present = Vec::new();
    for entry in std::fs::read_dir(repo_root().join(".github/workflows"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|x| x == "yml") {
            present.push(format!(
                ".github/workflows/{}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }
    present.sort();

    let mapped: BTreeSet<&str> = MAPPED_WORKFLOWS.iter().copied().collect();
    let unmapped: BTreeSet<&str> = UNMAPPED_WORKFLOWS.iter().map(|(f, _)| *f).collect();

    let missing: Vec<&String> = present
        .iter()
        .filter(|f| !mapped.contains(f.as_str()) && !unmapped.contains(f.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "these workflows are in neither MAPPED_WORKFLOWS nor UNMAPPED_WORKFLOWS, \
         so nothing says whether they are covered: {missing:?}"
    );

    let stale: Vec<&str> = present
        .iter()
        .map(String::as_str)
        .filter(|f| !mapped.contains(f))
        .filter(|f| {
            !UNMAPPED_WORKFLOWS
                .iter()
                .any(|(file, why)| file == f && !why.is_empty())
        })
        .collect();
    assert!(
        stale.is_empty(),
        "UNMAPPED_WORKFLOWS names {stale:?} with no reason, or names a workflow \
         that does not exist"
    );
    Ok(())
}

#[test]
fn the_exceptions_are_all_real() -> Result<()> {
    // A stale entry in the list is how the next genuine drift gets waved
    // through, so both directions are asserted.
    let makefiles = makefiles()?;
    let defined = all_tasks(&makefiles);
    let run = task_dependencies(&makefiles, "ci")?;
    let mapped: BTreeSet<&str> = MAPPING
        .iter()
        .flat_map(|job| job.tasks.iter().copied())
        .collect();

    for task in AGGREGATES {
        assert!(
            !mapped.contains(task),
            "AGGREGATES names `{task}`, which a CI job covers"
        );
        assert!(
            defined.contains(*task),
            "AGGREGATES names `{task}`, which the makefiles do not define"
        );
        assert!(
            !run.contains(*task),
            "AGGREGATES names `{task}`, which `./mk ci` runs -- it is an \
             entry point, not a leaf"
        );
    }
    Ok(())
}

/// The checks whose value is that they *fail*, not that they run.
///
/// Each of these is a line in a task that a passing gate could have without it,
/// and a check that cannot fail is worse than no check: it reports a green run
/// for a thing it never looked at.
#[test]
fn the_failing_checks_still_fail_on_failure() -> Result<()> {
    let makefiles = makefiles()?;
    let ci = read(".github/workflows/ci.yml")?;

    // `cargo doc` without this builds the docs and prints the warning, so the
    // task would pass on a broken intra-doc link -- which is a 404 in the
    // generated documentation and nothing else.
    assert!(
        makefiles.contains("RUSTDOCFLAGS=\"-D warnings"),
        "the `doc` task must run rustdoc with `-D warnings`, or it only proves \
         the docs build"
    );

    // The three `crates/upload` modules exist only under
    // `#[cfg(target_arch = \"wasm32\")]`, so nothing else compiles their code or
    // their tests. A test calling a method that does not exist lived in one of
    // them for the whole life of this gate.
    assert!(
        makefiles.contains("cargo clippy -p upload --target wasm32-unknown-unknown --all-targets"),
        "`lint-wasm-upload` must lint `upload` on its own, tests included"
    );
    assert!(
        ci.contains("./mk ci"),
        "the CI gate job must run `./mk ci` rather than its own list of \
         checks, or the two can drift"
    );

    // The license-header check is one `find` over the tree, in a script rather
    // than in the task; a `grep -x` on the copyright prefix would make it match
    // nothing and pass forever, so assert the shape it needs.
    let headers = read("make/scripts/license-headers.sh")?;
    assert!(
        headers.contains("SPDX-License-Identifier: AGPL-3.0-only"),
        "the license-header check must name the licence it enforces"
    );
    assert!(
        !headers.contains("grep -qxF '// SPDX-FileCopyrightText"),
        "the copyright line is a prefix, not an exact string: `grep -x` on it can \
         never match and the check would pass on every file"
    );
    assert!(
        makefiles.contains("make/scripts/license-headers.sh"),
        "the license-headers task must call the script; the check is the whole \
         gate otherwise"
    );

    // `extend` must be the first key in `Makefile.toml`. cargo-make accepts the
    // file with `[config]` above it, reports no error, and loads none of the
    // task files -- so every task is "not found" and the gate appears to have no
    // tasks at all.
    let root = read("Makefile.toml")?;
    let first_key = root
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .unwrap_or_default();
    assert!(
        first_key.starts_with("extend"),
        "the first key in Makefile.toml must be `extend`, and it is `{first_key}`. \
         With anything above it, cargo-make silently loads none of the task files."
    );

    // Every CI job installs the tools it calls. The `wasm` job needs binaryen as
    // well as `dx`, and the gate needs seven cargo-installed tools; a job that
    // calls one without installing it dies at 127, with a message about the
    // change under test.
    for job in ci_jobs(&ci)? {
        let block: String = ci
            .lines()
            .skip_while(|l| *l != format!("  {job}:"))
            .skip(1)
            .take_while(|l| l.starts_with("   ") || l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if block.contains("./mk ") {
            assert!(
                block.contains("cargo-make@"),
                "the {job} job runs `./mk` but does not install cargo-make"
            );
        }
    }
    Ok(())
}

/// `cargo-mutants` only reads `.cargo/mutants.toml` unless it is passed a
/// `--config` flag, and nothing in the gate passes one. A config file at the
/// repository root -- which is where most of the other config files in this
/// repository live -- is therefore read by nothing, and every exclusion in it is
/// a comment that reads as a decision.
#[test]
fn the_mutants_config_is_where_cargo_mutants_looks_for_it() {
    assert!(
        repo_root().join(".cargo/mutants.toml").exists(),
        "`.cargo/mutants.toml` is missing. `cargo mutants` reads that path and \
         no other without `--config`, and no task or CI job passes one."
    );
    assert!(
        !repo_root().join("mutants.toml").exists(),
        "there is a `mutants.toml` at the repository root, which `cargo mutants` \
         never reads. Move it to `.cargo/mutants.toml` or pass `--config` \
         everywhere; two configs is worse than none."
    );
}

/// The mutation scope is written out once in the `mutants` task and is what the
/// weekly job's survivor count is a claim about. If the scope and the claim
/// drift, the reported number is about a run nobody made.
#[test]
fn the_mutants_task_names_a_scope() -> Result<()> {
    let makefiles = makefiles()?;
    let scope = mutated_packages(&makefiles);
    assert!(
        !scope.is_empty(),
        "no `--package` in the makefiles, so `cargo mutants` has no scope to run"
    );
    for package in &scope {
        assert!(
            all_tasks(&makefiles).contains("mutants"),
            "the makefiles name `{package}` for mutation but define no `mutants` \
             task to run it"
        );
    }
    Ok(())
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

/// Every one of them has an ignore rule, and the rule is the path.
///
/// Text, not `git check-ignore`: this has to run in a checkout with no `.git` at
/// all, and a subprocess that fails is a test that fails for the wrong reason.
/// A rule that is present but does not match is a different bug, and the
/// simplest way to rule it out is to assert the rule *is* the path.
#[test]
fn the_local_data_files_are_ignored() -> Result<()> {
    let gitignore = read(".gitignore")?;
    let mut missing = Vec::new();

    for (path, why) in LOCAL_ONLY_DATA {
        let rule = format!("\n{path}\n");
        if !gitignore.contains(&rule) {
            missing.push(format!("{path} ({why})"));
        }
    }

    assert!(
        missing.is_empty(),
        "these are outputs of `smarts-evoliposuction` and its input data, not \
         source, and `git add -A` would commit them: {missing:?}. Add each to \
         `.gitignore`, or delete it here and in LOCAL_ONLY_DATA on purpose."
    );
    assert!(
        !LOCAL_ONLY_DATA.is_empty(),
        "LOCAL_ONLY_DATA is empty, so this test proves nothing"
    );
    Ok(())
}

/// `./mk` must exist, be executable, and hold `--no-workspace`.
///
/// Without the flag cargo-make re-runs every task once per workspace member:
/// eight executions of everything that is not a Cargo task, and the whole gate
/// measured at ~2 min against ~17s. There is no config key and no environment
/// variable for it in 0.37 -- `CARGO_MAKE_CRATE_IS_WORKSPACE`,
/// `CARGO_MAKE_WORKSPACE_EMULATION` and `CARGO_MAKE_WORKSPACE_SKIP_MEMBERS` were
/// all tried and all left the fan-out in place -- so the flag in this one file is
/// the whole mechanism, and a task run without it is a task run eight times.
#[test]
fn mk_holds_the_no_workspace_flag() -> Result<()> {
    let path = repo_root().join("mk");
    assert!(
        path.exists(),
        "`./mk` is missing: nothing has a way to run a task once"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)?.permissions().mode();
        assert!(
            mode & 0o111 != 0,
            "`./mk` is not executable (mode {mode:o}); the hooks and the workflow \
             call it directly, so they would fail with 126"
        );
    }

    let mk = read("mk")?;
    assert!(
        mk.contains("--no-workspace"),
        "`./mk` does not pass `--no-workspace`. Without it every task runs once \
         per workspace member; see the header comment in the file."
    );
    assert!(
        mk.contains("exec cargo make"),
        "`./mk` should `exec` the runner so a signal reaches it and the exit code \
         is the runner's"
    );
    Ok(())
}

/// Every `dtolnay/rust-toolchain` step must pass `toolchain:`.
///
/// The action's `toolchain` input is *required* and it does not fall back to
/// `rust-toolchain.toml`: a step without it exits 1 with "'toolchain' is a
/// required input", before a single check runs. Six steps across three workflows
/// were written that way, on the reasoning that rustup reads the pinned
/// file -- which is true, and is why those six steps do not exist any more, but
/// the action cannot be left in place with its required input unset.
///
/// The MSRV job is the one that must keep the action, because its whole point
/// is a toolchain other than the pinned one. That is why this asserts the input
/// rather than the absence of the action.
#[test]
fn every_rust_toolchain_action_passes_its_required_input() -> Result<()> {
    const REQUIRES_TOOLCHAIN: &str = "dtolnay/rust-toolchain";

    for file in [
        ".github/workflows/ci.yml",
        ".github/workflows/deploy.yml",
        ".github/workflows/scheduled.yml",
    ] {
        let yaml = read(file)?;
        let mut saw = false;
        for (n, line) in yaml.lines().enumerate() {
            if !line.contains(REQUIRES_TOOLCHAIN) {
                continue;
            }
            saw = true;
            // The `with:` block is the lines indented further than the `- uses:`.
            let rest = yaml.lines().skip(n + 1);
            let inputs: Vec<&str> = rest
                .take_while(|l| l.trim().is_empty() || l.starts_with("        "))
                .collect();
            assert!(
                inputs.iter().any(|l| l.contains("toolchain:")),
                "{file}:{} uses {REQUIRES_TOOLCHAIN} with no `toolchain:` input. \
                 The input is required and the step exits 1 before anything runs. \
                 Either give it the toolchain, or drop the action and let rustup \
                 read `rust-toolchain.toml`.",
                n + 1
            );
        }
        let _ = saw;
    }
    Ok(())
}

/// Nothing in the hooks or the workflows may invoke `cargo make` directly.
///
/// Every one of them delegates to `./mk`, and a bare `cargo make` in a hook is a
/// check that runs eight times — which reads as a slow hook and is not noticed,
/// because a check that runs eight times still passes.
#[test]
fn no_hook_or_workflow_bypasses_mk() -> Result<()> {
    for file in [
        ".github/workflows/ci.yml",
        ".github/workflows/deploy.yml",
        ".github/workflows/scheduled.yml",
    ] {
        let yaml = read(file)?;
        for (n, line) in yaml.lines().enumerate() {
            let code = line.split('#').next().unwrap_or("");
            if code.contains("cargo make") {
                return Err(format!(
                    "{file}:{} invokes `cargo make` directly. Use `./mk`, which holds \
                     `--no-workspace`: {line}",
                    n + 1
                )
                .into());
            }
        }
    }

    let hooks = read("prek.toml")?;
    for (n, line) in hooks.lines().enumerate() {
        if line.contains("entry = \"cargo make") {
            return Err(format!(
                "prek.toml:{} is a hook that runs `cargo make` directly, so it \
                 pays the eight-member fan-out on every commit: {line}",
                n + 1
            )
            .into());
        }
    }
    Ok(())
}

/// The `extend` indirection must actually name every file in `make/`.
///
/// A file added to `make/` and not added to `make/all.toml` is a task file
/// nothing loads, and `./mk --list-all-steps` shows the omission as an
/// absence rather than as an error.
#[test]
fn every_task_file_is_extended() -> Result<()> {
    let root = repo_root();
    let mut on_disk: Vec<String> = std::fs::read_dir(root.join("make"))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        // `all.toml` is the indirection `extend` names, so it is the one file
        // in this directory that is not in the list it holds.
        .filter(|p| p.file_name().is_some_and(|n| n != "all.toml"))
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    on_disk.sort();

    let all = read("make/all.toml")?;
    let named: BTreeSet<String> = all
        .lines()
        .filter_map(|line| line.trim().strip_prefix("{ path = "))
        .filter_map(|rest| rest.split('"').nth(1))
        .map(str::to_string)
        .collect();

    let missing: Vec<&String> = on_disk
        .iter()
        .filter(|f| !named.contains(f.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "these files are in `make/` but not in `make/all.toml`, so cargo-make \
         never loads them and their tasks do not exist: {missing:?}"
    );
    assert!(
        !named.is_empty(),
        "`make/all.toml` names no task files, so the gate has no tasks"
    );
    Ok(())
}

/// So a scraper that silently finds nothing cannot pass everything.
#[test]
fn the_declarations_are_not_vacuous() -> Result<()> {
    let makefiles = makefiles()?;
    let ci = task_dependencies(&makefiles, "ci")?;
    assert!(ci.len() >= 12, "`./mk ci` looks truncated: {ci:?}");
    assert!(MAPPING.len() >= 6, "MAPPING looks truncated");
    assert!(
        all_tasks(&makefiles).len() >= 40,
        "the makefiles look truncated: {} tasks",
        all_tasks(&makefiles).len()
    );
    assert!(
        hooked_tasks(&read("prek.toml")?).len() >= 12,
        "prek.toml looks truncated"
    );
    assert!(
        ci_jobs(&read(".github/workflows/ci.yml")?)?.len() >= 3,
        "the CI scrape found too few jobs"
    );
    Ok(())
}

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

/// Every app that renders a `DocumentHead`, against what its skip link points
/// at.
///
/// A skip link is two things that have to agree: an `href`, and a `main`
/// element carrying that id. Both are invisible to every linter, both are in a
/// different crate from the component that renders the link, and the failure
/// mode is a link that looks right in the source and goes nowhere in the
/// browser. The rule that un-hides it on focus lives in `ui`, so this only has
/// to check the pairing.
///
/// It also fails an app that renders a skip link pointing at nothing, and an
/// app with no skip link at all -- which is how `cxsmiles-yoga`,
/// `lipid-selecto-rs` and `smellfish-rs` had none until this test existed.
#[test]
fn every_app_pairs_its_skip_link_with_a_main_landmark() -> Result<()> {
    let root = repo_root();
    let mut apps: Vec<String> = std::fs::read_dir(root.join("apps"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    apps.sort();

    let mut missing: Vec<String> = Vec::new();

    for app in &apps {
        let src_dir = root.join("apps").join(app).join("src");
        let mut body = String::new();
        let mut stack = vec![src_dir];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    body.push_str(&std::fs::read_to_string(&path)?);
                    body.push('\n');
                }
            }
        }

        // An app with no `DocumentHead` is not rendering the head rules that
        // un-hide the link, so a skip link there would be broken by default.
        if !body.contains("DocumentHead") {
            if body.contains("skip_link {") {
                missing.push(format!("{app}: renders a skip link but no DocumentHead"));
            }
            continue;
        }

        let links = body.matches("skip_link {").count();
        // Scoped to the lines that actually open a `skip_link {` call: `target:`
        // is a common attribute name and the apps use it for other things.
        let targets: Vec<String> = body
            .lines()
            .filter(|line| line.contains("skip_link {"))
            .filter_map(|line| {
                line.find("target: \"").map(|i| {
                    line[i + 9..]
                        .split('"')
                        .next()
                        .unwrap_or_default()
                        .to_string()
                })
            })
            .collect();

        match (links, targets.len()) {
            (0, _) => missing.push(format!("{app}: no skip link at all")),
            (1, 0) => {
                if !body.contains("id: \"main-content\"") {
                    missing.push(format!(
                        "{app}: skip_link defaults to #main-content but no element has that id"
                    ));
                }
            }
            (1, 1) => {
                if let Some(target) = targets.first().map(|t| t.trim_start_matches('#')) {
                    let needle = format!("id: \"{target}\"");
                    if !body.contains(&needle) {
                        missing.push(format!("{app}: skip_link points at #{target}, no {needle}"));
                    }
                }
            }
            (n, t) => missing.push(format!(
                "{app}: {n} skip_link uses but {t} explicit targets, expected 1 and at most 1"
            )),
        }
    }

    assert!(
        missing.is_empty(),
        "skip-link wiring is wrong:\n  {}",
        missing.join("\n  ")
    );
    Ok(())
}
