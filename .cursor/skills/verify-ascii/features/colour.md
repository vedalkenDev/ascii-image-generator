# Colour

`:: COLOUR` tints the theater independently of the look. **SOURCE** uses the still, **VISOR** the Cosmic Archive visor ramp, **AMBER** phosphor, **CREAM** ground-to-cream. Switching looks does not reset the chip.

## Sub-features

- `colour-source` paints with the still's own colour.
- `colour-visor` is the default ramp.
- `colour-amber` is phosphor amber.
- `colour-cream` walks ground to cream `#F4F3E8`.
- `colour-keeps` stays on the chosen chip after applying another look.

## How to get to it (user POV)

- Choose `SOURCE`, `VISOR`, `AMBER`, or `CREAM` under `:: COLOUR`.

## Driving it with verify-ascii

Preconditions:

- Instance launched with `sample.png` (`verify-ascii launch`).
- `verify-ascii doctor` is ok.
- Look is Visor unless a step says otherwise.

- **Default visor.** Run `screenshot colour-visor`. Chip `VISOR` is cream-bordered. Theater cells walk the visor palette, not the still's rainbow.
- **Source.** Run `click colour.source` then `screenshot colour-source`. Chip `SOURCE` is selected. Theater uses the still's colours (rainbow bands on `sample.png`). Screenshots differ from `colour-visor`.
- **Amber.** Run `click colour.amber` then `screenshot colour-amber`. Chip `AMBER` is selected. Theater is phosphor amber, not visor teal.
- **Cream.** Run `click colour.cream` then `screenshot colour-cream`. Chip `CREAM` is selected. Occupied cells are cream on ground.
- **Keeps after look.** On Cream, apply Blocks. Run `click look.blocks` then `screenshot colour-keeps`. Chip `CREAM` is still selected. Theater stays cream, not Blocks' source colour.
- **Extra evidence (not a substitute).** `cream_maps_white_to_cream` and `source_keeps_still_colour` via `verify-ascii cargo-tests`.

## Gotchas

- Looks still set charset and the analog knob. Colour is the extra chip, not a fifth look.
- Thumbs keep each look's native palette. Do not require thumbs to follow `:: COLOUR`.
- A missed click on density or invert invalidates the colour proof — check the selected chip and a screenshot diff.
