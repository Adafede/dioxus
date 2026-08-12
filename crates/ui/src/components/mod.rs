//! Reusable Dioxus components for unified UI across all applications.
//!
//! All components use pure Rust styling via [`crate::theme`], no external CSS files.

pub mod button;
pub mod card;
pub mod footer;
pub mod header;
pub mod notice;
pub mod segmented_control;

pub use button::Button;
pub use card::Card;
pub use footer::Footer;
pub use header::Header;
pub use notice::{NoticeBar, NoticeTone};
pub use segmented_control::{SegmentedControl, SegmentedControlItem};
