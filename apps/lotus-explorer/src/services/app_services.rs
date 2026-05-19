// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Application-level services and dependency container.
//!
//! This module defines a single, unified dependency container that holds all
//! singleton/long-lived services needed by the app. It centralizes service
//! construction, caching, and context provision.
//!
//! ## Design
//!
//! - **Single ownership**: `AppServices` is created once at app bootstrap
//! - **Context provider**: Made available via Dioxus context
//! - **Zero-cost abstractions**: Copy-able, stateless wrappers around global services
//! - **Testability**: Services can be swapped via dependency injection

use crate::repositories::HybridRepository;
use std::marker::PhantomData;

/// Application-wide services container.
///
/// Holds references to all singleton dependencies needed throughout the app.
/// Designed to be provided via Dioxus context and used by hooks/components.
#[derive(Clone, Copy)]
pub struct AppServices {
    /// Data repository (API/SPARQL hybrid adapter).
    repo: HybridRepository,
    /// Placeholder for future services (telemetry, analytics, etc.)
    _marker: PhantomData<()>,
}

impl AppServices {
    /// Create a new services container with all dependencies initialized.
    pub fn new() -> Self {
        Self {
            repo: HybridRepository,
            _marker: PhantomData,
        }
    }

    /// Get the data repository.
    pub fn repository(&self) -> HybridRepository {
        self.repo
    }
}

impl Default for AppServices {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_services_is_copy() {
        let services = AppServices::new();
        let _copy = services;
        let _another_copy = services;
    }

    #[test]
    fn app_services_repository_is_consistent() {
        let services = AppServices::new();
        let repo1 = services.repository();
        let repo2 = services.repository();
        assert_eq!(repo1, repo2);
    }
}
