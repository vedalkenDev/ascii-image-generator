mod ascii;
mod io;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use ascii::{Cell, Look, Settings};
use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame, Galley,
    Margin, Pos2, Rect, RichText, Sense, Stroke, Vec2,
};
use eframe::emath::{self, Align};
use image::RgbaImage;

const GROUND: Color32 = Color32::from_rgb(0x05, 0x05, 0x08);
const CHARCOAL: Color32 = Color32::from_rgb(0x24, 0x2a, 0x30);
const CREAM: Color32 = Color32::from_rgb(0xf4, 0xf3, 0xe8);
const SAGE: Color32 = Color32::from_rgb(0x93, 0x9e, 0x7a);
const TEAL: Color32 = Color32::from_rgb(0x3f, 0xb6, 0xaf);
const AMBER: Color32 = Color32::from_rgb(0xc1, 0x8d, 0x5d);
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

#[derive(Clone)]
struct LookThumb {
    look: Look,
    cols: u32,
    rows: u32,
    grid: Vec<Cell>,
}

struct App {
    settings: Settings,
    look: Look,
    source: Option<RgbaImage>,
    source_name: String,
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
    source_gen: u64,
    preview: Option<(u64, Vec2, Vec<Arc<Galley>>)>,
    thumbs: Option<((u64, bool), [LookThumb; 4])>,
    notice: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        apply_theme(&cc.egui_ctx);
        let (jobs_tx, jobs_rx) = mpsc::channel();
        let mut settings = Settings::default();
        Look::Visor.apply(&mut settings);
        let mut app = Self {
            settings,
            look: Look::Visor,
            source: None,
            source_name: String::new(),
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
            source_gen: 0,
            preview: None,
            thumbs: None,
            notice: None,
        };
        if let Some(path) = std::env::args().nth(1) {
            app.busy = Some("Decoding…");
            spawn_load_path(app.jobs_tx.clone(), PathBuf::from(path));
        }
        app
    }

    fn set_source(&mut self, img: RgbaImage, name: String, _ctx: &egui::Context) {
        self.source = Some(img);
        self.source_name = name;
        self.error = None;
        self.notice = None;
        self.source_gen = self.source_gen.wrapping_add(1);
        self.thumbs = None;
        self.dirty = true;
    }

    fn apply_look(&mut self, look: Look) {
        self.look = look;
        look.apply(&mut self.settings);
        self.dirty = true;
    }

    fn ensure_thumbs(&mut self) {
        let key = (self.source_gen, self.settings.invert);
        if self
            .thumbs
            .as_ref()
            .is_some_and(|(k, _)| *k == key)
        {
            return;
        }
        let Some(img) = &self.source else {
            self.thumbs = None;
            return;
        };
        let invert = self.settings.invert;
        let cards = Look::ALL.map(|look| {
            let mut s = Settings::default();
            look.apply(&mut s);
            s.cols = 28;
            s.invert = invert;
            let (cols, rows, grid) = ascii::convert(img, &s);
            LookThumb {
                look,
                cols,
                rows,
                grid,
            }
        });
        self.thumbs = Some((key, cards));
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

    fn copy_txt(&mut self, ctx: &egui::Context) {
        if self.grid.is_empty() {
            self.error = Some("Load a still first.".into());
            return;
        }
        ctx.copy_text(io::to_txt(self.cols, self.rows, &self.grid));
        self.notice = Some("copied".into());
        self.error = None;
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
            .exact_width(320.0)
            .frame(
                Frame::new()
                    .fill(GROUND)
                    .inner_margin(Margin::same(14))
                    .stroke(Stroke::NONE),
            )
            .show(ctx, |ui| {
                let seam = ui.max_rect();
                ui.painter().rect_filled(
                    Rect::from_min_max(seam.min, Pos2::new(seam.max.x, seam.min.y + 3.0)),
                    0,
                    TEAL,
                );
                ui.painter().rect_filled(
                    Rect::from_min_max(
                        Pos2::new(seam.min.x, seam.min.y + 3.0),
                        Pos2::new(seam.max.x, seam.min.y + 16.0),
                    ),
                    0,
                    TEAL.gamma_multiply(0.10),
                );
                ui.painter().vline(
                    seam.left() + 0.5,
                    seam.y_range(),
                    Stroke::new(1.0_f32, CREAM.gamma_multiply(0.28)),
                );
                self.ensure_thumbs();
                let thumbs = self.thumbs.as_ref().map(|(_, cards)| cards.clone());
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                        ui.add_space(4.0);
                        archive_plate(
                            ui,
                            reduced,
                            &self.source_name,
                            self.look,
                            self.cols,
                            self.rows,
                        );
                        let open_label = busy.unwrap_or("OPEN STILL");
                        if press_button(ui, open_label, reduced).clicked() {
                            self.open_dialog();
                        }
                        if let Some(err) = &self.error {
                            ui.label(RichText::new(err).color(CORAL).size(12.0));
                        } else if let Some(note) = &self.notice {
                            ui.label(
                                RichText::new(note)
                                    .family(FontFamily::Monospace)
                                    .color(SAGE)
                                    .size(12.0),
                            );
                        }

                        section(ui, "LOOK");
                        let gap = 6.0;
                        let card_w = (ui.available_width() - gap) / 2.0;
                        for pair in Look::ALL.chunks(2) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                for look in pair {
                                    let thumb = thumbs.as_ref().and_then(|cards| {
                                        cards.iter().find(|t| t.look == *look)
                                    });
                                    if look_card(
                                        ui,
                                        card_w,
                                        *look,
                                        self.look == *look,
                                        thumb,
                                        reduced,
                                    )
                                    .clicked()
                                    {
                                        self.apply_look(*look);
                                    }
                                }
                            });
                        }

                        section(ui, "DENSITY");
                        ui.horizontal(|ui| {
                            let w = (ui.available_width() - 12.0) / 3.0;
                            for n in [32_u32, 80, 132] {
                                if chip(ui, w, &n.to_string(), self.settings.cols == n, reduced)
                                    .clicked()
                                {
                                    self.settings.cols = n;
                                    self.dirty = true;
                                }
                            }
                        });

                        section(ui, self.look.name());
                        if chip(
                            ui,
                            ui.available_width(),
                            if self.settings.invert {
                                "INVERT ON"
                            } else {
                                "INVERT"
                            },
                            self.settings.invert,
                            reduced,
                        )
                        .clicked()
                        {
                            self.settings.invert = !self.settings.invert;
                            self.dirty = true;
                        }
                        let (knob, range) = self.look.knob();
                        let analog = match self.look {
                            Look::Visor | Look::Blocks => &mut self.settings.contrast,
                            Look::Crt => &mut self.settings.brightness,
                            Look::Ansi => &mut self.settings.saturation,
                        };
                        if archive_dial_f32(ui, knob, analog, range, 2) {
                            self.dirty = true;
                        }

                        section(ui, "OUT");
                        ui.horizontal(|ui| {
                            let w = (ui.available_width() - 12.0) / 3.0;
                            for kind in [
                                io::ExportKind::Txt,
                                io::ExportKind::Svg,
                                io::ExportKind::Ansi,
                            ] {
                                if chip(ui, w, kind.label(), false, reduced).clicked() {
                                    self.save_export(kind);
                                }
                            }
                        });
                        if chip(ui, ui.available_width(), "COPY TXT", false, reduced).clicked()
                        {
                            self.copy_txt(ui.ctx());
                        }
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

fn archive_plate(
    ui: &mut egui::Ui,
    reduced: bool,
    name: &str,
    look: Look,
    cols: u32,
    rows: u32,
) {
    let h = if name.is_empty() { 52.0 } else { 68.0 };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::hover());
    let p = ui.painter();
    p.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0_f32, CREAM.gamma_multiply(0.38)),
        egui::StrokeKind::Inside,
    );
    let tick = 9.0;
    let stroke = Stroke::new(1.6_f32, CREAM);
    p.line_segment([rect.left_top(), rect.left_top() + Vec2::new(tick, 0.0)], stroke);
    p.line_segment([rect.left_top(), rect.left_top() + Vec2::new(0.0, tick)], stroke);
    p.line_segment([rect.right_top(), rect.right_top() + Vec2::new(-tick, 0.0)], stroke);
    p.line_segment([rect.right_top(), rect.right_top() + Vec2::new(0.0, tick)], stroke);
    p.line_segment(
        [rect.left_bottom(), rect.left_bottom() + Vec2::new(tick, 0.0)],
        stroke,
    );
    p.line_segment(
        [rect.left_bottom(), rect.left_bottom() + Vec2::new(0.0, -tick)],
        stroke,
    );
    p.line_segment(
        [rect.right_bottom(), rect.right_bottom() + Vec2::new(-tick, 0.0)],
        stroke,
    );
    p.line_segment(
        [rect.right_bottom(), rect.right_bottom() + Vec2::new(0.0, -tick)],
        stroke,
    );

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
    let pos = rect.min + Vec2::new(10.0, 8.0);
    if !reduced {
        p.galley(
            pos + Vec2::new(1.0, 0.0),
            p.layout_job(job(CORAL.gamma_multiply(0.75))),
            CORAL,
        );
        p.galley(
            pos + Vec2::new(-1.0, 0.0),
            p.layout_job(job(TEAL.gamma_multiply(0.75))),
            TEAL,
        );
    }
    p.galley(pos, p.layout_job(job(CREAM)), CREAM);

    let meta = if cols > 0 {
        format!("{}  {cols}×{rows}", look.name())
    } else {
        format!("{}  no still", look.name())
    };
    p.text(
        rect.min + Vec2::new(10.0, 28.0),
        egui::Align2::LEFT_TOP,
        meta,
        FontId::monospace(12.0),
        AMBER,
    );
    if !name.is_empty() {
        p.text(
            rect.min + Vec2::new(10.0, 46.0),
            egui::Align2::LEFT_TOP,
            name,
            FontId::monospace(12.0),
            SAGE,
        );
    }
}

fn look_card(
    ui: &mut egui::Ui,
    width: f32,
    look: Look,
    on: bool,
    thumb: Option<&LookThumb>,
    reduced: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 86.0), Sense::click());
    let s = press_scale(ui, resp.id, resp.is_pointer_button_down_on(), reduced);
    let r = Rect::from_center_size(rect.center(), rect.size() * s);
    ui.painter().rect_filled(r, CornerRadius::same(2), CHARCOAL);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(2),
        Stroke::new(1.0_f32, if on { CREAM } else { CREAM.gamma_multiply(0.22) }),
        egui::StrokeKind::Middle,
    );
    let label_h = 16.0;
    let preview = Rect::from_min_max(
        r.min + Vec2::new(6.0, 6.0),
        Pos2::new(r.max.x - 6.0, r.max.y - label_h - 4.0),
    );
    ui.painter()
        .rect_filled(preview, 0, Color32::from_rgb(0x08, 0x09, 0x0c));
    if let Some(thumb) = thumb {
        paint_cells(ui, preview, thumb.cols, thumb.rows, &thumb.grid);
    } else {
        ui.painter().text(
            preview.center(),
            egui::Align2::CENTER_CENTER,
            look.name(),
            FontId::monospace(11.0),
            SAGE,
        );
    }
    ui.painter().text(
        Pos2::new(r.center().x, r.max.y - 10.0),
        egui::Align2::CENTER_CENTER,
        look.name(),
        FontId::monospace(11.0),
        if on { CREAM } else { SAGE },
    );
    resp
}

fn paint_preview(ui: &egui::Ui, rect: Rect, app: &mut App) {
    ui.painter_at(rect).rect_filled(rect, 0, GROUND);
    if app.cols == 0 || app.rows == 0 {
        return;
    }
    let size = rect.size();
    let miss = match &app.preview {
        Some((gen, cached, _)) => *gen != app.grid_gen || (cached.x - size.x).abs() > 0.5,
        None => true,
    };
    if miss {
        let galleys = layout_cells(ui, rect, app.cols, app.rows, &app.grid);
        app.preview = Some((app.grid_gen, size, galleys));
    }
    if let Some((_, _, galleys)) = &app.preview {
        let font_size = rect.height() / app.rows as f32;
        let painter = ui.painter_at(rect);
        for (y, galley) in galleys.iter().enumerate() {
            let pos = Pos2::new(rect.left(), rect.top() + y as f32 * font_size);
            painter.galley(pos, galley.clone(), CREAM);
        }
    }
}

fn paint_cells(ui: &egui::Ui, rect: Rect, cols: u32, rows: u32, grid: &[Cell]) {
    if cols == 0 || rows == 0 || grid.is_empty() {
        return;
    }
    let galleys = layout_cells(ui, rect, cols, rows, grid);
    let font_size = rect.height() / rows as f32;
    let painter = ui.painter_at(rect);
    for (y, galley) in galleys.iter().enumerate() {
        let pos = Pos2::new(rect.left(), rect.top() + y as f32 * font_size);
        painter.galley(pos, galley.clone(), CREAM);
    }
}

fn layout_cells(
    ui: &egui::Ui,
    rect: Rect,
    cols: u32,
    rows: u32,
    grid: &[Cell],
) -> Vec<Arc<Galley>> {
    let painter = ui.painter();
    let font_size = rect.height() / rows as f32;
    let font = FontId::monospace(font_size);
    let gw = rect.width() / cols as f32;
    let mut galleys = Vec::with_capacity(rows as usize);
    for y in 0..rows {
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = f32::INFINITY;
        let row = y * cols;
        let mut x = 0;
        while x < cols {
            let idx = (row + x) as usize;
            if idx >= grid.len() {
                break;
            }
            let cell = grid[idx];
            let mut run = String::new();
            run.push(cell.ch);
            x += 1;
            while x < cols {
                let nidx = (row + x) as usize;
                if nidx >= grid.len() {
                    break;
                }
                let next = grid[nidx];
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
    galleys
}

fn section(ui: &mut egui::Ui, name: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(format!(":: {name}"))
            .family(FontFamily::Monospace)
            .size(11.0)
            .color(SAGE)
            .extra_letter_spacing(1.4),
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
        Color32::from_rgb(0x2e, 0x34, 0x3a)
    } else {
        CHARCOAL
    };
    ui.painter().rect_filled(r, CornerRadius::same(2), fill);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(2),
        Stroke::new(1.0_f32, CREAM.gamma_multiply(0.55)),
        egui::StrokeKind::Middle,
    );
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::monospace(13.0),
        CREAM,
    );
    resp
}

fn chip(
    ui: &mut egui::Ui,
    width: f32,
    label: &str,
    on: bool,
    reduced: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 26.0), Sense::click());
    let s = press_scale(ui, resp.id, resp.is_pointer_button_down_on(), reduced);
    let r = Rect::from_center_size(rect.center(), rect.size() * s);
    let fill = if on {
        CHARCOAL
    } else if resp.hovered() {
        Color32::from_rgb(0x1b, 0x20, 0x25)
    } else {
        Color32::from_rgb(0x0c, 0x0d, 0x10)
    };
    let stroke = if on {
        CREAM
    } else {
        CREAM.gamma_multiply(0.28)
    };
    ui.painter().rect_filled(r, CornerRadius::same(2), fill);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(2),
        Stroke::new(1.0_f32, stroke),
        egui::StrokeKind::Middle,
    );
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::monospace(11.0),
        if on { CREAM } else { SAGE },
    );
    resp
}

fn archive_dial(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    display: String,
) -> bool {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .family(FontFamily::Monospace)
                .size(12.0)
                .color(CREAM),
        );
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(display)
                    .family(FontFamily::Monospace)
                    .size(12.0)
                    .color(AMBER),
            );
        });
    });
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), Sense::click_and_drag());
    let min = *range.start();
    let max = *range.end();
    let span = (max - min).max(f32::EPSILON);
    let t = ((*value - min) / span).clamp(0.0, 1.0);
    let p = ui.painter();
    p.rect_filled(rect, 0, Color32::from_rgb(0x0c, 0x0d, 0x10));
    p.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0_f32, CREAM.gamma_multiply(0.22)),
        egui::StrokeKind::Inside,
    );
    let fill = Rect::from_min_size(rect.min, Vec2::new((rect.width() * t).max(2.0), rect.height()));
    p.rect_filled(fill, 0, TEAL.gamma_multiply(0.32));
    let x = rect.left() + rect.width() * t;
    p.rect_filled(
        Rect::from_center_size(
            Pos2::new(x, rect.center().y),
            Vec2::new(2.0, rect.height() - 2.0),
        ),
        0,
        CREAM,
    );

    if resp.dragged() || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let nt = ((pos.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
            let nv = min + nt * span;
            if (nv - *value).abs() > f32::EPSILON {
                *value = nv;
                return true;
            }
        }
    }
    false
}

fn archive_dial_f32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    decimals: usize,
) -> bool {
    let display = format!("{value:.decimals$}");
    archive_dial(ui, label, value, range, display)
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
    v.panel_fill = GROUND;
    v.window_fill = GROUND;
    v.extreme_bg_color = GROUND;
    v.faint_bg_color = Color32::from_rgb(0x1b, 0x20, 0x25);
    v.override_text_color = Some(CREAM);
    v.hyperlink_color = TEAL;
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
