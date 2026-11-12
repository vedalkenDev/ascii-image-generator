mod ascii;
mod io;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use ascii::{Cell, Charset, Mode, Settings};
use eframe::egui::{
    self, Color32, ColorImage, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame,
    Galley, Margin, Pos2, Rect, RichText, Sense, Stroke, TextureHandle, TextureOptions, Vec2,
};
use eframe::emath::{self, Align};
use image::RgbaImage;

const GROUND: Color32 = Color32::from_rgb(0x05, 0x05, 0x08);
const CHARCOAL: Color32 = Color32::from_rgb(0x24, 0x2a, 0x30);
const CREAM: Color32 = Color32::from_rgb(0xf4, 0xf3, 0xe8);
const SAGE: Color32 = Color32::from_rgb(0x93, 0x9e, 0x7a);
const TEAL: Color32 = Color32::from_rgb(0x3f, 0xb6, 0xaf);
const BLUE: Color32 = Color32::from_rgb(0x42, 0x77, 0xa2);
const CORAL: Color32 = Color32::from_rgb(0xd9, 0x63, 0x50);
const HIT: f32 = 40.0;
const OPEN_FILTERS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff", "svg",
];

enum Job {
    Loaded { name: String, img: RgbaImage },
    Failed(String),
    Idle,
    Saved,
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 680.0])
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
    reduced_motion: bool,
    jobs_tx: Sender<Job>,
    jobs_rx: Receiver<Job>,
    busy: Option<&'static str>,
    grid_gen: u64,
    preview: Option<(u64, Vec2, Vec<Arc<Galley>>)>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        apply_theme(&cc.egui_ctx);
        let (jobs_tx, jobs_rx) = mpsc::channel();
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
            reduced_motion: reduced_motion(),
            jobs_tx,
            jobs_rx,
            busy: None,
            grid_gen: 0,
            preview: None,
        };
        if let Some(path) = std::env::args().nth(1) {
            app.busy = Some("Decoding…");
            spawn_load_path(app.jobs_tx.clone(), PathBuf::from(path));
        }
        app
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
            self.grid_gen = self.grid_gen.wrapping_add(1);
            self.preview = None;
        }
        self.dirty = false;
    }

    fn poll_jobs(&mut self, ctx: &egui::Context) {
        while let Ok(job) = self.jobs_rx.try_recv() {
            match job {
                Job::Loaded { name, img } => {
                    self.set_source(img, name, ctx);
                    self.busy = None;
                }
                Job::Failed(e) => {
                    self.error = Some(e);
                    self.busy = None;
                }
                Job::Idle | Job::Saved => {
                    self.busy = None;
                }
            }
        }
        if self.busy.is_some() {
            ctx.request_repaint();
        }
    }

    fn poll_drops(&mut self, ctx: &egui::Context) {
        if self.busy.is_some() {
            return;
        }
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
            self.busy = Some("Decoding…");
            spawn_load_path(self.jobs_tx.clone(), p);
        } else if let Some((name, b)) = bytes {
            self.busy = Some("Decoding…");
            spawn_load_bytes(self.jobs_tx.clone(), name, b);
        }
    }

    fn open_dialog(&mut self) {
        if self.busy.is_some() {
            return;
        }
        self.busy = Some("Opening…");
        self.error = None;
        spawn_pick(self.jobs_tx.clone());
    }

    fn save_export(&mut self, kind: io::ExportKind) {
        if self.busy.is_some() {
            return;
        }
        if self.grid.is_empty() {
            self.error = Some("Load a still first.".into());
            return;
        }
        self.busy = Some("Exporting…");
        let stem = Path::new(&self.source_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("ascii")
            .to_string();
        spawn_save(
            self.jobs_tx.clone(),
            kind,
            stem,
            self.cols,
            self.rows,
            self.grid.clone(),
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_jobs(ctx);
        self.poll_drops(ctx);
        if self.dirty {
            self.rebuild();
        }
        let reduced = self.reduced_motion;
        let busy = self.busy;

        egui::SidePanel::right("controls")
            .resizable(false)
            .exact_width(280.0)
            .frame(
                Frame::new()
                    .fill(CHARCOAL)
                    .inner_margin(Margin::same(16))
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
                        ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);
                        brand(ui, reduced);
                        ui.label(
                            RichText::new("ASCII stills")
                                .family(FontFamily::Monospace)
                                .size(18.0)
                                .color(CREAM)
                                .extra_letter_spacing(0.6),
                        );
                        if !self.source_name.is_empty() {
                            ui.label(RichText::new(&self.source_name).size(12.0).color(SAGE));
                        }
                        let open_label = busy.unwrap_or("Open still");
                        if press_button(ui, open_label, reduced).clicked() {
                            self.open_dialog();
                        }
                        if let Some(err) = &self.error {
                            ui.label(RichText::new(err).color(CORAL).size(12.0));
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
                        if dial_f32(ui, "Brightness", &mut self.settings.brightness, -0.5..=0.5)
                        {
                            self.dirty = true;
                        }
                        if toggle_row(ui, "Invert", &mut self.settings.invert) {
                            self.dirty = true;
                        }
                        if toggle_row(ui, "Color", &mut self.settings.color) {
                            self.dirty = true;
                        }

                        section(ui, "Characters");
                        for set in [Charset::Alphanumeric, Charset::Blocks, Charset::Punctuation]
                        {
                            if segment(
                                ui,
                                ui.available_width(),
                                set.label(),
                                self.settings.charset == set,
                                reduced,
                            )
                            .clicked()
                            {
                                self.settings.charset = set;
                                self.dirty = true;
                            }
                        }

                        section(ui, "Mode");
                        ui.horizontal(|ui| {
                            let w = (ui.available_width() - 6.0) / 2.0;
                            if segment(ui, w, "Overlay", self.mode == Mode::Overlay, reduced)
                                .clicked()
                            {
                                self.mode = Mode::Overlay;
                            }
                            if segment(ui, w, "ASCII", self.mode == Mode::Ascii, reduced)
                                .clicked()
                            {
                                self.mode = Mode::Ascii;
                            }
                        });

                        section(ui, "Export");
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
                                if chip(ui, cell_w, kind.label(), reduced).clicked() {
                                    self.save_export(kind);
                                }
                            }
                        });
                        ui.label(
                            RichText::new("Drop PNG, JPEG, WebP, GIF, BMP, TIFF, or SVG.")
                                .size(11.0)
                                .color(SAGE),
                        );
                    });
            });

        egui::CentralPanel::default()
            .frame(Frame::new().fill(GROUND).inner_margin(Margin::same(16)))
            .show(ctx, |ui| {
                let avail = ui.available_rect_before_wrap();
                if self.source.is_none() {
                    let (resp, painter) = ui.allocate_painter(avail.size(), Sense::click());
                    painter.rect_filled(
                        resp.rect,
                        CornerRadius::same(4),
                        Color32::from_rgb(0x10, 0x11, 0x14),
                    );
                    painter.rect_stroke(
                        resp.rect.shrink(10.0),
                        CornerRadius::ZERO,
                        Stroke::new(1.0_f32, CREAM.gamma_multiply(0.35)),
                        egui::StrokeKind::Inside,
                    );
                    let c = resp.rect.center();
                    let (title, sub) = if busy.is_some() {
                        ("READING STILL", "the archive is decoding")
                    } else {
                        ("NO STILL LOADED", "drop a still into the archive")
                    };
                    painter.text(
                        c - Vec2::new(0.0, 14.0),
                        egui::Align2::CENTER_CENTER,
                        title,
                        FontId::monospace(16.0),
                        CREAM,
                    );
                    painter.text(
                        c + Vec2::new(0.0, 12.0),
                        egui::Align2::CENTER_CENTER,
                        sub,
                        FontId::monospace(13.0),
                        CREAM.gamma_multiply(0.55),
                    );
                    if resp.clicked() {
                        self.open_dialog();
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

fn spawn_pick(tx: Sender<Job>) {
    thread::spawn(move || {
        let picked = rfd::FileDialog::new()
            .add_filter("Image", OPEN_FILTERS)
            .pick_file();
        match picked {
            None => {
                let _ = tx.send(Job::Idle);
            }
            Some(p) => match io::load_path(&p) {
                Ok(img) => {
                    let _ = tx.send(Job::Loaded {
                        name: file_stem(&p),
                        img,
                    });
                }
                Err(e) => {
                    let _ = tx.send(Job::Failed(e));
                }
            },
        }
    });
}

fn spawn_load_path(tx: Sender<Job>, path: PathBuf) {
    thread::spawn(move || match io::load_path(&path) {
        Ok(img) => {
            let _ = tx.send(Job::Loaded {
                name: file_stem(&path),
                img,
            });
        }
        Err(e) => {
            let _ = tx.send(Job::Failed(e));
        }
    });
}

fn spawn_load_bytes(tx: Sender<Job>, name: String, bytes: Vec<u8>) {
    thread::spawn(move || match io::load_bytes(&bytes) {
        Ok(img) => {
            let _ = tx.send(Job::Loaded { name, img });
        }
        Err(e) => {
            let _ = tx.send(Job::Failed(e));
        }
    });
}

fn spawn_save(
    tx: Sender<Job>,
    kind: io::ExportKind,
    stem: String,
    cols: u32,
    rows: u32,
    grid: Vec<Cell>,
) {
    thread::spawn(move || {
        let name = format!("{stem}.{}", kind.ext());
        let Some(path) = rfd::FileDialog::new()
            .add_filter(kind.label(), &[kind.ext()])
            .set_file_name(&name)
            .save_file()
        else {
            let _ = tx.send(Job::Idle);
            return;
        };
        match io::encode(kind, cols, rows, &grid) {
            Ok(bytes) => {
                if let Err(e) = std::fs::write(&path, bytes) {
                    let _ = tx.send(Job::Failed(format!("Could not write file: {e}")));
                } else {
                    let _ = tx.send(Job::Saved);
                }
            }
            Err(e) => {
                let _ = tx.send(Job::Failed(e));
            }
        }
    });
}

fn brand(ui: &mut egui::Ui, reduced: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), Sense::hover());
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
    if !reduced {
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

fn paint_preview(ui: &egui::Ui, rect: Rect, app: &mut App) {
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
    let size = rect.size();
    let miss = match &app.preview {
        Some((gen, cached, _)) => *gen != app.grid_gen || (cached.x - size.x).abs() > 0.5,
        None => true,
    };
    if miss {
        let font_size = rect.height() / app.rows as f32;
        let font = FontId::monospace(font_size);
        let gw = rect.width() / app.cols as f32;
        let mut galleys = Vec::with_capacity(app.rows as usize);
        for y in 0..app.rows {
            let mut job = egui::text::LayoutJob::default();
            job.wrap.max_width = f32::INFINITY;
            let row = y * app.cols;
            let mut x = 0;
            while x < app.cols {
                let cell = app.grid[(row + x) as usize];
                let mut run = String::new();
                run.push(cell.ch);
                x += 1;
                while x < app.cols {
                    let next = app.grid[(row + x) as usize];
                    if next.color != cell.color {
                        break;
                    }
                    run.push(next.ch);
                    x += 1;
                }
                let c = cell.color;
                job.append(
                    &run,
                    0.0,
                    egui::TextFormat {
                        font_id: font.clone(),
                        color: Color32::from_rgb(c[0], c[1], c[2]),
                        extra_letter_spacing: gw - font_size * 0.6,
                        ..Default::default()
                    },
                );
            }
            galleys.push(painter.layout_job(job));
        }
        app.preview = Some((app.grid_gen, size, galleys));
    }
    if let Some((_, _, galleys)) = &app.preview {
        let font_size = rect.height() / app.rows as f32;
        for (y, galley) in galleys.iter().enumerate() {
            let pos = Pos2::new(rect.left(), rect.top() + y as f32 * font_size);
            painter.galley(pos, galley.clone(), CREAM);
        }
    }
}

fn section(ui: &mut egui::Ui, name: &str) {
    ui.add_space(6.0);
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
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::click());
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
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 34.0), Sense::click());
    let s = press_scale(ui, resp.id, resp.is_pointer_button_down_on(), reduced);
    let r = Rect::from_center_size(rect.center(), rect.size() * s);
    let fill = if on {
        Color32::from_rgb(0x1a, 0x3a, 0x3a)
    } else if resp.hovered() {
        Color32::from_rgb(0x2e, 0x36, 0x3c)
    } else {
        Color32::from_rgb(0x1b, 0x20, 0x25)
    };
    let stroke = if on {
        TEAL
    } else {
        Color32::from_rgb(0x3a, 0x42, 0x48)
    };
    ui.painter().rect_filled(r, CornerRadius::same(6), fill);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(6),
        Stroke::new(1.0_f32, stroke),
        egui::StrokeKind::Middle,
    );
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::monospace(12.0),
        if on { CREAM } else { SAGE },
    );
    resp
}

fn dial_u32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut u32,
    range: std::ops::RangeInclusive<u32>,
) -> bool {
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

fn dial_f32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) -> bool {
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
    ui.add(egui::Slider::new(value, range).show_value(false))
        .changed()
}

fn toggle_row(ui: &mut egui::Ui, label: &str, on: &mut bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    ui.painter().text(
        Pos2::new(rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        CREAM,
    );
    let knob_w = 40.0;
    let knob = Rect::from_center_size(
        Pos2::new(rect.right() - knob_w / 2.0 - 2.0, rect.center().y),
        Vec2::new(knob_w, 22.0),
    );
    ui.painter().rect_filled(
        knob,
        CornerRadius::same(11),
        if *on {
            TEAL
        } else {
            Color32::from_rgb(0x3a, 0x42, 0x48)
        },
    );
    let x = if *on {
        knob.right() - 11.0
    } else {
        knob.left() + 11.0
    };
    ui.painter()
        .circle_filled(Pos2::new(x, knob.center().y), 7.0, CREAM);
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

fn reduced_motion() -> bool {
    if let Ok(v) = std::env::var("PREFERS_REDUCED_MOTION") {
        let v = v.to_ascii_lowercase();
        if matches!(v.as_str(), "1" | "true" | "yes") {
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
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, TEAL);
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, TEAL);
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.spacing.interact_size = Vec2::new(40.0, HIT);
        s.spacing.slider_width = 240.0;
        s.spacing.button_padding = Vec2::new(12.0, 8.0);
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
