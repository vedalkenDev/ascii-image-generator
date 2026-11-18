# Foundations

These values are taken from the existing converter. The app currently ships one dark theme. `src/design_system.rs` holds the palette and semantic aliases; the conversion palette in `src/ascii.rs` produces image content and is separate from interface colours.

| Role | Rust token | Existing value |
| --- | --- | --- |
| Base surface | `color::SURFACE` | `#050508` |
| Raised surface | `color::SURFACE_RAISED` | `#242a30` |
| Primary text | `color::TEXT` | `#f4f3e8` |
| Subtle text | `color::TEXT_SUBTLE` | `#939e7a` |
| Accent | `color::ACCENT` | `#3fb6af` |
| Value text | `color::VALUE` | `#c18d5d` |
| Error text | `color::ERROR` | `#d96350` |
| Primary action height | `size::ACTION_HEIGHT` | 40 points |

Surface and text roles are paired as base surface with primary or subtle text, and raised surface with primary text. Contrast has not yet been audited.

The UI uses its bundled Noto Sans Mono font for the monospace family. The existing animation functions and reduced-motion behaviour remain in `src/main.rs`; no new motion values are introduced.

## Known gaps

Control hover, selected, border, canvas preview and colour picker values are still local to `src/main.rs`. Font sizes, spacing, radii and animation durations have not been mapped into semantic tokens. Image-derived colours are content, not interface tokens. The Rust-aware copy check is limited to static string literals and cannot verify dynamic labels or accessibility.
