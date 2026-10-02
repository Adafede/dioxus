// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `workflows` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
// separately.
use super::*;

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
