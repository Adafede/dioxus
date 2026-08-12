# json-count-rs

[![AGPL-3.0 license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-0-lightgray)](https://github.com/adafede/dioxus/actions)

`json-count-rs` — count non-null JSON fields from an uploaded file.

Drag-and-drop (or browse) a JSON file and see a live count of non-null
values per key, rendered as a sortable table.

### Run locally

```bash
dx serve --package json-count-rs
```

### Build for the website

```bash
dx build --release --platform web --package json-count-rs
```

## License

`AGPL-3.0-only` — see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html) for details.
