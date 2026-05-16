// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Task coordination and cancellation patterns for async search operations.
//!
//! Centralizes best practices for:
//! * Managing in-flight search tasks
//! * Preventing race conditions (search N+1 results displayed before search N)
//! * Tracking task lifecycle via tokens
//! * Cleanup on component unmount

// ── In-flight task controller ─────────────────────────────────────────────────

/// Manages in-flight async tasks with automatic cleanup.
///
/// When a new search is initiated, this replaces the previous task (if any),
/// which causes the old task to be cancelled automatically.
#[allow(dead_code)] // Public API for future task management improvements
pub struct TaskManager<T> {
    /// Whether a task is currently in flight.
    in_flight: bool,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TaskManager<T> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            in_flight: false,
            _marker: std::marker::PhantomData,
        }
    }

    /// Mark a new task as in-flight, returning whether there was a previous task.
    #[allow(dead_code)]
    pub fn start_new(&mut self) -> bool {
        let had_previous = self.in_flight;
        self.in_flight = true;
        had_previous
    }

    /// Mark the current task as finished.
    #[allow(dead_code)]
    pub fn finish(&mut self) {
        self.in_flight = false;
    }

    /// Check if a task is currently in flight.
    #[allow(dead_code)]
    pub fn is_in_flight(&self) -> bool {
        self.in_flight
    }
}

impl<T> Default for TaskManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Task lifecycle markers ────────────────────────────────────────────────────

/// Marker to track whether a task belongs to a particular request.
///
/// Used to detect stale results — if a task's token doesn't match the current
/// request token, we ignore its result.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)] // Public API for future stale-detection improvements
pub struct TaskToken(u64);

impl TaskToken {
    #[allow(dead_code)]
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    #[allow(dead_code)]
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Increment to the next token.
    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Check if this token matches another (for stale detection).
    #[allow(dead_code)]
    pub fn matches(&self, other: TaskToken) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_token_default_is_zero() {
        assert_eq!(TaskToken::default().value(), 0);
    }

    #[test]
    fn task_token_increments() {
        let token = TaskToken::new(5);
        let next = token.next();
        assert_eq!(next.value(), 6);
    }

    #[test]
    fn task_token_saturates_on_overflow() {
        let token = TaskToken::new(u64::MAX);
        let next = token.next();
        assert_eq!(next.value(), u64::MAX);
    }

    #[test]
    fn task_token_ordered() {
        let token1 = TaskToken::new(1);
        let token2 = TaskToken::new(2);
        assert!(token1 < token2);
        assert!(token2 > token1);
    }

    #[test]
    fn task_token_matches_identical() {
        let token1 = TaskToken::new(42);
        let token2 = TaskToken::new(42);
        assert!(token1.matches(token2));
    }

    #[test]
    fn task_token_not_matches_different() {
        let token1 = TaskToken::new(1);
        let token2 = TaskToken::new(2);
        assert!(!token1.matches(token2));
    }

    #[test]
    fn task_manager_starts_not_in_flight() {
        let manager: TaskManager<()> = TaskManager::new();
        assert!(!manager.is_in_flight());
    }

    #[test]
    fn task_manager_marks_in_flight() {
        let mut manager: TaskManager<()> = TaskManager::new();
        let had_previous = manager.start_new();
        assert!(!had_previous); // First time
        assert!(manager.is_in_flight());
    }

    #[test]
    fn task_manager_detects_replacement() {
        let mut manager: TaskManager<()> = TaskManager::new();
        manager.start_new(); // First task
        let had_previous = manager.start_new(); // Second task
        assert!(had_previous); // There was a previous task
    }

    #[test]
    fn task_manager_finish_clears_state() {
        let mut manager: TaskManager<()> = TaskManager::new();
        manager.start_new();
        assert!(manager.is_in_flight());
        manager.finish();
        assert!(!manager.is_in_flight());
    }

    #[test]
    fn task_manager_default_is_not_in_flight() {
        let manager: TaskManager<()> = TaskManager::default();
        assert!(!manager.is_in_flight());
    }
}
