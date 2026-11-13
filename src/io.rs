use std::io::Cursor;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, RgbaImage};
use resvg::{tiny_skia, usvg};

use crate::ascii::{Cell, Mark};

/// Longest side kept in memory and on the GPU. Bigger stills are downscaled.
pub const MAX_SIDE: u32 = 2048;
/// Raster/SVG long side. Same grid as the theater; cell size grows so 32-col
/// stills are not 8px stamps and 400-col stills are not hairline gaps.
const EXPORT_SIDE: u32 = 1600;
/// Refuse an in-process full decode above this (RGBA would exceed ~64 MiB).
const MAX_DECODE_PIXELS: u64 = 16 * 1024 * 1024;

pub fn downscale_max_side(img: RgbaImage, max_side: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let side = w.max(h);
    if side <= max_side {
        return img;
    }
    let nw = ((w as u64 * max_side as u64) / side as u64).max(1) as u32;
    let nh = ((h as u64 * max_side as u64) / side as u64).max(1) as u32;
    image::imageops::resize(&img, nw, nh, image::imageops::FilterType::Triangle)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_path(path: &Path) -> Result<RgbaImage, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Could not read file: {e}"))?;
    load_bytes(&bytes)
}

pub fn load_bytes(bytes: &[u8]) -> Result<RgbaImage, String> {
    if looks_like_svg(bytes) {
        return rasterize_svg(bytes);
    }
    let format = image::guess_format(bytes).ok();
    let (w, h) = peek_size(bytes).unwrap_or((0, 0));
    let pixels = (w as u64).saturating_mul(h as u64);
    let oversized = w.max(h) > MAX_SIDE || pixels > MAX_DECODE_PIXELS;

    if oversized {
        if format == Some(ImageFormat::Jpeg) {
            if let Ok(img) = load_jpeg_scaled(bytes) {
                return Ok(downscale_max_side(img, MAX_SIDE));
            }
        }
        if pixels > MAX_DECODE_PIXELS {
            return Err(format!(
                "Image is {w}×{h}, too large to decode. The generator caps the long side at {MAX_SIDE}px."
            ));
        }
    }

    let img = decode_rgba(bytes)?;
    Ok(downscale_max_side(img, MAX_SIDE))
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let n = bytes.len().min(256);
    let s = String::from_utf8_lossy(&bytes[..n]).to_ascii_lowercase();
    s.contains("<svg")
}

const SVG_OPEN_HINT: &str =
    "Could not rasterize this SVG. Export it as a PNG or JPEG and open that instead.";

fn rasterize_svg(bytes: &[u8]) -> Result<RgbaImage, String> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default())
        .map_err(|_| SVG_OPEN_HINT.to_string())?;
    let size = tree.size().to_int_size();
    let w = size.width();
    let h = size.height();
    if w == 0 || h == 0 {
        return Err(SVG_OPEN_HINT.into());
    }
    let side = w.max(h);
    let scale = if side > MAX_SIDE {
        MAX_SIDE as f32 / side as f32
    } else {
        1.0
    };
    let out_w = ((w as f32) * scale).round().max(1.0) as u32;
    let out_h = ((h as f32) * scale).round().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(out_w, out_h).ok_or(SVG_OPEN_HINT)?;
    pixmap.fill(tiny_skia::Color::WHITE);
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap_to_rgba(pixmap)
}

fn pixmap_to_rgba(pixmap: tiny_skia::Pixmap) -> Result<RgbaImage, String> {
    let w = pixmap.width();
    let h = pixmap.height();
    let mut data = pixmap.take();
    for px in data.chunks_exact_mut(4) {
        let a = px[3];
        if a > 0 && a < 255 {
            let ia = 255.0 / f32::from(a);
            px[0] = (f32::from(px[0]) * ia).round().min(255.0) as u8;
            px[1] = (f32::from(px[1]) * ia).round().min(255.0) as u8;
            px[2] = (f32::from(px[2]) * ia).round().min(255.0) as u8;
        }
    }
    RgbaImage::from_raw(w, h, data).ok_or_else(|| SVG_OPEN_HINT.into())
}

fn peek_size(bytes: &[u8]) -> Result<(u32, u32), String> {
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| e.to_string())
}

fn decode_rgba(bytes: &[u8]) -> Result<RgbaImage, String> {
    image::load_from_memory(bytes)
        .map(|img| img.to_rgba8())
        .map_err(|e| format!("Could not load image: {e}"))
}

fn load_jpeg_scaled(bytes: &[u8]) -> Result<RgbaImage, String> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    decoder.read_info().map_err(|e| e.to_string())?;
    let info = decoder.info().ok_or_else(|| "JPEG has no header".to_string())?;
    let side = info.width.max(info.height).max(1);
    if u32::from(side) > MAX_SIDE {
        let req_w = ((u32::from(info.width) * MAX_SIDE) / u32::from(side)).max(1) as u16;
        let req_h = ((u32::from(info.height) * MAX_SIDE) / u32::from(side)).max(1) as u16;
        decoder.scale(req_w, req_h).map_err(|e| e.to_string())?;
    }
    let pixels = decoder.decode().map_err(|e| e.to_string())?;
    let info = decoder.info().ok_or_else(|| "JPEG has no header".to_string())?;
    let mut img = jpeg_to_rgba(pixels, info.width, info.height, info.pixel_format)?;
    apply_jpeg_orientation(bytes, &mut img);
    Ok(img)
}

fn jpeg_to_rgba(
    pixels: Vec<u8>,
    w: u16,
    h: u16,
    fmt: jpeg_decoder::PixelFormat,
) -> Result<RgbaImage, String> {
    let (w, h) = (u32::from(w), u32::from(h));
    let rgba = match fmt {
        jpeg_decoder::PixelFormat::L8 => pixels
            .into_iter()
            .flat_map(|v| [v, v, v, 255])
            .collect::<Vec<_>>(),
        jpeg_decoder::PixelFormat::L16 => pixels
            .chunks_exact(2)
            .flat_map(|c| {
                let v = c[0];
                [v, v, v, 255]
            })
            .collect::<Vec<_>>(),
        jpeg_decoder::PixelFormat::RGB24 => {
            let mut out = Vec::with_capacity(pixels.len() / 3 * 4);
            for c in pixels.chunks_exact(3) {
                out.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
            out
        }
        jpeg_decoder::PixelFormat::CMYK32 => {
            return Err("CMYK JPEG".into());
        }
    };
    RgbaImage::from_raw(w, h, rgba).ok_or_else(|| "JPEG pixel buffer mismatch".into())
}

fn apply_jpeg_orientation(bytes: &[u8], img: &mut RgbaImage) {
    let Ok(mut dec) = image::codecs::jpeg::JpegDecoder::new(Cursor::new(bytes)) else {
        return;
    };
    let Ok(orientation) = dec.orientation() else {
        return;
    };
    let mut dynimg = DynamicImage::ImageRgba8(std::mem::take(img));
    dynimg.apply_orientation(orientation);
    *img = dynimg.to_rgba8();
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Txt,
    Png,
    Jpeg,
    Webp,
    Svg,
    #[allow(dead_code)]
    Html,
    Ansi,
}

impl ExportKind {
    pub fn ext(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Svg => "svg",
            Self::Html => "html",
            Self::Ansi => "ans",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Txt => "TXT",
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Webp => "WEBP",
            Self::Svg => "SVG",
            Self::Html => "HTML",
            Self::Ansi => "ANSI",
        }
    }

    pub fn mime(self) -> &'static str {
        match self {
            Self::Txt | Self::Ansi => "text/plain;charset=utf-8",
            Self::Svg => "image/svg+xml;charset=utf-8",
            Self::Html => "text/html;charset=utf-8",
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

pub fn to_txt(cols: u32, rows: u32, grid: &[Cell]) -> String {
    let mut out = String::with_capacity((cols as usize + 1) * rows as usize);
    for y in 0..rows {
        let row = (y * cols) as usize;
        for x in 0..cols {
            out.push(grid[row + x as usize].ch);
        }
        out.push('\n');
    }
    out
}

pub fn cell_px(cols: u32, rows: u32, mark: Mark) -> (u32, u32) {
    let cols = cols.max(1);
    let _rows = rows.max(1);
    match mark {
        Mark::Cells => {
            let cw = EXPORT_SIDE.div_ceil(cols).max(1);
            (cw, cw)
        }
        Mark::Chars => {
            let cw = EXPORT_SIDE.div_ceil(cols).max(1);
            (cw, (cw as f32 / crate::ascii::CHAR_ASPECT).round().max(1.0) as u32)
        }
    }
}

pub fn to_svg(cols: u32, rows: u32, grid: &[Cell], mark: Mark) -> String {
    let (cw, ch) = cell_px(cols, rows, mark);
    match mark {
        Mark::Cells => svg_rects(cols, rows, grid, cw, ch),
        Mark::Chars => svg_text_rows(cols, rows, grid, cw, ch),
    }
}

fn fill_attr(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

fn svg_open(w: u32, h: u32) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n\
         <rect width=\"{w}\" height=\"{h}\" fill=\"#050508\"/>\n"
    )
}

fn svg_rects(cols: u32, rows: u32, grid: &[Cell], cw: u32, ch: u32) -> String {
    let w = cols * cw;
    let h = rows * ch;
    let mut out = svg_open(w, h);
    for y in 0..rows {
        let row = (y * cols) as usize;
        let mut x = 0u32;
        while x < cols {
            let cell = grid[row + x as usize];
            if cell.ch == ' ' {
                x += 1;
                continue;
            }
            let color = cell.color;
            let mut n = 1u32;
            while x + n < cols {
                let next = grid[row + (x + n) as usize];
                if next.ch == ' ' || next.color != color {
                    break;
                }
                n += 1;
            }
            let px = x * cw;
            let py = y * ch;
            let rw = n * cw;
            let fill = fill_attr(cell.color);
            out.push_str(&format!(
                "<rect x=\"{px}\" y=\"{py}\" width=\"{rw}\" height=\"{ch}\" fill=\"{fill}\"/>\n"
            ));
            x += n;
        }
    }
    out.push_str("</svg>\n");
    out
}

fn svg_text_rows(cols: u32, rows: u32, grid: &[Cell], cw: u32, ch: u32) -> String {
    let w = cols * cw;
    let h = rows * ch;
    let fs = ((cw as f32 / crate::ascii::MONO_ADVANCE).min(ch as f32)).max(1.0);
    let mut out = svg_open(w, h);
    out.push_str(&format!(
        "<g font-family=\"Noto Sans Mono, ui-monospace, monospace\" font-size=\"{fs:.3}\">\n"
    ));
    for y in 0..rows {
        let row = (y * cols) as usize;
        let py = (y as f32 + 1.0) * ch as f32 - ch as f32 * 0.12;
        let mut line = format!("<text y=\"{py:.2}\" xml:space=\"preserve\">");
        let mut wrote = false;
        for x in 0..cols {
            let cell = grid[row + x as usize];
            if cell.ch == ' ' {
                continue;
            }
            let px = x * cw;
            let fill = fill_attr(cell.color);
            line.push_str(&format!(
                "<tspan x=\"{px}\" fill=\"{fill}\">{}</tspan>",
                xml_escape(cell.ch)
            ));
            wrote = true;
        }
        if wrote {
            line.push_str("</text>\n");
            out.push_str(&line);
        }
    }
    out.push_str("</g>\n</svg>\n");
    out
}

pub fn to_html(cols: u32, rows: u32, grid: &[Cell]) -> String {
    let mut out = String::from(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Cosmic Archive ASCII</title>\n\
         <style>body{background:#050508;margin:0;padding:1rem}pre{font:16px/1 monospace;letter-spacing:0}</style></head><body><pre>",
    );
    for y in 0..rows {
        let row = (y * cols) as usize;
        for x in 0..cols {
            let cell = grid[row + x as usize];
            let ch = html_escape(cell.ch);
            out.push_str(&format!(
                "<span style=\"color:#{:02x}{:02x}{:02x}\">{ch}</span>",
                cell.color[0], cell.color[1], cell.color[2]
            ));
        }
        out.push('\n');
    }
    out.push_str("</pre></body></html>\n");
    out
}

pub fn to_ansi(cols: u32, rows: u32, grid: &[Cell]) -> String {
    let mut out = String::new();
    for y in 0..rows {
        let row = (y * cols) as usize;
        let mut last = [255u8, 255, 255];
        let mut first = true;
        for x in 0..cols {
            let cell = grid[row + x as usize];
            if first || cell.color != last {
                out.push_str(&format!(
                    "\x1b[38;2;{};{};{}m",
                    cell.color[0], cell.color[1], cell.color[2]
                ));
                last = cell.color;
                first = false;
            }
            out.push(cell.ch);
        }
        out.push_str("\x1b[0m\n");
    }
    out
}

pub fn mosaic_rgba(cols: u32, rows: u32, grid: &[Cell], mark: Mark) -> RgbaImage {
    match mark {
        Mark::Chars => rasterize_export_svg(cols, rows, grid, mark)
            .unwrap_or_else(|_| mosaic_cells(cols, rows, grid, mark)),
        Mark::Cells => mosaic_cells(cols, rows, grid, mark),
    }
}

pub(crate) fn mosaic_cells(cols: u32, rows: u32, grid: &[Cell], mark: Mark) -> RgbaImage {
    let (cw, ch) = cell_px(cols, rows, mark);
    let mut img = RgbaImage::from_pixel(cols * cw, rows * ch, image::Rgba([0x05, 0x05, 0x08, 255]));
    for y in 0..rows {
        for x in 0..cols {
            let cell = grid[(y * cols + x) as usize];
            if cell.ch == ' ' {
                continue;
            }
            let px = image::Rgba([cell.color[0], cell.color[1], cell.color[2], 255]);
            let x0 = x * cw;
            let y0 = y * ch;
            for dy in 0..ch {
                for dx in 0..cw {
                    img.put_pixel(x0 + dx, y0 + dy, px);
                }
            }
        }
    }
    img
}

pub fn to_png(cols: u32, rows: u32, grid: &[Cell], mark: Mark) -> Result<Vec<u8>, String> {
    encode_mosaic(cols, rows, grid, mark, ImageFormat::Png)
}

fn encode_mosaic(
    cols: u32,
    rows: u32,
    grid: &[Cell],
    mark: Mark,
    format: ImageFormat,
) -> Result<Vec<u8>, String> {
    let img = mosaic_rgba(cols, rows, grid, mark);
    encode_image(img, format)
}

fn encode_image(img: RgbaImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    match format {
        ImageFormat::Jpeg => {
            let rgb = DynamicImage::ImageRgba8(img).to_rgb8();
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 92);
            enc.encode(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| e.to_string())?;
        }
        other => {
            DynamicImage::ImageRgba8(img)
                .write_to(&mut buf, other)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(buf.into_inner())
}

fn rasterize_export_svg(
    cols: u32,
    rows: u32,
    grid: &[Cell],
    mark: Mark,
) -> Result<RgbaImage, String> {
    let svg = to_svg(cols, rows, grid, mark);
    let mut opt = usvg::Options::default();
    if let Some(bytes) = mono_font_bytes() {
        opt.fontdb_mut().load_font_data(bytes);
        opt.fontdb_mut().set_monospace_family("Noto Sans Mono");
        opt.fontdb_mut().set_sans_serif_family("Noto Sans Mono");
    }
    let tree = usvg::Tree::from_data(svg.as_bytes(), &opt)
        .map_err(|e| format!("Could not render export: {e}"))?;
    let size = tree.size().to_int_size();
    let w = size.width();
    let h = size.height();
    if w == 0 || h == 0 {
        return Err("Could not render export.".into());
    }
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or("Could not render export.")?;
    pixmap.fill(tiny_skia::Color::from_rgba8(0x05, 0x05, 0x08, 255));
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap_to_rgba(pixmap)
}

pub fn mono_font_bytes() -> Option<Vec<u8>> {
    Some(include_bytes!("../assets/NotoSansMono-Regular.ttf").to_vec())
}

pub fn encode(
    kind: ExportKind,
    cols: u32,
    rows: u32,
    grid: &[Cell],
    mark: Mark,
) -> Result<Vec<u8>, String> {
    match kind {
        ExportKind::Txt => Ok(to_txt(cols, rows, grid).into_bytes()),
        ExportKind::Svg => Ok(to_svg(cols, rows, grid, mark).into_bytes()),
        ExportKind::Html => Ok(to_html(cols, rows, grid).into_bytes()),
        ExportKind::Ansi => Ok(to_ansi(cols, rows, grid).into_bytes()),
        ExportKind::Png => encode_mosaic(cols, rows, grid, mark, ImageFormat::Png),
        ExportKind::Jpeg => encode_mosaic(cols, rows, grid, mark, ImageFormat::Jpeg),
        ExportKind::Webp => encode_mosaic(cols, rows, grid, mark, ImageFormat::WebP),
    }
}

fn xml_escape(ch: char) -> String {
    match ch {
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        '&' => "&amp;".into(),
        '"' => "&quot;".into(),
        '\'' => "&apos;".into(),
        c => c.to_string(),
    }
}

fn html_escape(ch: char) -> String {
    match ch {
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        '&' => "&amp;".into(),
        c => c.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn downscale_caps_long_side() {
        let img = RgbaImage::from_pixel(3000, 1000, Rgba([10, 20, 30, 255]));
        let out = downscale_max_side(img, 2048);
        assert_eq!(out.width(), 2048);
        assert_eq!(out.height(), 682);
    }

    #[test]
    fn load_jpeg_over_max_side() {
        let src = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2400, 800, Rgba([200, 10, 10, 255])));
        let mut buf = Cursor::new(Vec::new());
        src.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
        let img = load_bytes(&buf.into_inner()).expect("load jpeg");
        assert!(img.width() <= MAX_SIDE);
        assert!(img.height() <= MAX_SIDE);
        assert!(img.width() >= img.height());
        assert!(img.width() >= 1000);
    }

    fn sample_grid() -> (u32, u32, Vec<Cell>) {
        let img = RgbaImage::from_pixel(40, 20, image::Rgba([255, 255, 255, 255]));
        let mut s = crate::ascii::Settings::default();
        s.cols = 8;
        s.charset = crate::ascii::Charset::Blocks;
        crate::ascii::convert(&img, &s)
    }

    #[test]
    fn load_gif_roundtrip() {
        let src = DynamicImage::ImageRgba8(RgbaImage::from_pixel(16, 16, image::Rgba([0, 200, 80, 255])));
        let mut buf = Cursor::new(Vec::new());
        src.write_to(&mut buf, ImageFormat::Gif).unwrap();
        let img = load_bytes(&buf.into_inner()).expect("gif");
        assert_eq!(img.width(), 16);
        assert_eq!(img.height(), 16);
    }

    #[test]
    fn export_txt_has_rows() {
        let (cols, rows, grid) = sample_grid();
        let txt = to_txt(cols, rows, &grid);
        let lines: Vec<_> = txt.lines().collect();
        assert_eq!(lines.len(), rows as usize);
        assert_eq!(lines[0].chars().count(), cols as usize);
        assert!(lines.iter().all(|l| l.contains('█')));
    }

    #[test]
    fn export_svg_has_glyphs() {
        let (cols, rows, grid) = sample_grid();
        let cells = to_svg(cols, rows, &grid, Mark::Cells);
        assert!(cells.contains("<svg"));
        assert!(cells.contains("<rect"));
        assert!(cells.contains("fill="));
        assert!(!cells.contains("<text"));

        let mut s = crate::ascii::Settings::default();
        s.cols = 8;
        s.charset = crate::ascii::Charset::Blocks;
        s.mark = Mark::Chars;
        let img = RgbaImage::from_pixel(40, 20, image::Rgba([255, 255, 255, 255]));
        let (cols, rows, grid) = crate::ascii::convert(&img, &s);
        let chars = to_svg(cols, rows, &grid, Mark::Chars);
        assert!(chars.contains("█"));
        assert!(chars.contains("<tspan"));
        assert!(!chars.contains("<text x="));
    }

    #[test]
    fn svg_merges_same_fill_runs() {
        let cell = Cell {
            ch: '█',
            color: [0x59, 0x2a, 0xa4],
        };
        let grid = vec![cell; 8];
        let svg = to_svg(8, 1, &grid, Mark::Cells);
        let rects = svg.matches("<rect").count();
        assert_eq!(rects, 2, "ground + one merged run, got {svg}");
        let (cw, _) = cell_px(8, 1, Mark::Cells);
        assert!(
            svg.contains(&format!("width=\"{}\"", cw * 8)),
            "{svg}"
        );
        assert!(svg.contains("#592aa4"), "{svg}");
    }

    #[test]
    fn import_svg_rasterizes_without_magick() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20">
            <rect width="40" height="20" fill="#cc2020"/>
        </svg>"##;
        let img = load_bytes(svg).expect("rasterize svg");
        assert_eq!(img.width(), 40);
        assert_eq!(img.height(), 20);
        let p = img.get_pixel(8, 8);
        assert!(p[0] > 160 && p[1] < 80 && p[2] < 80, "expected red fill, got {p:?}");
    }

    #[test]
    fn import_svg_caps_long_side() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4096" height="512">
            <rect width="4096" height="512" fill="#4277a2"/>
        </svg>"##;
        let img = load_bytes(svg).expect("rasterize wide svg");
        assert!(img.width() <= MAX_SIDE);
        assert!(img.height() <= MAX_SIDE);
        assert_eq!(img.width(), MAX_SIDE);
    }

    #[test]
    fn import_broken_svg_asks_for_png() {
        let err = load_bytes(b"<svg not-actually-svg").expect_err("bad svg");
        assert!(err.contains("PNG or JPEG"), "{err}");
    }

    #[test]
    fn export_html_and_png() {
        let (cols, rows, grid) = sample_grid();
        let html = to_html(cols, rows, &grid);
        assert!(html.contains("<pre>"));
        assert!(html.contains("█"));
        let png = to_png(cols, rows, &grid, Mark::Cells).expect("png");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
        let img = image::load_from_memory(&png).expect("decode png export");
        assert!(img.width() >= cols);
        assert!(img.height() >= rows);
        let mosaic = mosaic_rgba(cols, rows, &grid, Mark::Cells);
        assert_eq!(mosaic.width(), img.width());
        assert_eq!(mosaic.height(), img.height());
        let jpg = encode(ExportKind::Jpeg, cols, rows, &grid, Mark::Cells).expect("jpeg");
        assert_eq!(&jpg[..2], &[0xff, 0xd8]);
        let webp = encode(ExportKind::Webp, cols, rows, &grid, Mark::Cells).expect("webp");
        assert_eq!(&webp[..4], b"RIFF");
    }

    #[test]
    fn write_pipeline_proofs_if_asked() {
        let Ok(root) = std::env::var("ASCII_EVIDENCE") else {
            return;
        };
        let root = Path::new(&root);
        let hatch = root.join("hatching");
        let export = root.join("export-match");
        let looks = root.join("looks");
        std::fs::create_dir_all(&hatch).unwrap();
        std::fs::create_dir_all(&export).unwrap();
        std::fs::create_dir_all(&looks).unwrap();

        let still = Path::new("/cursor/stores/bc-af75a5c8-0b95-4049-b2af-9d16f70c2afa/media/ascii-before-eagle.jpg");
        let img = if still.exists() {
            load_path(still).expect("eagle")
        } else {
            load_path(&Path::new(env!("CARGO_MANIFEST_DIR")).join("sample.png")).expect("sample")
        };

        let mut s = crate::ascii::Settings::default();
        s.cols = 320;
        crate::ascii::Look::Visor.apply(&mut s);
        s.mark = Mark::Cells;
        let (cols, rows, grid) = crate::ascii::convert(&img, &s);

        let export_cells = mosaic_cells(cols, rows, &grid, Mark::Cells);
        export_cells.save(export.join("fixed-png.png")).unwrap();
        preview_inset(&grid, cols, rows, 800, 0.4)
            .save(hatch.join("cells-320-before.png"))
            .unwrap();
        preview_abut(&grid, cols, rows, 800)
            .save(hatch.join("cells-320-after.png"))
            .unwrap();
        old_mosaic(&grid, cols, rows, Mark::Cells)
            .save(export.join("bad-8px.png"))
            .unwrap();
        std::fs::write(export.join("fixed.svg"), to_svg(cols, rows, &grid, Mark::Cells)).unwrap();
        export_cells
            .save(export.join("preview-cells-320.png"))
            .unwrap();
        let mut s80 = s;
        s80.cols = 80;
        s80.mark = Mark::Cells;
        let (c80, r80, g80) = crate::ascii::convert(&img, &s80);
        std::fs::write(
            export.join("bad-4bit-80col.svg"),
            old_svg_4bit(&g80, c80, r80, Mark::Cells),
        )
        .unwrap();
        mosaic_cells(c80, r80, &g80, Mark::Cells)
            .save(export.join("preview-cells-80.png"))
            .unwrap();

        s.mark = Mark::Chars;
        let (ccols, crows, cgrid) = crate::ascii::convert(&img, &s);
        chars_overlap_preview(&cgrid, ccols, crows, 800)
            .save(hatch.join("chars-320-before.png"))
            .unwrap();
        mosaic_rgba(ccols, crows, &cgrid, Mark::Chars)
            .save(hatch.join("chars-320-after.png"))
            .unwrap();
        mosaic_rgba(ccols, crows, &cgrid, Mark::Chars)
            .save(export.join("fixed-chars-png.png"))
            .unwrap();
        old_mosaic(&cgrid, ccols, crows, Mark::Chars)
            .save(export.join("bad-chars-rects.png"))
            .unwrap();

        s.cols = 80;
        s.mark = Mark::Cells;
        for look in crate::ascii::Look::ALL {
            look.apply(&mut s);
            s.mark = Mark::Cells;
            let (lc, lr, lg) = crate::ascii::convert(&img, &s);
            mosaic_cells(lc, lr, &lg, Mark::Cells)
                .save(looks.join(format!("{}.png", look.name().to_ascii_lowercase())))
                .unwrap();
        }
    }

    fn preview_canvas(cols: u32, rows: u32, side: u32) -> (u32, u32, f32, f32) {
        let aspect = cols as f32 / rows.max(1) as f32;
        let (w, h) = if aspect >= 1.0 {
            (side, (side as f32 / aspect).round().max(1.0) as u32)
        } else {
            ((side as f32 * aspect).round().max(1.0) as u32, side)
        };
        (w, h, w as f32 / cols as f32, h as f32 / rows as f32)
    }

    fn preview_inset(grid: &[Cell], cols: u32, rows: u32, side: u32, inset: f32) -> RgbaImage {
        let (w, h, cw, ch) = preview_canvas(cols, rows, side);
        let mut img = RgbaImage::from_pixel(w, h, image::Rgba([0x05, 0x05, 0x08, 255]));
        for y in 0..rows {
            for x in 0..cols {
                let cell = grid[(y * cols + x) as usize];
                if cell.ch == ' ' {
                    continue;
                }
                let px = image::Rgba([cell.color[0], cell.color[1], cell.color[2], 255]);
                let x0 = ((x as f32 * cw + inset).round() as i32).max(0) as u32;
                let y0 = ((y as f32 * ch + inset).round() as i32).max(0) as u32;
                let x1 = ((((x + 1) as f32 * cw) - inset).round() as u32).min(w);
                let y1 = ((((y + 1) as f32 * ch) - inset).round() as u32).min(h);
                for yy in y0..y1 {
                    for xx in x0..x1 {
                        img.put_pixel(xx, yy, px);
                    }
                }
            }
        }
        img
    }

    fn preview_abut(grid: &[Cell], cols: u32, rows: u32, side: u32) -> RgbaImage {
        preview_inset(grid, cols, rows, side, 0.0)
    }

    fn chars_overlap_preview(grid: &[Cell], cols: u32, rows: u32, side: u32) -> RgbaImage {
        let (w, h, cw, ch) = preview_canvas(cols, rows, side);
        let mut img = RgbaImage::from_pixel(w, h, image::Rgba([0x05, 0x05, 0x08, 255]));
        let glyph_w = (ch * crate::ascii::MONO_ADVANCE).max(1.0);
        for y in 0..rows {
            for x in 0..cols {
                let cell = grid[(y * cols + x) as usize];
                if cell.ch == ' ' {
                    continue;
                }
                let px = image::Rgba([cell.color[0], cell.color[1], cell.color[2], 255]);
                let x0 = x as f32 * cw;
                let y0 = y as f32 * ch;
                let x1 = (x0 + glyph_w).min(w as f32);
                let y1 = (y0 + ch).min(h as f32);
                for yy in y0 as u32..y1 as u32 {
                    for xx in x0 as u32..x1 as u32 {
                        if xx < w && yy < h {
                            img.put_pixel(xx, yy, px);
                        }
                    }
                }
            }
        }
        img
    }

    fn old_mosaic(grid: &[Cell], cols: u32, rows: u32, mark: Mark) -> RgbaImage {
        let (cw, ch) = match mark {
            Mark::Cells => (8u32, 8u32),
            Mark::Chars => (8u32, 16u32),
        };
        let mut img = RgbaImage::from_pixel(cols * cw, rows * ch, image::Rgba([0x05, 0x05, 0x08, 255]));
        for y in 0..rows {
            for x in 0..cols {
                let cell = grid[(y * cols + x) as usize];
                if cell.ch == ' ' {
                    continue;
                }
                let px = image::Rgba([cell.color[0], cell.color[1], cell.color[2], 255]);
                let x0 = x * cw;
                let y0 = y * ch;
                for dy in 0..ch {
                    for dx in 0..cw {
                        img.put_pixel(x0 + dx, y0 + dy, px);
                    }
                }
            }
        }
        img
    }

    fn old_svg_4bit(grid: &[Cell], cols: u32, rows: u32, mark: Mark) -> String {
        let (cw, ch) = match mark {
            Mark::Cells => (8u32, 8u32),
            Mark::Chars => (8u32, 16u32),
        };
        let snap = |c: [u8; 3]| [c[0] & 0xf0, c[1] & 0xf0, c[2] & 0xf0];
        let w = cols * cw;
        let h = rows * ch;
        let mut out = svg_open(w, h);
        for y in 0..rows {
            let row = (y * cols) as usize;
            if row >= grid.len() {
                break;
            }
            let mut x = 0u32;
            while x < cols {
                let idx = row + x as usize;
                if idx >= grid.len() {
                    break;
                }
                let cell = grid[idx];
                if cell.ch == ' ' {
                    x += 1;
                    continue;
                }
                let color = snap(cell.color);
                let mut n = 1u32;
                while x + n < cols {
                    let nidx = row + (x + n) as usize;
                    if nidx >= grid.len() {
                        break;
                    }
                    let next = grid[nidx];
                    if next.ch == ' ' || snap(next.color) != color {
                        break;
                    }
                    n += 1;
                }
                out.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{ch}\" fill=\"#{:02x}{:02x}{:02x}\"/>\n",
                    x * cw,
                    y * ch,
                    n * cw,
                    color[0],
                    color[1],
                    color[2]
                ));
                x += n;
            }
        }
        out.push_str("</svg>\n");
        out
    }

    #[test]
    fn write_export_proofs_if_asked() {
        let Ok(dest) = std::env::var("ASCII_PROOF_DIR") else {
            return;
        };
        let dest = Path::new(&dest);
        std::fs::create_dir_all(dest).unwrap();
        let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("sample.png");
        let img = load_path(&sample).expect("sample.png");
        let mut s = crate::ascii::Settings::default();
        s.cols = 60;
        let (cols, rows, grid) = crate::ascii::convert(&img, &s);
        for kind in [
            ExportKind::Txt,
            ExportKind::Png,
            ExportKind::Jpeg,
            ExportKind::Webp,
            ExportKind::Svg,
            ExportKind::Html,
            ExportKind::Ansi,
        ] {
            let bytes = encode(kind, cols, rows, &grid, s.mark).expect(kind.label());
            let path = dest.join(format!("export-sample.{}", kind.ext()));
            std::fs::write(&path, bytes).unwrap();
            assert!(path.metadata().unwrap().len() > 0);
        }
    }

    #[test]
    #[ignore]
    fn caps_15mb_jpeg_if_present() {
        let p = Path::new(
            "/home/luqmaan/Projects/cosmic-archive/Documents/Projects/Brand/cosmic-archive/assets/TROUVELOT.jpg",
        );
        if !p.exists() {
            return;
        }
        let img = load_path(p).expect("15MB jpeg");
        assert!(img.width() <= MAX_SIDE);
        assert!(img.height() <= MAX_SIDE);
        assert_eq!(img.width(), 2048);
    }

    fn phone_still() -> RgbaImage {
        let mut img = RgbaImage::new(1080, 1920);
        for (x, y, p) in img.enumerate_pixels_mut() {
            let t = y as f32 / 1919.0;
            let s = x as f32 / 1079.0;
            p.0 = [
                (40.0 + 180.0 * t) as u8,
                (80.0 + 100.0 * s) as u8,
                (200.0 - 120.0 * t) as u8,
                255,
            ];
        }
        img
    }

    #[test]
    fn svg_phone_photo_at_density_200_is_figma_light() {
        let img = phone_still();
        let mut s = crate::ascii::Settings::default();
        s.cols = 200;
        crate::ascii::Look::Visor.apply(&mut s);
        s.mark = crate::ascii::Mark::Cells;
        s.color_kind = crate::ascii::ColorKind::Visor;
        let (cols, rows, grid) = crate::ascii::convert(&img, &s);
        let svg = to_svg(cols, rows, &grid, s.mark);
        let bytes = svg.len();
        let texts = svg.matches("<text").count();
        let rects = svg.matches("<rect").count();
        eprintln!(
            "SVG measure: {cols}×{rows} cells={} bytes={bytes} ({:.2} MB) texts={texts} rects={rects}",
            cols * rows,
            bytes as f64 / (1024.0 * 1024.0),
        );
        assert_eq!(cols, 200);
        assert!(!svg.contains("#50a0a0"), "4-bit snap leaked into SVG");
        assert_eq!(texts, 0);
        assert!(rects > 1);
    }

    #[test]
    fn high_cols_solid_cells_have_no_ground_gaps() {
        let img = RgbaImage::from_pixel(160, 160, image::Rgba([255, 255, 255, 255]));
        let mut s = crate::ascii::Settings::default();
        s.cols = 320;
        crate::ascii::Look::Blocks.apply(&mut s);
        s.color_kind = crate::ascii::ColorKind::Source;
        s.mark = Mark::Cells;
        let (cols, rows, grid) = crate::ascii::convert(&img, &s);
        assert_eq!(cols, 320);
        assert!(grid.iter().all(|c| c.ch != ' '));
        let mosaic = mosaic_rgba(cols, rows, &grid, Mark::Cells);
        let (cw, _) = cell_px(cols, rows, Mark::Cells);
        assert!(cw >= 1);
        assert_eq!(mosaic.width(), cols * cw);
        for p in mosaic.pixels() {
            assert_eq!(
                p.0,
                [255, 255, 255, 255],
                "ground or inset gap in high-col cells"
            );
        }
    }

    #[test]
    fn chars_export_paints_glyphs() {
        let img = RgbaImage::from_pixel(40, 20, image::Rgba([255, 255, 255, 255]));
        let mut s = crate::ascii::Settings::default();
        s.cols = 16;
        crate::ascii::Look::Blocks.apply(&mut s);
        s.color_kind = crate::ascii::ColorKind::Cream;
        s.mark = Mark::Chars;
        let (cols, rows, grid) = crate::ascii::convert(&img, &s);
        assert!(grid.iter().any(|c| c.ch != ' '));
        let mosaic = mosaic_rgba(cols, rows, &grid, Mark::Chars);
        let painted = mosaic.pixels().filter(|p| p.0 != [0x05, 0x05, 0x08, 255]).count();
        assert!(
            painted > 100,
            "CHARS raster should paint glyphs, got {painted} lit pixels"
        );
    }

    #[test]
    fn svg_keeps_exact_rgb_not_4bit() {
        let cell = Cell {
            ch: '█',
            color: [0x59, 0x2a, 0xa4],
        };
        let svg = to_svg(1, 1, &[cell], Mark::Cells);
        assert!(svg.contains("#592aa4"), "{svg}");
        assert!(!svg.contains("#5020a0"), "{svg}");
    }

    #[test]
    fn export_kind_mime_matches_extension() {
        assert_eq!(ExportKind::Txt.ext(), "txt");
        assert_eq!(ExportKind::Svg.ext(), "svg");
        assert_eq!(ExportKind::Ansi.ext(), "ans");
        assert_eq!(ExportKind::Png.ext(), "png");
        assert_eq!(ExportKind::Jpeg.ext(), "jpg");
        assert_eq!(ExportKind::Webp.ext(), "webp");
        assert!(ExportKind::Txt.mime().starts_with("text/plain"));
        assert!(ExportKind::Svg.mime().starts_with("image/svg+xml"));
        assert!(ExportKind::Ansi.mime().starts_with("text/plain"));
        assert_eq!(ExportKind::Png.mime(), "image/png");
        assert_eq!(ExportKind::Jpeg.mime(), "image/jpeg");
        assert_eq!(ExportKind::Webp.mime(), "image/webp");
    }
}
