# json-count-rs

[![AGPL-3.0 license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-0-lightgray)]()

`json-count-rs` is a small Dioxus web app for uploading a JSON file and counting
the number of non-null values seen under each top-level field.

## Run locally

```bash
dx serve --package json-count-rs
```

## Build for the website

```bash
dx build --release --platform web --package json-count-rs
```
