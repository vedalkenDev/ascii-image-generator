use image::RgbaImage;

const CHAR_ASPECT: f32 = 0.5;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Alphanumeric,
    Blocks,
    Punctuation,
}

impl Charset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Alphanumeric => "Letters / numbers",
            Self::Blocks => "Blocks ░▒▓█",
            Self::Punctuation => "Punctuation",
        }
    }

    pub fn ramp(self) -> &'static [char] {
        match self {
            Self::Alphanumeric => &[
                ' ', '.', '\'', '`', '^', '"', ':', ';', 'I', 'l', '!', 'i', '~', '+', '_', '-',
                '?', '1', '|', '(', ')', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X',
                'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k',
                'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$',
            ],
            Self::Blocks => &[' ', '░', '▒', '▓', '█'],
            Self::Punctuation => &[' ', '.', ',', ':', ';', '!', '?', '-', '+', '=', '*', '#'],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Overlay,
    Ascii,
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub cols: u32,
    pub charset: Charset,
    pub contrast: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub invert: bool,
    pub color: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cols: 100,
            charset: Charset::Blocks,
            contrast: 1.0,
            saturation: 1.0,
            brightness: 0.0,
            invert: false,
            color: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub color: [u8; 3],
}

pub fn convert(img: &RgbaImage, s: &Settings) -> (u32, u32, Vec<Cell>) {
    let cols = s.cols.clamp(8, 400);
    let img_aspect = img.width().max(1) as f32 / img.height().max(1) as f32;
    let rows = ((cols as f32 / img_aspect) * CHAR_ASPECT).round().max(1.0) as u32;
    let cell_w = img.width() as f32 / cols as f32;
    let cell_h = img.height() as f32 / rows as f32;
    let ramp = s.charset.ramp();
    let mut out = Vec::with_capacity((cols * rows) as usize);

    for y in 0..rows {
        for x in 0..cols {
            let rgb = sample(img, x, y, cell_w, cell_h);
            let rgb = enhance(rgb, s);
            let mut yv = luma(rgb);
            if s.invert {
                yv = 1.0 - yv;
            }
            let ch = ramp_char(ramp, yv);
            let color = if s.color {
                to_u8(rgb)
            } else {
                let g = (yv * 255.0).round() as u8;
                [g, g, g]
            };
            out.push(Cell { ch, color });
        }
    }
    (cols, rows, out)
}

pub fn ramp_char(ramp: &[char], luma: f32) -> char {
    if ramp.is_empty() {
        return ' ';
    }
    let t = luma.clamp(0.0, 1.0);
    let i = ((t * ramp.len() as f32) as usize).min(ramp.len() - 1);
    ramp[i]
}

fn sample(img: &RgbaImage, cx: u32, cy: u32, cw: f32, ch: f32) -> [f32; 3] {
    let x0 = (cx as f32 * cw) as u32;
    let y0 = (cy as f32 * ch) as u32;
    let x1 = (((cx + 1) as f32 * cw) as u32).min(img.width()).max(x0 + 1);
    let y1 = (((cy + 1) as f32 * ch) as u32).min(img.height()).max(y0 + 1);
    let sx = ((x1 - x0) / 4).max(1);
    let sy = ((y1 - y0) / 4).max(1);
    let mut acc = [0.0f32; 3];
    let mut n = 0.0;
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            let p = img.get_pixel(x, y).0;
            let a = p[3] as f32 / 255.0;
            acc[0] += p[0] as f32 / 255.0 * a;
            acc[1] += p[1] as f32 / 255.0 * a;
            acc[2] += p[2] as f32 / 255.0 * a;
            n += 1.0;
            x += sx;
        }
        y += sy;
    }
    if n == 0.0 {
        return [0.0; 3];
    }
    [acc[0] / n, acc[1] / n, acc[2] / n]
}

fn enhance(rgb: [f32; 3], s: &Settings) -> [f32; 3] {
    let y = luma(rgb);
    let sat = s.saturation;
    let mut c = [
        y + (rgb[0] - y) * sat,
        y + (rgb[1] - y) * sat,
        y + (rgb[2] - y) * sat,
    ];
    c = c.map(|v| (v - 0.5) * s.contrast + 0.5 + s.brightness);
    c.map(|v| v.clamp(0.0, 1.0))
}

fn luma(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

fn to_u8(rgb: [f32; 3]) -> [u8; 3] {
    rgb.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_maps_ends() {
        let r = Charset::Blocks.ramp();
        assert_eq!(ramp_char(r, 0.0), ' ');
        assert_eq!(ramp_char(r, 1.0), '█');
    }

    #[test]
    fn convert_grid_size() {
        let img = RgbaImage::from_pixel(100, 50, image::Rgba([255, 255, 255, 255]));
        let mut s = Settings::default();
        s.cols = 20;
        s.charset = Charset::Blocks;
        let (cols, rows, cells) = convert(&img, &s);
        assert_eq!(cols, 20);
        assert_eq!(rows, 5);
        assert_eq!(cells.len(), 100);
        assert!(cells.iter().all(|c| c.ch == '█'));
    }

    #[test]
    fn convert_invert_maps_white_to_space() {
        let img = RgbaImage::from_pixel(40, 40, image::Rgba([255, 255, 255, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        s.charset = Charset::Blocks;
        s.invert = true;
        s.color = false;
        let (_, _, cells) = convert(&img, &s);
        assert!(cells.iter().all(|c| c.ch == ' '));
    }
}
