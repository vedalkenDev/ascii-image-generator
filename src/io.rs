use std::io::{Cursor, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, RgbaImage};

use crate::ascii::Cell;

/// Longest side kept in memory and on the GPU. Bigger stills are downscaled.
pub const MAX_SIDE: u32 = 2048;
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

pub fn load_path(path: &Path) -> Result<RgbaImage, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Could not read file: {e}"))?;
    load_bytes(&bytes)
}

pub fn load_bytes(bytes: &[u8]) -> Result<RgbaImage, String> {
    if looks_like_svg(bytes) {
        let img = magick_downscale(bytes).map_err(|_| "Could not rasterize SVG.".to_string())?;
        return Ok(downscale_max_side(img, MAX_SIDE));
    }
    let format = image::guess_format(bytes).ok();
    let (w, h) = peek_size(bytes).unwrap_or((0, 0));
    let pixels = (w as u64).saturating_mul(h as u64);
    let oversized = w.max(h) > MAX_SIDE || pixels > MAX_DECODE_PIXELS;

    if oversized {
        if let Ok(img) = magick_downscale(bytes) {
            return Ok(downscale_max_side(img, MAX_SIDE));
        }
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

fn magick_downscale(bytes: &[u8]) -> Result<RgbaImage, String> {
    let mut child = Command::new("magick")
        .args([
            "-define",
            &format!("jpeg:size={MAX_SIDE}x{MAX_SIDE}"),
            "-",
            "-auto-orient",
            "-resize",
            &format!("{MAX_SIDE}x{MAX_SIDE}>"),
            "png:-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    {
        let mut stdin = child.stdin.take().ok_or("magick stdin")?;
        stdin.write_all(bytes).map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into());
    }
    decode_rgba(&out.stdout)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Txt,
    #[allow(dead_code)]
    Png,
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
            Self::Svg => "svg",
            Self::Html => "html",
            Self::Ansi => "ans",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Txt => "TXT",
            Self::Png => "PNG",
            Self::Svg => "SVG",
            Self::Html => "HTML",
            Self::Ansi => "ANSI",
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

pub fn to_svg(cols: u32, rows: u32, grid: &[Cell]) -> String {
    let cw = 8.0_f32;
    let ch = 16.0_f32;
    let w = cols as f32 * cw;
    let h = rows as f32 * ch;
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" font-family=\"monospace\" font-size=\"16\">\n\
         <rect width=\"100%\" height=\"100%\" fill=\"#050508\"/>\n"
    );
    for y in 0..rows {
        let row = (y * cols) as usize;
        for x in 0..cols {
            let cell = grid[row + x as usize];
            let fill = format!("#{:02x}{:02x}{:02x}", cell.color[0], cell.color[1], cell.color[2]);
            let glyph = xml_escape(cell.ch);
            let px = x as f32 * cw;
            let py = (y as f32 + 0.85) * ch;
            out.push_str(&format!(
                "<text x=\"{px}\" y=\"{py}\" fill=\"{fill}\">{glyph}</text>\n"
            ));
        }
    }
    out.push_str("</svg>\n");
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

pub fn to_png(cols: u32, rows: u32, grid: &[Cell]) -> Result<Vec<u8>, String> {
    let svg = to_svg(cols, rows, grid);
    if let Ok(bytes) = magick_svg_png(svg.as_bytes()) {
        return Ok(bytes);
    }
    png_mosaic(cols, rows, grid)
}

fn magick_svg_png(svg: &[u8]) -> Result<Vec<u8>, String> {
    let mut child = Command::new("magick")
        .args(["svg:-", "png:-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    {
        let mut stdin = child.stdin.take().ok_or("magick stdin")?;
        stdin.write_all(svg).map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into());
    }
    Ok(out.stdout)
}

fn png_mosaic(cols: u32, rows: u32, grid: &[Cell]) -> Result<Vec<u8>, String> {
    const CW: u32 = 8;
    const CH: u32 = 16;
    let mut img = RgbaImage::from_pixel(cols * CW, rows * CH, image::Rgba([0x05, 0x05, 0x08, 255]));
    for y in 0..rows {
        for x in 0..cols {
            let cell = grid[(y * cols + x) as usize];
            if cell.ch == ' ' {
                continue;
            }
            let px = image::Rgba([cell.color[0], cell.color[1], cell.color[2], 255]);
            for dy in 2..CH - 2 {
                for dx in 1..CW - 1 {
                    img.put_pixel(x * CW + dx, y * CH + dy, px);
                }
            }
        }
    }
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

pub fn encode(kind: ExportKind, cols: u32, rows: u32, grid: &[Cell]) -> Result<Vec<u8>, String> {
    match kind {
        ExportKind::Txt => Ok(to_txt(cols, rows, grid).into_bytes()),
        ExportKind::Svg => Ok(to_svg(cols, rows, grid).into_bytes()),
        ExportKind::Html => Ok(to_html(cols, rows, grid).into_bytes()),
        ExportKind::Ansi => Ok(to_ansi(cols, rows, grid).into_bytes()),
        ExportKind::Png => to_png(cols, rows, grid),
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
        let svg = to_svg(cols, rows, &grid);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("█"));
        assert!(svg.contains("fill="));
        let _ = (cols, rows);
    }

    #[test]
    fn export_html_and_png() {
        let (cols, rows, grid) = sample_grid();
        let html = to_html(cols, rows, &grid);
        assert!(html.contains("<pre>"));
        assert!(html.contains("█"));
        let png = to_png(cols, rows, &grid).expect("png");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
        let img = image::load_from_memory(&png).expect("decode png export");
        assert!(img.width() >= cols);
        assert!(img.height() >= rows);
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
            ExportKind::Svg,
            ExportKind::Html,
            ExportKind::Ansi,
        ] {
            let bytes = encode(kind, cols, rows, &grid).expect(kind.label());
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
}
