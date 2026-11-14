# ui

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Unified UI design system for Dioxus applications.

Provides a complete, type-safe design system with reusable components, theme
constants, and styling utilities---all defined in pure Rust.

## Design Philosophy

- **Pure-Rust theme tokens**: Colors, spacing, and typography defined in Rust
  via `StyleBuilder`, `ColorScheme`, `Spacing`, and `Typography`
- **Document head helper**: [`DocumentHead`](prelude::DocumentHead) manages page
  metadata, theme setup, and JSON-LD directly from Rust
- **Accessible components**: Semantic HTML landmarks, keyboard navigation, and
  high-contrast light/dark palettes

## Example

```rust
use dioxus::prelude::*;
use ui::prelude::*;

fn app() -> Element {
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

let mut dom = VirtualDom::new(app);
dom.rebuild_in_place();
```

## License

`AGPL-3.0-only` --- see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html)
for details.
