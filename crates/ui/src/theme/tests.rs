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
}

#[test]
fn style_builder_creates_valid_css() {
    let style = StyleBuilder::new()
        .color("#fff")
        .padding(Spacing::LG)
        .border_radius(Radius::MD)
        .build();

    assert!(style.contains("color: #fff"));
    assert!(style.contains("padding: 20px"));
    assert!(style.contains("border-radius: 10px"));
}

#[test]
fn style_builder_multiple_properties() {
    let style = StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap(Spacing::MD)
        .align_items("center")
        .build();

    assert_eq!(style.split("; ").count(), 4, "Should have 4 CSS properties");
}
