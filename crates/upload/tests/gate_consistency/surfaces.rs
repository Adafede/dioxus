// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Anti-drift gates for documentation surface separation, doctests, and ratchets.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{Result, read, repo_root};

const STALE_REMOVED_FILES: &[&str] = &[
    "PROJECT_METADATA.json",
    "apps/lipid-selecto-rs/assets/lipid_rules.yaml",
];

const FORBIDDEN_AGENT_TOKENS: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    "AGENT_AUDIT.md",
    ".github/ai",
    ".mcp",
    "graphify",
];

const FORBIDDEN_SLOP_WORDS: &[&str] = &["seamless", "cutting-edge", "leveraging", "blazing-fast"];

fn normalize_rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn has_extension(path: &Path, exts: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| exts.contains(&ext))
}

fn collect_dir(
    root: &Path,
    dir: &Path,
    recursive: bool,
    exts: &[&str],
    out: &mut BTreeMap<String, PathBuf>,
) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        if name == "target" || name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            if recursive {
                collect_dir(root, &path, true, exts, out)?;
            }
            continue;
        }
        if has_extension(&path, exts) {
            out.insert(normalize_rel(root, &path), path);
        }
    }
    Ok(())
}

fn human_doc_files(include_templates: bool) -> Result<Vec<(String, PathBuf)>> {
    let root = repo_root();
    let mut files = BTreeMap::new();

    for rel in ["README.md", "CHANGELOG.md", ".vscode/README.md"] {
        let path = root.join(rel);
        assert!(path.is_file(), "expected human doc {rel} is missing");
        files.insert(rel.to_owned(), path);
    }

    collect_dir(&root, &root.join("docs"), false, &["md"], &mut files)?;
    collect_dir(&root, &root.join(".github"), false, &["md"], &mut files)?;
    if include_templates {
        collect_dir(
            &root,
            &root.join(".github/ISSUE_TEMPLATE"),
            false,
            &["md", "yml"],
            &mut files,
        )?;
    }

    let sub_exts: &[&str] = if include_templates {
        &["md", "tpl"]
    } else {
        &["md"]
    };
    for tree in ["apps", "crates"] {
        collect_dir(&root, &root.join(tree), true, sub_exts, &mut files)?;
    }

    assert!(
        !files.is_empty(),
        "human_doc_files matched zero files; directory layout changed?"
    );
    Ok(files.into_iter().collect())
}

fn ignored_doctest_info(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let doc = trimmed
        .strip_prefix("///")
        .or_else(|| trimmed.strip_prefix("//!"))?
        .trim_start();
    doc.strip_prefix("```")
        .filter(|info| info.split([',', ' ']).any(|tok| tok.trim() == "ignore"))
}

fn is_ascii_word_match(haystack_lower: &str, needle: &str) -> bool {
    haystack_lower.match_indices(needle).any(|(start, _)| {
        let end = start + needle.len();
        let before_ok = haystack_lower
            .get(..start)
            .and_then(|prefix| prefix.chars().next_back())
            .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_');
        let after_ok = haystack_lower
            .get(end..)
            .and_then(|suffix| suffix.chars().next())
            .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_');
        before_ok && after_ok
    })
}

fn strip_inline_backticks(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut open_ticks = 0usize;
    while let Some(ch) = chars.next() {
        if ch == '`' {
            let mut run = 1usize;
            while chars.next_if_eq(&'`').is_some() {
                run += 1;
            }
            if open_ticks == 0 {
                open_ticks = run;
            } else if open_ticks == run {
                open_ticks = 0;
            }
        } else if open_ticks == 0 {
            out.push(ch);
        }
    }
    out
}

fn extract_markdown_link_targets(stripped: &str) -> Vec<&str> {
    let mut targets = Vec::new();
    let mut rest = stripped;
    while let Some((_, after)) = rest.split_once("](") {
        let Some((target, tail)) = after.split_once(')') else {
            break;
        };
        targets.push(target.trim());
        rest = tail;
    }
    targets
}

fn count_workspace_lint_allows(cargo_toml: &str) -> usize {
    let mut in_workspace_lints = false;
    let mut count = 0usize;
    for raw in cargo_toml.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_workspace_lints = line.starts_with("[workspace.lints.");
            continue;
        }
        if in_workspace_lints && line.contains("= \"allow\"") {
            count += 1;
        }
    }
    count
}

#[test]
fn removed_stale_artifacts_do_not_return() {
    let root = repo_root();
    let present: Vec<&str> = STALE_REMOVED_FILES
        .iter()
        .copied()
        .filter(|rel| root.join(rel).exists())
        .collect();
    assert!(
        present.is_empty(),
        "stale removed artifact(s) returned: {}",
        present.join(", ")
    );
}

#[test]
fn no_workspace_rust_source_uses_ignored_doctests() -> Result<()> {
    let root = repo_root();
    let mut rust_files = BTreeMap::new();
    for tree in ["apps", "crates"] {
        collect_dir(&root, &root.join(tree), true, &["rs"], &mut rust_files)?;
    }
    assert!(
        !rust_files.is_empty(),
        "no Rust sources found under apps/ or crates/"
    );

    let mut hits = Vec::new();
    for (rel, path) in rust_files {
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{rel}: {e}"))?;
        for (idx, line) in text.lines().enumerate() {
            if let Some(info) = ignored_doctest_info(line) {
                hits.push(format!("{rel}:{}: ```{info}", idx + 1));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "ignored doctests found (replace with compiled or `no_run` tests):\n  {}",
        hits.join("\n  ")
    );
    Ok(())
}

#[test]
fn human_docs_do_not_reference_agent_surface_or_slop() -> Result<()> {
    let docs = human_doc_files(true)?;
    let mut hits = Vec::new();

    for (rel, path) in docs {
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{rel}: {e}"))?;
        for (idx, line) in text.lines().enumerate() {
            let line_no = idx + 1;
            for token in FORBIDDEN_AGENT_TOKENS {
                if line.contains(token) {
                    hits.push(format!("{rel}:{line_no}: agent-surface token `{token}`"));
                }
            }
            let lower = line.to_ascii_lowercase();
            for word in FORBIDDEN_SLOP_WORDS {
                if is_ascii_word_match(&lower, word) {
                    hits.push(format!("{rel}:{line_no}: slop word `{word}`"));
                }
            }
        }
    }

    assert!(
        hits.is_empty(),
        "human documentation surface violation(s):\n  {}",
        hits.join("\n  ")
    );
    Ok(())
}

#[test]
fn relative_markdown_links_resolve_on_disk() -> Result<()> {
    let docs = human_doc_files(false)?;
    let mut broken = Vec::new();
    let mut checked = 0usize;

    for (rel, path) in docs {
        let parent = path.parent().unwrap_or(&path);
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{rel}: {e}"))?;
        let mut in_fence = false;

        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence {
                continue;
            }
            let stripped = strip_inline_backticks(line);
            for target in extract_markdown_link_targets(&stripped) {
                if target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with("mailto:")
                    || target.starts_with('#')
                    || target.contains("::")
                {
                    continue;
                }
                let file_part = target.split_once('#').map_or(target, |(p, _)| p).trim();
                if file_part.is_empty() {
                    continue;
                }
                checked += 1;
                if !parent.join(file_part).exists() {
                    broken.push(format!("{rel}:{}: `{target}`", idx + 1));
                }
            }
        }
    }

    assert!(
        checked > 0,
        "relative link checker scanned zero links; parser broken?"
    );
    assert!(
        broken.is_empty(),
        "broken relative markdown link(s):\n  {}",
        broken.join("\n  ")
    );
    Ok(())
}

#[test]
fn debt_ledger_matches_ratchet_config() -> Result<()> {
    let mk_local = read("mk.local")?;
    let coverage_floor = mk_local
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("COVERAGE_FAIL_UNDER_LINES=\"")
                .and_then(|rest| rest.strip_suffix('"'))
        })
        .ok_or("COVERAGE_FAIL_UNDER_LINES not found in mk.local")?;

    let debt_md = read("docs/DEBT.md")?;
    let expected_cov = format!("`COVERAGE_FAIL_UNDER_LINES=\"{coverage_floor}\"`");
    assert!(
        debt_md.contains(&expected_cov),
        "docs/DEBT.md does not match mk.local (`{expected_cov}` missing)"
    );

    let cargo_toml = read("Cargo.toml")?;
    let allow_count = count_workspace_lint_allows(&cargo_toml);
    let expected_allow = format!("- `Cargo.toml` lint `allow` count: `{allow_count}`");
    assert!(
        debt_md.contains(&expected_allow),
        "docs/DEBT.md allow count out of date (expected `{expected_allow}`)"
    );
    Ok(())
}
