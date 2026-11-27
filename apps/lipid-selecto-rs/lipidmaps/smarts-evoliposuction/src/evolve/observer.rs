// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Evolution progress observer recording every evaluated SMARTS candidate.

use std::sync::{Arc, Mutex};

use smarts_evolution::{EvolutionEvaluationProgress, EvolutionProgress, EvolutionProgressObserver};

/// One tested SMARTS candidate observed during evolution.
#[derive(Debug, Clone)]
pub struct TestedSmartsRecord {
    pub smarts: String,
    pub mcc: f64,
    pub smarts_len: usize,
    pub coverage_score: f64,
    pub limit_exceeded: bool,
    pub generation: u64,
}

impl TestedSmartsRecord {
    /// Total ordering by finite `mcc` descending, finite `coverage_score` descending,
    /// then `generation` ascending (with `NaN` values ranked last).
    #[must_use]
    pub fn cmp_priority(&self, other: &Self) -> std::cmp::Ordering {
        self.mcc
            .is_nan()
            .cmp(&other.mcc.is_nan())
            .then_with(|| other.mcc.total_cmp(&self.mcc))
            .then_with(|| {
                self.coverage_score
                    .is_nan()
                    .cmp(&other.coverage_score.is_nan())
            })
            .then_with(|| other.coverage_score.total_cmp(&self.coverage_score))
            .then_with(|| self.generation.cmp(&other.generation))
    }
}

#[derive(Debug)]
pub(super) struct TestedSmartsObserver {
    tested: Arc<Mutex<Vec<TestedSmartsRecord>>>,
    show_progress: bool,
}

impl TestedSmartsObserver {
    pub(super) const fn new(
        tested: Arc<Mutex<Vec<TestedSmartsRecord>>>,
        show_progress: bool,
    ) -> Self {
        Self {
            tested,
            show_progress,
        }
    }

    fn push_last(&self, progress: &EvolutionEvaluationProgress) {
        let Some(last) = progress.last() else {
            return;
        };
        let generation = progress.generation();
        if let Ok(mut tested) = self.tested.lock() {
            tested.push(TestedSmartsRecord {
                smarts: last.smarts().to_string(),
                mcc: last.mcc(),
                smarts_len: last.smarts_len(),
                coverage_score: last.coverage_score(),
                limit_exceeded: last.limit_exceeded(),
                generation,
            });
        }
    }
}

impl EvolutionProgressObserver for TestedSmartsObserver {
    fn on_evaluation(&mut self, progress: &EvolutionEvaluationProgress) {
        self.push_last(progress);
    }

    fn on_generation(&mut self, progress: &EvolutionProgress) {
        if !self.show_progress {
            return;
        }
        let generation = progress.generation();
        let total = progress.generation_limit();
        let best = progress.best_so_far();
        let display = truncate_str(best.smarts(), 60);
        eprintln!(
            "  gen {generation:>4}/{total:<4}  best_mcc={:.4}  best= {display}",
            best.mcc()
        );
    }
}

pub(super) fn truncate_str(s: &str, max_bytes: usize) -> &str {
    s.get(..s.floor_char_boundary(max_bytes)).unwrap_or(s)
}
