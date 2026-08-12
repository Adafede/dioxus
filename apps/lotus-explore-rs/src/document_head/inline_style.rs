// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Inline CSS for the document head, split into a font import and toast styles.

const INTER_FONT_IMPORT: &str = r#"
/* Inter Font */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap') layer;
"#;

const TOAST_STYLE: &str = r#"
#dx-toast-template {
    display: none;
    visibility: hidden;
}

.dx-toast {
    position: absolute;
    top: 10px;
    right: 0;
    padding-right: 10px;
    user-select: none;
    z-index: 2147483647;
}

.dx-toast .dx-toast-inner {
    position: fixed;
    background-color: #181B20;
    color: #ffffff;
    font-family: "Inter", sans-serif;
    display: grid;
    grid-template-columns: auto auto;
    max-width: 400px;
    min-height: 56px;
    border-radius: 5px;
}

.dx-toast .dx-toast-inner {
    cursor: pointer;
    margin-right: 10px;
}

.dx-toast .dx-toast-level-bar-container {
    height: 100%;
    width: 6px;
}

.dx-toast .dx-toast-level-bar-container .dx-toast-level-bar {
    width: 100%;
    height: 100%;
    border-radius: 5px 0 0 5px;
}

.dx-toast .dx-toast-content {
    padding: 8px;
}

.dx-toast .dx-toast-header {
    display: flex;
    flex-direction: row;
    justify-content: start;
    align-items: end;
    margin-bottom: 10px;
}

.dx-toast .dx-toast-header>svg {
    height: 18px;
    margin-right: 5px;
}

.dx-toast .dx-toast-header .dx-toast-header-text {
    font-size: 14px;
    font-weight: 700;
    padding: 0;
    margin: 0;
}

.dx-toast .dx-toast-msg {
    font-size: 11px;
    font-weight: 400;
    padding: 0;
    margin: 0;
}

.dx-toast-level-bar.info {
    background-color: #428EFF;
}

.dx-toast-level-bar.success {
    background-color: #42FF65;
}

.dx-toast-level-bar.error {
    background-color: #FF4242;
}
"#;

pub fn build_inline_style() -> String {
    [INTER_FONT_IMPORT, TOAST_STYLE].join("\n\n")
}
