#!/usr/bin/env python3
"""Bake a synchronous language bootstrap into every shipped `index.html`.

dioxus-cli 0.7 only bakes its own title/toast into the static index.html; the
app's `DocumentHead` applies `<html lang>` client-side *after* the async dioxus
module hydrates. Accessibility auditors (e.g. WAVE) flag that as:

    "Language*en* as the change happens AFTER loading"

for URLs like `/?lang=fr`.

This injects a tiny synchronous `<script>` as the very first child of `<head>`
(running during parse, before the deferred async module) that resolves the
language from the same `?lang=` / `?locale=` convention the app already uses,
defaulting to `"en"`. `<html lang>` is therefore correct on initial paint, so
no language change happens after load.

Run from the directory that contains `_site/` (the CI "Assemble site" step),
*before* `actions/upload-pages-artifact`. It targets the root landing page and
each app's top-level `index.html` only — never the nested Ketcher iframe
`index.html` under `assets/ketcher/`.
"""

import pathlib

# Inline, dependency-free, synchronous. Falls back to "en" (the app default)
# so even the un-parameterised URL paints with a lang before hydration.
LANG_BOOT = (
    '<script id="dx-lang-bootstrap">'
    'try{var p=new URLSearchParams(location.search);'
    'var l=p.get("lang")||p.get("locale");'
    'document.documentElement.lang=l||"en";}'
    'catch(e){}</script>'
)


def targets():
    """Every app's top-level index.html plus the root landing page."""
    yield pathlib.Path("_site/index.html")
    for d in pathlib.Path("_site").iterdir():
        if d.is_dir() and d.name == "assets":
            continue
        cand = d / "index.html"
        if cand.exists():
            yield cand


def bake(path: pathlib.Path) -> None:
    s = path.read_text()
    changed = False
    if "<html lang=" not in s and "<html>" in s:
        s = s.replace("<html>", '<html lang="en">', 1)
        changed = True
    if "dx-lang-bootstrap" not in s:
        s = s.replace("<head>", '<head>\n            ' + LANG_BOOT, 1)
        changed = True
    if changed:
        path.write_text(s)
        print(f"  baked lang bootstrap into {path}")


def main() -> None:
    for path in targets():
        bake(path)


if __name__ == "__main__":
    main()
