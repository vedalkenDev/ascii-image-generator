# Cosmic Archive — ASCII

A desktop stills converter. Drop a photo, pick a look, get glyphs you can copy or save.

This is the ASCII generator for [Cosmic Archive](https://vedalken.dev): cream on near-black, visor colour as light, ASCII as the medium. It is not a photo editor.

## Download

Built installers live on the [v0.1.0 release](https://github.com/vedalkenDev/ascii-image-generator/releases/tag/v0.1.0):

- [Windows x64 `.exe`](https://github.com/vedalkenDev/ascii-image-generator/releases/download/v0.1.0/Cosmic-Archive-ASCII-windows-x64.exe)
- [macOS `.dmg`](https://github.com/vedalkenDev/ascii-image-generator/releases/download/v0.1.0/Cosmic-Archive-ASCII-macos.dmg) (Intel and Apple Silicon)

A **web** build of the same app (WebAssembly, same looks catalog) is in this repo and avoids the Gatekeeper dialog. A GitHub Pages workflow ships with the repo; it is not a live public URL until Pages is turned on.

The Windows build is a single unsigned exe. The macOS disk image is **unsigned and not notarized**, so current macOS will show **“Cosmic Archive ASCII.app Not Opened”** / Apple could not verify it. That is Gatekeeper, not a broken app.

To open it:

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

Each look keeps three controls: density **32 / 80 / 132**, invert, and one knob that look actually uses (weight, glow, or chroma).

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

Drop PNG, JPEG, WebP, GIF, BMP, TIFF, or SVG onto the window.

There is no ImageMagick. The exe, the dmg, and the wasm build do not call `magick` and do not need it on `PATH`.

**Opening an SVG** rasterizes it in-process with [resvg](https://github.com/linebender/resvg) (paths, shapes, basic fills; long side capped at 2048px). Filters, scripts, and some text are not ImageMagick-complete. If that fails, the sidebar says to export the still as a PNG or JPEG and open that instead.

**Saving SVG** from **OUT** writes the glyph grid as vector XML. That path never rasterizes and never needs magick.

File size is not the limit. After decode, the long side is capped at **2048px** so a 15MB photograph does not take the process down.

## Export

From the sidebar:

- **TXT** — plain glyph grid
- **SVG** — vector glyphs (not a raster of the still)
- **ANSI** — coloured terminal art
- **COPY TXT** — the grid on the clipboard

## License

Personal project for Cosmic Archive. All rights reserved unless a license file is added later.
