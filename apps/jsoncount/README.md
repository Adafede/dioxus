# jsoncount

`jsoncount` is a small Dioxus web app for uploading a JSON file and counting the
number of non-null values seen under each top-level field.

## Run locally

```bash
dx serve --package jsoncount
```

Or with the workspace convenience target:

```bash
make serve APP=jsoncount
```

## Build for the website

```bash
dx build --release --platform web --package jsoncount
```
