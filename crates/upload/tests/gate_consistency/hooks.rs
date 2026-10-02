// The `hooks` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
// separately.
use super::*;

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
