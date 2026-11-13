---
name: verify-ascii
description: >-
  Drive the Cosmic Archive — ASCII eframe desktop app the way a user does:
  launch an isolated native window, apply looks, density, colour, invert, and export.
  Use when verifying Cosmic Archive ASCII, ascii-image-generator, stills
  conversion, looks catalog, or sidebar export — not unit tests alone.
---

# Verify Cosmic Archive — ASCII

Native Hyprland window (eframe 0.32). Title `Cosmic Archive — ASCII`. Not a browser; do not use Playwright.

The question to answer: does the current user-facing suite break when driven the way a user uses it? Drive the real binary. `cargo test` in `src/ascii.rs` / `src/io.rs` is extra evidence, not a substitute.

Read `features/README.md` before a run. Prove from the matching feature file. A convenient CLI-only check is incomplete when that file lists a GUI entry point.

Helper (executable, repo-relative):

```bash
.cursor/skills/verify-ascii/scripts/verify-ascii <command>
```

Put `~/.cargo/bin` on `PATH` before launch.

## Launch

From the repo root. The helper builds `target/release/ascii-image-generator` and execs that binary (same artifact as `cargo run --release -- sample.png`).

```bash
export PATH="$HOME/.cargo/bin:$PATH"
.cursor/skills/verify-ascii/scripts/verify-ascii launch
# empty theater instead of sample.png:
.cursor/skills/verify-ascii/scripts/verify-ascii launch --empty
# any still:
.cursor/skills/verify-ascii/scripts/verify-ascii launch --still /path/to/photo.jpg
```

Ready means all of:

- The helper printed `"ok": true` with a PID.
- `verify-ascii doctor` is ok.
- A Hyprland client exists with that PID and title `Cosmic Archive — ASCII`.
- With a still: theater is not `NO STILL LOADED` / `READING STILL`. With `--empty`: theater reads `NO STILL LOADED`.

Launch focuses workspace 5 (`hyprctl dispatch 'o.workspaces.focus(5)'`), floats **only** the new PID, and resizes it to 1100×820 so chrome handles stay stable (colour chips sit above invert). A Cosmic Archive window already on workspace 5 is foreign: record its PID, never click it, never kill it.

Teardown is Cleanup below. State lives in `$ASCII_VERIFY_DIR` (default `/tmp/ascii-verify-$USER/state.json`).

## Doctor

Run first whenever anything looks off, after every failed drive, and before the first click of a session.

```bash
.cursor/skills/verify-ascii/scripts/verify-ascii doctor
```

Worth driving only when JSON `"ok": true`: our PID is alive, cmdline contains `ascii-image-generator`, a mapped window with this title belongs to that PID, and the PID is not in `foreign_pids`. Exit 2 otherwise — relaunch; do not drive someone else's instance.

```bash
.cursor/skills/verify-ascii/scripts/verify-ascii window   # at/size/workspace
```

## Drive

Surface is a native window. Stable handles are window title, PID, named chrome, files written by `io::encode`, and `grim` of our window. Click coordinates are a last resort and are encoded inside the helper from the sidebar layout (right panel is exactly 320px).

```bash
.cursor/skills/verify-ascii/scripts/verify-ascii handles
.cursor/skills/verify-ascii/scripts/verify-ascii click look.blocks
.cursor/skills/verify-ascii/scripts/verify-ascii screenshot looks-blocks
.cursor/skills/verify-ascii/scripts/verify-ascii ocr $EVIDENCE/looks-blocks.png
.cursor/skills/verify-ascii/scripts/verify-ascii key Escape
.cursor/skills/verify-ascii/scripts/verify-ascii encode --dir $EVIDENCE/export
.cursor/skills/verify-ascii/scripts/verify-ascii cargo-tests
```

Handles (sidebar, top to bottom):

| Handle | User control |
| --- | --- |
| `open.still` | `CHOOSE STILL` / `CHANGE STILL` (native rfd; web clicks `#still-input`) |
| `theater` | central panel (empty-state click also opens the picker) |
| `look.visor` `look.blocks` `look.crt` `look.ansi` | look cards |
| `mark.cells` `mark.chars` | mosaic cells vs character glyphs |
| `density.32` `density.80` `density.132` | positions along the `cols` slider |
| `density` | 80% along the `cols` bar |
| `colour.source` `colour.visor` `colour.amber` `colour.cream` | colour chips |
| `invert` | `INVERT` / `INVERT ON` |
| `knob` | analog bar (weight / glow / chroma) |
| `out.txt` `out.svg` `out.ansi` | export chips (native rfd; web blob download) |
| `copy.txt` | `COPY TXT` |

Clicks go through `scripts/pointer-click` (wlr-virtual-pointer). Hyprland `send_key_state` / `send_shortcut` with `mouse:272` does **not** reach eframe; do not substitute that. The helper builds the clicker on first `click` (`wayland-scanner` + `cc` + `libwayland-client`).

Hyprland, if you must do it by hand:

```bash
hyprctl dispatch 'o.workspaces.focus(5)'
hyprctl clients -j   # match title + our PID
grim -g "X,Y WxH" evidence.png
```

## Evidence

Named location (survives cleanup):

`/home/luqmaan/.local/state/cursor/agent-stores/cursor_agent_stores/bc-af75a5c8-0b95-4049-b2af-9d16f70c2afa/files/media/ascii-generator/verify/`

Override with `ASCII_VERIFY_EVIDENCE`. Helper screenshots and `encode` write here unless `--path` / `--dir` say otherwise.

Proof standards:

- Exercise the real user path. Launch the GUI. Looks, density, invert, empty theater are proven on the painted window, not by calling `convert()` in a REPL.
- Capture the action and the resulting state (before + after screenshots). OCR sidecar is extra, not enough alone.
- Side effects: export files must be non-empty and parse as that format. `COPY TXT` must change `wl-paste`. The 2048px cap must leave a process that still paints.
- Unit tests (`verify-ascii cargo-tests`) and `verify-ascii encode` (the same `io::encode` the UI calls after the save dialog) are extra evidence. They do not replace a GUI journey the map lists.
- Mocks: none. SVG import is `resvg` in-process. Magick is not production; it may only mint test fixtures if present.

Seed fixture: repo `sample.png` (160×100 PNG). Scratch stills for format/cap journeys go under `$ASCII_VERIFY_DIR`, never over user photos.

## Cleanup

```bash
.cursor/skills/verify-ascii/scripts/verify-ascii cleanup
```

Kills **only** the PID `launch` recorded, and only if `/proc/$PID/cmdline` contains `ascii-image-generator` and the PID is not foreign. Then deletes the state file. Does **not** delete evidence. Does **not** `pkill ascii-image-generator`. After cleanup, `ls` the evidence directory and require the proof files still exist.

If a drive opened a native rfd dialog, send `Escape` before cleanup so the worker is not left in `Opening…`. Web export does not open a dialog.

Failed iterations must also cleanup so foreign PIDs and stray dialogs are not stranded.

## Helpers

`.cursor/skills/verify-ascii/scripts/verify-ascii` is executable. Subcommands: `launch`, `doctor`, `click`, `screenshot`, `ocr`, `window`, `handles`, `key`, `encode`, `cargo-tests`, `cleanup`. JSON on stdout. Do not reverse-engineer click math; use the named handles.

`scripts/pointer-click.c` plus `scripts/wlr-virtual-pointer-unstable-v1.xml` are the clicker. `verify-ascii click` compiles them into `$ASCII_VERIFY_DIR/pointer-click` on first use.
