# Component registry

| Family | Existing implementation | Status | Notes |
| --- | --- | --- | --- |
| Action button | `press_button` in `src/main.rs` | Proposed | See [button](button.md). Disabled, keyboard and screen-reader behaviour needs review. |
| Choice chip | `chip` in `src/main.rs` | Existing | Used for choices and export actions; behaviour parity is not yet documented. |
| Look card | `look_card` in `src/main.rs` | Existing | Renders a look preview and selection. |
| Dial | `archive_dial` in `src/main.rs` | Existing | Handles density and look values. |
| Colour picker | `duo_picker` in `src/main.rs` | Existing | Wraps an egui colour editor. |
| Section label | `section` in `src/main.rs` | Existing | Text primitive still needs extraction. |
