# Colour

`:: COLOUR` tints the theater independently of the look. **SOURCE** uses the still, **VISOR** the Cosmic Archive visor ramp, **AMBER** phosphor, **CREAM** ground-to-cream, **TEAL** and **CORAL** phosphor, **SAGE** charcoal-to-sage-to-cream, **DUO** luma mapped from INK to PAPER. Switching looks does not reset the chip.

## Sub-features

- `colour-source` paints with the still's own colour.
- `colour-visor` is the default ramp.
- `colour-amber` is phosphor amber.
- `colour-cream` walks ground to cream `#F4F3E8`.
- `colour-teal` is phosphor teal.
- `colour-coral` is phosphor coral.
- `colour-sage` walks ground to sage to cream.
- `colour-duo` maps black to INK and white to PAPER; INK/PAPER are RGB pickers.
- `colour-keeps` stays on the chosen chip after applying another look.

## How to get to it (user POV)

- Choose `SOURCE`, `VISOR`, `AMBER`, `CREAM`, `TEAL`, `CORAL`, `SAGE`, or `DUO` under `:: COLOUR`. On DUO, click the INK / PAPER swatches to open RGB pickers.

## Driving it with verify-ascii

Preconditions:

- Instance launched with `sample.png` (`verify-ascii launch`).
- `verify-ascii doctor` is ok.
- Look is Visor unless a step says otherwise.

- **Default visor.** Run `screenshot colour-visor`. Chip `VISOR` is cream-bordered. Theater cells walk the visor palette, not the still's rainbow.
- **Source.** Run `click colour.source` then `screenshot colour-source`. Chip `SOURCE` is selected. Theater uses the still's colours (rainbow bands on `sample.png`). Screenshots differ from `colour-visor`.
- **Amber.** Run `click colour.amber` then `screenshot colour-amber`. Chip `AMBER` is selected. Theater is phosphor amber, not visor teal.
- **Cream.** Run `click colour.cream` then `screenshot colour-cream`. Chip `CREAM` is selected. Occupied cells are cream on ground.
- **Teal.** Run `click colour.teal` then `screenshot colour-teal`. Chip `TEAL` is selected. Theater is teal phosphor, not amber.
- **Coral.** Run `click colour.coral` then `screenshot colour-coral`. Chip `CORAL` is selected. Theater is coral, not amber.
- **Sage.** Run `click colour.sage` then `screenshot colour-sage`. Chip `SAGE` is selected. Theater is sage, not cream-grey.
- **Duo.** Run `click colour.duo` then `screenshot colour-duo`. Chip `DUO` is selected. Theater is charcoal-to-cream luma, not visor teal. `INK` / `PAPER` RGB pickers are visible.
- **Keeps after look.** On Duo, apply Blocks. Run `click look.blocks` then `screenshot colour-keeps`. Chip `DUO` is still selected. Theater stays duotone, with Blocks marks.
- **Extra evidence (not a substitute).** `cream_maps_white_to_cream`, `source_keeps_still_colour`, `duotone_maps_black_to_ink_white_to_paper`, `duotone_accepts_arbitrary_rgb`, and `teal_and_coral_highlights_are_cream` via `verify-ascii cargo-tests`.

## Gotchas

- Looks still set charset and the analog knob. Colour is the extra chip, not a fifth look.
- Thumbs keep each look's native palette. Do not require thumbs to follow `:: COLOUR`.
- A missed click on density or invert invalidates the colour proof — check the selected chip and a screenshot diff.
- Violet and blue are not colour chips. Cream on charcoal is fine; violet/blue on charcoal is not.
