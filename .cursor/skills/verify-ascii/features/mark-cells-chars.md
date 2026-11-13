# Cells and characters

`:: MARK` chooses how the theater draws the grid. **CELLS** paints a square mosaic (Cosmic Archive visor language). **CHARS** paints monospace glyphs. Looks still set palette and charset. Default is CELLS.

## Sub-features

- `mark-cells` fills each occupied grid slot as a coloured rectangle, square aspect.
- `mark-chars` draws the look's glyph ramp (blocks, punctuation, or letters) at half-width character aspect.
- `mark-switch` redraws the theater and the look thumbs when the chip changes.

## How to get to it (user POV)

- Choose `CELLS` or `CHARS` under `:: MARK`.

## Driving it with verify-ascii

Preconditions:

- Instance launched with `sample.png` (`verify-ascii launch`).
- `verify-ascii doctor` is ok.

- **Default cells.** Run `screenshot mark-cells`. Chip `CELLS` is cream-bordered. Theater is a mosaic of filled squares, not a glyph field. Visor cells walk the visor palette.
- **Chars.** Run `click mark.chars` then `screenshot mark-chars`. Chip `CHARS` is selected. Theater shows glyphs (`░▒▓█` on Visor/Blocks, punctuation on CRT, letters on ANSI). Screenshots differ from `mark-cells`.
- **Back to cells.** Run `click mark.cells` then `screenshot mark-cells-again`. Mosaic matches the first cells capture.

## Gotchas

- CELLS uses square aspect, so the row count is larger than CHARS at the same density. Plate `80×…` will change.
- Empty cells (`ch == ' '`, including invert-to-black) stay ground. Do not require a full rectangle field.
- Thumbs follow MARK. A CHARS catalog is glyph thumbs; a CELLS catalog is mosaic thumbs.
