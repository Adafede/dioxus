// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Atomic UI components for reusable building blocks.
//!
//! These components are styled with Tailwind classes and take explicit props.
//! They can be composed into larger components in other modules.

pub mod badge;
pub mod button;
pub mod card;

#[allow(unused_imports)]
pub use badge::{BadgeVariant, StatusBadge};
#[allow(unused_imports)]
pub use button::{Button, ButtonSize, ButtonVariant};
#[allow(unused_imports)]
pub use card::{Card, CardProps};
