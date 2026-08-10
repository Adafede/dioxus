# file-upload

Streaming file upload and blob handling primitives for WASM file upload apps.

## Purpose

Provides reusable, type-safe components for file upload workflows in Dioxus/WASM
applications:

- Chunked streaming reads from browser Blobs
- Drag-and-drop event handling
- File input parsing
- Progress reporting integration

## Non-goals

- Native file I/O (WASM-only by design)
- HTTP upload to servers
- File validation beyond basic type checking
