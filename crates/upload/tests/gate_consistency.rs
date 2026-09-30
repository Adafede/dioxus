// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! The local gate, the git hooks and the CI gate must be the same gate.
//!
//! `just ci`, `prek.toml` and `.github/workflows/ci.yml` are three hand-written
//! lists of the same checks, and hand-written lists drift. They had: `just ci`
//! ran checks CI did not, CI ran a job `just ci` did not, the WASM clippy job
//! never compiled `crates/upload`'s three `#[cfg(wasm32)]` modules, and
//! `just doc` built the docs without failing on a rustdoc warning.
//!
//! `MAPPING` below is the correspondence, written out once. These tests hold it
//! to four things: it covers every CI job, every recipe it names really exists,
//! `just ci` runs exactly the part of it meant to be local, and every gated
//! recipe has a hook. Adding a check to one file and forgetting the others now
//! fails the build.
//!
//! ## Why this test lives in `crates/upload/tests/`
//!
//! The workspace has no root package, so there is no `tests/` directory that a
//! `--workspace` build is obliged to compile. `upload` is the one crate every
//! app depends on, so its `tests/` directory is compiled by `just test`, by
//! `just clippy --all-targets` and by the CI test job alike, on the host and on
//! `wasm32`. Put the test anywhere else and one of those three stops running it.
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

/// One CI job, the local recipes that cover it, and whether it gates.
struct Job {
    name: &'static str,
    /// Recipes whose combined commands are what this job runs.
    recipes: &'static [&'static str],
    /// `false` for the jobs `just ci` deliberately leaves to `just ci-slow`.
    in_local_gate: bool,
    /// Why, for a job that is not in the local gate.
    note: &'static str,
}

/// The correspondence, in CI's order.
const MAPPING: &[Job] = &[
    Job {
        name: "fmt",
        recipes: &["fmt"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "license-headers",
        recipes: &["license-headers"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "clippy",
        recipes: &["clippy"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "test",
        recipes: &["test"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "doc",
        recipes: &["doc"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "wasm",
        recipes: &["wasm"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "clippy-wasm",
        recipes: &["clippy-wasm"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "machete",
        recipes: &["machete"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "readme",
        recipes: &["readme"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "supply-chain",
        recipes: &["deny", "audit"],
        in_local_gate: true,
        note: "",
    },
    Job {
        name: "mutants",
        recipes: &["mutants"],
        in_local_gate: false,
        note: "non-blocking in CI: 170 of 421 mutants survive today, so a gate would be \
               permanently red; 421 mutants also take ~25 min",
    },
];

/// Checks `just ci` runs that have no CI job, and why.
const LOCAL_ONLY: &[(&str, &str)] = &[(
    "check",
    "a bare `cargo check`; the lints that follow already build everything",
)];

/// Recipes that are neither in `just ci` nor a CI job: run by hand, on purpose,
/// with a stated reason. Asserted in both directions so a stale entry cannot
/// quietly excuse a check that should be gating.
const UNGATED: &[(&str, &str)] = &[
    (
        "mutants-mgf",
        "717 mutants in `mgf-precursor-erro-rs`, about half an hour; a separate \
         recipe so `mutants` fits inside a CI job's budget",
    ),
    (
        "mutants-ui",
        "the Dioxus rendering crates `mutants.toml` excludes; minutes per run, and \
         a surviving mutant there is usually a string in a `class` attribute",
    ),
    (
        "mutants-list",
        "lists the mutants without running them, so a change can be sized before \
         the run is paid for",
    ),
    (
        "outdated",
        "a version report: it fails on any dependency at all, so as a gate it \
         would be red from the day it was added, and as a hook it could only \
         print noise on every push",
    ),
];

/// The recipes `just ci` runs, in order.
fn ci_recipes(justfile: &str) -> Vec<String> {
    let body = justfile
        .split_once("\nci:\n")
        .map(|(_, rest)| rest)
        .unwrap_or_default()
        .lines()
        .take_while(|line| line.starts_with('\t') || line.trim().is_empty());
    body.filter_map(|line| line.trim().strip_prefix("just ").map(str::to_string))
        .collect()
}

/// Every recipe name defined in the justfile.
fn all_recipes(justfile: &str) -> BTreeSet<String> {
    justfile
        .lines()
        .filter_map(|line| {
            // A recipe starts at column zero, is `name:`, and is not a comment.
            if line.is_empty() || line.starts_with([' ', '\t', '#']) {
                return None;
            }
            let (name, _) = line.split_once(':')?;
            let name = name.trim();
            if name.is_empty() || name.contains(char::is_whitespace) {
                return None;
            }
            Some(name.to_string())
        })
        .collect()
}

/// The recipes the git hooks delegate to.
///
/// Every hook is `just <recipe>` with no cargo flags of its own, so the flags
/// cannot drift — that is the point of delegating. What can still drift is the
/// *name*: a renamed or deleted recipe leaves the hook pointing at nothing and
/// failing on every push, and a new check can go into `just ci` with no hook to
/// catch it before the commit lands.
fn hooked_recipes(hooks: &str) -> BTreeSet<String> {
    let mut recipes = BTreeSet::new();
    for line in hooks.lines() {
        if let Some((_, rest)) = line.split_once("entry = \"just ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            if !name.is_empty() {
                recipes.insert(name);
            }
        }
    }
    recipes
}

/// The CI job names, read from the `jobs:` block only.
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

#[test]
fn the_mapping_covers_every_ci_job() -> Result<()> {
    let jobs = ci_jobs(&read(".github/workflows/ci.yml")?)?;
    let mapped: BTreeSet<&str> = MAPPING.iter().map(|job| job.name).collect();
    let unmapped: Vec<&String> = jobs
        .iter()
        .filter(|job| !mapped.contains(job.as_str()))
        .collect();

    assert!(
        unmapped.is_empty(),
        "CI has jobs {unmapped:?} that `MAPPING` does not cover. Add a row, or the \
         gate is silently unenforced locally."
    );
    assert!(
        !mapped.is_empty(),
        "MAPPING is empty, so this test proves nothing"
    );
    Ok(())
}

#[test]
fn every_mapped_recipe_exists() -> Result<()> {
    let defined = all_recipes(&read("justfile")?);
    let mut missing = Vec::new();
    for job in MAPPING {
        for recipe in job.recipes {
            if !defined.contains(*recipe) {
                missing.push(format!("{} -> {recipe}", job.name));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "MAPPING names recipes the justfile does not define: {missing:?}"
    );
    Ok(())
}

#[test]
fn just_ci_runs_exactly_the_mapped_local_gate() -> Result<()> {
    let expected: BTreeSet<String> = MAPPING
        .iter()
        .filter(|job| job.in_local_gate)
        .flat_map(|job| job.recipes.iter().map(|r| (*r).to_string()))
        .chain(LOCAL_ONLY.iter().map(|(name, _)| (*name).to_string()))
        .collect();

    let actual: BTreeSet<String> = ci_recipes(&read("justfile")?).into_iter().collect();
    let missing: Vec<&String> = expected.difference(&actual).collect();
    let extra: Vec<&String> = actual.difference(&expected).collect();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "`just ci` and MAPPING disagree. Missing from `just ci`: {missing:?}. \
         In `just ci` but not accounted for: {extra:?}. Add a row to MAPPING or \
         LOCAL_ONLY rather than editing the recipe alone."
    );
    Ok(())
}

#[test]
fn the_slow_jobs_are_reachable_and_documented() -> Result<()> {
    let justfile = read("justfile")?;
    let in_fast_gate = ci_recipes(&justfile);
    for job in MAPPING.iter().filter(|job| !job.in_local_gate) {
        assert!(
            !job.note.is_empty(),
            "{} is out of the local gate with no stated reason",
            job.name
        );
        for recipe in job.recipes {
            assert!(
                justfile.contains(&format!("\tjust {recipe}\n")),
                "`{recipe}` is not run by `just ci-slow`, so {} can only be run by hand",
                job.name
            );
            assert!(
                !in_fast_gate.contains(&recipe.to_string()),
                "`{recipe}` is in `just ci` but MAPPING marks it slow"
            );
        }
    }
    Ok(())
}

#[test]
fn every_hook_points_at_a_recipe_that_exists() -> Result<()> {
    let defined = all_recipes(&read("justfile")?);
    let missing: Vec<String> = hooked_recipes(&read("prek.toml")?)
        .into_iter()
        .filter(|r| !defined.contains(r))
        .collect();
    assert!(
        missing.is_empty(),
        "prek.toml delegates to {missing:?}, which the justfile does not define. \
         Every hook fails on push until the recipe is restored or the hook is cut."
    );
    Ok(())
}

#[test]
fn every_gated_recipe_has_a_hook() -> Result<()> {
    let hooked = hooked_recipes(&read("prek.toml")?);
    let mut missing: Vec<String> = ci_recipes(&read("justfile")?)
        .into_iter()
        .filter(|r| !hooked.contains(r))
        .collect();
    // `mutants` is `just ci-slow`: 421 mutants have no business running on
    // someone's commit, and the survivors are a to-do list rather than a gate.
    missing.retain(|r| r != "mutants");
    assert!(
        missing.is_empty(),
        "`just ci` runs {missing:?} with no prek hook. Add one at the stage matching \
         its cost, so the failure happens before the push."
    );
    Ok(())
}

#[test]
fn the_exceptions_are_all_real() -> Result<()> {
    // A stale entry in either list is how the next genuine drift gets waved
    // through, so both directions are asserted.
    let justfile = read("justfile")?;
    let run = ci_recipes(&justfile);
    let mapped: BTreeSet<&str> = MAPPING
        .iter()
        .flat_map(|job| job.recipes.iter().copied())
        .collect();

    let defined = all_recipes(&justfile);
    let hooked = hooked_recipes(&read("prek.toml")?);

    for (recipe, why) in LOCAL_ONLY {
        assert!(
            !why.is_empty(),
            "LOCAL_ONLY names `{recipe}` with no reason"
        );
        assert!(
            run.contains(&recipe.to_string()),
            "LOCAL_ONLY names `{recipe}`, which `just ci` does not run"
        );
        assert!(
            !mapped.contains(recipe),
            "LOCAL_ONLY names `{recipe}`, which a CI job covers"
        );
        assert!(
            defined.contains(*recipe),
            "LOCAL_ONLY names `{recipe}`, which the justfile does not define"
        );
    }

    for (recipe, why) in UNGATED {
        assert!(!why.is_empty(), "UNGATED names `{recipe}` with no reason");
        assert!(
            defined.contains(*recipe),
            "UNGATED names `{recipe}`, which the justfile does not define"
        );
        assert!(
            !run.contains(&recipe.to_string()),
            "UNGATED names `{recipe}`, which `just ci` runs — move it to MAPPING \
             or LOCAL_ONLY"
        );
        assert!(
            !mapped.contains(recipe),
            "UNGATED names `{recipe}`, which a CI job covers"
        );
        assert!(
            !hooked.contains(*recipe),
            "UNGATED names `{recipe}`, which has a git hook — it is being run \
             automatically, so it is not ungated"
        );
    }
    Ok(())
}

#[test]
fn the_declarations_are_not_vacuous() -> Result<()> {
    // So a scraper that silently finds nothing cannot pass everything.
    let justfile = read("justfile")?;
    let run = ci_recipes(&justfile);
    assert!(run.len() >= 10, "`just ci` looks truncated: {run:?}");
    assert!(MAPPING.len() >= 9, "MAPPING looks truncated");
    assert!(
        all_recipes(&justfile).contains("ci-slow"),
        "`just ci-slow` is referenced by the comments and must exist"
    );
    assert!(
        hooked_recipes(&read("prek.toml")?).len() >= 12,
        "prek.toml looks truncated"
    );
    let jobs = ci_jobs(&read(".github/workflows/ci.yml")?)?;
    assert!(jobs.len() >= 9, "the CI scrape found too few jobs");
    Ok(())
}

/// The checks whose value is that they *fail*, not that they run.
///
/// Each of these is a line in a recipe that a passing gate could have without it,
/// and a check that cannot fail is worse than no check: it reports a green run
/// for a thing it never looked at.
#[test]
fn the_failing_checks_still_fail_on_failure() -> Result<()> {
    let justfile = read("justfile")?;
    let ci = read(".github/workflows/ci.yml")?;

    // `cargo doc` without this builds the docs and prints the warning, so the
    // recipe would pass on a broken intra-doc link — which is a 404 in the
    // generated documentation and nothing else.
    assert!(
        justfile.contains("doc:\n\tRUSTDOCFLAGS=\"-D warnings\""),
        "`just doc` must run rustdoc with `-D warnings`, or it only proves the \
         docs build"
    );
    assert!(
        ci.contains("RUSTDOCFLAGS: -D warnings"),
        "the CI doc job must set RUSTDOCFLAGS, for the same reason"
    );

    // The three `crates/upload` modules exist only under
    // `#[cfg(target_arch = \"wasm32\")]`, so nothing else compiles their code or
    // their tests. A test calling a method that does not exist lived in one of
    // them for the whole life of this gate.
    assert!(
        justfile.contains("cargo clippy -p upload --target wasm32-unknown-unknown --all-targets"),
        "`just clippy-wasm` must lint `upload` on its own, tests included"
    );
    assert!(
        ci.contains("cargo clippy -p upload"),
        "the CI clippy-wasm job must do the same, or the gap is only closed locally"
    );

    // The license-header check is one `find` over the tree; a `grep -x` on the
    // copyright prefix would make it match nothing and pass forever, so assert
    // the shape it needs.
    assert!(
        justfile.contains("SPDX-License-Identifier: AGPL-3.0-only"),
        "`just license-headers` must name the licence it enforces"
    );
    assert!(
        !justfile.contains("grep -qxF '// SPDX-FileCopyrightText"),
        "the copyright line is a prefix, not an exact string: `grep -x` on it can \
         never match and the check would pass on every file"
    );
    Ok(())
}

/// `cargo-mutants` only reads `.cargo/mutants.toml` unless it is passed a
/// `--config` flag, and nothing in the gate passes one. A config file at the
/// repository root — which is where the other config files in this repo live —
/// is therefore read by nothing, and every exclusion in it is a comment that
/// reads as a decision.
#[test]
fn the_mutants_config_is_where_cargo_mutants_looks_for_it() {
    assert!(
        repo_root().join(".cargo/mutants.toml").exists(),
        "`.cargo/mutants.toml` is missing. `cargo mutants` reads that path and no \
         other without `--config`, and no recipe or CI job passes one."
    );
    assert!(
        !repo_root().join("mutants.toml").exists(),
        "there is a `mutants.toml` at the repository root, which `cargo mutants` \
         never reads. Move it to `.cargo/mutants.toml` or pass `--config` \
         everywhere; two configs is worse than none."
    );
}

/// The two lists of mutant packages are written out twice — once in the recipe
/// and once in the CI job — and mutation testing is the one check whose result
/// is a number rather than a pass or a fail. If the two scopes drift, the
/// survivor count one of them reports is a claim about a run the other never
/// made. `mgf-precursor-erro-rs` is deliberately not here: it is
/// `just mutants-mgf`, at 717 mutants, and it does not fit a CI job's budget.
const MUTATION_SCOPE: &[&str] = &["json-count-rs", "cxsmiles-yoga"];

/// The recipe line, asserted verbatim. A set comparison would also pass if the
/// flags drifted into a different order on one side and not the other.
const LOCAL_MUTANTS_COMMAND: &str =
    "cargo mutants --package json-count-rs --package cxsmiles-yoga --jobs 8 --timeout 300";

#[test]
fn the_local_and_ci_mutation_scopes_agree() -> Result<()> {
    let justfile = read("justfile")?;
    let ci = read(".github/workflows/ci.yml")?;

    assert!(
        justfile.contains(LOCAL_MUTANTS_COMMAND),
        "`just mutants` does not run `{LOCAL_MUTANTS_COMMAND}`. Update \
         MUTATION_SCOPE and LOCAL_MUTANTS_COMMAND together, or the survivor \
         count the CI job reports is a claim about a run `just ci-slow` never \
         made."
    );

    let in_ci = mutated_packages(&ci);
    let expected: std::collections::BTreeSet<String> = MUTATION_SCOPE
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    assert_eq!(
        in_ci, expected,
        "the CI mutants job names a different set of packages than \
         MUTATION_SCOPE says it should"
    );
    Ok(())
}

/// Data that must never be committed, and the reason each one exists.
///
/// This is a list of paths, not a list of rules: it is here so that deleting a
/// `.gitignore` line has to be done in two places, and so that the two places
/// can be compared. The largest of these is 33 MB of third-party LIPID MAPS
/// data, and a licence is not something a later commit can take back.
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
/// Text, not `git check-ignore`: this has to run in a checkout with no `.git`
/// at all, and a subprocess that fails is a test that fails for the wrong
/// reason. A rule that is present but does not match is a different bug, and
/// the simplest way to rule it out is to assert the rule *is* the path.
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

///
/// Reads the token pair rather than the structure around it, because the same
/// command is written once as a justfile line and once as a folded YAML scalar
/// whose first line holds only `cargo mutants` and whose flags are on the lines
/// after it.
fn mutated_packages(text: &str) -> std::collections::BTreeSet<String> {
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
