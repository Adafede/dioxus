// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `mutants` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in `main.rs` -- `read`, `makefiles` and the rest --
// so each module opens with `use super::*`. Split by what is being checked
// rather than by size: the groups fail for different reasons and get read
// separately.
use super::*;

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

/// The mutation scope is written out once, as `MUTANTS_PACKAGES` in `mk.local`,
/// which `./mk mutants` -- the bounded runner -- mutates, and which the weekly
/// job's survivor count is a claim about. The `mutants-list` task, which shows
/// that scope without running it, writes it out a second time; if the two
/// drift, the list is of a run nobody makes.
#[test]
fn the_mutants_task_names_a_scope() -> Result<()> {
    let scope: BTreeSet<String> = read("mk.local")?
        .lines()
        .find_map(|line| line.trim().strip_prefix("MUTANTS_PACKAGES=("))
        .and_then(|rest| rest.split(')').next())
        .map_or_default(|names| names.split_whitespace().map(str::to_string).collect());
    assert!(
        !scope.is_empty(),
        "no `MUTANTS_PACKAGES` in `mk.local`, so `./mk mutants` mutates the whole \
         workspace, rendering crates included"
    );
    assert!(
        read("mk")?.contains("    mutants)"),
        "`./mk` has no `mutants` gate to run the scope in `mk.local`"
    );
    let listed: String = makefiles()?
        .lines()
        .filter(|line| line.contains("cargo mutants") && line.contains("--list"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        mutated_packages(&listed),
        scope,
        "`mutants-list` lists a different scope from `MUTANTS_PACKAGES` in `mk.local`"
    );
    Ok(())
}
