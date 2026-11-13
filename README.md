# Cosmic Archive — ASCII

A desktop stills converter. Drop a photo, pick a look, get glyphs you can copy or save.

This is the ASCII generator for [Cosmic Archive](https://vedalken.dev): cream on near-black, visor colour as light, ASCII as the medium. It is not a photo editor.

## Download

Built installers live on the [v0.1.0 release](https://github.com/vedalkenDev/ascii-image-generator/releases/tag/v0.1.0):

- [Windows x64 `.exe`](https://github.com/vedalkenDev/ascii-image-generator/releases/download/v0.1.0/Cosmic-Archive-ASCII-windows-x64.exe)
- [macOS `.dmg`](https://github.com/vedalkenDev/ascii-image-generator/releases/download/v0.1.0/Cosmic-Archive-ASCII-macos.dmg) (Intel and Apple Silicon)

The Windows build is a single unsigned exe. On macOS, right-click Open the first time — the disk image is unsigned, so Gatekeeper will warn. Linux still builds from source below.

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

Open a still from the sidebar, or pass a path:

```bash
cargo run --release -- sample.png
```

Drop PNG, JPEG, WebP, GIF, BMP, TIFF, or SVG onto the window. SVG needs [ImageMagick](https://imagemagick.org/) (`magick`) on `PATH`.

File size is not the limit. After decode, the long side is capped at **2048px** so a 15MB photograph does not take the process down.

## Export

From the sidebar:

- **TXT** — plain glyph grid
- **SVG** — vector glyphs
- **ANSI** — coloured terminal art
- **COPY TXT** — the grid on the clipboard

## License

Personal project for Cosmic Archive. All rights reserved unless a license file is added later.
