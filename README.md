# Cosmic Archive — ASCII

A desktop stills converter. Drop a photo, pick a look, get glyphs you can copy or save.

This is the ASCII generator for [Cosmic Archive](https://vedalken.dev): cream on near-black, visor colour as light, ASCII as the medium. It is not a photo editor.

## Download

Use it in the browser. Same looks, no install, nothing for Gatekeeper to block:

**[vedalkendev.github.io/ascii-image-generator](https://vedalkendev.github.io/ascii-image-generator/)**

Optional unsigned desktop builds are on the [v0.1.0 release](https://github.com/vedalkenDev/ascii-image-generator/releases/tag/v0.1.0):

- [Windows x64 `.exe`](https://github.com/vedalkenDev/ascii-image-generator/releases/download/v0.1.0/Cosmic-Archive-ASCII-windows-x64.exe)
- [macOS `.dmg`](https://github.com/vedalkenDev/ascii-image-generator/releases/download/v0.1.0/Cosmic-Archive-ASCII-macos.dmg) (Intel and Apple Silicon)

The Windows exe is unsigned. The macOS disk image is **unsigned and not notarized**, so current macOS will show **“Cosmic Archive ASCII.app Not Opened”**. That is Gatekeeper, not a broken app. Prefer the web build. To force the Mac app open:

1. Dismiss **Done**.
2. Open **System Settings → Privacy & Security**, scroll to **Security**.
3. Choose **Open Anyway** for Cosmic Archive ASCII, then confirm.

Or in Terminal, after you copy the app out of the disk image:

```bash
xattr -dr com.apple.quarantine "/Applications/Cosmic Archive ASCII.app"
```

A paid Apple Developer ID and notarization would remove that dialog. This repo does not have those. Linux still builds from source below.

## Looks

The sidebar is a catalog of the loaded still, not a settings form.

| Look | What you get |
| --- | --- |
| **Visor** | Block glyphs tinted with the Cosmic Archive visor palette |
| **Blocks** | Block glyphs in the still’s own colour |
| **CRT** | Amber phosphor punctuation, terminal-style |
| **ANSI** | Letter and number glyphs in the still’s colour |
| **Dither** | Bayer stipple, occupied or empty |
| **Hatch** | Diagonal occupancy, sage phosphor |
| **Scan** | Interlace rows, teal phosphor |

Each look keeps **CELLS** (square mosaic) or **CHARS** (glyphs), a density slider (**16–400** columns, default **80**), colour (**SOURCE** / **VISOR** / **AMBER** / **CREAM** / **TEAL** / **CORAL** / **SAGE**), invert, and one knob that look actually uses (weight, glow, or chroma). CELLS is the default. Colour stays put when you switch looks. Weight goes **0.4–4.0**.

## Run on Linux

Needs a Rust toolchain (`rustup`).

```bash
git clone https://github.com/vedalkenDev/ascii-image-generator.git
cd ascii-image-generator
cargo run --release
```

The web target is the same crate. `trunk build --release` writes `dist/`.

Open a still from the sidebar, or pass a path:

```bash
cargo run --release -- sample.png
```

Drop PNG, JPEG, WebP, GIF, BMP, TIFF, or SVG onto the window. On the web build you can also drag a photo in from another tab (Pinterest, a news site, a gallery). The archive reads the image file when the browser supplies one; otherwise it fetches the image URL. If that site blocks the fetch, save the photo and use **CHOOSE STILL**.

There is no ImageMagick. The exe, the dmg, and the wasm build do not call `magick` and do not need it on `PATH`.

**Opening an SVG** rasterizes it in-process with [resvg](https://github.com/linebender/resvg) (paths, shapes, basic fills; long side capped at 2048px). Filters, scripts, and some text are not ImageMagick-complete. If that fails, the sidebar says to export the still as a PNG or JPEG and open that instead.

**Saving SVG** from **OUT** writes the glyph grid as vector XML. That path never rasterizes and never needs magick.

File size is not the limit. After decode, the long side is capped at **2048px** so a 15MB photograph does not take the process down.

## Export

From the sidebar **OUT**:

- **VECTOR** — SVG, TXT, ANSI
- **RASTER** — PNG, JPEG, WEBP of the theater mosaic (not the original still)
- **COPY TEXT** — the glyph grid on the clipboard
- **COPY IMAGE** — the mosaic on the clipboard (PNG)

On the web build, downloads go straight to the browser **Downloads** folder. Sage copy reads `saved still.svg` the same way **COPY TEXT** reads `copied text`. Desktop still asks where to put a download.

**COLOUR** includes **DUO**: the still is read as luma, then mapped from **INK** to **PAPER** (real RGB pickers). Looks still sit on top.

**SVG** uses the theater grid: CELLS as merged `<rect>` runs, CHARS as one `<text>` row of per-cell `<tspan>`s. Fills are the same 8-bit RGB as the preview. Raster export tiles the same cells (long side about 1600px) so high columns stay abutting instead of an 8px stamp.

## License

Personal project for Cosmic Archive. All rights reserved unless a license file is added later.
