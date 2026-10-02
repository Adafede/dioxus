// The `ci_mapping` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
// separately.
use super::*;

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
