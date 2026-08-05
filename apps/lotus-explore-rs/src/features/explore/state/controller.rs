// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::app::view::AppView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppLayoutClasses {
    pub app_layout: &'static str,
    pub main: &'static str,
}

pub fn classes_for_view(view: AppView) -> AppLayoutClasses {
    if view == AppView::Explore {
        AppLayoutClasses {
            app_layout: "app-layout",
            main: "main-content",
        }
    } else {
        AppLayoutClasses {
            app_layout: "app-layout no-sidebar",
            main: "main-content single-pane",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_for_explore_view_keeps_sidebar_layout() {
        let classes = classes_for_view(AppView::Explore);
        assert_eq!(classes.app_layout, "app-layout");
        assert_eq!(classes.main, "main-content");
    }

    #[test]
    fn classes_for_non_explore_view_uses_single_pane_layout() {
        let classes = classes_for_view(AppView::Curation);
        assert_eq!(classes.app_layout, "app-layout no-sidebar");
        assert_eq!(classes.main, "main-content single-pane");
    }
}
