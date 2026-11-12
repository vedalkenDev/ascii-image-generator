mod ascii;
mod io;

use std::path::{Path, PathBuf};

use ascii::{Cell, Charset, Mode, Settings};
use eframe::egui::{
    self, Color32, ColorImage, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame,
    Margin, Pos2, Rect, RichText, Sense, Stroke, TextureHandle, TextureOptions, Vec2,
};
use eframe::emath::{self, Align, TSTransform};
use image::RgbaImage;

const GROUND: Color32 = Color32::from_rgb(0x05, 0x05, 0x08);
const CHARCOAL: Color32 = Color32::from_rgb(0x24, 0x2a, 0x30);
const CREAM: Color32 = Color32::from_rgb(0xf4, 0xf3, 0xe8);
const SAGE: Color32 = Color32::from_rgb(0x93, 0x9e, 0x7a);
const TEAL: Color32 = Color32::from_rgb(0x3f, 0xb6, 0xaf);
const BLUE: Color32 = Color32::from_rgb(0x42, 0x77, 0xa2);
const CORAL: Color32 = Color32::from_rgb(0xd9, 0x63, 0x50);
const HIT: f32 = 44.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("Cosmic Archive — ASCII"),
        ..Default::default()
    };
    eframe::run_native(
        "Cosmic Archive — ASCII",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

struct App {
    settings: Settings,
    mode: Mode,
    source: Option<RgbaImage>,
    source_name: String,
    texture: Option<TextureHandle>,
    grid: Vec<Cell>,
    cols: u32,
    rows: u32,
    dirty: bool,
    error: Option<String>,
    born: f64,
    reduced_motion: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        apply_theme(&cc.egui_ctx);
        let mut app = Self {
            settings: Settings::default(),
            mode: Mode::Ascii,
            source: None,
            source_name: String::new(),
            texture: None,
            grid: Vec::new(),
            cols: 0,
            rows: 0,
            dirty: false,
            error: None,
            born: cc.egui_ctx.input(|i| i.time),
            reduced_motion: reduced_motion(),
        };
        if let Some(path) = std::env::args().nth(1) {
            app.load_path(Path::new(&path), &cc.egui_ctx);
        }
        app
    }

    fn appear(&self, ctx: &egui::Context) -> f32 {
        if self.reduced_motion {
            return 1.0;
        }
        let t = ((ctx.input(|i| i.time) - self.born) / 0.22).clamp(0.0, 1.0) as f32;
        emath::easing::cubic_out(t)
    }

    fn load_path(&mut self, path: &Path, ctx: &egui::Context) {
        match io::load_path(path) {
            Ok(img) => self.set_source(img, file_stem(path), ctx),
            Err(e) => self.error = Some(e),
        }
    }

    fn load_bytes(&mut self, name: &str, bytes: &[u8], ctx: &egui::Context) {
        match io::load_bytes(bytes) {
            Ok(img) => self.set_source(img, name.to_string(), ctx),
            Err(e) => self.error = Some(e),
        }
    }

    fn set_source(&mut self, img: RgbaImage, name: String, ctx: &egui::Context) {
        let size = [img.width() as usize, img.height() as usize];
        let color = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
        self.texture = Some(ctx.load_texture("source", color, TextureOptions::LINEAR));
        self.source = Some(img);
        self.source_name = name;
        self.error = None;
        self.dirty = true;
    }

    fn rebuild(&mut self) {
        if let Some(img) = &self.source {
            let (cols, rows, grid) = ascii::convert(img, &self.settings);
            self.cols = cols;
            self.rows = rows;
            self.grid = grid;
        }
        self.dirty = false;
    }

    fn poll_drops(&mut self, ctx: &egui::Context) {
        let mut path: Option<PathBuf> = None;
        let mut bytes: Option<(String, Vec<u8>)> = None;
        ctx.input(|i| {
            for f in &i.raw.dropped_files {
                if let Some(p) = &f.path {
                    path = Some(p.clone());
                    break;
                }
                if let Some(b) = &f.bytes {
                    let name = if f.name.is_empty() {
                        "dropped".into()
                    } else {
                        f.name.clone()
                    };
                    bytes = Some((name, b.to_vec()));
                    break;
                }
            }
        });
        if let Some(p) = path {
            self.load_path(&p, ctx);
        } else if let Some((name, b)) = bytes {
            self.load_bytes(&name, &b, ctx);
        }
    }

    fn open_dialog(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Image",
                &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff", "svg"],
            )
            .pick_file()
        {
            self.load_path(&path, ctx);
        }
    }

    fn save_export(&mut self, kind: io::ExportKind) {
        if self.grid.is_empty() {
            self.error = Some("Load a still first.".into());
            return;
        }
        let stem = Path::new(&self.source_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("ascii");
        let name = format!("{stem}.{}", kind.ext());
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(kind.label(), &[kind.ext()])
            .set_file_name(&name)
            .save_file()
        {
            match io::encode(kind, self.cols, self.rows, &self.grid) {
                Ok(bytes) => {
                    if let Err(e) = std::fs::write(&path, bytes) {
                        self.error = Some(format!("Could not write file: {e}"));
                    }
                }
                Err(e) => self.error = Some(e),
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);
        self.poll_drops(ctx);
        if self.dirty {
            self.rebuild();
        }
        let appear = self.appear(ctx);
        if appear < 1.0 {
            ctx.request_repaint();
        }

        egui::SidePanel::right("controls")
            .resizable(false)
            .exact_width(300.0)
            .frame(
                Frame::new()
                    .fill(CHARCOAL)
                    .inner_margin(Margin::same(20))
                    .stroke(Stroke::NONE),
            )
            .show(ctx, |ui| {
                let seam = ui.max_rect();
                ui.painter().vline(
                    seam.left() + 0.5,
                    seam.y_range(),
                    Stroke::new(1.0_f32, TEAL.gamma_multiply(0.45)),
                );
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);
                {
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), Sense::hover());
                    let font = FontId::monospace(11.0);
                    let job = |color: Color32| {
                        let mut job = egui::text::LayoutJob::default();
                        job.append(
                            "COSMIC ARCHIVE",
                            0.0,
                            egui::TextFormat {
                                font_id: font.clone(),
                                color,
                                extra_letter_spacing: 2.2,
                                ..Default::default()
                            },
                        );
                        job
                    };
                    let painter = ui.painter();
                    let pos = Pos2::new(rect.left(), rect.top());
                    if !self.reduced_motion {
                        painter.galley(
                            pos + Vec2::new(1.0, 0.0),
                            painter.layout_job(job(CORAL.gamma_multiply(0.75))),
                            CORAL,
                        );
                        painter.galley(
                            pos + Vec2::new(-1.0, 0.0),
                            painter.layout_job(job(TEAL.gamma_multiply(0.75))),
                            TEAL,
                        );
                    }
                    painter.galley(pos, painter.layout_job(job(CREAM)), CREAM);
                }
                ui.label(
                    RichText::new("ASCII stills")
                        .family(FontFamily::Monospace)
                        .size(20.0)
                        .color(CREAM)
                        .extra_letter_spacing(0.6),
                );
                if !self.source_name.is_empty() {
                    ui.label(RichText::new(&self.source_name).size(13.0).color(SAGE));
                }
                ui.add_space(4.0);
                if press_button(ui, "Open image", self.reduced_motion).clicked() {
                    self.open_dialog(ctx);
                }
                if let Some(err) = &self.error {
                    ui.label(RichText::new(err).color(CORAL).size(13.0));
                }

                section(ui, "Export");
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                let kinds = [
                    io::ExportKind::Txt,
                    io::ExportKind::Png,
                    io::ExportKind::Svg,
                    io::ExportKind::Html,
                    io::ExportKind::Ansi,
                ];
                let cell_w = (ui.available_width() - 12.0) / 3.0;
                ui.horizontal_wrapped(|ui| {
                    for kind in kinds {
                        if chip(ui, cell_w, kind.label(), self.reduced_motion).clicked()
                        {
                            self.save_export(kind);
                        }
                    }
                });
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);

                section(ui, "Mode");
                ui.horizontal(|ui| {
                    let w = (ui.available_width() - 8.0) / 2.0;
                    if segment(ui, w, "Overlay", self.mode == Mode::Overlay, self.reduced_motion)
                        .clicked()
                    {
                        self.mode = Mode::Overlay;
                    }
                    if segment(ui, w, "ASCII", self.mode == Mode::Ascii, self.reduced_motion)
                        .clicked()
                    {
                        self.mode = Mode::Ascii;
                    }
                });

                section(ui, "Characters");
                for set in [Charset::Alphanumeric, Charset::Blocks, Charset::Punctuation] {
                    if segment(
                        ui,
                        ui.available_width(),
                        set.label(),
                        self.settings.charset == set,
                        self.reduced_motion,
                    )
                    .clicked()
                    {
                        self.settings.charset = set;
                        self.dirty = true;
                    }
                }

                section(ui, "Dials");
                if dial_u32(ui, "Columns", &mut self.settings.cols, 20..=240) {
                    self.dirty = true;
                }
                if dial_f32(ui, "Contrast", &mut self.settings.contrast, 0.4..=2.4) {
                    self.dirty = true;
                }
                if dial_f32(ui, "Saturation", &mut self.settings.saturation, 0.0..=2.0) {
                    self.dirty = true;
                }
                if dial_f32(ui, "Brightness", &mut self.settings.brightness, -0.5..=0.5) {
                    self.dirty = true;
                }

                ui.add_space(4.0);
                if toggle_row(ui, "Invert", &mut self.settings.invert) {
                    self.dirty = true;
                }
                if toggle_row(ui, "Color", &mut self.settings.color) {
                    self.dirty = true;
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new("Drop PNG, JPEG, WebP, GIF, BMP, or TIFF. Dials resample the cache.")
                        .size(12.0)
                        .color(SAGE),
                );
                ui.add_space(12.0);
                    });
            });

        egui::CentralPanel::default()
            .frame(Frame::new().fill(GROUND).inner_margin(Margin::same(16)))
            .show(ctx, |ui| {
                let avail = ui.available_rect_before_wrap();
                let scale = 0.95 + 0.05 * appear;
                let layer = ui.layer_id();
                if appear < 1.0 {
                    let c = avail.center();
                    ctx.set_transform_layer(
                        layer,
                        TSTransform::from_translation(c.to_vec2())
                            * TSTransform::from_scaling(scale)
                            * TSTransform::from_translation(-c.to_vec2()),
                    );
                } else {
                    ctx.set_transform_layer(layer, TSTransform::IDENTITY);
                }

                if self.source.is_none() {
                    let (resp, painter) = ui.allocate_painter(avail.size(), Sense::click());
                    let fill = fade(Color32::from_rgb(0x10, 0x11, 0x14), appear);
                    painter.rect_filled(resp.rect, CornerRadius::same(4), fill);
                    painter.rect_stroke(
                        resp.rect.shrink(10.0),
                        CornerRadius::ZERO,
                        Stroke::new(1.0_f32, fade(CREAM.gamma_multiply(0.35), appear)),
                        egui::StrokeKind::Inside,
                    );
                    let c = resp.rect.center();
                    painter.text(
                        c - Vec2::new(0.0, 14.0),
                        egui::Align2::CENTER_CENTER,
                        "NO STILL LOADED",
                        FontId::monospace(16.0),
                        fade(CREAM, appear),
                    );
                    painter.text(
                        c + Vec2::new(0.0, 12.0),
                        egui::Align2::CENTER_CENTER,
                        "drop a still into the archive",
                        FontId::monospace(13.0),
                        fade(CREAM.gamma_multiply(0.55), appear),
                    );
                    if resp.clicked() {
                        self.open_dialog(ctx);
                    }
                    return;
                }

                let img = self.source.as_ref().unwrap();
                let fit = fit_rect(avail, img.width() as f32, img.height() as f32);
                ui.allocate_rect(avail, Sense::hover());
                paint_preview(ui, fit, self);
            });
    }
}

fn paint_preview(ui: &egui::Ui, rect: Rect, app: &App) {
    let painter = ui.painter_at(rect);
    match app.mode {
        Mode::Overlay => {
            if let Some(tex) = &app.texture {
                painter.image(
                    tex.id(),
                    rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            painter.rect_filled(rect, 0, Color32::from_black_alpha(90));
        }
        Mode::Ascii => {
            painter.rect_filled(rect, 0, GROUND);
        }
    }
    if app.cols == 0 || app.rows == 0 {
        return;
    }
    let font_size = rect.height() / app.rows as f32;
    let font = FontId::monospace(font_size);
    let gw = rect.width() / app.cols as f32;
    for y in 0..app.rows {
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = f32::INFINITY;
        let row = y * app.cols;
        for x in 0..app.cols {
            let cell = app.grid[(row + x) as usize];
            let c = cell.color;
            let mut buf = [0u8; 4];
            job.append(
                cell.ch.encode_utf8(&mut buf),
                0.0,
                egui::TextFormat {
                    font_id: font.clone(),
                    color: Color32::from_rgb(c[0], c[1], c[2]),
                    extra_letter_spacing: gw - font_size * 0.6,
                    ..Default::default()
                },
            );
        }
        let galley = painter.layout_job(job);
        let pos = Pos2::new(rect.left(), rect.top() + y as f32 * font_size);
        painter.galley(pos, galley, CREAM);
    }
}

fn section(ui: &mut egui::Ui, name: &str) {
    ui.add_space(8.0);
    ui.label(
        RichText::new(name)
            .size(11.0)
            .color(SAGE)
            .extra_letter_spacing(0.8),
    );
}

fn press_scale(ui: &egui::Ui, id: egui::Id, pressed: bool, reduced: bool) -> f32 {
    if reduced {
        return 1.0;
    }
    let t = ui.ctx().animate_bool_with_time_and_easing(
        id.with("press"),
        pressed,
        0.12,
        emath::easing::cubic_out,
    );
    1.0 - 0.03 * t
}

fn press_button(ui: &mut egui::Ui, label: &str, reduced: bool) -> egui::Response {
    let size = Vec2::new(ui.available_width(), HIT);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let s = press_scale(ui, resp.id, resp.is_pointer_button_down_on(), reduced);
    let r = Rect::from_center_size(rect.center(), rect.size() * s);
    let fill = if resp.hovered() {
        Color32::from_rgb(0x2e, 0x36, 0x3c)
    } else {
        Color32::from_rgb(0x1b, 0x20, 0x25)
    };
    ui.painter().rect_filled(r, CornerRadius::same(8), fill);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(8),
        Stroke::new(1.0_f32, TEAL.gamma_multiply(0.55)),
        egui::StrokeKind::Middle,
    );
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(15.0),
        CREAM,
    );
    resp
}

fn chip(ui: &mut egui::Ui, width: f32, label: &str, reduced: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::click());
    let s = press_scale(ui, resp.id, resp.is_pointer_button_down_on(), reduced);
    let r = Rect::from_center_size(rect.center(), rect.size() * s);
    let fill = if resp.hovered() {
        Color32::from_rgb(0x2e, 0x36, 0x3c)
    } else {
        Color32::from_rgb(0x1b, 0x20, 0x25)
    };
    ui.painter().rect_filled(r, CornerRadius::same(4), fill);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(4),
        Stroke::new(1.0_f32, TEAL.gamma_multiply(0.45)),
        egui::StrokeKind::Middle,
    );
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::monospace(12.0),
        CREAM,
    );
    resp
}

fn segment(ui: &mut egui::Ui, width: f32, label: &str, on: bool, reduced: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, HIT), Sense::click());
    let s = press_scale(ui, resp.id, resp.is_pointer_button_down_on(), reduced);
    let r = Rect::from_center_size(rect.center(), rect.size() * s);
    let fill = if on {
        Color32::from_rgb(0x1a, 0x3a, 0x3a)
    } else if resp.hovered() {
        Color32::from_rgb(0x2e, 0x36, 0x3c)
    } else {
        Color32::from_rgb(0x1b, 0x20, 0x25)
    };
    let stroke = if on { TEAL } else { Color32::from_rgb(0x3a, 0x42, 0x48) };
    ui.painter().rect_filled(r, CornerRadius::same(8), fill);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(8),
        Stroke::new(1.0_f32, stroke),
        egui::StrokeKind::Middle,
    );
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::monospace(13.0),
        if on { CREAM } else { SAGE },
    );
    resp
}

fn dial_u32(ui: &mut egui::Ui, label: &str, value: &mut u32, range: std::ops::RangeInclusive<u32>) -> bool {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(CREAM));
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{value}"))
                    .family(FontFamily::Monospace)
                    .size(13.0)
                    .color(TEAL),
            );
        });
    });
    ui.add(
        egui::Slider::new(value, range)
            .show_value(false)
            .min_decimals(0),
    )
    .changed()
}

fn dial_f32(ui: &mut egui::Ui, label: &str, value: &mut f32, range: std::ops::RangeInclusive<f32>) -> bool {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(CREAM));
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{value:.2}"))
                    .family(FontFamily::Monospace)
                    .size(13.0)
                    .color(TEAL),
            );
        });
    });
    ui.add(egui::Slider::new(value, range).show_value(false)).changed()
}

fn toggle_row(ui: &mut egui::Ui, label: &str, on: &mut bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), HIT), Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    ui.painter().text(
        Pos2::new(rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.0),
        CREAM,
    );
    let knob_w = 44.0;
    let knob = Rect::from_center_size(
        Pos2::new(rect.right() - knob_w / 2.0 - 2.0, rect.center().y),
        Vec2::new(knob_w, 24.0),
    );
    ui.painter().rect_filled(
        knob,
        CornerRadius::same(12),
        if *on { TEAL } else { Color32::from_rgb(0x3a, 0x42, 0x48) },
    );
    let x = if *on { knob.right() - 12.0 } else { knob.left() + 12.0 };
    ui.painter().circle_filled(Pos2::new(x, knob.center().y), 8.0, CREAM);
    resp.clicked()
}

fn fit_rect(avail: Rect, img_w: f32, img_h: f32) -> Rect {
    let ia = img_w.max(1.0) / img_h.max(1.0);
    let aa = avail.width() / avail.height().max(1.0);
    let size = if ia > aa {
        Vec2::new(avail.width(), avail.width() / ia)
    } else {
        Vec2::new(avail.height() * ia, avail.height())
    };
    Rect::from_center_size(avail.center(), size)
}

fn file_stem(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string()
}

fn fade(c: Color32, t: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as f32 * t).round() as u8)
}

fn reduced_motion() -> bool {
    if let Ok(v) = std::env::var("PREFERS_REDUCED_MOTION") {
        let v = v.to_ascii_lowercase();
        if matches!(v.as_str(), "1" | "true" | "yes") {
            return true;
        }
    }
    if let Ok(out) = std::process::Command::new("hyprctl")
        .args(["getoption", "animations:enabled", "-j"])
        .output()
    {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.contains("\"int\": 0") || s.contains("\"int\":0") {
            return true;
        }
    }
    false
}

fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.dark_mode = true;
    v.panel_fill = CHARCOAL;
    v.window_fill = CHARCOAL;
    v.extreme_bg_color = GROUND;
    v.faint_bg_color = Color32::from_rgb(0x1b, 0x20, 0x25);
    v.override_text_color = Some(CREAM);
    v.hyperlink_color = BLUE;
    v.warn_fg_color = TEAL;
    v.error_fg_color = CORAL;
    v.selection.bg_fill = TEAL.gamma_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0_f32, TEAL);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, CREAM);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, CREAM);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, CREAM);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, CREAM);
    v.widgets.open.fg_stroke = Stroke::new(1.0_f32, CREAM);
    v.widgets.inactive.bg_fill = Color32::from_rgb(0x1b, 0x20, 0x25);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0x2e, 0x36, 0x3c);
    v.widgets.active.bg_fill = Color32::from_rgb(0x1a, 0x3a, 0x3a);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x3a, 0x42, 0x48));
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, BLUE);
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, TEAL);
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.spacing.interact_size = Vec2::new(40.0, HIT);
        s.spacing.slider_width = 260.0;
        s.spacing.button_padding = Vec2::new(12.0, 10.0);
        s.interaction.tooltip_delay = 0.0;
        s.interaction.tooltip_grace_time = 0.0;
        s.animation_time = 0.12;
    });
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    if let Ok(bytes) = std::fs::read("/usr/share/fonts/noto/NotoSans-Regular.ttf") {
        fonts.font_data.insert(
            "ui".into(),
            std::sync::Arc::new(FontData::from_owned(bytes)),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "ui".into());
    }
    const MONO: &[&str] = &[
        "/usr/share/fonts/noto/NotoSansMono-Regular.ttf",
        "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
        "/usr/share/fonts/Adwaita/AdwaitaMono-Regular.ttf",
    ];
    for path in MONO {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "mono_blocks".into(),
                std::sync::Arc::new(FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "mono_blocks".into());
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push("mono_blocks".into());
            break;
        }
    }
    ctx.set_fonts(fonts);
}
