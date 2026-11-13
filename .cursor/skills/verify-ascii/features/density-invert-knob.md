# Density, invert, knob

Every look keeps three controls: density 32 / 80 / 132, invert, and one analog knob that look actually uses (weight on Visor/Blocks, glow on CRT, chroma on ANSI). Changing a control redraws the theater.

## Sub-features

- `density-32` sets a coarse grid.
- `density-80` is the default grid.
- `density-132` sets a fine grid.
- `invert-on` flips glyph mapping (`INVERT` becomes `INVERT ON`).
- `invert-off` restores the unflipped mapping.
- `knob-weight` is the analog bar while Visor or Blocks is selected (label `weight`).
- `knob-glow` is the analog bar while CRT is selected (label `glow`).
- `knob-chroma` is the analog bar while ANSI is selected (label `chroma`).

## How to get to it (user POV)

- Choose `32`, `80`, or `132` under `:: DENSITY`.
- Choose `INVERT` under the current look's section (`:: VISOR` / `:: BLOCKS` / `:: CRT` / `:: ANSI`).
- Drag the analog bar under that same section.

## Driving it with verify-ascii

Preconditions:

- Instance launched with `sample.png`.
- Look is Visor unless a step says otherwise.
- `verify-ascii doctor` is ok.

- **Default density.** Capture baseline. Run `.cursor/skills/verify-ascii/scripts/verify-ascii screenshot density-80`. Chip `80` is cream-bordered. Plate shows a column count of 80 (`VISOR  80×…`).
- **Density 32.** Choose `32`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click density.32` then `screenshot density-32`. Chip `32` is selected. Plate column count is 32. Theater glyphs are larger / fewer than at 80. Screenshots differ.
- **Density 132.** Choose `132`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click density.132` then `screenshot density-132`. Chip `132` is selected. Plate column count is 132. Theater is finer than at 80. Return to `80` before invert unless the journey wants 132 (`click density.80`).
- **Invert on.** Choose `INVERT`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click invert` then `screenshot invert-on`. The chip reads `INVERT ON`. Theater mapping is flipped versus `density-80`. Look thumbs also flip.
- **Invert off.** Choose `INVERT ON`. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click invert` then `screenshot invert-off`. Chip reads `INVERT`. Theater matches the pre-invert look.
- **Weight (Visor).** On Visor, drag the analog bar. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click knob` then `screenshot knob-weight`. Label is `weight`. Theater contrast/weight changes versus default `1.00`.
- **Glow (CRT).** Apply CRT, then the knob. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click look.crt` then `click knob` then `screenshot knob-glow`. Label is `glow`, not `weight`.
- **Chroma (ANSI).** Apply ANSI, then the knob. Run `.cursor/skills/verify-ascii/scripts/verify-ascii click look.ansi` then `click knob` then `screenshot knob-chroma`. Label is `chroma`.
- **Proof.** Evidence includes before/after for density, invert, and at least one knob. Plate `cols×rows` must match the selected density chip.
- **Extra evidence (not a substitute).** `convert_invert_maps_white_to_space` and `convert_grid_size` via `verify-ascii cargo-tests`.

## Gotchas

- The knob is one bar. If CRT still shows `weight`, the look did not apply — fail `looks-crt` first, do not pass this journey.
- `click knob` hits 80% along the bar (increases from the default). It is a click, not a full drag; the value must still visibly change. If it does not, report a product miss, do not call the control verified.
- Invert on a white-on-white region becomes empty glyphs; assert the chip text and a screenshot diff, not that “something is still visible”.
- Density chips sit above invert. A missed click that hits invert instead invalidates the density proof — check the plate column count.
