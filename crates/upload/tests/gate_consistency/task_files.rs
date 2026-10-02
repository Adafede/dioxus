// The `task_files` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
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
