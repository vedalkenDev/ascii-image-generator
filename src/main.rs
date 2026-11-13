#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ascii;
mod io;

use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;

use ascii::{Cell, ColorKind, Look, Mark, Settings};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
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
#[cfg(not(target_arch = "wasm32"))]
const OPEN_FILTERS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff", "svg",
];

#[allow(dead_code)]
enum Job {
    Loaded { name: String, img: RgbaImage },
    Failed(String),
    Idle,
    Saved(String),
    Busy(&'static str),
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("window")
            .document()
            .expect("document");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("canvas");
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await
            .expect("web start");
    });
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
    thumbs: Option<((u64, bool, Mark), [LookThumb; 4])>,
    notice: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        apply_theme(&cc.egui_ctx);
        let (jobs_tx, jobs_rx) = mpsc::channel();
        let mut settings = Settings::default();
        Look::Visor.apply(&mut settings);
        #[allow(unused_mut)]
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
        #[cfg(target_arch = "wasm32")]
        install_web_still_hooks(app.jobs_tx.clone(), cc.egui_ctx.clone());
        #[cfg(not(target_arch = "wasm32"))]
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
        let colour = self.settings.color_kind;
        self.look = look;
        look.apply(&mut self.settings);
        self.settings.color_kind = colour;
        self.dirty = true;
    }

    fn ensure_thumbs(&mut self) {
        let key = (self.source_gen, self.settings.invert, self.settings.mark);
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
                Job::Idle => {
                    self.busy = None;
                }
                Job::Saved(name) => {
                    self.busy = None;
                    self.notice = Some(format!("saved {name}"));
                    self.error = None;
                }
                Job::Busy(s) => {
                    self.busy = Some(s);
                    self.error = None;
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
        #[cfg(not(target_arch = "wasm32"))]
        let mut path: Option<PathBuf> = None;
        let mut bytes: Option<(String, Vec<u8>)> = None;
        ctx.input(|i| {
            for f in &i.raw.dropped_files {
                #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(p) = path {
            self.busy = Some("Decoding…");
            spawn_load_path(self.jobs_tx.clone(), p);
            return;
        }
        if let Some((name, b)) = bytes {
            self.busy = Some("Decoding…");
            ctx.request_repaint();
            spawn_load_bytes(self.jobs_tx.clone(), name, b);
        }
    }

    fn open_dialog(&mut self) {
        if self.busy.is_some() {
            return;
        }
        self.error = None;
        // Web: click the hidden <input> in this user-gesture. Do not flash
        // Opening… — that was the janky delay before the picker appeared.
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.busy = Some("Opening…");
        }
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
        let stem = Path::new(&self.source_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("ascii")
            .to_string();
        let name = format!("{stem}.{}", kind.ext());
        #[cfg(target_arch = "wasm32")]
        {
            match io::encode(kind, self.cols, self.rows, &self.grid) {
                Ok(bytes) => match web_download(&name, kind.mime(), &bytes) {
                    Ok(()) => {
                        self.notice = Some(format!("saved {name}"));
                        self.error = None;
                    }
                    Err(e) => self.error = Some(e),
                },
                Err(e) => self.error = Some(e),
            }
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.busy = Some("Exporting…");
            spawn_save(
                self.jobs_tx.clone(),
                kind,
                name,
                self.cols,
                self.rows,
                self.grid.clone(),
            );
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_jobs(ctx);
        self.poll_drops(ctx);
        // wasm drop-decode is sync, so a second drain picks up Loaded this frame.
        self.poll_jobs(ctx);
        if self.dirty {
            self.rebuild();
        }
        let reduced = self.reduced_motion;
        let busy = self.busy;

        egui::SidePanel::right("controls")
            .resizable(false)
            .exact_width(320.0)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(GROUND)
                    .inner_margin(Margin::same(14))
                    .stroke(Stroke::NONE),
            )
            .show(ctx, |ui| {
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
                        let open_label = match busy {
                            Some("Opening…") => "Opening…",
                            Some("Decoding…") => "Decoding…",
                            _ => {
                                if self.source.is_none() {
                                    "CHOOSE STILL"
                                } else {
                                    "CHANGE STILL"
                                }
                            }
                        };
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
                                        self.settings.mark,
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

                        section(ui, "MARK");
                        ui.horizontal(|ui| {
                            let w = (ui.available_width() - 6.0) / 2.0;
                            for mark in [Mark::Cells, Mark::Chars] {
                                if chip(
                                    ui,
                                    w,
                                    mark.name(),
                                    self.settings.mark == mark,
                                    reduced,
                                )
                                .clicked()
                                {
                                    self.settings.mark = mark;
                                    self.thumbs = None;
                                    self.dirty = true;
                                }
                            }
                        });

                        section(ui, "DENSITY");
                        if archive_dial_u32(ui, "cols", &mut self.settings.cols, 16..=200) {
                            self.dirty = true;
                        }

                        section(ui, "COLOUR");
                        let gap = 6.0;
                        let chip_w = (ui.available_width() - gap) / 2.0;
                        for pair in ColorKind::ALL.chunks(2) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                for kind in pair {
                                    if chip(
                                        ui,
                                        chip_w,
                                        kind.name(),
                                        self.settings.color_kind == *kind,
                                        reduced,
                                    )
                                    .clicked()
                                    {
                                        self.settings.color_kind = *kind;
                                        self.dirty = true;
                                    }
                                }
                            });
                        }

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
                        ("NO STILL LOADED", "drop a photo here, or click to choose")
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

fn finish_bytes(tx: &Sender<Job>, name: String, bytes: &[u8]) {
    match io::load_bytes(bytes) {
        Ok(img) => {
            let _ = tx.send(Job::Loaded { name, img });
        }
        Err(e) => {
            let _ = tx.send(Job::Failed(e));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn spawn_pick(_tx: Sender<Job>) {
    click_still_input();
}

#[cfg(target_arch = "wasm32")]
fn click_still_input() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(el) = document.get_element_by_id("still-input") else {
        return;
    };
    let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() else {
        return;
    };
    input.set_value("");
    let _ = input.click();
}

#[cfg(target_arch = "wasm32")]
fn read_web_file(tx: Sender<Job>, ctx: egui::Context, file: web_sys::File) {
    let name = file.name();
    let promise = file.array_buffer();
    wasm_bindgen_futures::spawn_local(async move {
        match wasm_bindgen_futures::JsFuture::from(promise).await {
            Ok(buf) => {
                let arr = js_sys::Uint8Array::new(&buf);
                let mut bytes = vec![0u8; arr.length() as usize];
                arr.copy_to(&mut bytes);
                finish_bytes(&tx, name, &bytes);
            }
            Err(_) => {
                let _ = tx.send(Job::Failed("Could not read that file.".into()));
            }
        }
        ctx.request_repaint();
    });
}

#[cfg(target_arch = "wasm32")]
fn install_web_still_hooks(tx: Sender<Job>, ctx: egui::Context) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use wasm_bindgen::closure::Closure;
    use web_sys::{ClipboardEvent, HtmlInputElement};

    static ONCE: AtomicBool = AtomicBool::new(false);
    if ONCE.swap(true, Ordering::SeqCst) {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    if let Some(el) = document.get_element_by_id("still-input") {
        if let Ok(input) = el.dyn_into::<HtmlInputElement>() {
            let tx_in = tx.clone();
            let ctx_in = ctx.clone();
            let on_change = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let Ok(target) = event
                    .target()
                    .ok_or(())
                    .and_then(|t| t.dyn_into::<HtmlInputElement>().map_err(|_| ()))
                else {
                    return;
                };
                let Some(files) = target.files() else {
                    return;
                };
                let Some(file) = files.get(0) else {
                    return;
                };
                let _ = tx_in.send(Job::Busy("Decoding…"));
                ctx_in.request_repaint();
                read_web_file(tx_in.clone(), ctx_in.clone(), file);
            }) as Box<dyn FnMut(_)>);
            let _ = input.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
            on_change.forget();
        }
    }

    let capture = {
        let o = web_sys::AddEventListenerOptions::new();
        o.set_capture(true);
        o
    };

    let on_over = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        event.prevent_default();
        if let Some(dt) = event.data_transfer() {
            let _ = dt.set_drop_effect("copy");
        }
    }) as Box<dyn FnMut(_)>);
    let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
        "dragover",
        on_over.as_ref().unchecked_ref(),
        &capture,
    );
    on_over.forget();

    let tx_drop = tx.clone();
    let ctx_drop = ctx.clone();
    let on_drop = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        event.prevent_default();
        event.stop_propagation();
        let Some(dt) = event.data_transfer() else {
            return;
        };
        if let Some(file) = first_drag_file(&dt) {
            let _ = tx_drop.send(Job::Busy("Decoding…"));
            ctx_drop.request_repaint();
            read_web_file(tx_drop.clone(), ctx_drop.clone(), file);
            return;
        }
        if let Some(url) = drag_urls(&dt).into_iter().next() {
            let _ = tx_drop.send(Job::Busy("Decoding…"));
            ctx_drop.request_repaint();
            fetch_web_still(tx_drop.clone(), ctx_drop.clone(), url);
            return;
        }
        let _ = tx_drop.send(Job::Failed(
            "Could not read that drop. Save the photo and choose it.".into(),
        ));
        ctx_drop.request_repaint();
    }) as Box<dyn FnMut(_)>);
    let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
        "drop",
        on_drop.as_ref().unchecked_ref(),
        &capture,
    );
    on_drop.forget();

    let tx_paste = tx;
    let ctx_paste = ctx;
    let on_paste = Closure::wrap(Box::new(move |event: ClipboardEvent| {
        let Some(dt) = event.clipboard_data() else {
            return;
        };
        if let Some(file) = first_drag_file(&dt) {
            event.prevent_default();
            let _ = tx_paste.send(Job::Busy("Decoding…"));
            ctx_paste.request_repaint();
            read_web_file(tx_paste.clone(), ctx_paste.clone(), file);
            return;
        }
        if let Some(url) = drag_urls(&dt).into_iter().next() {
            event.prevent_default();
            let _ = tx_paste.send(Job::Busy("Decoding…"));
            ctx_paste.request_repaint();
            fetch_web_still(tx_paste.clone(), ctx_paste.clone(), url);
        }
    }) as Box<dyn FnMut(_)>);
    let _ = window.add_event_listener_with_callback("paste", on_paste.as_ref().unchecked_ref());
    on_paste.forget();
}

#[cfg(target_arch = "wasm32")]
fn first_drag_file(dt: &web_sys::DataTransfer) -> Option<web_sys::File> {
    if let Some(files) = dt.files() {
        if let Some(file) = files.get(0) {
            return Some(file);
        }
    }
    let items = dt.items();
    for i in 0..items.length() {
        if let Some(item) = items.get(i) {
            if item.kind() == "file" {
                if let Ok(Some(file)) = item.get_as_file() {
                    return Some(file);
                }
            }
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn drag_urls(dt: &web_sys::DataTransfer) -> Vec<String> {
    let mut urls = Vec::new();
    for kind in ["text/uri-list", "text/x-moz-url", "text/plain", "text/html"] {
        if let Ok(raw) = dt.get_data(kind) {
            urls.extend(urls_from_payload(kind, &raw));
        }
    }
    rank_drop_urls(urls)
}

#[cfg(target_arch = "wasm32")]
fn fetch_web_still(tx: Sender<Job>, ctx: egui::Context, url: String) {
    wasm_bindgen_futures::spawn_local(async move {
        match fetch_bytes(&url).await {
            Ok((name, bytes)) => finish_bytes(&tx, name, &bytes),
            Err(e) => {
                let _ = tx.send(Job::Failed(e));
            }
        }
        ctx.request_repaint();
    });
}

#[cfg(target_arch = "wasm32")]
async fn fetch_bytes(url: &str) -> Result<(String, Vec<u8>), String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(web_sys::RequestMode::Cors);
    opts.set_credentials(web_sys::RequestCredentials::Omit);
    let promise = window.fetch_with_str_and_init(url, &opts);
    let resp_val = wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|_| {
        "That site will not let the archive fetch the photo. Save it, then CHOOSE STILL.".to_string()
    })?;
    let resp: web_sys::Response = resp_val.dyn_into().map_err(|_| "bad response".to_string())?;
    if !resp.ok() {
        return Err(
            "That site will not let the archive fetch the photo. Save it, then CHOOSE STILL."
                .into(),
        );
    }
    let buf = wasm_bindgen_futures::JsFuture::from(
        resp.array_buffer().map_err(|_| "empty body".to_string())?,
    )
    .await
    .map_err(|_| "empty body".to_string())?;
    let arr = js_sys::Uint8Array::new(&buf);
    if arr.length() == 0 {
        return Err("empty photo".into());
    }
    let mut bytes = vec![0u8; arr.length() as usize];
    arr.copy_to(&mut bytes);
    if looks_like_html_page(&bytes) {
        return Err(
            "That drop was a page, not a photo. Drag the image itself, or save it and choose it."
                .into(),
        );
    }
    Ok((still_name_from_url(url), bytes))
}

fn still_name_from_url(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    path.rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or("web-still")
        .to_string()
}

fn looks_like_html_page(bytes: &[u8]) -> bool {
    let n = bytes.len().min(64);
    let s = String::from_utf8_lossy(&bytes[..n]).to_ascii_lowercase();
    s.contains("<!doctype html") || s.contains("<html")
}

fn rank_drop_urls(mut urls: Vec<String>) -> Vec<String> {
    urls.sort_by_key(|u| std::cmp::Reverse(pinterest_rank(u)));
    urls.dedup();
    urls
}

fn urls_from_payload(kind: &str, raw: &str) -> Vec<String> {
    if kind == "text/html" {
        return urls_from_html(raw);
    }
    let mut out = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let first = line.split('\t').next().unwrap_or(line);
        if let Some(u) = normalize_drop_url(first) {
            out.push(u);
        }
    }
    out
}

fn urls_from_html(raw: &str) -> Vec<String> {
    let lower = raw.to_ascii_lowercase();
    let mut out = Vec::new();
    for value in html_attr_values(&lower, raw, "srcset=") {
        for part in value.split(',') {
            let tok = part.trim().split_whitespace().next().unwrap_or("");
            if let Some(u) = normalize_drop_url(&unescape_html(tok)) {
                out.push(u);
            }
        }
    }
    for value in html_attr_values(&lower, raw, "src=") {
        if let Some(u) = normalize_drop_url(&unescape_html(&value)) {
            out.push(u);
        }
    }
    out
}

fn html_attr_values(lower: &str, raw: &str, attr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(rel) = lower.get(pos..).and_then(|s| s.find(attr)) {
        let abs = pos + rel;
        if attr == "src=" && abs >= 3 && lower.get(abs - 3..abs) == Some("set") {
            pos = abs + attr.len();
            continue;
        }
        let start = abs + attr.len();
        let rest = raw.get(start..).unwrap_or("");
        let trimmed = rest.trim_start();
        let skip = rest.len() - trimmed.len();
        let (value, adv) = if trimmed.starts_with('"') || trimmed.starts_with('\'') {
            let q = trimmed.as_bytes()[0] as char;
            match trimmed[1..].find(q) {
                Some(i) => (trimmed[1..1 + i].to_string(), skip + i + 2),
                None => break,
            }
        } else {
            let tok = trimmed
                .split(|c: char| c.is_whitespace() || c == '>')
                .next()
                .unwrap_or("");
            (tok.to_string(), skip + tok.len())
        };
        out.push(value);
        pos = start + adv;
        if pos <= start {
            pos = start + 1;
        }
    }
    out
}

fn unescape_html(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn normalize_drop_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    let mut u = url.to_string();
    if u.starts_with("//") {
        u = format!("https:{u}");
    }
    if !(u.starts_with("https://")
        || u.starts_with("http://")
        || u.starts_with("data:image/")
        || u.starts_with("blob:"))
    {
        return None;
    }
    if u.contains("pinimg.com") {
        for size in ["/236x/", "/474x/", "/564x/", "/736x/", "/1200x/"] {
            if u.contains(size) {
                u = u.replace(size, "/originals/");
                break;
            }
        }
    }
    Some(u)
}

fn pinterest_rank(url: &str) -> u32 {
    let u = url.to_ascii_lowercase();
    let mut n = url.len() as u32;
    if u.contains("pinimg.com") {
        n += 1000;
    }
    if u.contains("/originals/") {
        n += 500;
    } else if u.contains("/736x/") {
        n += 300;
    }
    if u.ends_with(".jpg") || u.ends_with(".jpeg") || u.ends_with(".png") || u.ends_with(".webp") {
        n += 200;
    }
    n
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_load_path(tx: Sender<Job>, path: PathBuf) {
    thread::spawn(move || {
        let _ = tx.send(Job::Busy("Decoding…"));
        match io::load_path(&path) {
            Ok(img) => {
                let _ = tx.send(Job::Loaded {
                    name: file_stem(&path),
                    img,
                });
            }
            Err(e) => {
                let _ = tx.send(Job::Failed(e));
            }
        }
    });
}

fn spawn_load_bytes(tx: Sender<Job>, name: String, bytes: Vec<u8>) {
    #[cfg(not(target_arch = "wasm32"))]
    thread::spawn(move || finish_bytes(&tx, name, &bytes));
    #[cfg(target_arch = "wasm32")]
    finish_bytes(&tx, name, &bytes);
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_save(
    tx: Sender<Job>,
    kind: io::ExportKind,
    name: String,
    cols: u32,
    rows: u32,
    grid: Vec<Cell>,
) {
    thread::spawn(move || {
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
                    let _ = tx.send(Job::Saved(name));
                }
            }
            Err(e) => {
                let _ = tx.send(Job::Failed(e));
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn web_download(name: &str, mime: &str, bytes: &[u8]) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let document = window.document().ok_or_else(|| "no document".to_string())?;
    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::of1(&array);
    let props = web_sys::BlobPropertyBag::new();
    props.set_type(mime);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &props)
        .map_err(|_| "Could not prepare the download.".to_string())?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Could not prepare the download.".to_string())?;
    let a = document
        .create_element("a")
        .map_err(|_| "Could not start the download.".to_string())?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "Could not start the download.".to_string())?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
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
    mark: Mark,
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
        paint_grid(ui, preview, thumb.cols, thumb.rows, &thumb.grid, mark);
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
    if app.settings.mark == Mark::Cells {
        paint_mosaic(ui, rect, app.cols, app.rows, &app.grid);
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

fn paint_grid(ui: &egui::Ui, rect: Rect, cols: u32, rows: u32, grid: &[Cell], mark: Mark) {
    if cols == 0 || rows == 0 || grid.is_empty() {
        return;
    }
    if mark == Mark::Cells {
        paint_mosaic(ui, rect, cols, rows, grid);
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

fn paint_mosaic(ui: &egui::Ui, rect: Rect, cols: u32, rows: u32, grid: &[Cell]) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0, GROUND);
    for y in 0..rows {
        let y0 = rect.top() + (y as f32 / rows as f32) * rect.height();
        let y1 = rect.top() + ((y + 1) as f32 / rows as f32) * rect.height();
        for x in 0..cols {
            let idx = (y * cols + x) as usize;
            if idx >= grid.len() {
                continue;
            }
            let cell = grid[idx];
            if cell.ch == ' ' {
                continue;
            }
            let x0 = rect.left() + (x as f32 / cols as f32) * rect.width();
            let x1 = rect.left() + ((x + 1) as f32 / cols as f32) * rect.width();
            let r = Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1, y1)).shrink(0.4);
            painter.rect_filled(
                r,
                0,
                Color32::from_rgb(cell.color[0], cell.color[1], cell.color[2]),
            );
        }
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

fn archive_dial_u32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut u32,
    range: std::ops::RangeInclusive<u32>,
) -> bool {
    let mut f = *value as f32;
    let lo = *range.start() as f32;
    let hi = *range.end() as f32;
    let display = format!("{}", *value);
    let dragged = archive_dial(ui, label, &mut f, lo..=hi, display);
    let nv = f.round().clamp(lo, hi) as u32;
    if nv != *value {
        *value = nv;
        true
    } else {
        dragged && false
    }
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

#[cfg(not(target_arch = "wasm32"))]
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
    #[cfg(not(target_arch = "wasm32"))]
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
    let mono = {
        #[cfg(target_arch = "wasm32")]
        {
            Some(include_bytes!("../assets/NotoSansMono-Regular.ttf").to_vec())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            const MONO: &[&str] = &[
                "/usr/share/fonts/noto/NotoSansMono-Regular.ttf",
                "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
                "/usr/share/fonts/Adwaita/AdwaitaMono-Regular.ttf",
            ];
            MONO.iter().find_map(|path| std::fs::read(path).ok())
        }
    };
    if let Some(bytes) = mono {
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
    }
    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod drop_url_tests {
    use super::*;

    #[test]
    fn html_img_src_upgrades_pinterest_thumb() {
        let html = r#"<img src="//i.pinimg.com/236x/ab/cd/ef.jpg" alt="pin">"#;
        let urls = rank_drop_urls(urls_from_payload("text/html", html));
        assert_eq!(
            urls,
            vec!["https://i.pinimg.com/originals/ab/cd/ef.jpg".to_string()]
        );
    }

    #[test]
    fn html_srcset_prefers_pinimg_over_pin_page() {
        let html = concat!(
            r#"<a href="https://www.pinterest.com/pin/123">"#,
            r#"<img srcset="https://i.pinimg.com/236x/aa/bb/cc.jpg 236w, https://i.pinimg.com/736x/aa/bb/cc.jpg 736w" "#,
            r#"src="https://i.pinimg.com/236x/aa/bb/cc.jpg">"#,
            "</a>"
        );
        let urls = rank_drop_urls(urls_from_payload("text/html", html));
        assert_eq!(urls[0], "https://i.pinimg.com/originals/aa/bb/cc.jpg");
        assert!(!urls.iter().any(|u| u.contains("pinterest.com/pin/")));
    }

    #[test]
    fn uri_list_skips_comments_and_pages() {
        let raw = "# comment\nhttps://i.pinimg.com/474x/de/ad/be.jpg\nhttps://www.pinterest.com/pin/99/\n";
        let urls = rank_drop_urls(urls_from_payload("text/uri-list", raw));
        assert_eq!(urls[0], "https://i.pinimg.com/originals/de/ad/be.jpg");
    }

    #[test]
    fn rejects_javascript_and_bare_paths() {
        assert!(normalize_drop_url("javascript:alert(1)").is_none());
        assert!(normalize_drop_url("/local/photo.jpg").is_none());
        assert!(normalize_drop_url("").is_none());
    }

    #[test]
    fn html_page_bytes_are_rejected() {
        assert!(looks_like_html_page(b"<!DOCTYPE html><html>"));
        assert!(!looks_like_html_page(b"\xff\xd8\xff\xe0"));
    }

    #[test]
    fn still_name_strips_query() {
        assert_eq!(
            still_name_from_url("https://i.pinimg.com/originals/ab/cd/ef.jpg?foo=1"),
            "ef.jpg"
        );
    }
}
