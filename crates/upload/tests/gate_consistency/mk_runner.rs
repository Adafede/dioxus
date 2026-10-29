// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `mk_runner` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
// separately.
use super::*;

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
