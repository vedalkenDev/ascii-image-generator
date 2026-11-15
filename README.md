# Cosmic Archive — ASCII

Desktop stills converter (eframe 0.32 native) and the same crate as wasm on GitHub Pages. Drop a photo, pick a look, get glyphs you can copy or save.

This is the ASCII generator for [Cosmic Archive](https://vedalken.dev): cream on near-black, visor colour as light, ASCII as the medium. It is not a photo editor and it does not call ImageMagick.

Live: **https://vedalkendev.github.io/ascii-image-generator/**

Window title: `Cosmic Archive — ASCII`. Crate / binary: `ascii-image-generator` 0.1.0.

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

A paid Apple Developer ID and notarization would remove that dialog. This repo does not have those. Linux still builds from source below. New unsigned exe/dmg files come from `.github/workflows/release.yml` on a `v*` tag.

## Brand palette

Cosmic Archive visor palette only. Page / theater ground is near-black, not charcoal fills. Gradient is light only, never body text. Cream on charcoal is fine; violet/blue on charcoal is not (violet and blue are visor-ramp stops, not colour chips).

| Token | Hex | Role |
| --- | --- | --- |
| Violet | `#592AA4` | Visor ramp |
| Blue | `#4277A2` | Visor ramp |
| Teal | `#3FB6AF` | Visor ramp, phosphor, hyperlink |
| Sage | `#939E7A` | Visor ramp, Hatch default |
| Amber | `#C18D5D` | Visor ramp, CRT phosphor |
| Coral | `#D96350` | Visor ramp, errors |
| Cream | `#F4F3E8` | Body text, occupied cream cells |
| Charcoal | `#242A30` | DUO default INK |
| Ground | `#050508` | Window, theater, SVG fill |

Constants live in `src/main.rs` (`GROUND`, `CHARCOAL`, `CREAM`, …) and `src/ascii.rs` (`VISOR_RAMP`, `*_RGB`).

## Looks

The sidebar is a catalog of the loaded still, not a settings form. Seven live thumbs in `:: LOOK`. Default look on launch is **Visor**. Clicking a card applies that look; the colour chip stays put (`App::apply_look` restores `color_kind`).

| Look | What you get | Analog knob |
| --- | --- | --- |
| **Visor** | Block glyphs tinted with the visor palette | weight 0.4–4.0 |
| **Blocks** | Block glyphs in the still’s own colour | weight 0.4–4.0 |
| **CRT** | Amber phosphor punctuation, terminal-style | glow −0.5–0.5 |
| **ANSI** | Letter and number glyphs in the still’s colour | chroma 0.0–2.0 |
| **Dither** | Bayer stipple, occupied or empty | weight 0.4–4.0 |
| **Hatch** | Diagonal occupancy, sage phosphor | weight 0.4–4.0 |
| **Scan** | Interlace rows, teal phosphor | glow −0.5–0.5 |

Wire and Glitch were probed and skipped (Wire filled photos; Glitch read as a broken Visor). Do not add them back without a new request.

ANSI after CRT must restore chroma (`saturation` 1.00) and paint the still’s colour. Grey ANSI after CRT is a regression (`Look::Ansi.apply` used to leave CRT’s zeroed saturation). `cargo test ansi_after_crt_restores_chroma` covers the leak.

## Controls

Every look keeps these. Changing a control redraws the theater.

- **CELLS vs CHARS** (`:: MARK`). **CELLS** (default) is a square mosaic. **CHARS** is the look’s glyph ramp at half-width character aspect. Row count changes at the same density because CELLS uses aspect 1.0 and CHARS uses 0.5. Thumbs follow MARK.
- **Density** (`:: DENSITY` `cols` slider): **16–400** columns, default **80**. The convert path clamps columns 8–400; the UI dial is 16–400.
- **Weight** (and the one analog knob the current look uses): Visor / Blocks / Dither / Hatch show **weight 0.4–4.0** (default 1.00). CRT / Scan show **glow**. ANSI shows **chroma**. One bar; if CRT still says `weight`, the look did not apply.
- **Colour** (`:: COLOUR`): **SOURCE** / **VISOR** / **AMBER** / **CREAM** / **TEAL** / **CORAL** / **SAGE** / **DUO**. Independent of the look. Switching looks does not reset the chip. Thumbs keep each look’s native palette and do not follow this chip. Violet and blue are not chips.
- **DUO**: luma mapped from **INK** to **PAPER**. Defaults charcoal → cream. Both are real RGB pickers (`colour.ink` / `colour.paper`). Looks still sit on top.
- **INVERT**: flips glyph mapping. Chip reads `INVERT ON`. Thumbs flip too.

## Open a still

PNG, JPEG, WebP, GIF, BMP, TIFF, or SVG. No ImageMagick. The exe, the dmg, and the wasm build do not call `magick` and do not need it on `PATH`. Magick may only mint test fixtures if present.

- Sidebar **CHOOSE STILL** / **CHANGE STILL** (native `rfd`; web clicks hidden `#still-input`).
- Click the empty theater (`NO STILL LOADED`).
- Drop a file onto the window.
- CLI: `cargo run --release -- sample.png`.
- Web: drag a photo in from another tab (Pinterest, a news site, a gallery). The archive reads the image file when the browser supplies one; otherwise it fetches the image URL. If that site blocks the fetch, coral copy tells the user to save and **CHOOSE STILL**. A pin *page* URL must not decode as a still.

**Opening an SVG** rasterizes it in-process with [resvg](https://github.com/linebender/resvg) (paths, shapes, basic fills; long side capped at 2048px). Filters, scripts, and some text are not ImageMagick-complete. If that fails, the sidebar says to export the still as a PNG or JPEG and open that instead.

File size is not the limit. After decode, the long side is capped at **2048px** (`io::MAX_SIDE`) so a 15MB photograph does not take the process down. Decode also refuses a raw pixel count above 16×1024×1024.

Empty theater copy: `NO STILL LOADED` over `drop a photo here, or click to choose`. While a load is in flight: `READING STILL`.

Seed fixture: repo `sample.png` (160×100 PNG).

## Export

From the sidebar **OUT**:

- **VECTOR** — SVG, TXT, ANSI (`.svg` / `.txt` / `.ans`)
- **RASTER** — PNG, JPEG, WEBP of the theater mosaic (not the original still)
- **COPY TEXT** — the glyph grid on the clipboard; sage line `copied text`
- **COPY IMAGE** — the mosaic on the clipboard (PNG); sage line `copied image`

On the web build, downloads go straight to the browser **Downloads** folder. Sage copy reads `saved still.svg` the same way **COPY TEXT** reads `copied text`. Desktop still asks where to put a download (`rfd`). Web does not show `Exporting…` on **CHOOSE STILL**. Empty theater refuses with `Load a still first.`

**ANSI** the look and **ANSI** the export chip are different controls.

**Saving SVG** writes the glyph grid as vector XML. That path never rasterizes and never needs magick. CELLS as merged `<rect>` runs, CHARS as one `<text>` row of per-cell `<tspan>`s. Fills are the same 8-bit RGB as the preview. Raster export tiles the same cells (long side about **1600px**, `EXPORT_SIDE`) so high columns stay abutting instead of an 8px stamp.

The encoder also knows HTML; that is not sidebar chrome. Do not treat `verify-ascii encode` HTML/PNG extras as OUT chips.

## Repo map

| Path | What |
| --- | --- |
| `src/main.rs` | eframe/wasm app, sidebar, theater, copy, drops |
| `src/ascii.rs` | Looks, colour, CELLS/CHARS, `convert()` |
| `src/io.rs` | Load (incl. resvg SVG), 2048 cap, encode |
| `index.html` | wasm canvas + hidden file input |
| `Trunk.toml` | `trunk build --release` → `dist/` with `public_url = "./"` |
| `.cargo/config.toml` | wasm `web_sys_unstable_apis` + `getrandom_backend="wasm_js"` |
| `sample.png` | Seed still |
| `assets/NotoSansMono-Regular.ttf` | Embedded mono |
| `pack/macos-dmg.sh` | Universal unsigned `.app` + `.dmg` |
| `.github/workflows/pages.yml` | wasm Pages on push to `main` |
| `.github/workflows/release.yml` | unsigned Windows exe + macOS dmg on `v*` tags |
| `.cursor/skills/verify-ascii/` | Native Hyprland drive skill |

`cargo test` covers `src/ascii.rs` and `src/io.rs` (and drop-URL tests in `main.rs`). Tests are extra evidence, not a substitute for the GUI skill.

## Graphify before edits

Before changing product code in this repo, query the Graphify knowledge graph.

- Cursor rule (when installed): `.cursor/rules/graphify.mdc` (always-apply). Activate with `/graphify`.
- If `graphify-out/` is missing, build first: `/graphify .` (or `graphify extract` / `graphify update`). Source: https://github.com/Graphify-Labs/graphify
- To write the rule into a checkout that lacks it: `graphify cursor install --project` from the repo root.
- Do not skip the graph query because the files “look small.”
- Do not commit `graphify-out/`. Do not commit secrets. Docs-only bookkeeping may skip Graphify.

This Git history may not include `.cursor/rules/graphify.mdc`; treat a missing rule as “install then query,” not as permission to edit blind.

## Run on Linux

Needs a Rust toolchain (`rustup`). Native eframe needs a display (Wayland/X11). Put `~/.cargo/bin` on `PATH`.

```bash
git clone https://github.com/vedalkenDev/ascii-image-generator.git
cd ascii-image-generator
cargo run --release
# or with a still:
cargo run --release -- sample.png
```

The web target is the same crate:

```bash
rustup target add wasm32-unknown-unknown
# trunk 0.21.x matches CI (jetli/trunk-action v0.21.14)
trunk build --release   # writes dist/
trunk serve             # local wasm
```

`.github/workflows/pages.yml` deploys `dist/` to GitHub Pages on every push to `main`.

## Test and prove

Unit tests:

```bash
cargo test
```

GUI (required for user-facing claims). Native Hyprland window, not Playwright. Read `.cursor/skills/verify-ascii/SKILL.md` and `.cursor/skills/verify-ascii/features/README.md`, then the matching feature file.

```bash
export PATH="$HOME/.cargo/bin:$PATH"
.cursor/skills/verify-ascii/scripts/verify-ascii doctor
.cursor/skills/verify-ascii/scripts/verify-ascii launch          # loads sample.png
.cursor/skills/verify-ascii/scripts/verify-ascii launch --empty
.cursor/skills/verify-ascii/scripts/verify-ascii click look.blocks
.cursor/skills/verify-ascii/scripts/verify-ascii screenshot looks-blocks
.cursor/skills/verify-ascii/scripts/verify-ascii cargo-tests
.cursor/skills/verify-ascii/scripts/verify-ascii cleanup         # only our PID
```

Feature map: load-still, looks-catalog, mark-cells-chars, density-invert-knob, colour, export, empty-theater. Named handles only (`look.visor` … `copy.image`). Do not drive a foreign Cosmic Archive window. Magick is not production.

After any claimed completion (code, UI, export, preview), run **Prove it works**: exercise the real surface (running preview, Pages, or desktop app), capture evidence, and do not call it done until the check passes. One still is not enough for motion. Hit edges that used to fail (high cols, export vs preview, empty theater, invert). Compare evidence to the original request; if a clause is missing, it is not done.

On this Project, evidence belongs under the agent store `media/` (no localhost links for phone checks). Pure docs may skip research and still confirm the file exists.

## Ship

- **Web:** merge/push to GitHub `main`. Pages workflow builds wasm with trunk and deploys. Confirm https://vedalkendev.github.io/ascii-image-generator/
- **Desktop binaries:** tag `v*` (example `v0.1.0`). Release workflow builds the unsigned Windows x64 exe and the unsigned universal macOS dmg (`pack/macos-dmg.sh`) and attaches them to the GitHub release. Gatekeeper copy in this README stays accurate until notarization exists.
- **Linux:** ship source; `cargo run --release` / `cargo build --release`.
- Do not add `magick` as a runtime dependency. Do not change the brand palette without a request. Do not treat HTML encode as a sidebar export.

## License

Personal project for Cosmic Archive. All rights reserved unless a license file is added later.
