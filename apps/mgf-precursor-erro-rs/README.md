# mgf-precursor-erro-rs

`mgf-precursor-erro-rs` is a Dioxus web app for uploading an MGF file and
summarizing precursor mass errors. It uses `mascot-rs` to normalize each
spectrum block and reports error summaries in absolute Da and ppm, alongside
tolerance counts.

## Run locally

```bash
dx serve --package mgf-precursor-erro-rs
```

## Build for the website

```bash
dx build --release --platform web --package mgf-precursor-erro-rs
```
