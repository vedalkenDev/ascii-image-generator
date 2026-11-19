# Converter usability iterations

This is a code and browser review, not a user study. Each round records the observed problem, the selected change, and the available check.

## 1. Colour section

Discover: colour choices and conditional pickers occupied several rows even when the palette was already chosen.

Define and develop: use a collapsed control that names the current palette and opens the existing choices. Preserve the selected colour and its settings when collapsed.

Deliver: browser checks opened and closed the section without changing the preview, including a narrow-layout capture.

## 2. Names and help

Discover: Visor was used as a look and a colour even though look changes marks and colour changes the palette. The look choice preserved the selected colour, so the names suggested a dependency that did not exist.

Define and develop: call the colour Spectrum and add contextual hover help beside sections and slider labels. Keep Visor as the name of the look.

Deliver: tooltips explain look, colour and adjustments; the underlying palette is unchanged by renaming. Hover captures were inspected at desktop and narrow widths.

## 3. Visor look

Discover: Visor and Blocks selected the same character ramp with the same default tone settings, so with the same colour they looked identical.

Define and develop: add optional ordered texture to the Visor look, affecting marks independently of colour. Use a moderate initial texture, keep zero as the legacy setting, and preserve solid black and white endpoints.

Deliver: tests check that texture changes midtone glyphs, preserves colours, has no effect on the Blocks look, and visibly distinguishes the initial Visor setting from Blocks.

## 4. Spectrum palette

Discover: the original Visor palette had no controls despite being available independently of the look.

Define and develop: provide spectrum shift, spectrum range and adjusted image mix. Neutral values retain the original palette.

Deliver: tests check the controls change colours without changing glyphs and remain available under another look. Browser checks exercised shift and reset; a raster and SVG test checks the resulting cell colours.

## 5. Look previews

Discover: thumbnail cards used their own default colours and cell geometry, while choosing a card preserved the user's colour and mark settings. They could therefore predict the wrong result.

Define and develop: render previews from the active colour, mark, shared adjustments and relevant look state, at thumbnail density.

Deliver: tests check the selected and alternate thumbnail settings. Browser captures show the active palette applied to the cards. Thumbnails are approximations at lower density. Selecting the active look no longer resets its adjustment.

## Simplification pass

Thesis: choose a look, choose a palette, set the detail, save. Those four stay visible; anything that refines a result sits behind one **Tune** disclosure, never deeper.

| Before | After | Reason |
|---|---|---|
| Plate repeated the look name | Plate shows size and file only | The selected card already names the look. |
| Mark: two chip rows | One row under Look: Cells, Shaded, Chars | Two toggles were one choice. |
| Density | Detail | Plainer word. |
| Colour fold, Filter group, inline sliders | One row of live palette swatches | A main choice should not need a click to reveal; Filter hid three palettes a level deeper. |
| Look section (knob, texture, invert) | Tune | Refinement, not a choice. |
| Adjust image | Tune | Its contrast sat apart from the look's weight; now they sit together. |
| Two reset buttons | One Reset tuning | One way back. |
| Out: Vector/Raster switch, then formats | Save: all six formats, plus copy | The switch hid half the formats. |

Deliver: no setting removed. Desktop sidebar fits one screen with Tune closed. Browser captures checked default, open Tune, swatch hover and narrow layout.
