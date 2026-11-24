// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Static CSS stylesheet embedded at compile time from `assets/smellfish.css`.

pub(crate) const CSS: &str = include_str!("../assets/smellfish.css");
