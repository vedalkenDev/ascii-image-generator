# Empty theater

When no still is loaded, the theater is an empty archive frame: `NO STILL LOADED` over `drop a still into the archive`. Looks have no live thumbs. The plate reads `no still`.

## Sub-features

- `empty-frame` shows the cream frame, `NO STILL LOADED`, and the drop subtitle.
- `empty-decoding` shows `READING STILL` / `the archive is decoding` while a load is in flight.
- `empty-plate` shows `VISOR  no still` (or the current look) with no filename.
- `empty-thumbs` look cards show the look name, not a live thumb.
- `empty-open` clicking the theater opens the same picker as `OPEN STILL`.

## How to get to it (user POV)

- Start the app with no path (`cargo run --release`).
- Launch `verify-ascii launch --empty`.

## Driving it with verify-ascii

Preconditions:

- No still argument. Run `.cursor/skills/verify-ascii/scripts/verify-ascii launch --empty`.
- `verify-ascii doctor` is ok.
- This is not the `sample.png` baseline.

- **Empty frame.** Capture the theater. Run `.cursor/skills/verify-ascii/scripts/verify-ascii screenshot empty-theater`. OCR / the image show `NO STILL LOADED` and `drop a still into the archive`. Central panel is a dark frame with a cream inner stroke, not glyphs.
- **Plate.** The sidebar plate reads `no still` and has no `sample.png` filename line.
- **Thumbs.** Look cards show `VISOR` / `BLOCKS` / `CRT` / `ANSI` as labels without a rendered still.
- **Theater click.** Click the theater. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click theater`. The file picker opens. Cancel with `verify-ascii key Escape`. After cancel, the theater is still `NO STILL LOADED` (not a hang, not a loaded still).
- **Decoding copy.** Launch `--still` a large file and screenshot immediately if `READING STILL` is visible; otherwise record that decode finished before a frame was captured. Do not fake this state.
- **Proof.** Keep `empty-theater.png`. After cleanup, that file still exists. Loading `sample.png` afterwards (`cleanup` then `launch` without `--empty`) must replace this state — that belongs to [load-still](./load-still.md).

## Gotchas

- Default `verify-ascii launch` loads `sample.png`, so it will **fail** this feature. `--empty` is required.
- `NO STILL LOADED` vs `READING STILL` are different states. A busy decode is not the empty theater.
- Clicking the empty theater opens rfd. Escape-cancel must return to empty; if the app stays on `Opening…` or freezes, that is a product break.
- Do not screenshot a foreign Cosmic Archive that happens to be empty or loaded; doctor must match our PID.
