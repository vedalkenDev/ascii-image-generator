# Load a still

Load a still puts a photo into the theater so looks can run on it. The archive accepts PNG, JPEG, WebP, GIF, BMP, TIFF, and SVG (SVG is rasterized in-process with resvg; no `magick`), and it must stay alive when the long side is bigger than 2048px.

## Sub-features

- `load-open` opens a still from the sidebar `CHOOSE STILL` / `CHANGE STILL` control.
- `load-drop` loads a still dropped onto the window.
- `load-drop-url` (web) loads a photo dragged from another tab (Pinterest and similar). Files win when the browser supplies one; otherwise the image URL is fetched. A blocked CORS fetch must show coral copy, not hang.
- `load-cli` loads a still passed as the launch path (`sample.png` or `--still`).
- `load-formats` accepts PNG, JPEG, WebP, GIF, BMP, TIFF, and SVG.
- `load-cap` keeps a still whose long side is over 2048px from taking the process down; the theater still paints.

## How to get to it (user POV)

- Choose `CHOOSE STILL` (or `CHANGE STILL` once a photo is loaded) in the sidebar.
- Click the empty theater (`NO STILL LOADED`) to open the same picker.
- Drop a still onto the window.
- On the web build, drag a photo from another tab (the image itself, not only the pin page).
- Start the app with a path: `cargo run --release -- sample.png`.

## Driving it with verify-ascii

Preconditions:

- `verify-ascii doctor` is ok for an instance this run started.
- Fixture `sample.png` exists in the repo root.
- Scratch stills, if needed, live under `$ASCII_VERIFY_DIR` — never overwrite user photos.

- **CLI entry.** Launch with the seed still. Run `.cursor/skills/verify-ascii/scripts/verify-ascii launch`. Doctor is ok, the plate shows `sample.png`, and the theater is not `NO STILL LOADED`.
- **Proof of CLI load.** Capture the loaded theater. Run `.cursor/skills/verify-ascii/scripts/verify-ascii screenshot load-cli`. The artifact shows ASCII in the theater and `sample.png` in the sidebar plate.
- **Open still.** On a `--empty` instance, choose `CHOOSE STILL`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii launch --empty` then `.cursor/skills/verify-ascii/scripts/verify-ascii click open.still`. On desktop an xdg-portal file dialog appears and the button may read `Opening…`. On web the browser file picker opens from a user click, without an Opening flash. Pick `sample.png`. After dismiss, the theater shows the still and the plate names it.
- **Empty-theater picker.** On a `--empty` instance, click the theater. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click theater`. The same picker appears as `CHOOSE STILL`.
- **Drag-drop.** On a `--empty` instance, drop `sample.png` onto the Cosmic Archive window (the verification PID, not a foreign one). The theater loads the still without using the picker. If the harness cannot synthesize a Wayland drop, record `load-drop` as unreachable with that reason — do not mark it verified via CLI.
- **Web image drop.** On the wasm build, drag an image from another tab (Pinterest CDN `i.pinimg.com` is the named case). Prefer the file the browser puts on the drop; otherwise fetch the image URL. Theater loads, or coral copy tells the user to save and **CHOOSE STILL** if the site blocks the fetch. A pin *page* URL (`pinterest.com/pin/…`) must not decode as a still. Extra evidence: `html_img_src_upgrades_pinterest_thumb`, `html_srcset_prefers_pinimg_over_pin_page`, `uri_list_skips_comments_and_pages` via `verify-ascii cargo-tests`. Those tests do not replace a real drag when one is possible.
- **Formats.** For each of PNG, JPEG, WebP, GIF, BMP, TIFF, SVG, launch `--still` a scratch file of that type into `$ASCII_VERIFY_DIR`. Magick may mint those fixtures if it is on the machine; it is not part of the product. A tiny SVG fixture is enough: `<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#cc2020"/></svg>`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii launch --still $ASCII_VERIFY_DIR/fmt.<ext>` then `screenshot load-<ext>`. Each theater paints glyphs and the process stays up.
- **2048 cap.** Make a still with long side > 2048px in `$ASCII_VERIFY_DIR` (any tool; magick is only a fixture helper). Run `verify-ascii launch --still $ASCII_VERIFY_DIR/wide.jpg` then `screenshot load-cap`. Doctor stays ok; the window still paints; the process does not exit.
- **Extra evidence (not a substitute).** Run `.cursor/skills/verify-ascii/scripts/verify-ascii cargo-tests`. `load_gif_roundtrip`, `load_jpeg_over_max_side`, `downscale_caps_long_side`, `import_svg_rasterizes_without_magick`, and `import_svg_caps_long_side` pass.

## Gotchas

- `CHOOSE STILL` / theater click open the picker. Native rfd may show `Opening…` until confirmed or cancelled. Web does not. Dismiss with `verify-ascii key Escape` if you are not completing the pick. Never leave the dialog up across another journey.
- Clicking `open.still` on a foreign Cosmic Archive window is a failed isolation check, not a load proof.
- A still SVG that resvg cannot parse must surface coral copy telling the user to open a PNG or JPEG instead, not a silent empty theater. That is a failed format journey, not a skip. Production does not call `magick`.
- File size is not the limit; the long side is. A small-byte image with a huge pixel size is the cap case.
- Drag-drop is a listed entry point. Proving only `--still` does not prove `load-drop`.
- Web URL drops are CORS-limited. A coral “save then CHOOSE STILL” after a blocked `i.pinimg.com` fetch is the product behaving, not a miss — unless the browser also supplied a File and that File was ignored.
