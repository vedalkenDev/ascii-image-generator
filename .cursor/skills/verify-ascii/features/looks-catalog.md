# Looks catalog

Looks catalog is four live thumbs of the loaded still (Visor, Blocks, CRT, ANSI). Clicking a card applies that look to the theater. Default look on launch is Visor.

## Sub-features

- `looks-thumbs` shows four thumbs of the current still, not placeholder names alone.
- `looks-visor` applies Visor (block glyphs, visor palette).
- `looks-blocks` applies Blocks (block glyphs, the still's colour).
- `looks-crt` applies CRT (amber phosphor punctuation).
- `looks-ansi` applies ANSI (letter and number glyphs, the still's colour).
- `looks-selected` shows the active card with a cream border and the plate line `VISOR` / `BLOCKS` / `CRT` / `ANSI`.

## How to get to it (user POV)

- With a still loaded, read the `:: LOOK` cards in the sidebar.
- Click `VISOR`, `BLOCKS`, `CRT`, or `ANSI`.

## Driving it with verify-ascii

Preconditions:

- Instance launched with `sample.png` (`verify-ascii launch`).
- `verify-ascii doctor` is ok.
- Theater is not `NO STILL LOADED`.

- **Thumbs present.** Capture the default Visor state. Run `.cursor/skills/verify-ascii/scripts/verify-ascii screenshot looks-visor`. The sidebar shows four thumbs of `sample.png`. Visor's card is cream-bordered. The plate reads `VISOR` and a column×row count. Default MARK is CELLS: theater is a visor-palette mosaic, not the raw rainbow of Blocks. Switch to CHARS (`click mark.chars`) to assert block glyphs instead.
- **Apply Blocks.** Click the Blocks card. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click look.blocks` then `screenshot looks-blocks`. The Blocks card is cream-bordered, the plate reads `BLOCKS`, and the theater uses the still's own colours (rainbow bands on `sample.png`). The two screenshots must differ.
- **Apply CRT.** Click the CRT card. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click look.crt` then `screenshot looks-crt`. The plate reads `CRT`. Theater glyphs are amber punctuation, not blocks. Screenshot differs from Visor and Blocks.
- **Apply ANSI.** Click the ANSI card. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click look.ansi` then `screenshot looks-ansi`. The plate reads `ANSI`. Theater glyphs are letters/numbers in the still's colour. Screenshot differs from CRT.
- **Return to Visor.** Click Visor. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click look.visor` then `screenshot looks-visor-again`. Plate reads `VISOR` and the theater matches the first Visor look, not ANSI.
- **Proof.** Keep all five screenshots under the evidence directory with this feature id. OCR may show `VISOR` / `BLOCKS` / `CRT` / `ANSI`; pixel difference between looks is required even if OCR misses the plate.
- **Extra evidence (not a substitute).** `verify-ascii cargo-tests` — `visor_and_crt_use_named_palettes` passes. That does not replace the click journey.

## Gotchas

- Thumbs only exist after a still is loaded. On `--empty`, cards show the look name as text; do not call that a catalog proof.
- Invert is shared with thumbs. If invert is on, thumbs flip too. Reset invert before comparing looks unless the journey is about invert.
- Click coordinates assume the helper floated the window to 1100×820. A tiled foreign window will miss. Doctor + `window` first.
- Do not click a look on the user's already-open Cosmic Archive window.
- ANSI after CRT must restore chroma (`saturation` 1.00) and paint the still's colour. Grey ANSI after CRT is a product regression (`Look::Ansi.apply` used to leave CRT's zeroed saturation). `cargo test ansi_after_crt_restores_chroma` covers the leak.
