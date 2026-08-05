# json-count-rs

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
