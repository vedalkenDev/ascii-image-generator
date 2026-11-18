//! Values derived from the existing application interface.
use eframe::egui::Color32;

mod palette {
    use super::Color32;
    pub const NEAR_BLACK: Color32 = Color32::from_rgb(0x05, 0x05, 0x08);
    pub const CHARCOAL: Color32 = Color32::from_rgb(0x24, 0x2a, 0x30);
    pub const CREAM: Color32 = Color32::from_rgb(0xf4, 0xf3, 0xe8);
    pub const SAGE: Color32 = Color32::from_rgb(0x93, 0x9e, 0x7a);
    pub const TEAL: Color32 = Color32::from_rgb(0x3f, 0xb6, 0xaf);
    pub const AMBER: Color32 = Color32::from_rgb(0xc1, 0x8d, 0x5d);
    pub const CORAL: Color32 = Color32::from_rgb(0xd9, 0x63, 0x50);
}

pub mod color {
    use super::{palette, Color32};
    pub const SURFACE: Color32 = palette::NEAR_BLACK;
    pub const SURFACE_RAISED: Color32 = palette::CHARCOAL;
    pub const TEXT: Color32 = palette::CREAM;
    pub const TEXT_SUBTLE: Color32 = palette::SAGE;
    pub const ACCENT: Color32 = palette::TEAL;
    pub const VALUE: Color32 = palette::AMBER;
    pub const ERROR: Color32 = palette::CORAL;
}

pub mod size {
    pub const ACTION_HEIGHT: f32 = 40.0;
}
