# ASCII Image Generator

Cosmic Archive desktop app: drop a still, get a live ASCII preview, export the glyph grid.

## Run

```bash
cargo run --release
```

Needs Rust (`rustup`).

## Import

PNG, JPEG, WebP, GIF (first frame), BMP, TIFF. SVG rasterizes when ImageMagick (`magick`) is installed.

File size is not the limit. The long side is capped at **2048px** after decode (a 15MB photo is often 8–9k pixels on a side; that used to crash). JPEG shrinks on load; huge stills can also go through `magick`.

## Export

From the sidebar: **TXT**, **PNG** of the ASCII render, **SVG** of the glyph grid, **HTML**, and **ANSI**.
