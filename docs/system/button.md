<!-- Draft: based on the current egui implementation and call site. -->
# Action button

## Description

The action button starts an operation in the converter.

`press_button`, in `src/main.rs`; egui application code; proposed in the component registry.

## Examples

The existing still-picker call site:

```rust
if press_button(ui, open_label, reduced).clicked() {
    self.open_dialog();
}
```

### Example files

| File | Covers | Caption |
| --- | --- | --- |
| `src/main.rs` | Still picker | Open or change the source image. |

## Variants

None. The function has no variant argument.

## States

- Rest: the label and outline are visible.
- Hover: the fill and outline animate.
- Pressed: the button scales down unless reduced motion is enabled.
- Disabled, error and pending: not implemented by this function.

## Props

| Parameter | Type | Purpose |
| --- | --- | --- |
| `ui` | `&mut egui::Ui` | Allocates and paints the control. |
| `label` | `&str` | Supplies its visible text. |
| `reduced` | `bool` | Disables press animation when true. |

The return value is an `egui::Response`; the caller handles activation.

## Usage

### When to use

Use for the still-picker action, as in the existing converter.

### When not to use

Use the choice chip for a selection in the converter.

### Rules

Not yet specified; only one action-button call site was found.

### Content

Use sentence case. `python3 scripts/check-design-system.py` checks static uppercase literals within its stated scope.

### Anti-slop

Not evaluated.

### Limits

Not measured: test long labels and narrow layouts.

## Accessibility

The implementation uses an egui click response and paints its text separately. Keyboard activation, accessible naming and focus indication have not been confirmed in a rendered browser. The canvas accessibility tree does not expose these controls in the captured build.

## Tokens

`color::SURFACE_RAISED`, `color::TEXT`, and `size::ACTION_HEIGHT` are used through aliases in `src/main.rs`. The hover fill, radius, border and text size remain local values.

## Related

Choice chip: selects a mark, colour or output mode in the existing converter.

Sources: `src/main.rs`, `src/design_system.rs`, `docs/system/components.md`.
