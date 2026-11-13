# Load a still

Load a still puts a photo into the theater so looks can run on it. The archive accepts PNG, JPEG, WebP, GIF, BMP, TIFF, and SVG (SVG needs `magick`), and it must stay alive when the long side is bigger than 2048px.

## Sub-features

- `load-open` opens a still from the sidebar `OPEN STILL` control.
- `load-drop` loads a still dropped onto the window.
- `load-cli` loads a still passed as the launch path (`sample.png` or `--still`).
- `load-formats` accepts PNG, JPEG, WebP, GIF, BMP, TIFF, and SVG.
- `load-cap` keeps a still whose long side is over 2048px from taking the process down; the theater still paints.

## How to get to it (user POV)

- Choose `OPEN STILL` in the sidebar.
- Click the empty theater (`NO STILL LOADED`) to open the same picker.
- Drop a still onto the window.
- Start the app with a path: `cargo run --release -- sample.png`.

## Driving it with verify-ascii

Preconditions:

- `verify-ascii doctor` is ok for an instance this run started.
- Fixture `sample.png` exists in the repo root.
- Scratch stills, if needed, live under `$ASCII_VERIFY_DIR` — never overwrite user photos.

- **CLI entry.** Launch with the seed still. Run `.cursor/skills/verify-ascii/scripts/verify-ascii launch`. Doctor is ok, the plate shows `sample.png`, and the theater is not `NO STILL LOADED`.
- **Proof of CLI load.** Capture the loaded theater. Run `.cursor/skills/verify-ascii/scripts/verify-ascii screenshot load-cli`. The artifact shows ASCII in the theater and `sample.png` in the sidebar plate.
- **Open still.** On a `--empty` instance, choose `OPEN STILL`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii launch --empty` then `.cursor/skills/verify-ascii/scripts/verify-ascii click open.still`. An xdg-portal file dialog appears and the button label becomes `Opening…`. Pick `sample.png` in that dialog. After dismiss, the theater shows the still and the plate names it.
- **Empty-theater picker.** On a `--empty` instance, click the theater. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click theater`. The same file dialog appears as `OPEN STILL`.
- **Drag-drop.** On a `--empty` instance, drop `sample.png` onto the Cosmic Archive window (the verification PID, not a foreign one). The theater loads the still without using the picker. If the harness cannot synthesize a Wayland drop, record `load-drop` as unreachable with that reason — do not mark it verified via CLI.
- **Formats.** For each of PNG, JPEG, WebP, GIF, BMP, TIFF, SVG, launch `--still` a scratch file of that type (build JPEG/WebP/GIF/BMP/TIFF/SVG from `sample.png` with `magick` into `$ASCII_VERIFY_DIR`). Run `.cursor/skills/verify-ascii/scripts/verify-ascii launch --still $ASCII_VERIFY_DIR/fmt.<ext>` then `screenshot load-<ext>`. Each theater paints glyphs and the process stays up. SVG requires `magick` on `PATH`.
- **2048 cap.** Make a still with long side > 2048px in `$ASCII_VERIFY_DIR` (`magick sample.png -resize 2400x800 jpeg:$ASCII_VERIFY_DIR/wide.jpg`). Run `verify-ascii launch --still $ASCII_VERIFY_DIR/wide.jpg` then `screenshot load-cap`. Doctor stays ok; the window still paints; the process does not exit.
- **Extra evidence (not a substitute).** Run `.cursor/skills/verify-ascii/scripts/verify-ascii cargo-tests`. `load_gif_roundtrip`, `load_jpeg_over_max_side`, and `downscale_caps_long_side` pass.

## Gotchas

- `OPEN STILL` and a theater click open an rfd/xdg-portal dialog. That worker stays `Opening…` until the dialog is confirmed or cancelled. Dismiss with `verify-ascii key Escape` if you are not completing the pick. Never leave the dialog up across another journey.
- Clicking `open.still` on a foreign Cosmic Archive window is a failed isolation check, not a load proof.
- SVG without `magick` must surface `Could not rasterize SVG.` rather than a silent empty theater. That is a failed format journey, not a skip.
- File size is not the limit; the long side is. A small-byte image with a huge pixel size is the cap case.
- Drag-drop is a listed entry point. Proving only `--still` does not prove `load-drop`.
