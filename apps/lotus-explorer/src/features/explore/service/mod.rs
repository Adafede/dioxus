// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Application service layer for the Explore feature.
//!
//! Each module is a pure use-case function with no dependency on the Dioxus
//! runtime, making every unit independently testable without a virtual DOM.
//!
//! | Module            | Responsibility                                      |
//! |-------------------|-----------------------------------------------------|
//! | `strategy`        | Select execution path (API-first / SPARQL / DL-only)|
//! | `resolve_taxon`   | Resolve a free-text taxon name to a Wikidata QID    |
//! | `build_query`     | Build the SPARQL query from criteria + QID          |
//! | `fetch_preview`   | Fetch count + display rows from SPARQL endpoint     |
//! | `finalize`        | Assemble hashes, metadata JSON, and stats           |

pub mod build_query;
pub mod fetch_preview;
pub mod finalize;
pub mod resolve_taxon;
pub mod strategy;
