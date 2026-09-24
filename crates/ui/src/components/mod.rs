// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Reusable Dioxus components for unified UI across all applications.
//!
//! All components use pure Rust styling via [`crate::theme`], no external CSS files.

mod button;
mod card;
mod footer;
mod header;
mod notice;
mod segmented_control;
mod upload_zone;

pub use button::{Button, ButtonVariant};
pub use card::Card;
pub use footer::Footer;
pub use header::Header;
pub use notice::{NoticeBar, NoticeTone};
pub use segmented_control::{SegmentedControl, SegmentedControlItem};
pub use upload_zone::UploadZone;
