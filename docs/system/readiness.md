# Readiness

The system is not ready for wider adoption. The converter uses an egui canvas, so its interactive elements need an accessibility and keyboard contract before extracted components can be considered ready.

| Audit | Verdict | Evidence or next step |
| --- | --- | --- |
| Slots and props | Not assessed | Map the existing button and chip call sites to a proposed API. |
| Behaviour contracts | Not assessed | Exercise activation, focus, disabled and reduced-motion states. |
| Tone parity | Not assessed | The existing controls do not have a shared tone API. |
| Accessible names | Gap | The browser accessibility capture lists the hidden file input and a textbox, but not the painted controls. A keyboard probe left DOM focus on the canvas through five Tab presses; egui's internal focus was not measured. |
| Difficult call sites | Incomplete | The converter was captured empty and loaded at 390 and 1280 pixels. No component-state showcase was captured. |

## Checks and limitations

`python3 scripts/check-design-system.py` checks em dashes and a subset of static uppercase string literals. It is not a Rust parser and does not cover dynamic strings, generated output, or all documentation. Protocol constants, binary magic and decoder error strings are excluded. A planted uppercase label and em dash were rejected in this run.

The build-design-system `check-system.mjs` reports that no supported source files exist: it does not scan Rust. Its self-test passed but does not validate this application's code. Do not treat it as a zero-gap token audit. The remaining hardcoded interface values are noted in [foundations](foundations.md).

The screenshot comparison of the value-only token changes returned no changed pixels for the captured empty and loaded states. After sentence-case changes, differences were observed; the images have not been fully reviewed for unrelated changes. Loading, errors, export, focus, hover, press and reduced-motion transitions need separate checks.
