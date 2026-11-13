# Export

Export writes the current theater off the machine: TXT (plain glyph grid), SVG (vector glyphs), ANSI (coloured terminal art), and COPY TXT (clipboard). PNG and HTML encoders exist in the binary but are not in the sidebar. Native asks for a save path. Web sends the file to the browser Downloads folder on the same click — no dialog, no `Exporting…` on **CHOOSE STILL**.

## Sub-features

- `export-txt` saves a `.txt` glyph grid from the `TXT` chip.
- `export-svg` saves a `.svg` from the `SVG` chip.
- `export-ansi` saves a `.ans` from the `ANSI` chip.
- `export-copy` puts the glyph grid on the clipboard from `COPY TXT`.
- `export-empty` refuses to export when no still is loaded (`Load a still first.`).

## How to get to it (user POV)

- Choose `TXT`, `SVG`, or `ANSI` under `:: OUT`. Native: pick a save path. Web: the file appears in Downloads and the sidebar reads `saved <name>.<ext>`.
- Choose `COPY TXT` (no dialog; sidebar reads `copied`).

## Driving it with verify-ascii

Preconditions:

- Instance launched with `sample.png` unless testing `export-empty`.
- `verify-ascii doctor` is ok.
- Save targets are a fresh directory under `$ASCII_VERIFY_DIR` or the evidence `export/` folder. Never overwrite user files.

- **COPY TXT.** Choose `COPY TXT`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click copy.txt`. Sidebar shows `copied`. Run `wl-paste`. Clipboard is a non-empty glyph grid whose line length matches the plate column count (80 on baseline). Capture `verify-ascii screenshot export-copy`.
- **TXT via dialog (native).** Choose `TXT`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click out.txt`. An rfd save dialog opens. **CHOOSE STILL** must still read `CHANGE STILL`, not `Exporting…`. The sage line reads `saved …` after dismiss. Save as `$ASCII_VERIFY_DIR/out.txt`. After dismiss, the file exists, size > 0, row count equals the plate row count, each line's character count equals the plate column count. If the dialog cannot be driven, do **not** skip this bullet: record the dialog as blocking, keep the GUI up, and still run the encode path below as extra evidence — the dialog journey stays unverified, not passed.
- **TXT download (web).** Choose `TXT`. No save dialog. The browser downloads `<still>.txt`. Sidebar sage line is `saved <still>.txt`. **CHANGE STILL** stays **CHANGE STILL**.
- **SVG via dialog.** Same as TXT with `click out.svg` and `$ASCII_VERIFY_DIR/out.svg`. File starts with `<svg`. CELLS is merged `<rect>` runs (no per-glyph `<text>`). CHARS is `<tspan>` runs. Size stays Figma-loadable (phone still at density 200 was 3.47MB / 71k texts; compact+80-col cap is the product). Extra evidence: `svg_phone_photo_at_density_200_is_figma_light` and `svg_merges_same_fill_runs`.
- **ANSI via dialog.** Same with `click out.ansi` and `$ASCII_VERIFY_DIR/out.ans`. File contains CSI colour sequences (`ESC[38;2;`) and glyphs, size > 0.
- **Encode path (extra, same function the dialog calls).** Run `.cursor/skills/verify-ascii/scripts/verify-ascii encode --dir $EVIDENCE/export`. Writes `export-sample.txt`, `export-sample.svg`, `export-sample.ans` (also `.png` and `.html` from the encoder — those two are not sidebar chrome). Each sidebar format file is non-empty.
- **Empty refuse.** On `--empty`, click `COPY TXT`. Run `verify-ascii launch --empty` then `click copy.txt` then `screenshot export-empty`. Coral status `Load a still first.` Clipboard is not a new grid. Same status if `TXT` is used without a still (dialog must not write a file).
- **Proof.** Keep the saved files and the COPY screenshot. A passing `encode` run without a GUI COPY/dialog attempt does not prove the sidebar.

## Gotchas

- File-save dialogs block the export worker until confirmed or cancelled. Cancel with `verify-ascii key Escape`. Never save over `~/` photos or the repo's `docs/` samples.
- Sidebar chrome is TXT, SVG, ANSI, COPY TXT only. Do not fail the product for missing PNG/HTML chips; do not claim PNG/HTML as a user-visible export path.
- `ANSI` the look and `ANSI` the export chip are different controls. The look card is under `:: LOOK`; the export chip is under `:: OUT`.
- COPY TXT uses the compositor clipboard. `wl-paste` must run while our instance still lives; cleanup wipes the process, not necessarily the clipboard.
- Never weaken a dialog journey to “encode passed, so export works”.
