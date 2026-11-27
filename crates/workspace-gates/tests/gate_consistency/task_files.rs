// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `task_files` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in `main.rs` -- `read`, `makefiles` and the rest --
// so each module opens with `use super::*`. Split by what is being checked
// rather than by size: the groups fail for different reasons and get read
// separately.
use super::*;

/// The `extend` indirection must actually name every file in `make/`.
///
/// A file added to `make/` and not added to `make/all.toml` is a task file
/// nothing loads, and `./mk --list-all-steps` shows the omission as an
/// absence rather than as an error.
#[test]
fn every_task_file_is_extended() -> Result<()> {
    let root = repo_root();
    let on_disk: Vec<String> = make_toml_paths(&root)?
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n != "all.toml"))
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();

    let all = read("make/all.toml")?;
    let named: BTreeSet<&str> = all
        .lines()
        .filter_map(|line| line.trim().strip_prefix("{ path = "))
        .filter_map(|rest| rest.split('"').nth(1))
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
