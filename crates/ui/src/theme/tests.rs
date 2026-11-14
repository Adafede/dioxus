// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `tests` tests, extracted from `theme.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `theme` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn color_scheme_light_has_values() {
    assert_ne!(ColorScheme::LIGHT.accent, "");
    assert_ne!(ColorScheme::LIGHT.accent, ColorScheme::DARK.accent);
    assert_ne!(ColorScheme::LIGHT.bg, ColorScheme::DARK.bg);
    assert_ne!(ColorScheme::LIGHT.surface, ColorScheme::DARK.surface);
    assert_ne!(ColorScheme::LIGHT.text, ColorScheme::DARK.text);
    assert!(!Spacing::XL.is_empty(), "spacing scale should be non-empty");
    assert!(!Shadow::MD.is_empty(), "shadow tokens should be non-empty");
    assert!(
        !Typography::SANS.is_empty(),
        "font stack should be non-empty"
    );
    assert!(
        !Interaction::TRANSITION_DEFAULT.is_empty(),
        "transition token should be non-empty"
    );
}

#[test]
fn style_builder_creates_valid_css() {
    let style = StyleBuilder::new()
        .color("#fff")
        .padding(Spacing::LG)
        .border_radius(Radius::MD)
        .build();

    assert_eq!(style, "color: #fff; padding: 20px; border-radius: 10px");
}

#[test]
fn style_builder_multiple_properties() {
    let builder = StyleBuilder::new()
        .display("flex")
        .flex("1 1 auto")
        .flex_direction("column")
        .flex_wrap("wrap")
        .gap(Spacing::MD)
        .align_items("center")
        .justify_content("space-between")
        .background_color(ColorScheme::LIGHT.bg)
        .margin("0")
        .border("1px solid #000")
        .border_bottom("2px solid #111")
        .border_left("4px solid #222")
        .font_size(Typography::BODY)
        .font_family(Typography::SANS)
        .font_weight("600")
        .line_height(Typography::LINE_HEIGHT)
        .text_align("left")
        .width("100%")
        .height("40px")
        .min_height("30px")
        .box_shadow(Shadow::XS)
        .transition(Interaction::TRANSITION_DEFAULT)
        .opacity("0.9")
        .text_decoration("none")
        .cursor("pointer");

    let displayed = builder.to_string();
    let style = builder.build();
    assert_eq!(displayed, style);
    assert_eq!(
        style.split("; ").count(),
        25,
        "Should have 25 CSS properties"
    );
    assert!(style.contains("flex: 1 1 auto"), "missing flex: {style}");
    assert!(
        style.contains("border-left: 4px solid #222"),
        "missing border-left: {style}"
    );
    assert!(style.contains("cursor: pointer"), "missing cursor: {style}");
    assert_eq!(StyleBuilder::default().build(), "");
}
