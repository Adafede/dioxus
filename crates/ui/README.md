# ui

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-8-brightgreen)](https://github.com/adafede/dioxus/actions)

Unified UI design system for Dioxus applications.

Provides a complete, type-safe design system with reusable components, theme
constants, and styling utilities---all defined in pure Rust.

## Design Philosophy

- **CSS bundled as static assets**: App-wide CSS lives in external files (e.g.
  `lotus-explore.css`) loaded via `<link rel="stylesheet">` in the generated
  `index.html` for optimal mobile performance
- **Type-safe theming**: Compile-time checked colors, spacing, typography via
  `StyleBuilder`, `ColorScheme`, `Spacing` and `Typography`
- **Accessible components**: WCAG AAA contrast, keyboard navigation, semantic
  HTML
- **Lotus aesthetic**: Clean, professional design inspired by lotus-explore-rs

## Example

```rust
use dioxus::prelude::*;
use ui::prelude::*;
use ui::theme::{ColorScheme, Spacing, StyleBuilder};

fn app() -> Element {
    let colors = ColorScheme::LIGHT;

    rsx! {
        Header {
            title: "My App".to_string(),
        }
        div { style: StyleBuilder::new().padding(Spacing::LG).build(),
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
