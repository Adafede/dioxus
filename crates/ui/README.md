# ui

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-8-brightgreen)](https://github.com/adafede/dioxus/actions)

Unified UI design system for Dioxus applications.

Provides a complete, type-safe design system with reusable components, theme
constants, and styling utilities---all defined in pure Rust.

## Design Philosophy

- **No external CSS files**: All styling defined as Rust constants via [`theme`]
- **Type-safe theming**: Compile-time checked colors, spacing, typography
- **Accessible components**: WCAG AAA contrast, keyboard navigation, semantic
  HTML
- **Lotus aesthetic**: Clean, professional design inspired by lotus-explore-rs
- **Zero runtime overhead**: All styles inline, no dynamic CSS generation

## Example

```rust
use dioxus::prelude::*;
use ui::prelude::*;
use ui::theme::{ColorScheme, Spacing};

fn app() -> Element {
    let colors = ColorScheme::LIGHT;

    rsx! {
        Header {
            title: "My App".to_string(),
        }
        div { style: "padding: {}", Spacing::LG,
            Card {
                title: "Content".to_string(),
                "Body text here"
            }
        }
        Footer {}
    }
}
```

## License

`AGPL-3.0-only` --- see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html)
for details.
