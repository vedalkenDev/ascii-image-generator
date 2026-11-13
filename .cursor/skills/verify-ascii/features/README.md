# Cosmic Archive — ASCII verification map

This directory is the maintained source for verifying the user-facing behavior of Cosmic Archive — ASCII. Read the index before driving the app, then use the matching feature file as the recipe.

## Baseline preconditions

- Repo root of `ascii-image-generator` with `sample.png` present.
- `~/.cargo/bin` on `PATH`. `hyprctl` and `grim` available. Magick is optional fixture minting, not a product dependency.
- Launch with `.cursor/skills/verify-ascii/scripts/verify-ascii launch` (loads `sample.png`) unless a feature says `--empty` or `--still`.
- Run `verify-ascii doctor` and require `"ok": true` for **our** PID. Title must be `Cosmic Archive — ASCII`.
- Never drive an instance this run did not start. A window already on workspace 5 is foreign.
- Prefer named handles over raw coordinates. File-save / open dialogs are rfd/xdg-portal and block the worker until dismissed.

## Driving conventions

- Start every recipe from the baseline state unless its preconditions say otherwise.
- Treat every command as literal. Keep handle names unchanged.
- GUI actions go through `verify-ascii click` / `screenshot` / `key`.
- Convert/export side effects that the UI itself calls may also go through `verify-ascii encode` and `verify-ascii cargo-tests`, but only as extra evidence next to the GUI path the feature lists.
- Restore nothing in the user's own Cosmic Archive window. Cleanup kills only our PID. Proof artifacts stay in the evidence directory named in the skill.

## Proof and skip reporting

- Capture the user action and the resulting state, not only the final screen.
- GUI proof includes a `grim` of our window (title bar / chrome / theater visible) plus OCR when asserting a string (`NO STILL LOADED`, `INVERT ON`, look names).
- Mutation proof includes a second view: another screenshot, a file on disk, or `wl-paste`.
- Record the feature ID and entry point used with every artifact.
- Report an unreachable path with the attempted command and the unmet precondition.
- Do not report a skipped entry point as verified through a different path.

## Feature entry contract

Each feature file starts with an H1 title and one paragraph describing the user-visible behavior. It then uses exactly four H2 sections in this order.

1. `Sub-features` lists short IDs with one line for each behavior.
2. `How to get to it (user POV)` lists every user entry point.
3. `Driving it with verify-ascii` starts with `Preconditions:` and uses labeled bullets that pair each user action with an exact command and observable result.
4. `Gotchas` lists traps that can waste or invalidate a verification run.

Keep implementation details out of the map. Name only user paths, stable handles, required state, commands, and observable proof.

## Features

- [Load a still](./load-still.md) covers Open still, drag-drop, CLI path, formats, and the 2048px long-side cap.
- [Looks catalog](./looks-catalog.md) covers Visor, Blocks, CRT, ANSI live thumbs and applying a look by clicking its card.
- [Cells and characters](./mark-cells-chars.md) covers `:: MARK` CELLS (mosaic) vs CHARS (glyphs).
- [Density, invert, knob](./density-invert-knob.md) covers the 16–200 `cols` slider, invert, and the one analog knob the current look uses.
- [Colour](./colour.md) covers SOURCE / VISOR / AMBER / CREAM / DUO (INK and PAPER swatches), independent of the look.
- [Export](./export.md) covers VECTOR (SVG / TXT / ANSI) and RASTER (PNG / JPEG / WEBP mosaic), plus COPY TEXT and COPY IMAGE. Native uses a save dialog; web sends the file to Downloads.
- [Empty theater](./empty-theater.md) covers `NO STILL LOADED` when nothing is loaded.
