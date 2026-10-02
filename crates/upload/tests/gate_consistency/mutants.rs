// The `mutants` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
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
